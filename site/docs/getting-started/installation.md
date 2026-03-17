# Installation

## Prerequisites

- **Claude Code CLI** (optional, for AI features): `npm install -g @anthropic-ai/claude-code`

## Install via Homebrew (macOS & Linux)

```bash
brew tap mbaneshi/codeilus
brew install codeilus
```

Upgrade:
```bash
brew upgrade codeilus
```

## Install via Cargo

```bash
cargo install --git https://github.com/mbaneshi/codeilus.git codeilus-app
```

## Download Pre-built Binary

Download the latest release for your platform from [GitHub Releases](https://github.com/mbaneshi/codeilus/releases):

| Platform | Download |
|---|---|
| macOS (Apple Silicon) | `codeilus-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `codeilus-x86_64-apple-darwin.tar.gz` |
| Linux (x86_64) | `codeilus-x86_64-unknown-linux-gnu.tar.gz` |
| Linux (ARM64) | `codeilus-aarch64-unknown-linux-gnu.tar.gz` |

```bash
# Example: macOS Apple Silicon
curl -LO https://github.com/mbaneshi/codeilus/releases/latest/download/codeilus-aarch64-apple-darwin.tar.gz
tar xzf codeilus-aarch64-apple-darwin.tar.gz
sudo mv codeilus /usr/local/bin/
```

## Build from Source

```bash
git clone https://github.com/mbaneshi/codeilus.git
cd codeilus
cargo build --release
# Binary at ./target/release/codeilus
```

## Verify Installation

```bash
codeilus --version
# codeilus 0.1.0

codeilus --help
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
