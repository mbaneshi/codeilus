//! MCP server setup with stdio transport.

use crate::tools::CodeilusTools;
use codeilus_core::CodeilusResult;
use codeilus_db::DbPool;
use std::sync::Arc;
use tracing::info;

/// Start the MCP server on stdio.
///
/// The server runs until stdin is closed (e.g. when the AI agent disconnects).
pub async fn start_mcp_server(db: DbPool) -> CodeilusResult<()> {
    let tools = CodeilusTools::new(Arc::new(db));

    info!("starting MCP server on stdio");

    let transport = rmcp::transport::io::stdio();

    let server = rmcp::service::serve_server(tools, transport)
        .await
        .map_err(|e| codeilus_core::CodeilusError::Internal(format!("MCP server init error: {}", e)))?;

    info!("MCP server initialized, waiting for requests");

    // Wait for the server to complete (stdin closed)
    server
        .waiting()
        .await
        .map_err(|e| codeilus_core::CodeilusError::Internal(format!("MCP server error: {}", e)))?;

    info!("MCP server shut down");
    Ok(())
}
