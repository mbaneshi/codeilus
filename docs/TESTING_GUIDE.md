# Codeilus — Testing & Running Guide

## Prerequisites

- Rust toolchain (stable)
- Node.js 20+
- A repo to analyze (any git repo works)

---

## Step 1: Build the Frontend

```bash
cd /Users/bm/codeilus/codeilus/frontend
npm install
npm run build
```

This creates `frontend/build/` which gets embedded into the Rust binary via `rust-embed`.

## Step 2: Build the Binary

```bash
cd /Users/bm/codeilus/codeilus
cargo build --release
```

Binary will be at `target/release/codeilus`.

## Step 3: Analyze a Repo

```bash
# Analyze any local repo
./target/release/codeilus analyze /path/to/any/repo

# Or analyze codeilus itself
./target/release/codeilus analyze .
```

You should see output like:
```
Step 1/8: Parsing repository...
Step 2/8: Storing parsed data...
Step 3/8: Building knowledge graph...
Step 4/8: Computing metrics...
Step 5/8: Detecting patterns...
Step 6/8: Generating diagrams...
Step 7/8: Generating narratives...
Step 8/8: Building curriculum...
═══════════════════════════════════════════
Analysis complete!
  Files:       122
  Symbols:     614
  ...
═══════════════════════════════════════════
```

## Step 4: Start the Server

```bash
./target/release/codeilus serve
```

Or use the shorthand (analyze + serve in one command):

```bash
./target/release/codeilus /path/to/any/repo
```

Server starts at: **http://localhost:4174**

## Step 5: Open in Browser

Navigate to http://localhost:4174 and explore:

| Page | URL | What It Shows |
|------|-----|---------------|
| Home | `/` | Stats overview, server health |
| File Tree | `/explore/tree` | All parsed files, click to see symbols |
| Graph | `/explore/graph` | Force-directed knowledge graph |
| Metrics | `/explore/metrics` | SLOC, language breakdown, top files |
| Diagrams | `/explore/diagrams` | Communities + processes |
| Learn | `/learn` | Chapter cards from communities |
| Ask | `/ask` | Symbol search type-ahead |

## Step 6: Try the Search API

```bash
# Search symbols
curl "http://localhost:4174/api/v1/search?q=parse&type=symbol"

# Search files
curl "http://localhost:4174/api/v1/search?q=main&type=file"

# Search everything
curl "http://localhost:4174/api/v1/search?q=graph"
```

## Step 7: Try the API Directly

```bash
# Health check
curl http://localhost:4174/api/v1/health

# List all files
curl http://localhost:4174/api/v1/files

# List files by language
curl "http://localhost:4174/api/v1/files?language=Rust"

# Get symbols for a file (replace 1 with actual file ID)
curl http://localhost:4174/api/v1/files/1/symbols

# Get the knowledge graph
curl http://localhost:4174/api/v1/graph

# Get communities
curl http://localhost:4174/api/v1/communities

# Get processes
curl http://localhost:4174/api/v1/processes

# Search symbols by prefix
curl "http://localhost:4174/api/v1/symbols/search?q=parse"
```

---

## Useful Options

```bash
# Custom port
./target/release/codeilus serve --port 8080

# Custom DB path
CODEILUS_DB_PATH=/tmp/test.db ./target/release/codeilus analyze .

# Debug logging
RUST_LOG=debug ./target/release/codeilus analyze .

# Delete old DB to start fresh
rm ~/.codeilus/codeilus.db
```

## Troubleshooting

| Issue | Fix |
|-------|-----|
| "disk I/O error" | Delete `~/.codeilus/codeilus.db` (stale WAL files) |
| Empty pages in browser | Make sure you ran `analyze` before `serve` |
| Frontend shows 404 | Rebuild frontend: `cd frontend && npm run build` then rebuild Rust |
| Port already in use | Use `--port 8081` or kill the existing process |

---

## Running Tests

```bash
# All tests
cargo test --workspace

# Single crate
cargo test -p codeilus-parse
cargo test -p codeilus-search

# Clippy
cargo clippy --workspace -- -D warnings

# Frontend checks
cd frontend && npm run check
```

## Dev Mode (Hot Reload Frontend)

Run these in separate terminals:

```bash
# Terminal 1: Rust server
cargo run -- serve

# Terminal 2: Frontend dev server (proxies API to :4174)
cd frontend && npm run dev
```

Then open http://localhost:5173 (Vite dev server with hot reload).
