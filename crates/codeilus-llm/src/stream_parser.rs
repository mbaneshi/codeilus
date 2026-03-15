//! Parse Claude CLI stream-json output into LlmEvent values.

use crate::types::{LlmEvent, LlmResponse};

/// Parse a single line of stream-json output into an LlmEvent.
///
/// Claude CLI `--output-format stream-json` emits these event types:
/// - `{"type":"system",...}` — init info
/// - `{"type":"assistant","message":{"content":[{"type":"text","text":"..."}],...}}` — message
/// - `{"type":"result","result":"...","total_cost_usd":...}` — final result
///
/// Returns `None` for unknown/ignorable event types or invalid JSON.
pub fn parse_stream_line(line: &str) -> Option<LlmEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let value: serde_json::Value = serde_json::from_str(trimmed).ok()?;
    let event_type = value.get("type")?.as_str()?;

    match event_type {
        // Claude CLI format: assistant message with full content
        "assistant" => {
            let message = value.get("message")?;
            let content = message.get("content")?.as_array()?;
            let mut text = String::new();
            for block in content {
                if block.get("type")?.as_str()? == "text" {
                    if let Some(t) = block.get("text").and_then(|v| v.as_str()) {
                        text.push_str(t);
                    }
                }
            }
            if !text.is_empty() {
                Some(LlmEvent::ContentDelta(text))
            } else {
                None
            }
        }
        // Claude CLI format: final result with accumulated text
        "result" => {
            let result_text = value
                .get("result")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let tokens = value
                .get("usage")
                .and_then(|u| u.get("output_tokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0) as usize;
            Some(LlmEvent::Complete(LlmResponse {
                text: result_text,
                tokens_used: tokens,
            }))
        }
        // Anthropic API format (for compatibility)
        "content_block_delta" => {
            let delta = value.get("delta")?;
            let delta_type = delta.get("type")?.as_str()?;
            if delta_type == "text_delta" {
                let text = delta.get("text")?.as_str()?.to_string();
                Some(LlmEvent::ContentDelta(text))
            } else {
                None
            }
        }
        "tool_use" => {
            let name = value.get("name")?.as_str()?.to_string();
            let input = value
                .get("input")
                .map(|v| v.to_string())
                .unwrap_or_default();
            Some(LlmEvent::ToolUse { name, input })
        }
        "message_stop" => None,
        // Ignore: system, message_start, content_block_start, content_block_stop, message_delta
        _ => None,
    }
}

/// Returns true if the line represents a completion event.
pub fn is_message_stop(line: &str) -> bool {
    let trimmed = line.trim();
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        let event_type = value.get("type").and_then(|t| t.as_str());
        matches!(event_type, Some("message_stop") | Some("result"))
    } else {
        false
    }
}

/// Accumulates content deltas into a complete response.
pub struct StreamAccumulator {
    text: String,
    tokens: usize,
    /// If we get a Complete event, store it directly.
    complete: Option<LlmResponse>,
}

impl StreamAccumulator {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            tokens: 0,
            complete: None,
        }
    }

    /// Feed an event into the accumulator.
    pub fn feed(&mut self, event: &LlmEvent) {
        match event {
            LlmEvent::ContentDelta(text) => {
                self.text.push_str(text);
                self.tokens = self.text.len() / 4;
            }
            LlmEvent::Complete(response) => {
                self.complete = Some(response.clone());
            }
            _ => {}
        }
    }

    /// Finish accumulation and return the complete response.
    pub fn finish(self) -> LlmResponse {
        // Prefer the Complete event if we got one (it has the final result text)
        if let Some(complete) = self.complete {
            // Use complete.text if it's non-empty, otherwise fall back to accumulated text
            if !complete.text.is_empty() {
                return complete;
            }
        }

        let tokens = if self.tokens == 0 && !self.text.is_empty() {
            self.text.len() / 4
        } else {
            self.tokens
        };
        LlmResponse {
            text: self.text,
            tokens_used: tokens,
        }
    }
}

impl Default for StreamAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_claude_cli_assistant() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Hello world!"}],"role":"assistant"}}"#;
        let event = parse_stream_line(line);
        match event {
            Some(LlmEvent::ContentDelta(text)) => assert_eq!(text, "Hello world!"),
            other => panic!("Expected ContentDelta, got {:?}", other),
        }
    }

    #[test]
    fn parse_claude_cli_result() {
        let line = r#"{"type":"result","subtype":"success","result":"Hello!","usage":{"output_tokens":5}}"#;
        let event = parse_stream_line(line);
        match event {
            Some(LlmEvent::Complete(response)) => {
                assert_eq!(response.text, "Hello!");
                assert_eq!(response.tokens_used, 5);
            }
            other => panic!("Expected Complete, got {:?}", other),
        }
    }

    #[test]
    fn parse_content_delta() {
        let line = r#"{"type":"content_block_delta","delta":{"type":"text_delta","text":"Hello"}}"#;
        let event = parse_stream_line(line);
        match event {
            Some(LlmEvent::ContentDelta(text)) => assert_eq!(text, "Hello"),
            other => panic!("Expected ContentDelta, got {:?}", other),
        }
    }

    #[test]
    fn parse_message_stop() {
        let line = r#"{"type":"message_stop"}"#;
        assert!(parse_stream_line(line).is_none());
        assert!(is_message_stop(line));
    }

    #[test]
    fn parse_result_is_stop() {
        let line = r#"{"type":"result","result":"done"}"#;
        assert!(is_message_stop(line));
    }

    #[test]
    fn parse_tool_use() {
        let line = r#"{"type":"tool_use","name":"read_file","input":{"path":"src/main.rs"}}"#;
        let event = parse_stream_line(line);
        match event {
            Some(LlmEvent::ToolUse { name, input }) => {
                assert_eq!(name, "read_file");
                assert!(input.contains("src/main.rs"));
            }
            other => panic!("Expected ToolUse, got {:?}", other),
        }
    }

    #[test]
    fn parse_unknown_event() {
        let line = r#"{"type":"system","cwd":"/tmp"}"#;
        assert!(parse_stream_line(line).is_none());
    }

    #[test]
    fn parse_invalid_json() {
        let line = "not valid json {{{";
        assert!(parse_stream_line(line).is_none());
    }

    #[test]
    fn accumulator_with_result() {
        let mut acc = StreamAccumulator::new();
        acc.feed(&LlmEvent::ContentDelta("Hello ".to_string()));
        acc.feed(&LlmEvent::Complete(LlmResponse {
            text: "Hello world!".to_string(),
            tokens_used: 3,
        }));
        let response = acc.finish();
        assert_eq!(response.text, "Hello world!");
        assert_eq!(response.tokens_used, 3);
    }

    #[test]
    fn accumulator_basic() {
        let mut acc = StreamAccumulator::new();
        acc.feed(&LlmEvent::ContentDelta("Hello ".to_string()));
        acc.feed(&LlmEvent::ContentDelta("world".to_string()));
        acc.feed(&LlmEvent::ContentDelta("!".to_string()));
        let response = acc.finish();
        assert_eq!(response.text, "Hello world!");
        assert!(response.tokens_used > 0);
    }

    #[test]
    fn accumulator_empty() {
        let acc = StreamAccumulator::new();
        let response = acc.finish();
        assert_eq!(response.text, "");
        assert_eq!(response.tokens_used, 0);
    }
}
