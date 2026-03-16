# Codeilus

**Turn any codebase into an interactive learning experience.**

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![CI](https://img.shields.io/github/actions/workflow/status/codeilus/codeilus/ci.yml?branch=main&label=CI)](https://github.com/codeilus/codeilus/actions)

Codeilus is a single Rust binary that analyzes any codebase and transforms it
into a gamified, browser-based learning experience -- complete with a 3D graph
explorer, guided chapters, AI-powered Q&A, and auto-generated quizzes.

<!-- TODO: Add screenshot / demo GIF -->

---

## Quick Start

```bash
cargo install --path crates/codeilus-app
codeilus analyze ./your-repo
codeilus serve
```

Then open [http://localhost:3000](http://localhost:3000) in your browser.

## Key Features

- **8-step analysis pipeline** -- parse, graph, metrics, analyze, narrate, learn, harvest, export
- **16 focused Rust crates** -- each with a single responsibility and clean dependency boundaries
- **3D graph visualization** -- explore files, symbols, and dependencies interactively
- **AI-generated chapters** -- pedagogically ordered explanations of how the codebase works
- **Quizzes and gamification** -- test understanding with auto-generated questions
- **Claude Code CLI integration** -- use as an MCP tool from Claude Code
- **Incremental parsing** -- re-analyze only what changed
- **SQLite storage** -- portable, zero-config persistence

## Architecture

Codeilus is organized as a Cargo workspace of 16 crates:

```
codeilus-core       Contract types, IDs, traits (zero internal deps)
codeilus-db         SQLite repositories (depends only on core)
codeilus-parse      Tree-sitter incremental parsing
codeilus-graph      Dependency graph construction
codeilus-metrics    Complexity, churn, coupling metrics
codeilus-analyze    Orchestrates parse + graph + metrics
codeilus-narrate    LLM-powered chapter generation
codeilus-learn      Learning path and quiz generation
codeilus-harvest    Multi-repo aggregation
codeilus-export     Static site and PDF export
codeilus-llm        LLM provider abstraction (Claude, OpenAI, Ollama)
codeilus-diagram    Architecture diagram generation
codeilus-mcp        Model Context Protocol server
codeilus-api        Axum HTTP/WebSocket API
codeilus-app        CLI entry point
codeilus-event      Event bus (tokio broadcast)
```

The frontend is a SvelteKit 5 application under `frontend/`.

Data flows through: **parse** -> **graph** -> **metrics** -> **analyze** -> **narrate** -> **learn** -> **export**.

## Tech Stack

| Layer     | Technology              |
|-----------|-------------------------|
| Backend   | Rust, Tokio, Axum       |
| Frontend  | SvelteKit 5, TypeScript |
| Database  | SQLite (via sqlx)       |
| Parsing   | tree-sitter             |
| AI        | Claude, OpenAI, Ollama  |
| CLI       | clap                    |

## Building from Source

```bash
# Build all crates
cargo build

# Run all tests
cargo test

# Lint (must be zero warnings)
cargo clippy

# Test a single crate
cargo test -p codeilus-parse
```

## Documentation

- [NORTH_STAR.md](NORTH_STAR.md) -- Vision, architecture, and roadmap
- [CLAUDE.md](CLAUDE.md) -- Agent instructions and architecture rules
- [docs/](docs/) -- Additional documentation and agent prompts

## Contributing

Contributions are welcome. Please read the documentation in `docs/` before
submitting a pull request. Run `cargo clippy` and `cargo test` to ensure zero
warnings and passing tests.

## License

[MIT](LICENSE)
