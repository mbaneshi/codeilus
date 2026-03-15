//! Claude CLI subprocess management.

use codeilus_core::error::{CodeilusError, CodeilusResult};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, warn};

use crate::stream_parser::{is_message_stop, parse_stream_line, StreamAccumulator};
use crate::types::{LlmEvent, LlmRequest, LlmResponse};

/// Phrases in LLM responses that indicate a billing/rate-limit error rather than real content.
const GARBAGE_PHRASES: &[&str] = &[
    "Credit balance",
    "rate limit",
    "too many requests",
    "Rate limit",
    "Too many requests",
];

/// Check if an LLM response looks like a billing/rate-limit error.
fn is_garbage_response(text: &str) -> bool {
    GARBAGE_PHRASES.iter().any(|phrase| text.contains(phrase))
}

/// Claude CLI subprocess wrapper.
pub struct ClaudeCli {
    timeout_secs: u64,
}

impl ClaudeCli {
    /// Create a new ClaudeCli with default 180-second timeout.
    pub fn new() -> Self {
        Self { timeout_secs: 180 }
    }

    /// Create a new ClaudeCli with a custom timeout.
    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self { timeout_secs }
    }

    /// Check if the `claude` CLI is available on the system.
    pub async fn is_available(&self) -> bool {
        match Command::new("which").arg("claude").output().await {
            Ok(output) => output.status.success(),
            Err(_) => {
                // Fallback: try `command -v claude` via sh
                match Command::new("sh")
                    .arg("-c")
                    .arg("command -v claude")
                    .output()
                    .await
                {
                    Ok(output) => output.status.success(),
                    Err(_) => false,
                }
            }
        }
    }

    /// Run a prompt and return the full response.
    /// Retries once on error (timeout, non-zero exit, or garbage response) after a 2s delay.
    pub async fn prompt(&self, request: &LlmRequest) -> CodeilusResult<LlmResponse> {
        if !self.is_available().await {
            return Err(CodeilusError::Llm(
                "Claude CLI not found. Install it with: npm install -g @anthropic-ai/claude-code"
                    .to_string(),
            ));
        }

        match self.prompt_once(request).await {
            Ok(response) if !is_garbage_response(&response.text) => Ok(response),
            Ok(response) => {
                warn!(
                    text_preview = &response.text[..response.text.len().min(100)],
                    "garbage response detected, retrying after 2s"
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                let retry = self.prompt_once(request).await?;
                if is_garbage_response(&retry.text) {
                    Err(CodeilusError::Llm(
                        "LLM returned a billing/rate-limit error instead of content".to_string(),
                    ))
                } else {
                    Ok(retry)
                }
            }
            Err(e) => {
                warn!(error = %e, "prompt failed, retrying after 2s");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                self.prompt_once(request).await
            }
        }
    }

    /// Single attempt at running a prompt (no retry).
    async fn prompt_once(&self, request: &LlmRequest) -> CodeilusResult<LlmResponse> {
        let mut cmd = Command::new("claude");
        cmd.arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--print")
            .arg(&request.prompt);

        if let Some(system) = &request.system {
            cmd.arg("--system-prompt").arg(system);
        }

        // Unset CLAUDECODE env var to allow nested Claude CLI invocations
        cmd.env_remove("CLAUDECODE");
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        debug!(prompt_len = request.prompt.len(), "spawning claude CLI");

        let mut child = cmd
            .spawn()
            .map_err(|e| CodeilusError::Llm(format!("Failed to spawn claude: {}", e)))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| CodeilusError::Llm("Failed to capture stdout".to_string()))?;

        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut accumulator = StreamAccumulator::new();

        let timeout = tokio::time::Duration::from_secs(self.timeout_secs);

        let result = tokio::time::timeout(timeout, async {
            while let Ok(Some(line)) = lines.next_line().await {
                // Parse first, then check for stop — the result line contains the text
                if let Some(event) = parse_stream_line(&line) {
                    accumulator.feed(&event);
                }
                if is_message_stop(&line) {
                    break;
                }
            }
            accumulator.finish()
        })
        .await;

        // Clean up the child process
        let _ = child.kill().await;

        match result {
            Ok(response) => Ok(response),
            Err(_) => Err(CodeilusError::Llm(format!(
                "Claude CLI timed out after {} seconds",
                self.timeout_secs
            ))),
        }
    }

    /// Run a prompt and stream events through an mpsc channel.
    pub async fn prompt_stream(
        &self,
        request: &LlmRequest,
    ) -> CodeilusResult<tokio::sync::mpsc::Receiver<LlmEvent>> {
        if !self.is_available().await {
            return Err(CodeilusError::Llm(
                "Claude CLI not found. Install it with: npm install -g @anthropic-ai/claude-code"
                    .to_string(),
            ));
        }

        let mut cmd = Command::new("claude");
        cmd.arg("--output-format")
            .arg("stream-json")
            .arg("--verbose")
            .arg("--print")
            .arg(&request.prompt);

        if let Some(system) = &request.system {
            cmd.arg("--system-prompt").arg(system);
        }

        // Unset CLAUDECODE env var to allow nested Claude CLI invocations
        cmd.env_remove("CLAUDECODE");
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| CodeilusError::Llm(format!("Failed to spawn claude: {}", e)))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| CodeilusError::Llm("Failed to capture stdout".to_string()))?;

        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let timeout_secs = self.timeout_secs;

        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            let mut accumulator = StreamAccumulator::new();
            let timeout = tokio::time::Duration::from_secs(timeout_secs);

            let stream_result = tokio::time::timeout(timeout, async {
                while let Ok(Some(line)) = lines.next_line().await {
                    if let Some(event) = parse_stream_line(&line) {
                        accumulator.feed(&event);
                        if tx.send(event).await.is_err() {
                            break;
                        }
                    }
                    if is_message_stop(&line) {
                        break;
                    }
                }
                accumulator.finish()
            })
            .await;

            match stream_result {
                Ok(response) => {
                    let _ = tx.send(LlmEvent::Complete(response)).await;
                }
                Err(_) => {
                    let _ = tx
                        .send(LlmEvent::Error(format!(
                            "Claude CLI timed out after {} seconds",
                            timeout_secs
                        )))
                        .await;
                }
            }

            let _ = child.kill().await;
        });

        Ok(rx)
    }
}

impl Default for ClaudeCli {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn is_available_check() {
        // This test is environment-dependent — it just verifies
        // that is_available() doesn't panic.
        let cli = ClaudeCli::new();
        let _available = cli.is_available().await;
        // No assertion on the value — just checking no panic
    }

    #[tokio::test]
    async fn prompt_graceful_degradation() {
        // Use an impossible binary name to simulate claude not found
        let cli = ClaudeCli::new();
        // We can't easily mock `which`, but we can test that the
        // error path produces the right error type when CLI is unavailable.
        // If claude IS installed, this test would actually succeed,
        // so we test the error message format instead.
        let request = LlmRequest {
            prompt: "test".to_string(),
            system: None,
            max_tokens: None,
        };

        let result = cli.prompt(&request).await;
        // If claude is not installed, we get an Llm error
        // If it is installed, we'd get a response (which is also fine)
        if let Err(CodeilusError::Llm(msg)) = &result {
            assert!(
                msg.contains("not found")
                    || msg.contains("Failed")
                    || msg.contains("billing")
                    || msg.contains("timed out"),
                "Error should be descriptive: {}",
                msg
            );
        }
        // Either way, no panic
    }
}
