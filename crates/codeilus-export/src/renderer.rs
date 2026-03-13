use crate::template;
use crate::types::ExportData;
use codeilus_core::error::{CodeilusError, CodeilusResult};
use std::path::Path;

/// Render ExportData into a self-contained HTML file.
pub fn render_html(data: &ExportData, output_path: &Path) -> CodeilusResult<()> {
    let data_json = serde_json::to_string(data)
        .map_err(|e| CodeilusError::Internal(format!("JSON serialization failed: {e}")))?;

    let mut env = minijinja::Environment::new();
    env.add_template("repo", template::REPO_TEMPLATE)
        .map_err(|e| CodeilusError::Internal(format!("Template parse error: {e}")))?;

    let tmpl = env
        .get_template("repo")
        .map_err(|e| CodeilusError::Internal(format!("Template not found: {e}")))?;

    let ctx = minijinja::context! {
        repo_name => &data.repo_name,
        repo_description => &data.repo_description,
        language_badges => &data.language_badges,
        overview => &data.overview,
        architecture_mermaid => &data.architecture_mermaid,
        reading_order => &data.reading_order,
        entry_points => &data.entry_points,
        architecture_narrative => &data.architecture_narrative,
        extension_guide => &data.extension_guide,
        contribution_guide => &data.contribution_guide,
        why_trending => &data.why_trending,
        metrics_snapshot => &data.metrics_snapshot,
        file_tree => &data.file_tree,
        communities => &data.communities,
        patterns => &data.patterns,
        data_json => &data_json,
    };

    let html = tmpl
        .render(&ctx)
        .map_err(|e| CodeilusError::Internal(format!("Template render error: {e}")))?;

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CodeilusError::Internal(format!("Cannot create output dir: {e}")))?;
    }

    std::fs::write(output_path, &html)
        .map_err(|e| CodeilusError::Internal(format!("Cannot write HTML: {e}")))?;

    tracing::info!(
        path = %output_path.display(),
        size_kb = html.len() / 1024,
        "Exported repo HTML"
    );

    Ok(())
}

/// Render ExportData to an HTML string (without writing to disk).
pub fn render_html_string(data: &ExportData) -> CodeilusResult<String> {
    let data_json = serde_json::to_string(data)
        .map_err(|e| CodeilusError::Internal(format!("JSON serialization failed: {e}")))?;

    let mut env = minijinja::Environment::new();
    env.add_template("repo", template::REPO_TEMPLATE)
        .map_err(|e| CodeilusError::Internal(format!("Template parse error: {e}")))?;

    let tmpl = env
        .get_template("repo")
        .map_err(|e| CodeilusError::Internal(format!("Template not found: {e}")))?;

    let ctx = minijinja::context! {
        repo_name => &data.repo_name,
        repo_description => &data.repo_description,
        language_badges => &data.language_badges,
        overview => &data.overview,
        architecture_mermaid => &data.architecture_mermaid,
        reading_order => &data.reading_order,
        entry_points => &data.entry_points,
        architecture_narrative => &data.architecture_narrative,
        extension_guide => &data.extension_guide,
        contribution_guide => &data.contribution_guide,
        why_trending => &data.why_trending,
        metrics_snapshot => &data.metrics_snapshot,
        file_tree => &data.file_tree,
        communities => &data.communities,
        patterns => &data.patterns,
        data_json => &data_json,
    };

    tmpl.render(&ctx)
        .map_err(|e| CodeilusError::Internal(format!("Template render error: {e}")))
}
