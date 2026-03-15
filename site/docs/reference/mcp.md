# MCP Server

Codeilus includes an MCP (Model Context Protocol) server for AI agent integration.

## Start the Server

```bash
codeilus mcp
```

This starts an MCP server on stdio, compatible with Claude Code, Cursor, and other MCP clients.

## Configuration

Add to your Claude Code MCP config (`~/.claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "codeilus": {
      "command": "codeilus",
      "args": ["mcp"]
    }
  }
}
```

## Available Tools

| Tool | Description |
|---|---|
| `list_repos` | List analyzed repositories |
| `current_repo` | Get the currently analyzed repository |
| `query` | BM25/RRF search across files, symbols, narratives |
| `context` | 360-degree symbol view (callers, callees, community, metrics) |
| `impact` | Blast radius analysis with depth scoring |
| `detect_changes` | Map diffs to changed symbols and affected processes |
| `rename` | Graph-aware multi-file rename with dry-run support |
| `diagram` | Generate architecture or flowchart diagrams |
| `learn_status` | Get learning progress and curriculum overview |

## Example Usage

Once configured, you can ask Claude Code:

- "What does the GraphBuilder do?" &rarr; uses `context`
- "What would break if I change this function?" &rarr; uses `impact`
- "Search for all database-related code" &rarr; uses `query`
- "Show me the architecture diagram" &rarr; uses `diagram`
