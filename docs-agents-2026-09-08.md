# Codeilus — Multi-Agent Build Strategy

> How to use parallel Claude Code agents to build Codeilus professionally.
>
> **Related docs:**
> - `codeilus/docs/AGENT_PROMPTS.md` — Copy-paste prompts for each agent (6 waves)
> - `codeilus/CLAUDE.md` — Shared context every agent reads first
> - `codeilus/sprints/sprint-N.md` — Per-sprint tracking files
> - `codeilus/NORTH_STAR.md` — Full vision and architecture

---

## 1. The Reality: How Many Agents?

### Practical Limits

| Constraint | Limit | Why |
|-----------|-------|-----|
| Claude Code parallel agents | **5-7 concurrent** | Beyond this, context quality degrades and merge conflicts multiply |
| Git worktree isolation | **1 worktree per agent** | Each agent works on an isolated branch, merged via PR |
| Crate independence | **12 stub crates** = 12 potential parallel tracks | But dependencies between them limit true parallelism |
| Human review bandwidth | **3-4 PRs/day** | You need to review and merge — agents can't self-merge |

### Sweet Spot: 3-5 Agents Per Sprint

Not "as many as possible" — but **the right agents at the right time**, based on the dependency graph.

---

## 2. Agent Types

### Type A: Crate Builder
- Builds one crate from stub to complete
- Works in isolated git worktree
- Has clear input contract (types from `core`) and output contract (public API)
- Runs `cargo test -p <crate>` and `cargo clippy` before finishing
- Produces a PR with tests

### Type B: Frontend Builder
- Builds Svelte pages/components
- Works in `frontend/` directory only
- No Rust conflicts possible
- Can run in parallel with ANY crate builder

### Type C: Integration Agent
- Wires crates together (API routes, CLI commands, AppState)
- Runs AFTER crate builders finish
- Works on `codeilus-api` and `codeilus-app`
- Runs full `cargo test` and E2E smoke tests

### Type D: Research Agent
- Reads reference repos, extracts patterns
- Produces documentation (prompts, schemas, algorithms)
- No code changes — pure research
- Can run in parallel with everything

---

## 3. Sprint-by-Sprint Agent Allocation

### Sprint 0: Foundation [DONE]
Already complete. 4 crates implemented, 12 stubs.

---

### Sprint 1: Parsing Engine (Week 2)

```
┌──────────────────────────────────────────────────────────────┐
│  SPRINT 1 — 3 agents, 1 sequential                          │
│                                                              │
│  ┌─ Agent 1A: codeilus-parse ─────────────────────────────┐  │
│  │  Build: tree-sitter integration, 6 language grammars,  │  │
│  │  filesystem walker, chunked parsing, import resolution │  │
│  │  Worktree: sprint1/parse                               │  │
│  │  Port from: GitNexus src/core/ingestion/               │  │
│  │  Tests: parse Python/TS/Rust fixture repos             │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌─ Agent 1B: codeilus-db repos ──────────────────────────┐  │
│  │  Build: FileRepo, SymbolRepo with batch inserts        │  │
│  │  Worktree: sprint1/db-repos                            │  │
│  │  Tests: insert/query fixtures                          │  │
│  │  CAN RUN IN PARALLEL with 1A (independent files)       │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌─ Agent 1C: Frontend file tree ─────────────────────────┐  │
│  │  Build: SvelteKit skeleton + /explore/tree page        │  │
│  │  Worktree: sprint1/frontend                            │  │
│  │  CAN RUN IN PARALLEL (separate directory)              │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ── THEN (sequential, after 1A + 1B merge) ──                │
│                                                              │
│  ┌─ Agent 1D: Integration ────────────────────────────────┐  │
│  │  Wire: API routes (GET /files, /symbols),              │  │
│  │        CLI analyze command, EventBus progress           │  │
│  │  Tests: E2E analyze fixture repo → query API           │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

**Parallelism: 3 agents → then 1 sequential**

---

### Sprint 2: Knowledge Graph (Week 3)

```
┌──────────────────────────────────────────────────────────────┐
│  SPRINT 2 — 3 parallel + 1 sequential                       │
│                                                              │
│  ┌─ Agent 2A: codeilus-graph ─────────────────────────────┐  │
│  │  Build: call tracing, heritage edges, dependency edges,│  │
│  │  Louvain community detection, process detection        │  │
│  │  Port from: GitNexus ingestion + emerge graph.py       │  │
│  │  Worktree: sprint2/graph                               │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌─ Agent 2B: codeilus-db graph repos ────────────────────┐  │
│  │  Build: EdgeRepo, CommunityRepo, ProcessRepo           │  │
│  │  Worktree: sprint2/db-repos                            │  │
│  │  PARALLEL with 2A                                      │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌─ Agent 2C: Frontend graph explorer ────────────────────┐  │
│  │  Build: /explore/graph page, D3 force-directed graph   │  │
│  │  Worktree: sprint2/frontend                            │  │
│  │  PARALLEL (separate directory)                         │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  ── THEN ──                                                  │
│                                                              │
│  ┌─ Agent 2D: Integration ────────────────────────────────┐  │
│  │  Wire: API routes, graph build in analyze pipeline     │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

---

### Sprint 3: Metrics & Analysis (Week 4)

```
┌──────────────────────────────────────────────────────────────┐
│  SPRINT 3 — 4 parallel + 1 sequential                       │
│                                                              │
│  Agent 3A: codeilus-metrics    (SLOC, fan-in/out, git churn) │
│  Agent 3B: codeilus-analyze    (god classes, circular deps)  │
│  Agent 3C: codeilus-db repos   (MetricsRepo, PatternRepo)   │
│  Agent 3D: Frontend metrics    (/explore/metrics, heatmap)  │
│  ── ALL PARALLEL ──                                          │
│  Agent 3E: Integration         (API routes, pipeline wiring) │
└──────────────────────────────────────────────────────────────┘
```

**Parallelism: 4 agents → then 1**

---

### Sprint 4: Diagrams (Week 5)

```
┌──────────────────────────────────────────────────────────────┐
│  SPRINT 4 — 3 parallel + 1 sequential                       │
│                                                              │
│  Agent 4A: codeilus-diagram    (FlowchartIR, Mermaid gen)   │
│  Agent 4B: Research agent      (study CodeVisualizer IR,     │
│            gitdiagram prompts, extract reusable patterns)    │
│  Agent 4C: Frontend diagrams   (/explore/architecture,       │
│            /explore/flowchart, Mermaid renderer component)   │
│  ── ALL PARALLEL ──                                          │
│  Agent 4D: Integration         (API routes, LLM auto-fix)   │
└──────────────────────────────────────────────────────────────┘
```

---

### Sprint 5: LLM Integration (Week 6)

```
┌──────────────────────────────────────────────────────────────┐
│  SPRINT 5 — 4 parallel + 1 sequential                       │
│                                                              │
│  Agent 5A: codeilus-llm        (Claude CLI spawn, stream     │
│            parsing — port from forge-process)                 │
│  Agent 5B: codeilus-narrate    (8 narrative prompts, context  │
│            builder, caching in narratives table)              │
│  Agent 5C: Research agent      (study PocketFlow prompts,     │
│            deep-research streaming UX, extract patterns)      │
│  Agent 5D: Frontend Q&A        (/ask chat page, explain       │
│            popover, streaming UX, relevant files sidebar)     │
│  ── ALL PARALLEL ──                                           │
│  Agent 5E: Integration          (API streaming routes,        │
│            EventBus LLM events, analyze pipeline wiring)      │
└──────────────────────────────────────────────────────────────┘
```

---

### Sprint 6: Learning Engine (Weeks 7–8)

```
┌──────────────────────────────────────────────────────────────┐
│  SPRINT 6 — 5 parallel + 1 sequential (2-week sprint)       │
│                                                              │
│  Agent 6A: Curriculum generator (topo-sort + LLM ordering,  │
│            community naming, chapter structure)               │
│  Agent 6B: Quiz generator       (MCQ, T/F, impact, free-    │
│            form, adaptive difficulty)                         │
│  Agent 6C: Gamification engine  (XP, badges, streaks,        │
│            progress tracking)                                 │
│  Agent 6D: codeilus-db repos    (ChapterRepo, ProgressRepo,  │
│            QuizRepo, BadgeRepo, StatsRepo)                    │
│  Agent 6E: Frontend learning    (/learn grid, /learn/[ch],   │
│            progress bars, XP counter, badge shelf, quiz)      │
│  ── ALL PARALLEL ──                                           │
│  Agent 6F: Integration          (API routes, full pipeline)   │
└──────────────────────────────────────────────────────────────┘
```

**Peak parallelism: 5 agents** — this is the biggest sprint.

---

### Sprint 7: Search & Impact (Week 9)

```
┌──────────────────────────────────────────────────────────────┐
│  SPRINT 7 — 3 parallel + 1 sequential                       │
│                                                              │
│  Agent 7A: codeilus-search     (FTS5, BM25, RRF ranking)    │
│  Agent 7B: Impact analysis     (blast radius, what-if,       │
│            reverse BFS, depth scoring — in codeilus-graph)    │
│  Agent 7C: Frontend search     (/search, /impact pages)      │
│  ── ALL PARALLEL ──                                           │
│  Agent 7D: Integration          (API routes, FTS5 indexing)  │
└──────────────────────────────────────────────────────────────┘
```

---

### Sprint 8: MCP, Export & Polish (Week 10)

```
┌──────────────────────────────────────────────────────────────┐
│  SPRINT 8 — 5 parallel                                       │
│                                                              │
│  Agent 8A: codeilus-mcp        (rmcp stdio, 7 tools)        │
│  Agent 8B: codeilus-export     (static HTML + markdown)      │
│  Agent 8C: codeilus-harvest    (GitHub trending scraper)     │
│  Agent 8D: Frontend polish     (responsive, theme, shortcuts)│
│  Agent 8E: CI/CD              (GitHub Actions, release bins) │
│  ── ALL PARALLEL (no integration needed, independent) ──     │
└──────────────────────────────────────────────────────────────┘
```

---

## 4. Coordination System

### 4.1 Branch Strategy

```
main
├── sprint1/parse          ← Agent 1A worktree
├── sprint1/db-repos       ← Agent 1B worktree
├── sprint1/frontend       ← Agent 1C worktree
├── sprint1/integration    ← Agent 1D (after merge of 1A+1B)
├── sprint2/graph          ← Agent 2A worktree
├── sprint2/db-repos       ← ...
└── ...
```

**Rules:**
- Each agent works in its own git worktree (isolated copy of repo)
- Agents NEVER touch files outside their assigned crate/directory
- Integration agent runs on main AFTER parallel agents merge
- PRs are the merge mechanism — human reviews before merge

### 4.2 File Ownership (No Conflicts)

This is the key to parallel agents. Each agent owns specific files:

| Agent Type | Owns | Never Touches |
|-----------|------|--------------|
| `parse` agent | `crates/codeilus-parse/src/**` | Any other crate |
| `graph` agent | `crates/codeilus-graph/src/**` | Any other crate |
| `metrics` agent | `crates/codeilus-metrics/src/**` | Any other crate |
| `db-repos` agent | `crates/codeilus-db/src/repos/**` | pool.rs, batch_writer.rs, migrations.rs |
| `frontend` agent | `frontend/src/**` | Any Rust code |
| `integration` agent | `crates/codeilus-api/src/routes/**`, `crates/codeilus-app/src/main.rs` | Domain crates |

**Shared files (conflict risk):**

| File | Who Can Edit | Resolution |
|------|-------------|-----------|
| `Cargo.toml` (workspace) | Integration agent only | Others request dep additions in PR description |
| `crates/codeilus-core/src/types.rs` | FROZEN during sprint | New types proposed in PR, added by integration agent |
| `crates/codeilus-db/src/repos/mod.rs` | db-repos agent only | Exports new repo modules |
| `migrations/0001_init.sql` | FROZEN | New migrations get new files (0002, 0003...) |

### 4.3 Contract-First Development

Before launching parallel agents, define the contract:

```rust
// This goes in codeilus-core BEFORE agents start.
// Agents code against these types, not against each other.

// Sprint 1 contract:
pub struct ParsedFile {
    pub path: String,
    pub language: Language,
    pub symbols: Vec<Symbol>,
    pub imports: Vec<Import>,
    pub calls: Vec<Call>,
    pub heritage: Vec<Heritage>,
}

// Sprint 2 contract:
pub struct GraphResult {
    pub edges: Vec<Edge>,
    pub communities: Vec<Community>,
    pub processes: Vec<Process>,
}
```

**Pattern:** Define types in `core` → agents implement against those types → integration agent wires them together.

### 4.4 Agent Prompt Template

Every agent gets launched with a structured prompt:

```markdown
## Agent Assignment: [Sprint X] [Crate Name]

### Your Scope
- Crate: `crates/codeilus-[name]/`
- Branch: `sprintX/[name]`
- You may ONLY modify files under your crate directory

### Input Contract
[Types from codeilus-core that you consume]

### Output Contract
[Public API your crate must expose]

### Reference Code
[Specific files from reference repos to study]

### Acceptance Criteria
[Checklist from PLAN.md for this agent's deliverable]

### Tests Required
[Specific test cases that must pass]

### Dependencies
[What workspace deps you may add — request in PR if new ones needed]
```

---

## 5. Documentation & Tracking

### 5.1 Sprint Tracking File

Each sprint gets a tracking file:

```
codeilus/
├── sprints/
│   ├── sprint-1.md      # tracking for current sprint
│   ├── sprint-2.md
│   └── ...
```

**Format:**

```markdown
# Sprint 1: Parsing Engine

**Status:** In Progress
**Week:** 2 (2026-03-16 to 2026-03-22)

## Agents

| Agent | Crate | Branch | Status | PR |
|-------|-------|--------|--------|-----|
| 1A | codeilus-parse | sprint1/parse | In Progress | — |
| 1B | codeilus-db repos | sprint1/db-repos | In Progress | — |
| 1C | Frontend tree | sprint1/frontend | In Progress | — |
| 1D | Integration | sprint1/integration | Blocked on 1A+1B | — |

## Contracts Defined
- [x] ParsedFile struct in core
- [x] FileRepo trait in db
- [x] SymbolRepo trait in db

## Acceptance Criteria
- [ ] Analyze 10K-line Python repo in < 5 seconds
- [ ] All functions, classes, imports extracted for 6 languages
- [ ] File tree page shows hierarchy with symbol counts
- [ ] Progress events stream to WebSocket

## Blockers
(none yet)

## Decisions Made
(log decisions here as they arise)
```

### 5.2 Agent Handoff Protocol

When an agent completes:

1. **Agent creates PR** with:
   - Summary of what was built
   - Test results (`cargo test -p <crate>`)
   - Clippy results
   - Any new dependencies requested
   - Any types it needs added to `core`

2. **Human reviews** the PR

3. **Integration agent picks up** after merge:
   - Adds API routes
   - Wires into CLI pipeline
   - Runs full `cargo test`
   - Updates sprint tracking file

### 5.3 CLAUDE.md as Shared Context

`codeilus/CLAUDE.md` is the single source of truth that ALL agents read:
- Architecture rules
- Code style
- Build commands
- What NOT to touch

Every agent reads this file first. It prevents agents from making conflicting decisions.

---

## 6. Example: Launching Sprint 1

Here's exactly what you'd do:

### Step 1: Define contracts in core (5 min, manual)

Add `ParsedFile`, `Symbol`, `Import`, `Call`, `Heritage` types to `codeilus-core/src/types.rs`.
Add `FileRepo`, `SymbolRepo` trait stubs to `codeilus-db/src/repos/mod.rs`.
Commit to main.

### Step 2: Create sprint tracking file (2 min, manual)

Create `sprints/sprint-1.md` with the template above.

### Step 3: Launch agents (parallel)

```
Agent 1A: "Build codeilus-parse..."  (worktree isolation)
Agent 1B: "Build FileRepo..."       (worktree isolation)
Agent 1C: "Build frontend skeleton..." (worktree isolation)
```

All three run simultaneously. No conflicts because they touch different files.

### Step 4: Review & merge (human)

Review each PR. Merge to main.

### Step 5: Launch integration agent (sequential)

```
Agent 1D: "Wire parse + db into API routes and CLI analyze command..."
```

### Step 6: Update tracking

Mark sprint-1.md items as complete. Move to Sprint 2.

---

## 7. Anti-Patterns to Avoid

| Anti-Pattern | Why It Fails | Do This Instead |
|-------------|-------------|----------------|
| Launch 10 agents at once | Merge conflicts, context degradation, review bottleneck | Max 5 parallel, scoped to non-overlapping files |
| Let agents edit shared files | Merge conflicts guarantee | Freeze shared files, use integration agent |
| Skip contract definition | Agents make incompatible types | Always define types in core FIRST |
| No sprint tracking | Lose track of what's done/blocked | Sprint file updated after each PR merge |
| Agent does research + build | Slow, unfocused, context bloat | Separate research agents from builder agents |
| Skip integration agent | Crates compile alone but don't connect | Integration agent runs full test suite |
| Auto-merge agent PRs | Quality degrades fast | Human review every PR |

---

## 8. Total Agent Count Per Sprint

| Sprint | Parallel Agents | Sequential | Total | Peak Concurrency |
|--------|----------------|------------|-------|-----------------|
| 0 | — | — | — | Done |
| 1 | 3 (parse, db, frontend) | 1 (integration) | 4 | 3 |
| 2 | 3 (graph, db, frontend) | 1 (integration) | 4 | 3 |
| 3 | 4 (metrics, analyze, db, frontend) | 1 (integration) | 5 | 4 |
| 4 | 3 (diagram, research, frontend) | 1 (integration) | 4 | 3 |
| 5 | 4 (llm, narrate, research, frontend) | 1 (integration) | 5 | 4 |
| 6 | 5 (curriculum, quiz, gamification, db, frontend) | 1 (integration) | 6 | 5 |
| 7 | 3 (search, impact, frontend) | 1 (integration) | 4 | 3 |
| 8 | 5 (mcp, export, harvest, frontend, CI) | 0 | 5 | 5 |
| **Total** | | | **37 agent sessions** | **Max 5** |

---

## 9. Quick Reference: The Rules

1. **Contract first** — Define types in `core` before launching agents
2. **File ownership** — Each agent owns specific directories, never crosses boundaries
3. **Worktree isolation** — Each agent gets its own git worktree
4. **Integration last** — Parallel crate agents → merge → integration agent wires everything
5. **Human reviews all PRs** — No auto-merge, no skipping review
6. **Sprint tracking** — One markdown file per sprint, updated after each merge
7. **CLAUDE.md is law** — All agents read it, all agents follow it
8. **Max 5 concurrent** — More agents = more conflicts, worse quality
9. **Research separate from building** — Don't make a builder agent also read 5 reference repos
10. **Freeze shared files** — `core/types.rs`, `migrations/*.sql` don't change mid-sprint
