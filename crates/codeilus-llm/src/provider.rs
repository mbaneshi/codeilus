//! LLM provider trait and configuration for provider-agnostic LLM integration.

use async_trait::async_trait;
use codeilus_core::error::{CodeilusError, CodeilusResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::cli::ClaudeCli;
use crate::types::{LlmEvent, LlmRequest, LlmResponse};

/// Trait for LLM providers. Implementations handle prompt execution
/// via different backends (Claude Code CLI, Anthropic API, etc.).
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Human-readable name of this provider.
    fn name(&self) -> &str;

    /// Check if this provider is currently usable.
    async fn is_available(&self) -> bool;

    /// Send a prompt and get the full response.
    async fn prompt(&self, request: &LlmRequest) -> CodeilusResult<LlmResponse>;

    /// Send a prompt and stream events through an mpsc channel.
    async fn prompt_stream(
        &self,
        request: &LlmRequest,
    ) -> CodeilusResult<tokio::sync::mpsc::Receiver<LlmEvent>>;
}

/// Which LLM provider to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmProviderKind {
    /// Claude Code CLI (default — uses the installed `claude` binary).
    ClaudeCode,
    /// Direct Anthropic API (requires ANTHROPIC_API_KEY).
    AnthropicApi,
}

/// Configuration for the LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: LlmProviderKind,
    pub timeout_secs: u64,
    /// Only needed for AnthropicApi provider.
    pub api_key: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProviderKind::ClaudeCode,
            timeout_secs: 180,
            api_key: None,
        }
    }
}

/// Create a provider from explicit configuration.
pub fn create_provider(config: &LlmConfig) -> Arc<dyn LlmProvider> {
    match config.provider {
        LlmProviderKind::ClaudeCode => Arc::new(ClaudeCli::with_timeout(config.timeout_secs)),
        LlmProviderKind::AnthropicApi => Arc::new(AnthropicApiStub),
    }
}

/// Auto-detect the best available provider.
///
/// Resolution order:
/// 0. `CODEILUS_SKIP_LLM` env var — if "1" or "true", return a no-op provider
/// 1. `CODEILUS_LLM_PROVIDER` env var (if set)
/// 2. Claude Code CLI (if `claude` binary found)
/// 3. Anthropic API (if `ANTHROPIC_API_KEY` set)
/// 4. Falls back to Claude Code CLI (will error on use)
pub async fn auto_detect_provider() -> Arc<dyn LlmProvider> {
    // Check skip flag first
    if let Ok(val) = std::env::var("CODEILUS_SKIP_LLM") {
        if val == "1" || val.eq_ignore_ascii_case("true") {
            info!("LLM disabled via CODEILUS_SKIP_LLM");
            return Arc::new(NoOpProvider);
        }
    }

    // Check env var override
    if let Ok(kind) = std::env::var("CODEILUS_LLM_PROVIDER") {
        match kind.to_lowercase().as_str() {
            "claude_code" | "claudecode" | "claude-code" => {
                info!("LLM provider override: Claude Code CLI");
                return create_provider(&LlmConfig::default());
            }
            "anthropic_api" | "anthropicapi" | "anthropic-api" => {
                info!("LLM provider override: Anthropic API");
                return create_provider(&LlmConfig {
                    provider: LlmProviderKind::AnthropicApi,
                    api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
                    ..Default::default()
                });
            }
            _ => {
                tracing::warn!(
                    value = %kind,
                    "Unknown CODEILUS_LLM_PROVIDER value, falling through to auto-detect"
                );
            }
        }
    }

    // Auto-detect: prefer Claude Code CLI
    let cli = ClaudeCli::with_timeout(180);
    if cli.is_available().await {
        info!("Auto-detected LLM provider: Claude Code CLI");
        return Arc::new(cli);
    }

    // Fall back to Anthropic API if key exists
    if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        info!("Auto-detected LLM provider: Anthropic API (ANTHROPIC_API_KEY found)");
        return Arc::new(AnthropicApiStub);
    }

    // Default to Claude Code CLI even if unavailable (will error on use)
    tracing::warn!("No LLM provider detected — LLM features will be unavailable");
    Arc::new(ClaudeCli::with_timeout(180))
}

// ---------------------------------------------------------------------------
// No-op provider — returned when CODEILUS_SKIP_LLM is set
// ---------------------------------------------------------------------------

/// Provider that always errors. Used when LLM is explicitly disabled.
struct NoOpProvider;

#[async_trait]
impl LlmProvider for NoOpProvider {
    fn name(&self) -> &str {
        "none (disabled)"
    }

    async fn is_available(&self) -> bool {
        false
    }

    async fn prompt(&self, _request: &LlmRequest) -> CodeilusResult<LlmResponse> {
        Err(CodeilusError::Llm(
            "LLM disabled via CODEILUS_SKIP_LLM".to_string(),
        ))
    }

    async fn prompt_stream(
        &self,
        _request: &LlmRequest,
    ) -> CodeilusResult<tokio::sync::mpsc::Receiver<LlmEvent>> {
        Err(CodeilusError::Llm(
            "LLM disabled via CODEILUS_SKIP_LLM".to_string(),
        ))
    }
}

// ---------------------------------------------------------------------------
// Anthropic API stub — placeholder for future direct API integration
// ---------------------------------------------------------------------------

/// Stub provider for direct Anthropic API calls.
/// Currently returns an error directing users to use Claude Code CLI instead.
struct AnthropicApiStub;

#[async_trait]
impl LlmProvider for AnthropicApiStub {
    fn name(&self) -> &str {
        "Anthropic API (not yet implemented)"
    }

    async fn is_available(&self) -> bool {
        std::env::var("ANTHROPIC_API_KEY").is_ok()
    }

    async fn prompt(&self, _request: &LlmRequest) -> CodeilusResult<LlmResponse> {
        Err(CodeilusError::Llm(
            "Direct Anthropic API provider is not yet implemented. \
             Use Claude Code CLI instead: npm install -g @anthropic-ai/claude-code"
                .to_string(),
        ))
    }

    async fn prompt_stream(
        &self,
        _request: &LlmRequest,
    ) -> CodeilusResult<tokio::sync::mpsc::Receiver<LlmEvent>> {
        Err(CodeilusError::Llm(
            "Direct Anthropic API provider is not yet implemented. \
             Use Claude Code CLI instead: npm install -g @anthropic-ai/claude-code"
                .to_string(),
        ))
    }
}
