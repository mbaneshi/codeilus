//! Static single-HTML page generator with inlined data.

pub mod data_loader;
pub mod index;
pub mod renderer;
pub mod template;
pub mod types;

pub use types::*;

use codeilus_core::error::CodeilusResult;
use codeilus_db::DbPool;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Export a repo to a self-contained HTML file.
pub fn export_repo(repo_name: &str, db: &Arc<DbPool>, output_dir: &Path) -> CodeilusResult<PathBuf> {
    let data = data_loader::load_export_data(repo_name, db)?;
    let filename = format!("{}.html", repo_name.replace('/', "-"));
    let output_path = output_dir.join(&filename);
    renderer::render_html(&data, &output_path)?;
    Ok(output_path)
}

/// Generate an index page listing all exported repos.
pub fn generate_index(repos: &[ExportedRepo], output_dir: &Path) -> CodeilusResult<PathBuf> {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    index::generate_index(repos, &date, output_dir)
}
