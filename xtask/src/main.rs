use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
#[command(name = "xtask", about = "Codeilus development tasks")]
struct Cli {
    #[command(subcommand)]
    command: XTask,
}

#[derive(Subcommand)]
enum XTask {
    /// Run database migrations
    Migrate,
    /// Build the frontend
    BuildFrontend,
    /// Clean build artifacts and database
    Clean,
    /// Run all checks (build + clippy + test)
    Check,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        XTask::Migrate => {
            println!("Running migrations...");
            run("cargo", &["run", "-p", "codeilus-app", "--", "serve"])?;
            // Note: migrations run automatically on startup
            println!("Migrations applied (server started and stopped)");
        }
        XTask::BuildFrontend => {
            println!("Building frontend...");
            run_in("frontend", "npm", &["run", "build"])?;
            println!("Frontend build complete");
        }
        XTask::Clean => {
            println!("Cleaning...");
            run("cargo", &["clean"])?;
            let db_path = std::env::var("CODEILUS_DB_PATH").unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_default();
                format!("{home}/.codeilus/codeilus.db")
            });
            if std::path::Path::new(&db_path).exists() {
                std::fs::remove_file(&db_path)?;
                println!("Removed {db_path}");
            }
            println!("Clean complete");
        }
        XTask::Check => {
            println!("Running checks...");
            run("cargo", &["build"])?;
            run("cargo", &["clippy", "--", "-D", "warnings"])?;
            run("cargo", &["test"])?;
            println!("All checks passed!");
        }
    }
    Ok(())
}

fn run(cmd: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new(cmd).args(args).status()?;
    if !status.success() {
        return Err(format!("{cmd} failed with {status}").into());
    }
    Ok(())
}

fn run_in(dir: &str, cmd: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new(cmd).args(args).current_dir(dir).status()?;
    if !status.success() {
        return Err(format!("{cmd} in {dir} failed with {status}").into());
    }
    Ok(())
}
