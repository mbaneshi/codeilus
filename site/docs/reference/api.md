# API Reference

All endpoints are under `/api/v1`.

## System

| Method | Endpoint | Description |
|---|---|---|
| GET | `/health` | Health check (`{ "status": "ok" }`) |
| GET | `/llm/status` | LLM provider status (`{ "available": bool, "provider": string }`) |

## Files

| Method | Endpoint | Description |
|---|---|---|
| GET | `/files` | List all files. Optional `?language=rust` filter |
| GET | `/files/:id` | Get file by ID |
| GET | `/files/:id/symbols` | List symbols in a file |
| GET | `/files/:id/source` | Get file source. Optional `?start=1&end=50` |

## Symbols

| Method | Endpoint | Description |
|---|---|---|
| GET | `/symbols` | List all symbols. Optional `?kind=Function` filter |
| GET | `/symbols/:id` | Get symbol by ID |
| GET | `/symbols/search?q=query` | Search symbols by name prefix |

## Graph

| Method | Endpoint | Description |
|---|---|---|
| GET | `/graph` | Paginated graph. Optional `?community_id=X&limit=500&offset=0` |
| GET | `/communities` | List communities with members |
| GET | `/processes` | List execution flows with steps |

## Narratives

| Method | Endpoint | Description |
|---|---|---|
| GET | `/narratives/:kind` | Get narrative by kind (overview, architecture, etc.) |
| GET | `/narratives/:kind/:target_id` | Get narrative for a specific target |

## Learning

| Method | Endpoint | Description |
|---|---|---|
| GET | `/chapters` | List all chapters with sections |
| GET | `/chapters/:id` | Get chapter detail with narrative |
| GET | `/chapters/:id/quiz` | Get quiz questions for a chapter |
| POST | `/quiz/:id/answer` | Submit quiz answer (`{ "answer": "..." }`) |
| GET | `/progress` | Get section completion progress |
| POST | `/chapters/:cid/sections/:sid/complete` | Mark section as complete |
| GET | `/learner/stats` | Get XP, streak, badges, chapters completed |

## Annotations

| Method | Endpoint | Description |
|---|---|---|
| GET | `/annotations` | List all annotations. Optional `?flagged=true` filter |
| GET | `/annotations/:target_type/:target_id` | List annotations for a specific node/edge |
| POST | `/annotations` | Create annotation. Body: `{ "target_type": "node", "target_id": 42, "content": "..." }` |
| PUT | `/annotations/:id` | Update annotation content. Body: `{ "content": "..." }` |
| POST | `/annotations/:id/flag` | Toggle flagged status |
| DELETE | `/annotations/:id` | Delete annotation |

## Search

| Method | Endpoint | Description |
|---|---|---|
| GET | `/search?q=query` | Unified BM25 search across files, symbols, narratives |

## Ask

| Method | Endpoint | Description |
|---|---|---|
| POST | `/ask` | Streaming Q&A via SSE. Body: `{ "question": "...", "context_symbol_ids": [] }` |

### SSE Events

```
event: delta
data: {"type": "delta", "content": "partial response text"}

event: done
data: {"type": "done", "content": "42 tokens"}

event: error
data: {"type": "error", "content": "error message"}
```
