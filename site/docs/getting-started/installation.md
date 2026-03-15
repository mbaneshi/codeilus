# Installation

## Prerequisites

- **Rust toolchain** (1.75+): [rustup.rs](https://rustup.rs)
- **Claude Code CLI** (optional, for AI features): `npm install -g @anthropic-ai/claude-code`

## Install from Source

```bash
git clone https://github.com/codeilus/codeilus.git
cd codeilus
cargo install --path crates/codeilus-app
```

## Verify Installation

```bash
codeilus --help
```

You should see:

```
Turn any codebase into an interactive learning experience

Usage: codeilus [PATH] [COMMAND]

Commands:
  analyze   Analyze a codebase
  serve     Start the interactive server
  harvest   Scrape GitHub trending repos
  export    Export analyzed repo as static HTML
  deploy    Deploy static output to CDN
  mcp       Start MCP stdio server
  help      Print help
```

## Claude Code Setup (Optional)

Claude Code powers the AI features: narrative generation, Q&A, and diagram enhancement. Without it, all analysis, graphs, metrics, and diagrams still work.

```bash
# Install
npm install -g @anthropic-ai/claude-code

# Authenticate
claude auth

# Verify
claude --version
```

!!! tip "Claude Code subscription"
    Codeilus uses the `claude` CLI binary directly. It works with your Claude Code subscription &mdash; no separate API key needed.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `CODEILUS_DB_PATH` | `~/.codeilus/codeilus.db` | SQLite database location |
| `CODEILUS_SKIP_LLM` | `false` | Skip all LLM calls (use placeholders) |
| `CODEILUS_LLM_PROVIDER` | auto-detect | Force LLM provider: `claude_code` or `anthropic_api` |
| `CODEILUS_CLONE_DIR` | `/tmp/codeilus-clones` | Directory for harvested repo clones |
| `RUST_LOG` | `info` | Log level (`debug`, `info`, `warn`, `error`) |
