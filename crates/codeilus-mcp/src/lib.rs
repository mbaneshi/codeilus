//! MCP server for AI agent integration.
//!
//! Provides 8 tools for querying the codebase knowledge graph:
//! query_symbols, query_graph, get_context, get_impact,
//! get_diagram, get_metrics, get_learning_status, explain_symbol.

pub mod server;
pub mod tools;
pub mod types;

pub use server::start_mcp_server;
