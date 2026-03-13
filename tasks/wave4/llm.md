# Task: Claude Code LLM Integration

> **Crate:** `crates/codeilus-llm/`
> **Wave:** 4 (parallel with narrate, learn)
> **Depends on:** codeilus-core (done), codeilus-graph (wave 2)
> **Status:** pending

---

## Context

Read these files first:
- `CLAUDE.md` — project rules and conventions
- `NORTH_STAR.md` — section 9, Sprint 5 LLM deliverables, section 8 (Claude Code CLI subprocess choice)
- `crates/codeilus-core/src/types.rs` — NarrativeKind
- `crates/codeilus-core/src/error.rs` — CodeilusError::Llm variant
- `crates/codeilus-graph/src/types.rs` — KnowledgeGraph, Community, GraphNode
- Reference: `../forge-project/crates/forge-process/src/` — subprocess spawning, stream-json parsing, process management patterns

## Objective

Spawn Claude Code CLI as a subprocess, parse its stream-json output, and provide a clean async API for sending prompts and receiving streamed responses. Build context from graph data. Gracefully degrade if `claude` CLI is not found.

Public API:
```rust
pub async fn prompt(request: LlmRequest) -> CodeilusResult<LlmResponse>
pub async fn prompt_stream(request: LlmRequest) -> CodeilusResult<impl Stream<Item = LlmEvent>>
pub fn build_context(graph: &KnowledgeGraph, focus: ContextFocus) -> String
pub async fn is_available() -> bool
```

## Files to Create/Modify

### 1. Update `crates/codeilus-llm/Cargo.toml`

```toml
[package]
name = "codeilus-llm"
version = "0.1.0"
edition = "2021"

[dependencies]
codeilus-core = { path = "../codeilus-core" }
tokio = { workspace = true, features = ["process", "io-util"] }
tokio-stream = "0.1"
futures = "0.3"
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

### 2. `src/types.rs` — LLM types

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub prompt: String,
    pub system: Option<String>,
    pub max_tokens: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub text: String,
    pub tokens_used: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmEvent {
    /// Partial text content received
    ContentDelta(String),
    /// Tool usage by the agent (for streaming UX)
    ToolUse { name: String, input: String },
    /// Final result
    Complete(LlmResponse),
    /// Error during streaming
    Error(String),
}

/// What to focus the context on.
#[derive(Debug, Clone)]
pub enum ContextFocus {
    /// Overall repo overview
    Overview,
    /// Specific community
    Community(i64),
    /// Specific symbol and its neighbors
    Symbol(i64),
    /// Custom file paths
    Files(Vec<String>),
}
```

### 3. `src/cli.rs` — Claude CLI subprocess management

- Check if `claude` binary exists: `which claude` or `command -v claude`
- Spawn: `claude --output-format stream-json --print "<prompt>"`
- If system prompt: pass via `--system "<system>"`
- Read stdout line-by-line (each line is a JSON event)
- Parse stream-json format:
  - `{"type": "content_block_delta", "delta": {"type": "text_delta", "text": "..."}}`  → accumulate text
  - `{"type": "content_block_start", ...}` → start of content block
  - `{"type": "message_stop"}` → end of message
  - `{"type": "tool_use", "name": "...", "input": {...}}` → tool use event
- Handle subprocess errors: non-zero exit code, stderr output
- Timeout: 60 seconds default, configurable
- Reference: `../forge-project/crates/forge-process/src/` for subprocess patterns

```rust
pub struct ClaudeCli {
    timeout_secs: u64,
}

impl ClaudeCli {
    pub fn new() -> Self;
    pub fn with_timeout(timeout_secs: u64) -> Self;

    /// Check if claude CLI is available.
    pub async fn is_available(&self) -> bool;

    /// Run a prompt and return the full response.
    pub async fn prompt(&self, request: &LlmRequest) -> CodeilusResult<LlmResponse>;

    /// Run a prompt and stream events.
    pub async fn prompt_stream(&self, request: &LlmRequest) -> CodeilusResult<tokio::sync::mpsc::Receiver<LlmEvent>>;
}
```

### 4. `src/stream_parser.rs` — Stream-json event parser

```rust
/// Parse a single line of stream-json output into an LlmEvent.
pub fn parse_stream_line(line: &str) -> Option<LlmEvent> { ... }

/// Accumulate content deltas into a complete response.
pub struct StreamAccumulator {
    text: String,
    tokens: usize,
}

impl StreamAccumulator {
    pub fn new() -> Self;
    pub fn feed(&mut self, event: &LlmEvent);
    pub fn finish(self) -> LlmResponse;
}
```

Parse the actual Claude CLI stream-json format. Key event types:
- `content_block_delta` with `text_delta` → `LlmEvent::ContentDelta`
- `tool_use` → `LlmEvent::ToolUse`
- `message_stop` → trigger `LlmEvent::Complete`
- Ignore `message_start`, `content_block_start`, `content_block_stop`, `message_delta`

### 5. `src/context.rs` — Context builder

- Build context string from KnowledgeGraph for LLM prompts
- Target: **<8K tokens** (~32K chars as rough estimate)
- Context structure by focus:
  - **Overview**: top-level file list + language stats + community names + entry points
  - **Community**: community members + inter-community edges + community keywords
  - **Symbol**: symbol signature + callers + callees + containing file + community context
  - **Files**: file contents (truncated) + symbols in those files
- Truncation strategy: prioritize by fan-in score, truncate long lists with "... and N more"

```rust
use codeilus_graph::KnowledgeGraph;

pub fn build_context(graph: &KnowledgeGraph, focus: ContextFocus) -> String { ... }
```

### 6. `src/lib.rs` — Module entry point

```rust
pub mod cli;
pub mod context;
pub mod stream_parser;
pub mod types;

pub use cli::ClaudeCli;
pub use context::build_context;
pub use types::*;

use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;

/// Check if Claude Code CLI is available.
pub async fn is_available() -> bool {
    ClaudeCli::new().is_available().await
}

/// Send a prompt and get the full response.
pub async fn prompt(request: LlmRequest) -> CodeilusResult<LlmResponse> {
    ClaudeCli::new().prompt(&request).await
}
```

## Tests

### Test cases:
1. `parse_content_delta` — `{"type":"content_block_delta","delta":{"type":"text_delta","text":"Hello"}}` → `ContentDelta("Hello")`
2. `parse_message_stop` — `{"type":"message_stop"}` → triggers Complete
3. `parse_tool_use` — Tool use JSON → `ToolUse { name, input }`
4. `parse_unknown_event` — Unknown type → None (ignored)
5. `parse_invalid_json` — Malformed JSON → None (ignored)
6. `accumulator_basic` — Feed 3 ContentDeltas → finish() concatenates all text
7. `accumulator_empty` — No events → finish() returns empty text
8. `context_overview` — Build overview context → contains file count, languages, communities
9. `context_overview_truncation` — Large graph → context is <8K tokens (~32K chars)
10. `context_symbol` — Build symbol context → contains symbol name, callers, callees
11. `context_community` — Build community context → contains member names
12. `is_available_mock` — Test availability check (mock the which command or accept that this is environment-dependent)
13. `prompt_graceful_degradation` — When claude not found → returns CodeilusError::Llm with descriptive message

Note: Tests for actual CLI subprocess are integration tests that require `claude` to be installed. Unit tests should test the parser and context builder. Use `#[ignore]` for tests requiring the real CLI.

## Acceptance Criteria

- [ ] `cargo test -p codeilus-llm` — all unit tests pass
- [ ] `cargo clippy -p codeilus-llm` — zero warnings
- [ ] Stream-json parser correctly handles content_block_delta, message_stop, tool_use
- [ ] StreamAccumulator collects text from multiple deltas
- [ ] Context builder respects <8K token budget
- [ ] Context varies by focus (overview vs symbol vs community)
- [ ] `is_available()` returns false when claude CLI not found (no panic)
- [ ] `prompt()` returns `CodeilusError::Llm` with helpful message when CLI unavailable
- [ ] No hard dependency on claude being installed — all analysis works without it

## Do NOT Touch
- `crates/codeilus-core/` (read-only)
- `crates/codeilus-graph/` (wave 2)
- `crates/codeilus-parse/` (wave 1)
- Any DB files — this crate has no DB dependency
- `Cargo.toml` at workspace root
- Any files outside `crates/codeilus-llm/`

---

## Report

> **Agent: fill this section when done.**

### Status: complete

### Files Created/Modified:
- `crates/codeilus-llm/Cargo.toml` — Updated: added codeilus-graph, petgraph, tokio-stream, futures, serde, serde_json deps
- `crates/codeilus-llm/src/lib.rs` — Created: module declarations, `is_available()`, `prompt()` public API
- `crates/codeilus-llm/src/types.rs` — Created: LlmRequest, LlmResponse, LlmEvent, ContextFocus types
- `crates/codeilus-llm/src/cli.rs` — Created: ClaudeCli subprocess wrapper with is_available(), prompt(), prompt_stream()
- `crates/codeilus-llm/src/stream_parser.rs` — Created: parse_stream_line(), is_message_stop(), StreamAccumulator
- `crates/codeilus-llm/src/context.rs` — Created: build_context() with Overview/Community/Symbol/Files focus modes

### Tests:
```
running 13 tests
test stream_parser::tests::accumulator_basic ... ok
test stream_parser::tests::accumulator_empty ... ok
test context::tests::context_symbol ... ok
test context::tests::context_community ... ok
test context::tests::context_overview ... ok
test stream_parser::tests::parse_message_stop ... ok
test stream_parser::tests::parse_invalid_json ... ok
test stream_parser::tests::parse_unknown_event ... ok
test stream_parser::tests::parse_tool_use ... ok
test stream_parser::tests::parse_content_delta ... ok
test context::tests::context_overview_truncation ... ok
test cli::tests::is_available_check ... ok
test cli::tests::prompt_graceful_degradation ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Clippy:
Zero warnings.

### Issues / Blockers:
- None.

### Notes:
- `is_available()` checks for `claude` binary via `which` with `command -v` fallback. No panic if not found.
- `prompt()` returns `CodeilusError::Llm` with install instructions when CLI is unavailable.
- `prompt_stream()` returns an `mpsc::Receiver<LlmEvent>` channel for streaming events.
- Stream parser handles content_block_delta (text_delta), tool_use, and message_stop events. All other event types are silently ignored.
- Context builder respects ~32K char budget (~8K tokens). Truncation adds "... (context truncated for token budget)" suffix.
- Context `Files` focus is limited since KnowledgeGraph only has FileId (not full paths) — callers should augment with actual file content.
- The `futures` and `tokio-stream` deps are included for downstream consumers that may need `Stream` trait; the current implementation uses mpsc channels directly.
