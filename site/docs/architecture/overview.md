# Architecture Overview

## Single Binary, Everything Embedded

```
codeilus (single binary, ~15MB)
+-- Rust backend (Axum HTTP + WebSocket)
+-- SQLite database (WAL mode, zero config)
+-- SvelteKit 5 frontend (embedded via rust-embed)
+-- Claude Code integration (subprocess, stream-json)
```

No Docker. No PostgreSQL. No Redis. No Node.js runtime.

## Technology Choices

| Decision | Choice | Why |
|---|---|---|
| Language | Rust | Single binary, no runtime, memory safe, fast parsing |
| HTTP server | Axum 0.7 | Tokio-native, tower middleware, WebSocket built-in |
| Database | SQLite (rusqlite, WAL) | Zero config, single file, adequate for single-repo analysis |
| Frontend | SvelteKit 5 + adapter-static | Compiled to static files, embedded via rust-embed |
| CSS | TailwindCSS 4 | Utility-first, small bundle, dark mode built-in |
| Parsing | tree-sitter | Battle-tested, incremental, 12+ language grammars |
| CPU parallelism | rayon | Tree-sitter parsing is CPU-bound |
| Graph algorithms | petgraph | In-memory Louvain, BFS, cycle detection |
| LLM | Claude Code CLI subprocess | Provider-agnostic trait, stream-json output |
| Static export | Vanilla HTML/JS | Self-contained, no framework runtime, ~300KB |
| MCP server | rmcp (Rust MCP SDK) | Official SDK, stdio transport |
| IDs | i64 newtype wrappers | Graph-heavy workload, faster joins than UUID |

## LLM Architecture

Codeilus uses a provider-agnostic `LlmProvider` trait:

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn is_available(&self) -> bool;
    async fn prompt(&self, request: &LlmRequest) -> Result<LlmResponse>;
    async fn prompt_stream(&self, request: &LlmRequest)
        -> Result<Receiver<LlmEvent>>;
}
```

Current implementations:

- **`ClaudeCli`** &mdash; spawns the `claude` binary (default, uses subscription)
- **`AnthropicApiStub`** &mdash; placeholder for direct API integration

Provider is auto-detected at startup or overridden via `CODEILUS_LLM_PROVIDER`.

## Database Schema

20 tables across 7 domains:

```
Core:       files, symbols, edges
Graph:      communities, community_members, processes, process_steps
Metrics:    file_metrics, patterns
Narratives: narratives
Learning:   chapters, chapter_sections, progress,
            quiz_questions, quiz_attempts, badges, learner_stats
Harvest:    harvested_repos
System:     events, schema_version
```

Key design: i64 IDs (SQLite rowid) for fast graph joins, WAL mode for concurrent reads, BatchWriter for event persistence.
