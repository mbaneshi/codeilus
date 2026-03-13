use crate::template;
use crate::types::ExportedRepo;
use codeilus_core::error::{CodeilusError, CodeilusResult};
use std::path::{Path, PathBuf};

/// Generate an index.html listing all exported repos for a given date.
pub fn generate_index(
    repos: &[ExportedRepo],
    date: &str,
    output_dir: &Path,
) -> CodeilusResult<PathBuf> {
    let mut env = minijinja::Environment::new();
    env.add_template("index", template::INDEX_TEMPLATE)
        .map_err(|e| CodeilusError::Internal(format!("Index template parse error: {e}")))?;

    let tmpl = env
        .get_template("index")
        .map_err(|e| CodeilusError::Internal(format!("Index template not found: {e}")))?;

    let ctx = minijinja::context! {
        date => date,
        repos => repos,
    };

    let html = tmpl
        .render(&ctx)
        .map_err(|e| CodeilusError::Internal(format!("Index render error: {e}")))?;

    std::fs::create_dir_all(output_dir)
        .map_err(|e| CodeilusError::Internal(format!("Cannot create output dir: {e}")))?;

    let output_path = output_dir.join("index.html");
    std::fs::write(&output_path, &html)
        .map_err(|e| CodeilusError::Internal(format!("Cannot write index HTML: {e}")))?;

    tracing::info!(
        path = %output_path.display(),
        repos = repos.len(),
        "Generated index page"
    );

    Ok(output_path)
}
