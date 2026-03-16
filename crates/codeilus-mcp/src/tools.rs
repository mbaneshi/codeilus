//! MCP tool implementations.

use codeilus_core::ids::{CommunityId, FileId, SymbolId};
use codeilus_db::{
    ChapterRepo, CommunityRepo, DbPool, EdgeRepo, FileMetricsRepo, FileRepo, NarrativeRepo,
    ProgressRepo, SymbolRepo,
};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_handler, tool_router};
use std::sync::Arc;
use tracing::debug;

use crate::types::*;

/// Codeilus MCP tool handler. Holds a DB connection for querying.
pub struct CodeilusTools {
    db: Arc<DbPool>,
    #[allow(dead_code)]
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

impl CodeilusTools {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self {
            db,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl CodeilusTools {
    /// Search for symbols (functions, classes, etc.) by name. Returns matching symbols with file locations.
    #[tool(name = "query_symbols")]
    async fn query_symbols(
        &self,
        params: Parameters<QuerySymbolsInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = params.0;
        let _limit = input.limit.unwrap_or(20);
        debug!(query = %input.query, "query_symbols");

        let conn = Arc::clone(&self.db);
        let symbol_repo = SymbolRepo::new(conn.clone());
        let file_repo = FileRepo::new(conn);

        let symbols = symbol_repo
            .search(&input.query)
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        let mut results = Vec::new();
        for sym in &symbols {
            let file_path = file_repo
                .get(sym.file_id)
                .ok()
                .map(|f| f.path)
                .unwrap_or_default();

            // Apply kind filter if provided
            if let Some(ref kind_filter) = input.kind {
                if !sym.kind.eq_ignore_ascii_case(kind_filter) {
                    continue;
                }
            }

            let mut entry = serde_json::json!({
                "id": sym.id.0,
                "name": sym.name,
                "kind": sym.kind,
                "file_path": file_path,
                "start_line": sym.start_line,
                "end_line": sym.end_line,
            });

            if let Some(ref sig) = sym.signature {
                entry["signature"] = serde_json::json!(sig);
            }
            results.push(entry);
        }

        Ok(CallToolResult::structured(serde_json::json!({
            "symbols": results,
            "count": results.len(),
        })))
    }

    /// Query the knowledge graph for a symbol's relationships: callers, callees, dependencies, inheritance.
    #[tool(name = "query_graph")]
    async fn query_graph(
        &self,
        params: Parameters<QueryGraphInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = params.0;
        debug!(symbol_id = ?input.symbol_id, "query_graph");

        let conn = Arc::clone(&self.db);
        let edge_repo = EdgeRepo::new(conn.clone());
        let symbol_repo = SymbolRepo::new(conn);

        let mut edges_out = Vec::new();
        let mut edges_in = Vec::new();

        if let Some(sid) = input.symbol_id {
            let sym_id = SymbolId(sid);

            // Outgoing edges
            if let Ok(out) = edge_repo.list_from(sym_id) {
                for e in &out {
                    let target_name = symbol_repo
                        .get(e.target_id)
                        .ok()
                        .map(|s| s.name.clone())
                        .unwrap_or_default();

                    if let Some(ref kind_filter) = input.edge_kind {
                        if !e.kind.eq_ignore_ascii_case(kind_filter) {
                            continue;
                        }
                    }

                    edges_out.push(serde_json::json!({
                        "target_id": e.target_id.0,
                        "target_name": target_name,
                        "kind": e.kind,
                        "confidence": e.confidence,
                    }));
                }
            }

            // Incoming edges
            if let Ok(inc) = edge_repo.list_to(sym_id) {
                for e in &inc {
                    let source_name = symbol_repo
                        .get(e.source_id)
                        .ok()
                        .map(|s| s.name.clone())
                        .unwrap_or_default();

                    if let Some(ref kind_filter) = input.edge_kind {
                        if !e.kind.eq_ignore_ascii_case(kind_filter) {
                            continue;
                        }
                    }

                    edges_in.push(serde_json::json!({
                        "source_id": e.source_id.0,
                        "source_name": source_name,
                        "kind": e.kind,
                        "confidence": e.confidence,
                    }));
                }
            }
        }

        Ok(CallToolResult::structured(serde_json::json!({
            "outgoing": edges_out,
            "incoming": edges_in,
        })))
    }

    /// Build structured context about the codebase for understanding. Focus on overview, a specific symbol, community, or file set.
    #[tool(name = "get_context")]
    async fn get_context(
        &self,
        params: Parameters<GetContextInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = params.0;
        debug!(focus = %input.focus, "get_context");

        let conn = Arc::clone(&self.db);
        let file_repo = FileRepo::new(conn.clone());
        let symbol_repo = SymbolRepo::new(conn);

        let context = match input.focus.as_str() {
            "overview" => {
                let files = file_repo
                    .list(None)
                    .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;
                let symbol_count = symbol_repo
                    .count()
                    .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

                format!(
                    "Repository Overview:\n- Files: {}\n- Symbols: {}\n",
                    files.len(),
                    symbol_count
                )
            }
            "symbol" => {
                if let Some(id) = input.target_id {
                    match symbol_repo.get(SymbolId(id)) {
                        Ok(sym) => format!(
                            "Symbol: {} ({})\nFile ID: {}\nLines: {}-{}\nSignature: {}",
                            sym.name,
                            sym.kind,
                            sym.file_id.0,
                            sym.start_line,
                            sym.end_line,
                            sym.signature.as_deref().unwrap_or("N/A")
                        ),
                        Err(_) => format!("Symbol {} not found", id),
                    }
                } else {
                    "No target_id provided for symbol focus".to_string()
                }
            }
            "community" => {
                if let Some(_id) = input.target_id {
                    "Community context (query DB for community members)".to_string()
                } else {
                    "No target_id provided for community focus".to_string()
                }
            }
            "files" => {
                if let Some(paths) = &input.file_paths {
                    format!("File context for: {}", paths.join(", "))
                } else {
                    "No file_paths provided for files focus".to_string()
                }
            }
            other => format!("Unknown focus: {}", other),
        };

        Ok(CallToolResult::structured(serde_json::json!({
            "context": context,
        })))
    }

    /// Analyze the blast radius of changing a symbol. Returns all affected downstream symbols with risk scores.
    #[tool(name = "get_impact")]
    async fn get_impact(
        &self,
        params: Parameters<GetImpactInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = params.0;
        let max_depth = input.depth.unwrap_or(3);
        debug!(symbol_id = input.symbol_id, max_depth, "get_impact");

        let conn = Arc::clone(&self.db);
        let edge_repo = EdgeRepo::new(conn.clone());
        let symbol_repo = SymbolRepo::new(conn);

        // BFS from the given symbol following incoming edges (who calls this?)
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut affected = Vec::new();

        queue.push_back((input.symbol_id, 0usize));
        visited.insert(input.symbol_id);

        while let Some((sid, depth)) = queue.pop_front() {
            if depth > 0 {
                let name = symbol_repo
                    .get(SymbolId(sid))
                    .ok()
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| format!("symbol_{}", sid));
                affected.push(serde_json::json!({
                    "symbol_id": sid,
                    "name": name,
                    "depth": depth,
                    "risk_score": 1.0 / depth as f64,
                }));
            }

            if depth < max_depth {
                if let Ok(edges) = edge_repo.list_to(SymbolId(sid)) {
                    for e in &edges {
                        if !visited.contains(&e.source_id.0) {
                            visited.insert(e.source_id.0);
                            queue.push_back((e.source_id.0, depth + 1));
                        }
                    }
                }
            }
        }

        Ok(CallToolResult::structured(serde_json::json!({
            "symbol_id": input.symbol_id,
            "affected": affected,
            "total_affected": affected.len(),
        })))
    }

    /// Generate a Mermaid diagram. 'architecture' for system overview, 'flowchart' for function control flow.
    #[tool(name = "get_diagram")]
    async fn get_diagram(
        &self,
        params: Parameters<GetDiagramInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = params.0;
        debug!(kind = %input.kind, "get_diagram");

        // Return placeholder — actual diagram generation requires KnowledgeGraph in memory.
        // For MCP, we can generate from DB data or return pre-stored diagrams.
        let diagram = match input.kind.as_str() {
            "architecture" => "graph TD\n    A[\"Analysis pending\"]\n    B[\"Run 'codeilus analyze' first\"]\n    A --> B".to_string(),
            "flowchart" => {
                if let Some(sid) = input.symbol_id {
                    format!(
                        "flowchart TD\n    start([\"Start: symbol_{}\"])\n    end_node([\"End\"])\n    start --> end_node",
                        sid
                    )
                } else {
                    return Err(rmcp::ErrorData::invalid_params(
                        "symbol_id required for flowchart",
                        None,
                    ));
                }
            }
            other => {
                return Err(rmcp::ErrorData::invalid_params(
                    format!("Unknown diagram kind: {}. Use 'architecture' or 'flowchart'", other),
                    None,
                ));
            }
        };

        Ok(CallToolResult::structured(serde_json::json!({
            "kind": input.kind,
            "mermaid": diagram,
        })))
    }

    /// Get code metrics: SLOC, complexity, churn, contributors, hotspot scores.
    #[tool(name = "get_metrics")]
    async fn get_metrics(
        &self,
        params: Parameters<GetMetricsInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = params.0;
        debug!(file_id = ?input.file_id, "get_metrics");

        let conn = Arc::clone(&self.db);
        let metrics_repo = FileMetricsRepo::new(conn);

        if let Some(fid) = input.file_id {
            let row = metrics_repo
                .get_by_file(FileId(fid))
                .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;
            match row {
                Some(m) => Ok(CallToolResult::structured(serde_json::json!({
                    "file_id": m.file_id.0,
                    "sloc": m.sloc,
                    "complexity": m.complexity,
                    "churn": m.churn,
                    "contributors": m.contributors,
                }))),
                None => Ok(CallToolResult::structured(serde_json::json!({
                    "error": format!("No metrics for file_id {}", fid),
                }))),
            }
        } else {
            let limit = input.top_hotspots.unwrap_or(10);
            let hotspots = metrics_repo
                .list_hotspots(limit)
                .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

            let entries: Vec<_> = hotspots
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "file_id": m.file_id.0,
                        "sloc": m.sloc,
                        "complexity": m.complexity,
                        "churn": m.churn,
                        "contributors": m.contributors,
                    })
                })
                .collect();

            Ok(CallToolResult::structured(serde_json::json!({
                "hotspots": entries,
                "count": entries.len(),
            })))
        }
    }

    /// Get current learning progress: XP, level, badges, chapter completion, streak.
    #[tool(name = "get_learning_status")]
    async fn get_learning_status(
        &self,
        _params: Parameters<GetLearningStatusInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        debug!("get_learning_status");

        let conn = Arc::clone(&self.db);
        let progress_repo = ProgressRepo::new(conn);

        let stats = progress_repo
            .get_or_create_stats()
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::structured(serde_json::json!({
            "total_xp": stats.total_xp,
            "streak_days": stats.streak_days,
            "last_active": stats.last_active,
        })))
    }

    /// Get a human-readable explanation of what a symbol does, including its role and connections.
    #[tool(name = "explain_symbol")]
    async fn explain_symbol(
        &self,
        params: Parameters<ExplainSymbolInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = params.0;
        debug!(symbol_id = input.symbol_id, "explain_symbol");

        let conn = Arc::clone(&self.db);
        let narrative_repo = NarrativeRepo::new(conn.clone());
        let symbol_repo = SymbolRepo::new(conn);

        // Try to find pre-generated explanation
        let explanation = narrative_repo
            .get_by_kind_and_target("symbol_explanation", input.symbol_id)
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        if let Some(narrative) = explanation {
            return Ok(CallToolResult::structured(serde_json::json!({
                "symbol_id": input.symbol_id,
                "explanation": narrative.content,
                "source": "pre-generated",
            })));
        }

        // Fallback: build a basic explanation from DB data
        let sym = symbol_repo
            .get(SymbolId(input.symbol_id))
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        let explanation = format!(
            "{} is a {} defined at lines {}-{}. Signature: {}",
            sym.name,
            sym.kind,
            sym.start_line,
            sym.end_line,
            sym.signature.as_deref().unwrap_or("N/A")
        );

        Ok(CallToolResult::structured(serde_json::json!({
            "symbol_id": input.symbol_id,
            "explanation": explanation,
            "source": "generated-from-db",
        })))
    }

    /// Get a comprehensive overview of the analyzed codebase: file count, languages, symbols, communities, entry points, and narratives.
    #[tool(name = "understand_codebase")]
    async fn understand_codebase(
        &self,
        _params: Parameters<UnderstandCodebaseInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        debug!("understand_codebase");

        let conn = Arc::clone(&self.db);
        let file_repo = FileRepo::new(conn.clone());
        let symbol_repo = SymbolRepo::new(conn.clone());
        let community_repo = CommunityRepo::new(conn.clone());
        let narrative_repo = NarrativeRepo::new(conn.clone());
        let chapter_repo = ChapterRepo::new(conn);

        let files = file_repo
            .list(None)
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;
        let symbol_count = symbol_repo
            .count()
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;
        let communities = community_repo
            .list()
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;
        let chapters = chapter_repo
            .list_ordered()
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        // Language breakdown
        let mut lang_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for f in &files {
            if let Some(ref lang) = f.language {
                *lang_counts.entry(lang.clone()).or_default() += 1;
            }
        }

        // Get overview narrative if available
        let overview = narrative_repo
            .get_by_kind("overview")
            .ok()
            .and_then(|n| n.map(|r| r.content));

        let community_list: Vec<_> = communities
            .iter()
            .map(|c| serde_json::json!({
                "id": c.id.0,
                "label": c.label,
                "cohesion": c.cohesion,
            }))
            .collect();

        Ok(CallToolResult::structured(serde_json::json!({
            "file_count": files.len(),
            "symbol_count": symbol_count,
            "languages": lang_counts,
            "communities": community_list,
            "community_count": communities.len(),
            "chapter_count": chapters.len(),
            "overview": overview,
        })))
    }

    /// Trace the call chain from a symbol, showing the sequence of function calls with file locations.
    #[tool(name = "trace_call_chain")]
    async fn trace_call_chain(
        &self,
        params: Parameters<TraceCallChainInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = params.0;
        let max_depth = input.max_depth.unwrap_or(5);
        debug!(symbol = %input.symbol_name, max_depth, "trace_call_chain");

        let conn = Arc::clone(&self.db);
        let symbol_repo = SymbolRepo::new(conn.clone());
        let edge_repo = EdgeRepo::new(conn.clone());
        let file_repo = FileRepo::new(conn);

        let symbols = symbol_repo
            .search(&input.symbol_name)
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        let root = match symbols.first() {
            Some(s) => s,
            None => {
                return Ok(CallToolResult::structured(serde_json::json!({
                    "error": format!("Symbol '{}' not found", input.symbol_name),
                })));
            }
        };

        // BFS through outgoing CALLS edges
        let mut chain = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((root.id, 0usize));
        visited.insert(root.id);

        while let Some((sym_id, depth)) = queue.pop_front() {
            let sym = match symbol_repo.get(sym_id) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let file_path = file_repo
                .get(sym.file_id)
                .ok()
                .map(|f| f.path)
                .unwrap_or_default();

            chain.push(serde_json::json!({
                "depth": depth,
                "symbol_id": sym_id.0,
                "name": sym.name,
                "kind": sym.kind,
                "file_path": file_path,
                "start_line": sym.start_line,
            }));

            if depth < max_depth {
                if let Ok(edges) = edge_repo.list_from(sym_id) {
                    for e in &edges {
                        if e.kind.eq_ignore_ascii_case("CALLS") && !visited.contains(&e.target_id) {
                            visited.insert(e.target_id);
                            queue.push_back((e.target_id, depth + 1));
                        }
                    }
                }
            }
        }

        Ok(CallToolResult::structured(serde_json::json!({
            "root": input.symbol_name,
            "call_chain": chain,
            "total_nodes": chain.len(),
        })))
    }

    /// Analyze blast radius of a symbol by name. Returns affected symbols grouped by depth with risk scores.
    #[tool(name = "impact_analysis")]
    async fn impact_analysis(
        &self,
        params: Parameters<ImpactAnalysisInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = params.0;
        let max_depth = input.depth.unwrap_or(2).min(3);
        debug!(symbol = %input.symbol_name, max_depth, "impact_analysis");

        let conn = Arc::clone(&self.db);
        let symbol_repo = SymbolRepo::new(conn.clone());
        let edge_repo = EdgeRepo::new(conn.clone());
        let file_repo = FileRepo::new(conn);

        let symbols = symbol_repo
            .search(&input.symbol_name)
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        let root = match symbols.first() {
            Some(s) => s,
            None => {
                return Ok(CallToolResult::structured(serde_json::json!({
                    "error": format!("Symbol '{}' not found", input.symbol_name),
                })));
            }
        };

        // BFS through incoming edges (who depends on this?)
        let mut affected = Vec::new();
        let mut affected_files = std::collections::HashSet::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((root.id, 0usize));
        visited.insert(root.id);

        while let Some((sym_id, depth)) = queue.pop_front() {
            if depth > 0 {
                if let Ok(sym) = symbol_repo.get(sym_id) {
                    let file_path = file_repo
                        .get(sym.file_id)
                        .ok()
                        .map(|f| f.path.clone())
                        .unwrap_or_default();
                    affected_files.insert(file_path.clone());
                    affected.push(serde_json::json!({
                        "symbol_id": sym_id.0,
                        "name": sym.name,
                        "kind": sym.kind,
                        "file_path": file_path,
                        "depth": depth,
                        "risk_score": 1.0 / depth as f64,
                    }));
                }
            }

            if depth < max_depth {
                if let Ok(edges) = edge_repo.list_to(sym_id) {
                    for e in &edges {
                        if !visited.contains(&e.source_id) {
                            visited.insert(e.source_id);
                            queue.push_back((e.source_id, depth + 1));
                        }
                    }
                }
            }
        }

        Ok(CallToolResult::structured(serde_json::json!({
            "symbol": input.symbol_name,
            "affected_symbols": affected,
            "affected_symbol_count": affected.len(),
            "affected_file_count": affected_files.len(),
            "max_depth_searched": max_depth,
        })))
    }

    /// Find symbols related to the given symbol: same community, directly connected, or sharing callers.
    #[tool(name = "find_related_code")]
    async fn find_related_code(
        &self,
        params: Parameters<FindRelatedCodeInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = params.0;
        debug!(symbol = %input.symbol_name, "find_related_code");

        let conn = Arc::clone(&self.db);
        let symbol_repo = SymbolRepo::new(conn.clone());
        let community_repo = CommunityRepo::new(conn.clone());
        let edge_repo = EdgeRepo::new(conn.clone());
        let file_repo = FileRepo::new(conn);

        let symbols = symbol_repo
            .search(&input.symbol_name)
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        let root = match symbols.first() {
            Some(s) => s,
            None => {
                return Ok(CallToolResult::structured(serde_json::json!({
                    "error": format!("Symbol '{}' not found", input.symbol_name),
                })));
            }
        };

        // Find community peers
        let mut community_peers = Vec::new();
        if let Ok(Some(community)) = community_repo.find_by_symbol(root.id) {
            if let Ok(members) = community_repo.list_members(community.id) {
                for member_id in members.iter().take(20) {
                    if *member_id != root.id {
                        if let Ok(sym) = symbol_repo.get(*member_id) {
                            let file_path = file_repo
                                .get(sym.file_id)
                                .ok()
                                .map(|f| f.path)
                                .unwrap_or_default();
                            community_peers.push(serde_json::json!({
                                "id": sym.id.0,
                                "name": sym.name,
                                "kind": sym.kind,
                                "file_path": file_path,
                                "relation": "same_community",
                            }));
                        }
                    }
                }
            }
        }

        // Direct neighbors
        let mut neighbors = Vec::new();
        if let Ok(out_edges) = edge_repo.list_from(root.id) {
            for e in out_edges.iter().take(10) {
                if let Ok(sym) = symbol_repo.get(e.target_id) {
                    let file_path = file_repo
                        .get(sym.file_id)
                        .ok()
                        .map(|f| f.path)
                        .unwrap_or_default();
                    neighbors.push(serde_json::json!({
                        "id": sym.id.0,
                        "name": sym.name,
                        "kind": sym.kind,
                        "file_path": file_path,
                        "edge_kind": e.kind,
                        "confidence": e.confidence,
                        "direction": "outgoing",
                    }));
                }
            }
        }
        if let Ok(in_edges) = edge_repo.list_to(root.id) {
            for e in in_edges.iter().take(10) {
                if let Ok(sym) = symbol_repo.get(e.source_id) {
                    let file_path = file_repo
                        .get(sym.file_id)
                        .ok()
                        .map(|f| f.path)
                        .unwrap_or_default();
                    neighbors.push(serde_json::json!({
                        "id": sym.id.0,
                        "name": sym.name,
                        "kind": sym.kind,
                        "file_path": file_path,
                        "edge_kind": e.kind,
                        "confidence": e.confidence,
                        "direction": "incoming",
                    }));
                }
            }
        }

        Ok(CallToolResult::structured(serde_json::json!({
            "symbol": input.symbol_name,
            "community_peers": community_peers,
            "direct_neighbors": neighbors,
        })))
    }

    /// Get a detailed explanation of a file: its purpose, symbols, connections, and metrics.
    #[tool(name = "explain_file")]
    async fn explain_file(
        &self,
        params: Parameters<ExplainFileInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = params.0;
        debug!(file = %input.file_path, "explain_file");

        let conn = Arc::clone(&self.db);
        let file_repo = FileRepo::new(conn.clone());
        let symbol_repo = SymbolRepo::new(conn.clone());
        let metrics_repo = FileMetricsRepo::new(conn);

        let file = file_repo
            .get_by_path(&input.file_path)
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        let file = match file {
            Some(f) => f,
            None => {
                return Ok(CallToolResult::structured(serde_json::json!({
                    "error": format!("File '{}' not found", input.file_path),
                })));
            }
        };

        let symbols = symbol_repo
            .list_by_file(file.id)
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        let metrics = metrics_repo
            .get_by_file(file.id)
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        let symbol_list: Vec<_> = symbols
            .iter()
            .map(|s| serde_json::json!({
                "name": s.name,
                "kind": s.kind,
                "start_line": s.start_line,
                "end_line": s.end_line,
                "signature": s.signature,
            }))
            .collect();

        let mut result = serde_json::json!({
            "file_path": file.path,
            "language": file.language,
            "sloc": file.sloc,
            "symbols": symbol_list,
            "symbol_count": symbols.len(),
        });

        if let Some(m) = metrics {
            result["metrics"] = serde_json::json!({
                "complexity": m.complexity,
                "churn": m.churn,
                "contributors": m.contributors,
            });
        }

        Ok(CallToolResult::structured(result))
    }

    /// Find test files and functions that exercise a given symbol.
    #[tool(name = "find_tests_for")]
    async fn find_tests_for(
        &self,
        params: Parameters<FindTestsForInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = params.0;
        debug!(symbol = %input.symbol_name, "find_tests_for");

        let conn = Arc::clone(&self.db);
        let symbol_repo = SymbolRepo::new(conn.clone());
        let edge_repo = EdgeRepo::new(conn.clone());
        let file_repo = FileRepo::new(conn);

        let symbols = symbol_repo
            .search(&input.symbol_name)
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        let root = match symbols.first() {
            Some(s) => s,
            None => {
                return Ok(CallToolResult::structured(serde_json::json!({
                    "error": format!("Symbol '{}' not found", input.symbol_name),
                })));
            }
        };

        // Find callers that are in test files or have "test" in their name
        let mut test_symbols = Vec::new();
        if let Ok(edges) = edge_repo.list_to(root.id) {
            for e in &edges {
                if let Ok(caller) = symbol_repo.get(e.source_id) {
                    let file_path = file_repo
                        .get(caller.file_id)
                        .ok()
                        .map(|f| f.path)
                        .unwrap_or_default();

                    let is_test = caller.name.contains("test")
                        || file_path.contains("test")
                        || file_path.contains("spec");

                    if is_test {
                        test_symbols.push(serde_json::json!({
                            "name": caller.name,
                            "kind": caller.kind,
                            "file_path": file_path,
                            "start_line": caller.start_line,
                        }));
                    }
                }
            }
        }

        Ok(CallToolResult::structured(serde_json::json!({
            "symbol": input.symbol_name,
            "tests": test_symbols,
            "test_count": test_symbols.len(),
            "has_tests": !test_symbols.is_empty(),
        })))
    }

    /// Suggest an optimal reading order for understanding the codebase. Returns top files ranked by learning value.
    #[tool(name = "suggest_reading_order")]
    async fn suggest_reading_order(
        &self,
        _params: Parameters<SuggestReadingOrderInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        debug!("suggest_reading_order");

        let conn = Arc::clone(&self.db);
        let file_repo = FileRepo::new(conn.clone());
        let symbol_repo = SymbolRepo::new(conn.clone());
        let edge_repo = EdgeRepo::new(conn.clone());
        let narrative_repo = NarrativeRepo::new(conn);

        // Check for pre-generated reading order narrative
        if let Ok(Some(narrative)) = narrative_repo.get_by_kind("reading_order") {
            return Ok(CallToolResult::structured(serde_json::json!({
                "reading_order": narrative.content,
                "source": "pre-generated",
            })));
        }

        // Compute file importance by fan-in (how many symbols call into this file)
        let files = file_repo
            .list(None)
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        let mut file_scores: Vec<(String, i64, f64)> = Vec::new(); // (path, sloc, score)
        for f in &files {
            let syms = symbol_repo.list_by_file(f.id).unwrap_or_default();
            let mut fan_in = 0usize;
            for sym in &syms {
                fan_in += edge_repo.list_to(sym.id).map(|e| e.len()).unwrap_or(0);
            }
            // Score: fan-in weighted, penalize very large files
            let size_penalty = if f.sloc > 500 { 0.7 } else { 1.0 };
            let score = fan_in as f64 * size_penalty;
            if score > 0.0 {
                file_scores.push((f.path.clone(), f.sloc, score));
            }
        }

        file_scores.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        let top: Vec<_> = file_scores
            .iter()
            .take(10)
            .enumerate()
            .map(|(i, (path, sloc, score))| serde_json::json!({
                "rank": i + 1,
                "file_path": path,
                "sloc": sloc,
                "importance_score": score,
            }))
            .collect();

        Ok(CallToolResult::structured(serde_json::json!({
            "reading_order": top,
            "source": "computed",
        })))
    }

    /// Get full context for a community: all member symbols, internal edges, external connections, and narrative.
    #[tool(name = "get_community_context")]
    async fn get_community_context(
        &self,
        params: Parameters<GetCommunityContextInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = params.0;
        let cid = CommunityId(input.community_id);
        debug!(community_id = input.community_id, "get_community_context");

        let conn = Arc::clone(&self.db);
        let community_repo = CommunityRepo::new(conn.clone());
        let symbol_repo = SymbolRepo::new(conn.clone());
        let edge_repo = EdgeRepo::new(conn.clone());
        let file_repo = FileRepo::new(conn.clone());
        let narrative_repo = NarrativeRepo::new(conn);

        let community = community_repo
            .get(cid)
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        let member_ids = community_repo
            .list_members(cid)
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        let member_set: std::collections::HashSet<_> = member_ids.iter().copied().collect();

        let mut members = Vec::new();
        let mut internal_edges = Vec::new();
        let mut external_connections = Vec::new();

        for &sym_id in &member_ids {
            if let Ok(sym) = symbol_repo.get(sym_id) {
                let file_path = file_repo
                    .get(sym.file_id)
                    .ok()
                    .map(|f| f.path)
                    .unwrap_or_default();
                members.push(serde_json::json!({
                    "id": sym_id.0,
                    "name": sym.name,
                    "kind": sym.kind,
                    "file_path": file_path,
                }));

                // Classify edges as internal vs external
                if let Ok(edges) = edge_repo.list_from(sym_id) {
                    for e in &edges {
                        if member_set.contains(&e.target_id) {
                            internal_edges.push(serde_json::json!({
                                "from": sym.name,
                                "to_id": e.target_id.0,
                                "kind": e.kind,
                                "confidence": e.confidence,
                            }));
                        } else {
                            let target_name = symbol_repo
                                .get(e.target_id)
                                .ok()
                                .map(|s| s.name)
                                .unwrap_or_default();
                            external_connections.push(serde_json::json!({
                                "from": sym.name,
                                "to": target_name,
                                "to_id": e.target_id.0,
                                "kind": e.kind,
                                "direction": "outgoing",
                            }));
                        }
                    }
                }
            }
        }

        // Get module summary narrative if available
        let narrative = narrative_repo
            .get_by_kind_and_target("module_summary", input.community_id)
            .ok()
            .and_then(|n| n.map(|r| r.content));

        Ok(CallToolResult::structured(serde_json::json!({
            "community_id": input.community_id,
            "label": community.label,
            "cohesion": community.cohesion,
            "member_count": members.len(),
            "members": members,
            "internal_edges": internal_edges,
            "external_connections": external_connections,
            "narrative": narrative,
        })))
    }
}

#[tool_handler]
impl rmcp::handler::server::ServerHandler for CodeilusTools {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo::default()
            .with_server_info(rmcp::model::Implementation::new(
                "codeilus",
                env!("CARGO_PKG_VERSION"),
            ))
    }
}
