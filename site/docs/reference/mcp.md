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

## Available Tools (16)

### Query & Search

| Tool | Description |
|---|---|
| `query_symbols` | Search symbols by name, kind, or file |
| `query_graph` | Query graph edges with filters |
| `find_related_code` | Find code related to a symbol |
| `find_tests_for` | Find test files/functions for a given symbol |

### Context & Understanding

| Tool | Description |
|---|---|
| `get_context` | 360-degree symbol view (callers, callees, community, metrics) |
| `explain_symbol` | LLM-generated explanation of a specific symbol |
| `explain_file` | LLM-generated explanation of a file's role |
| `understand_codebase` | High-level codebase overview |
| `get_community_context` | Understand a community/module |

### Analysis

| Tool | Description |
|---|---|
| `get_metrics` | Code metrics for files or symbols |
| `get_impact` | Blast radius analysis with depth scoring |
| `impact_analysis` | Extended impact analysis with change propagation |
| `trace_call_chain` | Trace call paths between symbols |

### Navigation & Learning

| Tool | Description |
|---|---|
| `suggest_reading_order` | Recommended file reading order |
| `get_learning_status` | Learning progress and curriculum overview |
| `get_diagram` | Generate architecture or flowchart diagrams |

## Example Usage

Once configured, you can ask Claude Code:

- "What does the GraphBuilder do?" &rarr; uses `context`
- "What would break if I change this function?" &rarr; uses `impact`
- "Search for all database-related code" &rarr; uses `query`
- "Show me the architecture diagram" &rarr; uses `diagram`
