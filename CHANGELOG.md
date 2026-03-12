# Changelog

All notable changes to Codeilus are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Sprint 0: Foundation
#### Added
- 16-crate Rust workspace with shared dependencies
- `codeilus-core`: EventBus (tokio broadcast), 18 event types, 12 error variants, 5 typed ID wrappers, Language/SymbolKind/EdgeKind enums
- `codeilus-db`: DbPool with SQLite WAL mode, Migrator, BatchWriter (crossbeam, 50-event/2s flush)
- `codeilus-api`: Axum HTTP server with CORS, WebSocket event streaming, rust-embed SPA fallback
- `codeilus-app`: clap CLI with analyze/serve/harvest/export/deploy/mcp subcommands
- `codeilus-parse`: File walker (gitignore-aware), language detection, basic heuristic parsers for 6 languages
- `migrations/0001_init.sql`: 20-table schema (files, symbols, edges, communities, metrics, learning, harvest, events)
- 12 stub crates ready for Sprint 1+

#### Known Issues
- Parsers are heuristic (line-by-line regex), not tree-sitter — no call/heritage extraction
- DB is write-only (no SELECT queries, no read-side repos)
- Test suite has compilation error (EventBus::new signature mismatch)
- 8 clippy warnings in codeilus-parse
- No frontend exists yet
