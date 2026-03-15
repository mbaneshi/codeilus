# CLI Reference

## Usage

```
codeilus [PATH] [COMMAND]
```

If `PATH` is given without a subcommand, Codeilus runs `analyze` then `serve`.

## Commands

### `codeilus analyze <path>`

Run the full analysis pipeline on a repository.

```bash
codeilus analyze ./my-project
```

Steps: parse, store, graph, metrics, analyze, diagram, narrate, learn.

### `codeilus serve`

Start the interactive web server.

```bash
codeilus serve --port 4174
```

| Flag | Default | Description |
|---|---|---|
| `--port`, `-p` | `4174` | Port to listen on |

### `codeilus <path>`

Shorthand for `analyze` + `serve`:

```bash
codeilus ./my-project
# Equivalent to:
# codeilus analyze ./my-project && codeilus serve
```

### `codeilus harvest`

Scrape GitHub trending repositories.

```bash
codeilus harvest --trending --languages rust,python
```

| Flag | Description |
|---|---|
| `--trending` | Scrape trending repos |
| `--date` | Date to harvest (YYYY-MM-DD) |
| `--languages` | Filter by languages (comma-separated) |

### `codeilus export`

Export analyzed repos as static HTML.

```bash
codeilus export ./my-project --output ./output
```

| Flag | Default | Description |
|---|---|---|
| `--output`, `-o` | `./output` | Output directory |
| `--all-harvested` | | Export all harvested repos |
| `--date` | | Date for harvested repos |

### `codeilus deploy`

Deploy static output to CDN.

```bash
codeilus deploy ./output --cloudflare
```

| Flag | Description |
|---|---|
| `--cloudflare` | Deploy to Cloudflare Pages |
| `--gh-pages` | Deploy to GitHub Pages |

### `codeilus mcp`

Start the MCP stdio server for AI agent integration.

```bash
codeilus mcp
```

See [MCP Server](mcp.md) for available tools.
