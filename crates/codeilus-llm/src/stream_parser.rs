//! Parse Claude CLI stream-json output into LlmEvent values.

use crate::types::{LlmEvent, LlmResponse};

/// Parse a single line of stream-json output into an LlmEvent.
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
        "message_stop" => {
            // Signal that the message is complete.
            // The caller should use StreamAccumulator::finish() to build the response.
            None
        }
        // Ignore: message_start, content_block_start, content_block_stop, message_delta
        _ => None,
    }
}

/// Returns true if the line represents a message_stop event.
pub fn is_message_stop(line: &str) -> bool {
    let trimmed = line.trim();
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        value
            .get("type")
            .and_then(|t| t.as_str())
            == Some("message_stop")
    } else {
        false
    }
}

/// Accumulates content deltas into a complete response.
pub struct StreamAccumulator {
    text: String,
    tokens: usize,
}

impl StreamAccumulator {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            tokens: 0,
        }
    }

    /// Feed an event into the accumulator.
    pub fn feed(&mut self, event: &LlmEvent) {
        if let LlmEvent::ContentDelta(text) = event {
            self.text.push_str(text);
            // Rough token estimate: ~4 chars per token
            self.tokens = self.text.len() / 4;
        }
    }

    /// Finish accumulation and return the complete response.
    pub fn finish(self) -> LlmResponse {
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
        // message_stop returns None (it's a signal, not an event)
        assert!(parse_stream_line(line).is_none());
        // But is_message_stop should detect it
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
        let line = r#"{"type":"message_start","message":{}}"#;
        assert!(parse_stream_line(line).is_none());
    }

    #[test]
    fn parse_invalid_json() {
        let line = "not valid json {{{";
        assert!(parse_stream_line(line).is_none());
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
