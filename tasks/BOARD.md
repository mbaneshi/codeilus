# Codeilus — Task Board

> Each task is a self-contained markdown file. Agents read their task, do the work, update the report section.

## How to Launch an Agent

Give each Cursor agent this one-liner (replace the path):

```
Read and execute the task in tasks/wave1/parse.md — follow every instruction exactly, update the Report section when done.
```

## Status

### Wave 1 (Sprint 1: Parse + DB + Frontend) — run all 3 in parallel
| Task | File | Agent | Status |
|---|---|---|---|
| Tree-sitter parsing engine | `tasks/wave1/parse.md` | — | pending |
| DB repositories (file, symbol, edge) | `tasks/wave1/db-repos.md` | — | pending |
| Frontend skeleton (SvelteKit 5) | `tasks/wave1/frontend.md` | — | pending |

### Wave 2 (Sprint 2: Graph + API) — after Wave 1 complete
| Task | File | Agent | Status |
|---|---|---|---|
| Knowledge graph builder | `tasks/wave2/graph.md` | — | pending |
| API routes (files, symbols, graph) | `tasks/wave2/api-routes.md` | — | pending |

### Wave 3 (Sprint 3+4: Metrics + Analysis + Diagrams) — after Wave 2
| Task | File | Agent | Status |
|---|---|---|---|
| Code metrics engine | `tasks/wave3/metrics.md` | — | pending |
| Anti-pattern analyzer | `tasks/wave3/analyze.md` | — | pending |
| Diagram generator | `tasks/wave3/diagram.md` | — | pending |

### Wave 4 (Sprint 5+6: LLM + Narrate + Learn) — after Wave 3
| Task | File | Agent | Status |
|---|---|---|---|
| Claude Code LLM integration | `tasks/wave4/llm.md` | — | pending |
| Narrative generator | `tasks/wave4/narrate.md` | — | pending |
| Learning engine | `tasks/wave4/learn.md` | — | pending |

### Wave 5 (Sprint 7: Harvest + Export) — after Wave 4
| Task | File | Agent | Status |
|---|---|---|---|
| GitHub trending harvester | `tasks/wave5/harvest.md` | — | pending |
| Static HTML exporter | `tasks/wave5/export.md` | — | pending |

### Wave 6 (Sprint 8: MCP + Wiring) — after Wave 5
| Task | File | Agent | Status |
|---|---|---|---|
| MCP server + pipeline wiring | `tasks/wave6/mcp-wiring.md` | — | pending |

## Rules
1. Agents within the same wave run in parallel — they own separate files
2. Wait for all agents in a wave to report "done" before starting next wave
3. Between waves: run `cargo build && cargo test && cargo clippy` to verify
4. If an agent reports blockers, resolve before continuing
