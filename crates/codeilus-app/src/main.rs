//! Codeilus binary: DB setup, migrations, EventBus, API server, CLI subcommands.

use codeilus_api::{serve_until_signal, AppState};
use codeilus_core::EventBus;
use codeilus_core::ids::SymbolId;
use codeilus_db::{
    BatchWriter, ChapterRepo, CommunityRepo, DbPool, EdgeRepo, FileMetricsRepo, Migrator,
    NarrativeRepo, PatternRepo, PatternRow, ProcessRepo, QuizRepo,
};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

#[derive(Parser)]
#[command(name = "codeilus", about = "Turn any codebase into an interactive learning experience")]
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
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(path = %path.display(), "analyzing codebase");

    // 0. CLEAR previous analysis data (enables re-analyze)
    info!("Clearing previous analysis data...");
    db.clear_analysis_data()?;

    // 1. PARSE
    info!("Step 1/5: Parsing repository...");
    let config = codeilus_parse::ParseConfig::new(path.to_path_buf());
    let parsed_files = codeilus_parse::parse_repository(&config, Some(event_bus))?;
    info!(files = parsed_files.len(), symbols = parsed_files.iter().map(|f| f.symbols.len()).sum::<usize>(), "Parsing complete");

    // 2. STORE
    info!("Step 2/5: Storing parsed data...");
    db.persist_parsed_files(&parsed_files)?;
    info!("Stored files and symbols in database");

    // 3. GRAPH
    info!("Step 3/8: Building knowledge graph...");
    let graph = codeilus_graph::GraphBuilder::new().build(&parsed_files)?;
    info!(
        nodes = graph.graph.node_count(),
        edges = graph.graph.edge_count(),
        communities = graph.communities.len(),
        entry_points = graph.entry_points.len(),
        processes = graph.processes.len(),
        "Knowledge graph built"
    );

    // 3b. PERSIST GRAPH DATA
    info!("Persisting graph data...");
    {
        // Persist edges
        let edge_repo = EdgeRepo::new(db.conn_arc());
        let edges: Vec<(SymbolId, SymbolId, String, f64)> = graph
            .graph
            .edge_indices()
            .filter_map(|ei| {
                let (src_idx, tgt_idx) = graph.graph.edge_endpoints(ei)?;
                let edge = graph.graph.edge_weight(ei)?;
                let src_node = graph.graph.node_weight(src_idx)?;
                let tgt_node = graph.graph.node_weight(tgt_idx)?;
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

        // Persist communities + members
        let community_repo = CommunityRepo::new(db.conn_arc());
        for community in &graph.communities {
            let cid = community_repo.insert(&community.label, community.cohesion)?;
            let members: Vec<_> = community.members.iter().map(|sid| (cid, *sid)).collect();
            if !members.is_empty() {
                community_repo.insert_members_batch(&members)?;
            }
        }
        info!(count = graph.communities.len(), "Communities persisted");

        // Persist processes + steps
        let process_repo = ProcessRepo::new(db.conn_arc());
        for process in &graph.processes {
            let pid = process_repo.insert(&process.name, process.entry_symbol_id)?;
            for step in &process.steps {
                process_repo.insert_step(pid, step.order as i64, step.symbol_id, &step.description)?;
            }
        }
        info!(count = graph.processes.len(), "Processes persisted");
    }

    // 4. METRICS
    info!("Step 4/8: Computing metrics...");
    let metrics = codeilus_metrics::compute_metrics(&parsed_files, &graph, path)?;
    info!(
        total_files = metrics.repo_metrics.total_files,
        total_sloc = metrics.repo_metrics.total_sloc,
        avg_complexity = format!("{:.1}", metrics.repo_metrics.avg_complexity),
        modularity = format!("{:.3}", metrics.repo_metrics.modularity_q),
        "Metrics computed"
    );

    // 4b. PERSIST METRICS
    {
        let metrics_repo = FileMetricsRepo::new(db.conn_arc());
        let batch: Vec<_> = metrics
            .file_metrics
            .iter()
            .map(|fm| (fm.file_id, fm.sloc as i64, fm.complexity, fm.churn as i64, fm.contributors as i64, fm.heatmap_score))
            .collect();
        if !batch.is_empty() {
            metrics_repo.insert_batch(&batch)?;
            info!(count = batch.len(), "File metrics persisted");
        }
    }

    // 5. ANALYZE
    info!("Step 5/8: Detecting patterns...");
    let patterns = codeilus_analyze::analyze(&parsed_files, &graph)?;
    info!(patterns = patterns.len(), "Pattern detection complete");

    // 5b. PERSIST PATTERNS
    {
        let pattern_repo = PatternRepo::new(db.conn_arc());
        let rows: Vec<PatternRow> = patterns
            .iter()
            .map(|p| PatternRow {
                id: 0,
                kind: p.kind.as_str().to_string(),
                severity: p.severity.as_str().to_string(),
                file_id: p.file_id.map(|fid| fid.0),
                symbol_id: p.symbol_id.map(|sid| sid.0),
                description: p.message.clone(),
            })
            .collect();
        if !rows.is_empty() {
            pattern_repo.insert_batch(&rows)?;
            info!(count = rows.len(), "Patterns persisted");
        }
    }

    // 6. DIAGRAM
    info!("Step 6/8: Generating diagrams...");
    match codeilus_diagram::generate_architecture(&graph) {
        Ok(diagram) => info!(len = diagram.len(), "Architecture diagram generated"),
        Err(e) => tracing::warn!(error = %e, "Diagram generation failed (non-fatal)"),
    }

    // 7. NARRATE
    info!("Step 7/8: Generating narratives...");
    match codeilus_narrate::generate_all_narratives(&graph, &parsed_files, path).await {
        Ok(narratives) => {
            // Persist narratives
            let narrative_repo = NarrativeRepo::new(db.conn_arc());
            let batch: Vec<(String, Option<i64>, String)> = narratives
                .iter()
                .map(|n| {
                    let kind_key = codeilus_narrate::narrative_kind_key(n.kind).to_string();
                    (kind_key, n.target_id, n.content.clone())
                })
                .collect();
            if !batch.is_empty() {
                narrative_repo.insert_batch(&batch)?;
            }
            info!(count = narratives.len(), "Narratives generated and persisted");
        }
        Err(e) => tracing::warn!(error = %e, "Narrative generation failed (non-fatal)"),
    }

    // 8. LEARN
    info!("Step 8/8: Building curriculum...");
    match codeilus_learn::generate_curriculum(&graph) {
        Ok(curriculum) => {
            // Persist chapters + sections
            let chapter_repo = ChapterRepo::new(db.conn_arc());
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

            // Generate quizzes for each chapter
            let quiz_repo = QuizRepo::new(db.conn_arc());
            let mut quiz_count = 0;
            for chapter in &curriculum.chapters {
                match codeilus_learn::generate_quiz(chapter, &graph) {
                    Ok(quiz) => {
                        // Map chapter title → DB chapter id via community_id
                        // We need the DB chapter id, not the in-memory one
                        // Find the DB chapter by matching community_id
                        let db_chapter_id = if let Some(comm_id) = chapter.community_id {
                            let chapter_repo2 = ChapterRepo::new(db.conn_arc());
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
        }
        Err(e) => tracing::warn!(error = %e, "Curriculum generation failed (non-fatal)"),
    }

    // Summary
    info!("═══════════════════════════════════════════");
    info!("Analysis complete!");
    info!("  Files:       {}", metrics.repo_metrics.total_files);
    info!("  Symbols:     {}", metrics.repo_metrics.total_symbols);
    info!("  SLOC:        {}", metrics.repo_metrics.total_sloc);
    info!("  Communities: {}", graph.communities.len());
    info!("  Entry points:{}", graph.entry_points.len());
    info!("  Patterns:    {}", patterns.len());
    info!("  Complexity:  {:.1}", metrics.repo_metrics.avg_complexity);
    info!("  Modularity:  {:.3}", metrics.repo_metrics.modularity_q);
    info!("═══════════════════════════════════════════");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

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
    let event_bus = Arc::new(EventBus::new(256));

    // Wire BatchWriter to EventBus
    let batch_writer = Arc::new(BatchWriter::spawn(db.conn_arc()));
    let bw = Arc::clone(&batch_writer);
    let mut event_rx = event_bus.subscribe();
    tokio::spawn(async move {
        loop {
            match event_rx.recv().await {
                Ok(event) => {
                    if let Err(e) = bw.write(event) {
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

    let mut state = AppState::new(Arc::clone(&db), Arc::clone(&event_bus));

    match cli.command {
        Some(Command::Serve { port }) => {
            state = state.with_repo_root(std::env::current_dir()?);
            let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
            info!(%addr, "starting codeilus server");
            serve_until_signal(addr, state, shutdown_signal()).await?;
        }
        Some(Command::Analyze { path }) => {
            run_analyze(&path, &db, &event_bus).await?;
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
                run_analyze(repo_path, &db, &event_bus).await?;
                state = state.with_repo_root(std::fs::canonicalize(repo_path)?);
            } else {
                state = state.with_repo_root(std::env::current_dir()?);
            }
            let addr: SocketAddr = "127.0.0.1:4174".parse()?;
            info!(%addr, "starting codeilus server");
            serve_until_signal(addr, state, shutdown_signal()).await?;
        }
    }

    // Shutdown BatchWriter
    match Arc::try_unwrap(batch_writer) {
        Ok(bw) => {
            if let Err(e) = bw.shutdown() {
                tracing::warn!(error = %e, "batch writer shutdown error");
            }
        }
        Err(arc) => { drop(arc); }
    }

    Ok(())
}
