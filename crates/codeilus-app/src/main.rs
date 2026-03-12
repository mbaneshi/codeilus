//! Codeilus binary: DB setup, migrations, EventBus, API server, CLI subcommands.

use codeilus_api::{serve_until_signal, AppState};
use codeilus_core::EventBus;
use codeilus_db::{BatchWriter, DbPool, Migrator};
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

    let state = AppState::new(Arc::clone(&db), Arc::clone(&event_bus));

    match cli.command {
        Some(Command::Serve { port }) => {
            let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
            info!(%addr, "starting codeilus server");
            serve_until_signal(addr, state, shutdown_signal()).await?;
        }
        Some(Command::Analyze { path }) => {
            info!(path = %path.display(), "analyzing codebase (not yet implemented)");
            // TODO: Sprint 1
        }
        Some(Command::Harvest { trending, date, languages }) => {
            info!(trending, ?date, ?languages, "harvesting (not yet implemented)");
            // TODO: Sprint 7
        }
        Some(Command::Export { path, all_harvested, date, output }) => {
            info!(?path, all_harvested, ?date, output = %output.display(), "exporting (not yet implemented)");
            // TODO: Sprint 7
        }
        Some(Command::Deploy { path, cloudflare, gh_pages }) => {
            info!(path = %path.display(), cloudflare, gh_pages, "deploying (not yet implemented)");
            // TODO: Sprint 7
        }
        Some(Command::Mcp) => {
            info!("MCP server (not yet implemented)");
            // TODO: Sprint 8
        }
        None => {
            // Default: if path given, analyze + serve; otherwise just serve
            if let Some(repo_path) = cli.path {
                info!(path = %repo_path.display(), "analyzing codebase (not yet implemented)");
                // TODO: Sprint 1 - analyze first, then serve
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
