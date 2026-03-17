//! Codeilus binary: DB setup, migrations, EventBus, API server, CLI subcommands.

use codeilus_api::{serve_until_signal, AppState};
use codeilus_core::{CodeilusConfig, EventBus};
use codeilus_core::ids::SymbolId;
use codeilus_db::{
    BatchWriter, ChapterRepo, CommunityRepo, DbPool, EdgeRepo, FileMetricsRepo, FileRepo,
    Migrator, NarrativeRepo, PatternRepo, PatternRow, PipelineRepo, ProcessRepo, QuizRepo,
};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

#[derive(Parser)]
#[command(name = "codeilus", version, about = "Turn any codebase into an interactive learning experience")]
struct Cli {
    /// Path to analyze (shorthand for `analyze <path> && serve`)
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Analyze a codebase
    Analyze {
        /// Path to the repository
        path: PathBuf,
        /// Force full re-analysis, ignoring checkpoints
        #[arg(long)]
        force: bool,
    },
    /// Start the interactive server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "4174")]
        port: u16,
    },
    /// Scrape GitHub trending repos, clone, analyze, narrate
    Harvest {
        /// Scrape trending repos
        #[arg(long)]
        trending: bool,
        /// Date to harvest (YYYY-MM-DD)
        #[arg(long)]
        date: Option<String>,
        /// Filter by languages (comma-separated)
        #[arg(long)]
        languages: Option<String>,
    },
    /// Export analyzed repo as static HTML
    Export {
        /// Path to analyzed repo (or --all-harvested)
        path: Option<PathBuf>,
        /// Export all harvested repos for a date
        #[arg(long)]
        all_harvested: bool,
        /// Date for harvested repos
        #[arg(long)]
        date: Option<String>,
        /// Output directory
        #[arg(short, long, default_value = "./output")]
        output: PathBuf,
    },
    /// Deploy static output to CDN
    Deploy {
        /// Path to static output directory
        path: PathBuf,
        /// Deploy to Cloudflare Pages
        #[arg(long)]
        cloudflare: bool,
        /// Deploy to GitHub Pages
        #[arg(long)]
        gh_pages: bool,
    },
    /// Start MCP stdio server
    Mcp,
}

fn default_db_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    format!("{}/.codeilus/codeilus.db", home)
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    info!("shutdown signal received");
}

async fn run_analyze(
    path: &Path,
    db: &Arc<DbPool>,
    event_bus: &Arc<EventBus>,
    llm: &Arc<dyn codeilus_llm::LlmProvider>,
    force: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(path = %path.display(), "analyzing codebase");

    let pipeline = PipelineRepo::new(Arc::clone(db));
    let repo_path = path.to_string_lossy().to_string();
    if force {
        info!("Force flag set, resetting pipeline checkpoints and clearing data");
        pipeline.reset(&repo_path)?;
        db.clear_analysis_data()?;
    }

    // 1. PARSE (incremental if possible)
    let parsed_files = if pipeline.is_completed(&repo_path, "parse") {
        info!(step = 1, total_steps = 8, phase = "parsing", "Skipping parse (checkpoint found)");
        let config = codeilus_parse::ParseConfig::new(path.to_path_buf());
        codeilus_parse::parse_repository(&config, None)?
    } else {
        pipeline.mark_started(&repo_path, "parse")?;
        let result = (|| -> Result<Vec<codeilus_parse::ParsedFile>, Box<dyn std::error::Error + Send + Sync>> {
            let config = codeilus_parse::ParseConfig::new(path.to_path_buf());
            let file_repo = FileRepo::new(Arc::clone(db));
            let existing_files = file_repo.list_existing().unwrap_or_default();

            let files = if existing_files.is_empty() {
                info!("No existing files found, performing full parse");
                info!(step = 1, total_steps = 8, phase = "parsing", "Parsing repository");
                db.clear_analysis_data()?;
                let files = codeilus_parse::parse_repository(&config, Some(event_bus))?;
                info!(files = files.len(), symbols = files.iter().map(|f| f.symbols.len()).sum::<usize>(), "Full parse complete");
                files
            } else {
                info!(existing = existing_files.len(), "Found existing files, attempting incremental parse");
                info!(step = 1, total_steps = 8, phase = "parsing", "Incremental parsing repository");

                let canonical_root = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

                let mut existing_map = std::collections::HashMap::new();
                for (db_path, last_mod_str) in &existing_files {
                    let rel_path = std::path::Path::new(db_path)
                        .strip_prefix(&canonical_root)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| db_path.clone());

                    let last_modified = last_mod_str.as_ref().and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| {
                            let secs = dt.timestamp() as u64;
                            let nanos = dt.timestamp_subsec_nanos();
                            std::time::UNIX_EPOCH + std::time::Duration::new(secs, nanos)
                        })
                    });

                    existing_map.insert(
                        rel_path,
                        codeilus_parse::ExistingFile {
                            id: codeilus_core::FileId(0),
                            last_modified,
                        },
                    );
                }

                let result = codeilus_parse::parse_repository_incremental(&config, &existing_map, Some(event_bus))?;
                info!(
                    changed = result.changed_files.len(),
                    unchanged = result.unchanged_ids.len(),
                    "Incremental parse complete"
                );

                if result.changed_files.is_empty() {
                    info!("No files changed since last analysis, skipping re-persist");
                    codeilus_parse::parse_repository(&config, None)?
                } else {
                    info!("Changes detected, clearing and re-persisting");
                    db.clear_analysis_data()?;
                    codeilus_parse::parse_repository(&config, Some(event_bus))?
                }
            };
            Ok(files)
        })();
        match result {
            Ok(files) => {
                pipeline.mark_completed(&repo_path, "parse")?;
                files
            }
            Err(e) => {
                let err_msg = e.to_string();
                pipeline.mark_failed(&repo_path, "parse", &err_msg)?;
                return Err(e);
            }
        }
    };

    // 2. STORE
    if pipeline.is_completed(&repo_path, "store") {
        info!(step = 2, total_steps = 8, phase = "storing", "Skipping store (checkpoint found)");
    } else {
        pipeline.mark_started(&repo_path, "store")?;
        info!(step = 2, total_steps = 8, phase = "storing", "Storing parsed data");
        match db.persist_parsed_files(&parsed_files) {
            Ok(()) => {
                pipeline.mark_completed(&repo_path, "store")?;
                info!("Stored files and symbols in database");
            }
            Err(e) => {
                let err_msg = e.to_string();
                pipeline.mark_failed(&repo_path, "store", &err_msg)?;
                return Err(e.into());
            }
        }
    }

    // 3. GRAPH
    let graph = if pipeline.is_completed(&repo_path, "graph") {
        info!(step = 3, total_steps = 8, phase = "graph", "Rebuilding graph from parsed data (checkpoint found, re-deriving in memory)");
        // Graph is in-memory only; rebuild it but skip DB persist
        match codeilus_graph::GraphBuilder::new().build(&parsed_files) {
            Ok(g) => Some(g),
            Err(e) => {
                tracing::warn!(error = %e, "Graph rebuild failed (non-fatal)");
                None
            }
        }
    } else {
        pipeline.mark_started(&repo_path, "graph")?;
        info!(step = 3, total_steps = 8, phase = "graph", "Building knowledge graph");
        match codeilus_graph::GraphBuilder::new().build(&parsed_files) {
            Ok(g) => {
                info!(
                    nodes = g.graph.node_count(),
                    edges = g.graph.edge_count(),
                    communities = g.communities.len(),
                    entry_points = g.entry_points.len(),
                    processes = g.processes.len(),
                    "Knowledge graph built"
                );

                // 3b. PERSIST GRAPH DATA
                info!("Persisting graph data...");
                {
                    let edge_repo = EdgeRepo::new(Arc::clone(db));
                    let edges: Vec<(SymbolId, SymbolId, String, f64)> = g
                        .graph
                        .edge_indices()
                        .filter_map(|ei| {
                            let (src_idx, tgt_idx) = g.graph.edge_endpoints(ei)?;
                            let edge = g.graph.edge_weight(ei)?;
                            let src_node = g.graph.node_weight(src_idx)?;
                            let tgt_node = g.graph.node_weight(tgt_idx)?;
                            let kind_str = match edge.kind {
                                codeilus_core::types::EdgeKind::Calls => "CALLS",
                                codeilus_core::types::EdgeKind::Imports => "IMPORTS",
                                codeilus_core::types::EdgeKind::Extends => "EXTENDS",
                                codeilus_core::types::EdgeKind::Implements => "IMPLEMENTS",
                                codeilus_core::types::EdgeKind::Contains => "CONTAINS",
                            };
                            Some((src_node.symbol_id, tgt_node.symbol_id, kind_str.to_string(), edge.confidence.0))
                        })
                        .collect();
                    if !edges.is_empty() {
                        edge_repo.insert_batch(&edges)?;
                        info!(count = edges.len(), "Edges persisted");
                    }

                    let community_repo = CommunityRepo::new(Arc::clone(db));
                    for community in &g.communities {
                        let cid = community_repo.insert(&community.label, community.cohesion)?;
                        let members: Vec<_> = community.members.iter().map(|sid| (cid, *sid)).collect();
                        if !members.is_empty() {
                            community_repo.insert_members_batch(&members)?;
                        }
                    }
                    info!(count = g.communities.len(), "Communities persisted");

                    let process_repo = ProcessRepo::new(Arc::clone(db));
                    for process in &g.processes {
                        let pid = process_repo.insert(&process.name, process.entry_symbol_id)?;
                        for step in &process.steps {
                            process_repo.insert_step(pid, step.order as i64, step.symbol_id, &step.description)?;
                        }
                    }
                    info!(count = g.processes.len(), "Processes persisted");
                }

                pipeline.mark_completed(&repo_path, "graph")?;
                Some(g)
            }
            Err(e) => {
                let err_msg = e.to_string();
                pipeline.mark_failed(&repo_path, "graph", &err_msg)?;
                tracing::warn!(error = %e, "Graph building failed (non-fatal), skipping graph-dependent steps");
                None
            }
        }
    };

    // 4. METRICS (depends on graph)
    let metrics = if pipeline.is_completed(&repo_path, "metrics") {
        info!(step = 4, total_steps = 8, phase = "metrics", "Skipping metrics (checkpoint found)");
        // Re-derive metrics in memory for summary display
        if let Some(ref graph) = graph {
            codeilus_metrics::compute_metrics(&parsed_files, graph, path).ok()
        } else {
            None
        }
    } else {
        pipeline.mark_started(&repo_path, "metrics")?;
        info!(step = 4, total_steps = 8, phase = "metrics", "Computing metrics");
        if let Some(ref graph) = graph {
            match codeilus_metrics::compute_metrics(&parsed_files, graph, path) {
                Ok(m) => {
                    info!(
                        total_files = m.repo_metrics.total_files,
                        total_sloc = m.repo_metrics.total_sloc,
                        avg_complexity = format!("{:.1}", m.repo_metrics.avg_complexity),
                        modularity = format!("{:.3}", m.repo_metrics.modularity_q),
                        "Metrics computed"
                    );

                    {
                        let metrics_repo = FileMetricsRepo::new(Arc::clone(db));
                        let batch: Vec<_> = m
                            .file_metrics
                            .iter()
                            .map(|fm| (fm.file_id, fm.sloc as i64, fm.complexity, fm.churn as i64, fm.contributors as i64, fm.heatmap_score))
                            .collect();
                        if !batch.is_empty() {
                            metrics_repo.insert_batch(&batch)?;
                            info!(count = batch.len(), "File metrics persisted");
                        }
                    }

                    pipeline.mark_completed(&repo_path, "metrics")?;
                    Some(m)
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    pipeline.mark_failed(&repo_path, "metrics", &err_msg)?;
                    tracing::warn!(error = %e, "Metrics computation failed (non-fatal)");
                    None
                }
            }
        } else {
            tracing::warn!("Skipping metrics (no graph available)");
            None
        }
    };

    // 5. ANALYZE (depends on graph)
    let patterns = if pipeline.is_completed(&repo_path, "analyze") {
        info!(step = 5, total_steps = 8, phase = "analyze", "Skipping analyze (checkpoint found)");
        None
    } else {
        pipeline.mark_started(&repo_path, "analyze")?;
        info!(step = 5, total_steps = 8, phase = "analyze", "Detecting patterns");
        if let Some(ref graph) = graph {
            match codeilus_analyze::analyze(&parsed_files, graph) {
                Ok(p) => {
                    info!(patterns = p.len(), "Pattern detection complete");

                    {
                        let pattern_repo = PatternRepo::new(Arc::clone(db));
                        let rows: Vec<PatternRow> = p
                            .iter()
                            .map(|pat| PatternRow {
                                id: 0,
                                kind: pat.kind.as_str().to_string(),
                                severity: pat.severity.as_str().to_string(),
                                file_id: pat.file_id.map(|fid| fid.0),
                                symbol_id: pat.symbol_id.map(|sid| sid.0),
                                description: pat.message.clone(),
                            })
                            .collect();
                        if !rows.is_empty() {
                            pattern_repo.insert_batch(&rows)?;
                            info!(count = rows.len(), "Patterns persisted");
                        }
                    }

                    pipeline.mark_completed(&repo_path, "analyze")?;
                    Some(p)
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    pipeline.mark_failed(&repo_path, "analyze", &err_msg)?;
                    tracing::warn!(error = %e, "Pattern detection failed (non-fatal)");
                    None
                }
            }
        } else {
            tracing::warn!("Skipping pattern analysis (no graph available)");
            None
        }
    };

    // 6. DIAGRAM (depends on graph)
    if pipeline.is_completed(&repo_path, "diagram") {
        info!(step = 6, total_steps = 8, phase = "diagram", "Skipping diagram (checkpoint found)");
    } else {
        pipeline.mark_started(&repo_path, "diagram")?;
        info!(step = 6, total_steps = 8, phase = "diagram", "Generating diagrams");
        if let Some(ref graph) = graph {
            match codeilus_diagram::generate_architecture(graph) {
                Ok(diagram) => {
                    info!(len = diagram.len(), "Architecture diagram generated");
                    pipeline.mark_completed(&repo_path, "diagram")?;
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    pipeline.mark_failed(&repo_path, "diagram", &err_msg)?;
                    tracing::warn!(error = %e, "Diagram generation failed (non-fatal)");
                }
            }
        } else {
            tracing::warn!("Skipping diagram generation (no graph available)");
        }
    }

    // 7. NARRATE (depends on graph)
    if pipeline.is_completed(&repo_path, "narrate") {
        info!(step = 7, total_steps = 8, phase = "narrate", "Skipping narrate (checkpoint found)");
    } else {
        pipeline.mark_started(&repo_path, "narrate")?;
        info!(step = 7, total_steps = 8, phase = "narrate", "Generating narratives");
        if let Some(ref graph) = graph {
            match codeilus_narrate::generate_all_narratives(graph, &parsed_files, path, Arc::clone(llm)).await {
                Ok(narratives) => {
                    let narrative_repo = NarrativeRepo::new(Arc::clone(db));
                    let batch: Vec<(String, Option<i64>, String, bool)> = narratives
                        .iter()
                        .map(|n| {
                            let kind_key = codeilus_narrate::narrative_kind_key(n.kind).to_string();
                            (kind_key, n.target_id, n.content.clone(), n.is_placeholder)
                        })
                        .collect();
                    if !batch.is_empty() {
                        narrative_repo.insert_batch(&batch)?;
                    }
                    pipeline.mark_completed(&repo_path, "narrate")?;
                    info!(count = narratives.len(), "Narratives generated and persisted");
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    pipeline.mark_failed(&repo_path, "narrate", &err_msg)?;
                    tracing::warn!(error = %e, "Narrative generation failed (non-fatal)");
                }
            }
        } else {
            tracing::warn!("Skipping narrative generation (no graph available)");
        }
    }

    // 8. LEARN (depends on graph)
    if pipeline.is_completed(&repo_path, "learn") {
        info!(step = 8, total_steps = 8, phase = "learn", "Skipping learn (checkpoint found)");
    } else {
        pipeline.mark_started(&repo_path, "learn")?;
        info!(step = 8, total_steps = 8, phase = "learn", "Building curriculum");
        if let Some(ref graph) = graph {
            match codeilus_learn::generate_curriculum(graph) {
                Ok(curriculum) => {
                    let chapter_repo = ChapterRepo::new(Arc::clone(db));
                    for chapter in &curriculum.chapters {
                        let cid = chapter_repo.insert(
                            chapter.order as i64,
                            &chapter.title,
                            &chapter.description,
                            chapter.community_id.map(|c| c.0),
                            chapter.difficulty.as_str(),
                        )?;
                        for section in &chapter.sections {
                            chapter_repo.insert_section(cid, &section.id, &section.title, section.kind.as_str(), &section.content)?;
                        }
                    }
                    info!(chapters = curriculum.chapters.len(), "Curriculum built and persisted");

                    let quiz_repo = QuizRepo::new(Arc::clone(db));
                    let mut quiz_count = 0;
                    for chapter in &curriculum.chapters {
                        match codeilus_learn::generate_quiz(chapter, graph) {
                            Ok(quiz) => {
                                let db_chapter_id = if let Some(comm_id) = chapter.community_id {
                                    let chapter_repo2 = ChapterRepo::new(Arc::clone(db));
                                    chapter_repo2.list_ordered().ok().and_then(|chs| {
                                        chs.iter().find(|c| c.community_id == Some(comm_id.0)).map(|c| c.id)
                                    })
                                } else {
                                    None
                                };

                                if let Some(db_cid) = db_chapter_id {
                                    for q in &quiz.questions {
                                        let kind_str = match q.kind {
                                            codeilus_learn::QuizQuestionKind::MultipleChoice => "multiple_choice",
                                            codeilus_learn::QuizQuestionKind::TrueFalse => "true_false",
                                            codeilus_learn::QuizQuestionKind::ImpactAnalysis => "impact_analysis",
                                        };
                                        if let Err(e) = quiz_repo.insert(
                                            db_cid,
                                            &q.question,
                                            kind_str,
                                            &q.options,
                                            q.correct_index,
                                            &q.explanation,
                                        ) {
                                            tracing::warn!(error = %e, "Failed to insert quiz question");
                                        } else {
                                            quiz_count += 1;
                                        }
                                    }
                                }
                            }
                            Err(e) => tracing::warn!(chapter = chapter.title, error = %e, "Quiz generation failed"),
                        }
                    }
                    info!(count = quiz_count, "Quiz questions generated and persisted");
                    pipeline.mark_completed(&repo_path, "learn")?;
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    pipeline.mark_failed(&repo_path, "learn", &err_msg)?;
                    tracing::warn!(error = %e, "Curriculum generation failed (non-fatal)");
                }
            }
        } else {
            tracing::warn!("Skipping curriculum generation (no graph available)");
        }
    }

    // Summary
    info!("═══════════════════════════════════════════");
    info!("Analysis complete!");
    if let Some(ref metrics) = metrics {
        info!("  Files:       {}", metrics.repo_metrics.total_files);
        info!("  Symbols:     {}", metrics.repo_metrics.total_symbols);
        info!("  SLOC:        {}", metrics.repo_metrics.total_sloc);
        info!("  Complexity:  {:.1}", metrics.repo_metrics.avg_complexity);
        info!("  Modularity:  {:.3}", metrics.repo_metrics.modularity_q);
    }
    if let Some(ref graph) = graph {
        info!("  Communities: {}", graph.communities.len());
        info!("  Entry points:{}", graph.entry_points.len());
    }
    if let Some(ref patterns) = patterns {
        info!("  Patterns:    {}", patterns.len());
    }
    info!("═══════════════════════════════════════════");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let log_format = std::env::var("CODEILUS_LOG_FORMAT").unwrap_or_default();
    if log_format == "json" {
        tracing_subscriber::fmt().json().with_env_filter(filter).init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }

    let cli = Cli::parse();

    // Open DB
    let db_path = std::env::var("CODEILUS_DB_PATH").unwrap_or_else(|_| default_db_path());
    let path = Path::new(&db_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    info!(path = %db_path, "opening database");
    let db = DbPool::new(path)?;
    {
        let conn = db.connection();
        let migrator = Migrator::new(&conn);
        let applied = migrator.apply_pending()?;
        if applied > 0 {
            info!(count = applied, "migrations applied");
        }
    }

    let db = Arc::new(db);
    let config = CodeilusConfig::from_env();
    info!(?config, "loaded configuration");
    let config = Arc::new(config);
    let event_bus = Arc::new(EventBus::new(config.event_bus_capacity));

    // Initialize LLM provider (auto-detect best available)
    let llm_provider = codeilus_llm::auto_detect_provider().await;
    info!(provider = llm_provider.name(), "LLM provider initialized");

    // Wire BatchWriter to EventBus (only if persist_events is enabled)
    let batch_writer = if config.persist_events {
        let bw = Arc::new(BatchWriter::spawn(Arc::clone(&db)));
        let bw_clone = Arc::clone(&bw);
        let mut event_rx = event_bus.subscribe();
        tokio::spawn(async move {
            loop {
                match event_rx.recv().await {
                    Ok(event) => {
                        if let Err(e) = bw_clone.write(event) {
                            tracing::warn!(error = %e, "batch writer: failed to queue event");
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(count = n, "batch writer: subscriber lagged");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        info!("event bus closed, stopping event persistence");
                        break;
                    }
                }
            }
        });
        Some(bw)
    } else {
        None
    };

    let mut state = AppState::new(Arc::clone(&db), Arc::clone(&event_bus), Arc::clone(&llm_provider), Arc::clone(&config));

    match cli.command {
        Some(Command::Serve { port }) => {
            state = state.with_repo_root(std::env::current_dir()?);
            let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
            info!(%addr, "starting codeilus server");
            serve_until_signal(addr, state, shutdown_signal()).await?;
        }
        Some(Command::Analyze { path, force }) => {
            run_analyze(&path, &db, &event_bus, &llm_provider, force).await?;
        }
        Some(Command::Harvest { trending, date, languages }) => {
            if !trending {
                info!("Use --trending to scrape GitHub trending repos");
                return Ok(());
            }
            let clone_dir = std::env::var("CODEILUS_CLONE_DIR")
                .unwrap_or_else(|_| "/tmp/codeilus-clones".into());
            let config = codeilus_harvest::HarvestConfig {
                since: codeilus_harvest::TrendingSince::Daily,
                language: languages,
                clone_dir: PathBuf::from(clone_dir),
                ..Default::default()
            };
            info!(?date, "harvesting trending repos");
            let repos = codeilus_harvest::harvest_trending(config, Some(&db)).await?;
            info!(count = repos.len(), "harvest complete");
            for repo in &repos {
                info!(
                    name = %format!("{}/{}", repo.owner, repo.name),
                    status = ?repo.status,
                    "harvested repo"
                );
            }
        }
        Some(Command::Export { path, all_harvested, date, output }) => {
            if let Some(ref repo_path) = path {
                let repo_name = repo_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                std::fs::create_dir_all(&output).ok();
                let out = codeilus_export::export_repo(repo_name, &db, &output)?;
                info!(path = %out.display(), "exported");
            } else if all_harvested {
                info!(?date, output = %output.display(), "batch export (not yet implemented)");
            } else {
                info!("Provide a path or use --all-harvested");
            }
        }
        Some(Command::Deploy { path, cloudflare, gh_pages }) => {
            info!(path = %path.display(), cloudflare, gh_pages, "deploying (not yet implemented)");
        }
        Some(Command::Mcp) => {
            info!("starting MCP server on stdio");
            let db_owned = Arc::try_unwrap(db).unwrap_or_else(|_arc| {
                DbPool::new(Path::new(&db_path)).expect("failed to open DB for MCP")
            });
            codeilus_mcp::start_mcp_server(db_owned).await?;
        }
        None => {
            // Default: if path given, analyze + serve; otherwise just serve
            if let Some(ref repo_path) = cli.path {
                run_analyze(repo_path, &db, &event_bus, &llm_provider, false).await?;
                state = state.with_repo_root(std::fs::canonicalize(repo_path)?);
            } else {
                state = state.with_repo_root(std::env::current_dir()?);
            }
            let addr: SocketAddr = format!("127.0.0.1:{}", config.server_port).parse()?;
            info!(%addr, "starting codeilus server");
            serve_until_signal(addr, state, shutdown_signal()).await?;
        }
    }

    // Shutdown BatchWriter
    if let Some(bw) = batch_writer {
        match Arc::try_unwrap(bw) {
            Ok(bw) => {
                if let Err(e) = bw.shutdown() {
                    tracing::warn!(error = %e, "batch writer shutdown error");
                }
            }
            Err(arc) => { drop(arc); }
        }
    }

    Ok(())
}
