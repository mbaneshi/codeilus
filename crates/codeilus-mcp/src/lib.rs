//! MCP server for AI agent integration.
//!
//! Provides 16 tools for querying the codebase knowledge graph:
//! query_symbols, query_graph, get_context, get_impact,
//! get_diagram, get_metrics, get_learning_status, explain_symbol,
//! understand_codebase, trace_call_chain, impact_analysis,
//! find_related_code, explain_file, find_tests_for,
//! suggest_reading_order, get_community_context.

pub mod server;
pub mod tools;
pub mod types;

pub use server::start_mcp_server;
