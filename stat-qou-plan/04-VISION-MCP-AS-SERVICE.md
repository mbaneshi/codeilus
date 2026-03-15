# Codeilus as Downstream MCP Server — Vision

## The Big Idea

Codeilus becomes a **codebase intelligence service** that any AI application can query via MCP.

```
┌─────────────────────────────────────────────┐
│              UPSTREAM CLIENTS               │
│                                             │
│   Claude Code    Cursor    Windsurf         │
│   Claude Cowork  Cline     ADK Apps         │
│   Antigravity    Custom    Any MCP Client   │
│                                             │
└──────────────┬──────────────────────────────┘
               │ MCP Protocol (stdio / SSE / HTTP)
               ▼
┌─────────────────────────────────────────────┐
│          CODEILUS MCP SERVER                 │
│                                             │
│  Tools:                                     │
│  ├── understand_codebase     (overview)     │
│  ├── explain_symbol          (deep dive)    │
│  ├── find_entry_points       (where to start)│
│  ├── trace_call_chain        (A calls B...) │
│  ├── impact_analysis         (what breaks?) │
│  ├── find_related_code       (graph walk)   │
│  ├── get_architecture        (diagram)      │
│  ├── suggest_reading_order   (learn path)   │
│  ├── search_code             (BM25 + graph) │
│  ├── get_community_context   (module view)  │
│  ├── detect_patterns         (anti-patterns)│
│  ├── get_metrics             (complexity)   │
│  ├── explain_file            (file purpose) │
│  ├── find_tests_for          (test coverage)│
│  └── suggest_changes         (refactoring)  │
│                                             │
│  Resources:                                 │
│  ├── codeilus://graph         (full graph)  │
│  ├── codeilus://communities   (clusters)    │
│  ├── codeilus://metrics       (all metrics) │
│  └── codeilus://narrative/:id (cached text) │
│                                             │
│  Prompts:                                   │
│  ├── review_pr               (context-aware)│
│  ├── onboard_developer       (guided tour)  │
│  └── plan_feature            (impact-aware) │
│                                             │
└──────────────┬──────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────┐
│          CODEILUS ENGINE                     │
│                                             │
│  Parse → Graph → Metrics → Analyze          │
│  → Narrate → Learn → Search                 │
│  All backed by SQLite + petgraph in-memory  │
│                                             │
└─────────────────────────────────────────────┘
```

## Why This Matters

### For AI Coding Assistants

Today, Claude Code/Cursor read files and grep for context. They don't understand:
- **Architecture** — which modules exist and how they connect
- **Impact** — "if I change this function, what tests break?"
- **Reading order** — "which 3 files should I read first?"
- **Patterns** — "this codebase uses the repository pattern"
- **Communities** — "this is the auth module, it connects to the API layer here"

Codeilus gives them this intelligence as structured data, not raw file contents.

### For ADK / Agent Frameworks

Google ADK, LangGraph, CrewAI, etc. can use codeilus as a tool:
```python
# ADK example
from google.adk import Agent

agent = Agent(
    tools=[
        McpTool("codeilus", transport="stdio", command="codeilus mcp")
    ]
)

# Agent can now call:
# codeilus.understand_codebase() → structured overview
# codeilus.impact_analysis("GraphBuilder::build") → blast radius
# codeilus.trace_call_chain("main") → execution flow
```

### For Claude Cowork / Multi-Agent Systems

A team of AI agents working on a codebase can share one codeilus instance:
- **Architect agent** calls `get_architecture` and `get_community_context`
- **Code review agent** calls `impact_analysis` and `detect_patterns`
- **Onboarding agent** calls `suggest_reading_order` and `explain_symbol`
- **Testing agent** calls `find_tests_for` and `detect_patterns`

---

## Transport Modes

### Mode 1: stdio (Current)
```bash
codeilus mcp
# AI tool spawns this as subprocess, communicates via stdin/stdout
```
Best for: local development, single-user, Claude Code / Cursor integration.

### Mode 2: SSE (Server-Sent Events)
```bash
codeilus serve --mcp-sse --port 3001
# AI tools connect via HTTP SSE
```
Best for: shared team server, multiple AI clients connecting to one analysis.

### Mode 3: HTTP Streamable (Modern MCP)
```bash
codeilus serve --mcp-http --port 3001
# Full HTTP-based MCP with streaming
```
Best for: cloud deployment, API gateway integration, production use.

---

## Tool Design Principles

### 1. Return Structured Data, Not Prose

Bad: `"The function process_data calls validate_input and then save_to_db"`
Good:
```json
{
  "symbol": "process_data",
  "calls": [
    {"name": "validate_input", "file": "src/validation.rs", "confidence": 0.95},
    {"name": "save_to_db", "file": "src/db.rs", "confidence": 0.8}
  ],
  "called_by": [...],
  "community": "data_processing",
  "complexity": 4.2
}
```

### 2. Layer Information Depth

Each tool should accept a `depth` parameter:
- `shallow` — name, file, one-liner (for listing)
- `medium` — + connections, metrics, community (for understanding)
- `deep` — + source code, narrative, full graph walk (for deep dive)

### 3. Context-Aware Responses

Tools should accept optional context (current file, current function, recent changes) to tailor responses:
```json
{
  "tool": "find_related_code",
  "input": {
    "symbol": "EventBus",
    "context": {
      "current_file": "src/api/routes.rs",
      "task": "adding a new WebSocket endpoint"
    }
  }
}
```

---

## Implementation Roadmap

### Phase 1: Expand Current MCP Tools (Now)
- Upgrade 8 existing tools with structured JSON output
- Add `depth` parameter to all tools
- Add `impact_analysis`, `trace_call_chain`, `find_related_code`
- Test with Claude Code: `codeilus setup` auto-configures MCP

### Phase 2: Add SSE Transport (Next)
- Axum SSE endpoint at `/mcp/sse`
- Session management for concurrent clients
- Share analysis state across connections

### Phase 3: Rich Resources & Prompts
- Expose graph, communities, metrics as MCP Resources
- Add MCP Prompts for common workflows (PR review, onboarding, planning)
- Sampling support for LLM-powered answers

### Phase 4: Cloud-Ready
- HTTP Streamable transport
- Multi-repo support (analyze multiple repos, query across them)
- Authentication / API keys for shared deployments
- Caching layer for expensive graph operations

---

## Competitive Position as MCP Server

| Feature | Codeilus MCP | Raw File Access | Repomix |
|---------|-------------|-----------------|---------|
| Structured symbols | Yes | No | No |
| Call graph | Yes | No | No |
| Community detection | Yes | No | No |
| Impact analysis | Yes | No | No |
| Architecture diagrams | Yes | No | No |
| Reading order | Yes | No | No |
| Anti-pattern detection | Yes | No | No |
| Metrics | Yes | No | No |
| Search (BM25 + graph) | Yes | Basic grep | No |
| Multi-language | 6 languages | Any | Any |
| Single binary | Yes | N/A | Node.js |

The key insight: **AI tools don't need more files — they need understanding.**
Codeilus transforms raw source into structured intelligence that AI agents can reason about.
