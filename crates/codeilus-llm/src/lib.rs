//! LLM integration: provider-agnostic trait, Claude Code CLI implementation, stream parsing.

pub mod cli;
pub mod context;
pub mod provider;
pub mod stream_parser;
pub mod types;

pub use cli::ClaudeCli;
pub use context::build_context;
pub use provider::{auto_detect_provider, create_provider, LlmConfig, LlmProvider, LlmProviderKind};
pub use types::*;

use codeilus_core::CodeilusResult;

/// Convenience: check if the default Claude Code CLI is available.
pub async fn is_available() -> bool {
    ClaudeCli::new().is_available().await
}

/// Convenience: send a prompt via the default Claude Code CLI.
pub async fn prompt(request: LlmRequest) -> CodeilusResult<LlmResponse> {
    use provider::LlmProvider as _;
    ClaudeCli::new().prompt(&request).await
}
