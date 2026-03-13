//! MCP tool implementations.

use codeilus_core::ids::{FileId, SymbolId};
use codeilus_db::{
    DbPool, EdgeRepo, FileMetricsRepo, FileRepo, NarrativeRepo, ProgressRepo, SymbolRepo,
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

        let conn = self.db.conn_arc();
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

        let conn = self.db.conn_arc();
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

        let conn = self.db.conn_arc();
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

        let conn = self.db.conn_arc();
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

        let conn = self.db.conn_arc();
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

        let conn = self.db.conn_arc();
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

        let conn = self.db.conn_arc();
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
