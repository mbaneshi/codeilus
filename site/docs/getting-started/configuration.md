# Configuration

## LLM Provider

Codeilus supports a provider-agnostic LLM architecture. The default provider is **Claude Code CLI**.

### Auto-Detection

On startup, Codeilus automatically detects the best available provider:

1. Check `CODEILUS_LLM_PROVIDER` env var
2. Look for `claude` CLI binary in PATH
3. Check for `ANTHROPIC_API_KEY`
4. Fall back to Claude Code CLI (errors on use if unavailable)

### Manual Override

```bash
# Force Claude Code CLI
CODEILUS_LLM_PROVIDER=claude_code codeilus ./repo

# Force Anthropic API (requires ANTHROPIC_API_KEY)
CODEILUS_LLM_PROVIDER=anthropic_api codeilus ./repo

# Skip LLM entirely
CODEILUS_SKIP_LLM=1 codeilus ./repo
```

### Settings Page

The Settings page (`/settings`) shows the current LLM provider status and availability. You can verify your setup there after starting the server.

## Database

Codeilus uses SQLite with WAL mode. The database is created automatically.

```bash
# Custom database path
CODEILUS_DB_PATH=/path/to/my.db codeilus ./repo
```

Default: `~/.codeilus/codeilus.db`

Re-analyzing a repository clears previous data automatically.

## Server

```bash
# Custom port
codeilus serve --port 8080
```

Default: `127.0.0.1:4174`

## Logging

Codeilus uses the `tracing` framework. Control log levels with `RUST_LOG`:

```bash
# Debug LLM and narrative generation
RUST_LOG=codeilus_llm=debug,codeilus_narrate=debug codeilus ./repo

# Verbose everything
RUST_LOG=debug codeilus ./repo

# Quiet mode
RUST_LOG=warn codeilus ./repo
```
