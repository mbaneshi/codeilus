# Codeilus — Development Tracking & Engineering Process

> How humans, AI agents, and documentation stay in sync throughout the build.

---

## 1. The Three Pillars

```
┌─────────────────────────────────────────────────────────────────┐
│                    CODEILUS DEV SYSTEM                           │
│                                                                  │
│  ┌──────────┐     ┌──────────┐     ┌──────────────────┐        │
│  │  HUMAN   │ ◄──►│    AI    │ ◄──►│  DOCUMENTATION   │        │
│  │          │     │  AGENTS  │     │                  │        │
│  │ Reviews  │     │ Build    │     │ Records truth    │        │
│  │ Decides  │     │ Code     │     │ Tracks state     │        │
│  │ Merges   │     │ Test     │     │ Enables pickup   │        │
│  │ Steers   │     │ Report   │     │ across sessions  │        │
│  └──────────┘     └──────────┘     └──────────────────┘        │
│       │                │                    │                    │
│       └────────────────┼────────────────────┘                   │
│                        │                                        │
│              Shared Source of Truth:                             │
│              Git + Sprint Files + Memory                         │
└─────────────────────────────────────────────────────────────────┘
```

### Human Responsibilities
- Define contracts (types in `core`) before agents start
- Review every agent PR before merge
- Run quality gates between waves
- Make architectural decisions when agents disagree
- Update sprint tracking after merges
- Steer priorities when scope changes

### AI Agent Responsibilities
- Read CLAUDE.md + task file before starting
- Work only within assigned crate/directory
- Write tests for everything built
- Report: what was built, what was skipped, what's blocked
- Update the Report section of their task file
- Never modify shared files without coordination

### Documentation Responsibilities
- `sprints/sprint-N.md` — single source of truth for sprint state
- `tasks/BOARD.md` — kanban view of all agent tasks
- `docs/adr/` — architecture decisions with context and rationale
- `CHANGELOG.md` — human-readable history of what shipped
- Claude memory files — cross-session context for AI continuity

---

## 2. Quality Gates

### Gate 1: Pre-Sprint (before launching agents)

```
□ Contracts defined in codeilus-core/src/types.rs
□ DB repo trait stubs created
□ Sprint tracking file created (sprints/sprint-N.md)
□ Agent task files reviewed and ready
□ Previous sprint's quality gate passed
□ `cargo test && cargo clippy` clean on main
```

### Gate 2: Per-Agent (before PR review)

```
□ Agent updated Report section in task file
□ cargo test -p <crate> — all pass
□ cargo clippy -p <crate> — zero warnings
□ Only modified files within assigned scope
□ No new dependencies without justification
□ Tests cover happy path + at least 1 error case
```

### Gate 3: Per-Wave (before starting next wave)

```
□ All agents in wave reported done
□ cargo build — full workspace compiles
□ cargo test — all tests pass (not just per-crate)
□ cargo clippy — zero warnings workspace-wide
□ Sprint tracking file updated with results
□ CHANGELOG.md updated with wave deliverables
□ Any new shared types committed to core
□ Blockers for next wave identified and resolved
```

### Gate 4: Per-Sprint (before moving to next sprint)

```
□ All acceptance criteria from PLAN.md checked
□ E2E smoke test passes (analyze → serve → verify)
□ Known issues documented in sprint file
□ ADR written for any significant decisions made
□ Memory files updated for AI session continuity
□ BOARD.md statuses all marked done or carried over
```

---

## 3. Architecture Decision Records (ADRs)

When an agent or human makes a non-obvious technical choice, record it.

**Location:** `docs/adr/NNNN-title.md`

**Template:**
```markdown
# ADR-NNNN: Title

**Status:** Accepted | Superseded | Deprecated
**Date:** YYYY-MM-DD
**Sprint:** N
**Decider:** Human | Agent (which one)

## Context
What situation prompted this decision?

## Decision
What did we decide?

## Alternatives Considered
What else could we have done?

## Consequences
What are the trade-offs? What becomes easier/harder?
```

**When to write an ADR:**
- Choosing between two viable approaches (e.g., tree-sitter vs. heuristic parsing)
- Adding a significant dependency
- Changing a data model or API contract
- Deviating from the plan in NORTH_STAR.md
- Any decision a future developer would ask "why?"

---

## 4. Changelog

**Location:** `CHANGELOG.md` in codeilus root

**Format:**
```markdown
# Changelog

## [Unreleased]

### Wave 1 (Sprint 1) — YYYY-MM-DD
#### Added
- Tree-sitter parsing for Python, TypeScript, JavaScript, Rust, Go, Java
- FileRepo and SymbolRepo with batch inserts and queries
- Frontend skeleton with sidebar navigation and file tree page
#### Fixed
- EventBus::new() test signature mismatch
- 8 clippy warnings in codeilus-parse
#### Changed
- Replaced heuristic parsers with tree-sitter AST extraction
```

**Rules:**
- Updated after each wave merges (not per-agent)
- Uses [Keep a Changelog](https://keepachangelog.com) format
- Added/Fixed/Changed/Removed categories
- Each entry is one sentence, links to relevant code if needed

---

## 5. Sprint Tracking Protocol

### Before Sprint Starts
1. Create `sprints/sprint-N.md` from template
2. Define contracts in `codeilus-core`
3. Verify quality gate 1

### During Sprint
1. Launch agents per wave plan
2. Each agent updates their `tasks/waveN/*.md` report section
3. Human reviews PRs, runs quality gate 2
4. After each wave: run quality gate 3
5. Log decisions in sprint file's "Decisions Made" section

### After Sprint Ends
1. Run quality gate 4
2. Update `CHANGELOG.md`
3. Update `tasks/BOARD.md` statuses
4. Update memory files for AI continuity
5. Write ADRs for any significant decisions
6. Carry over incomplete items to next sprint

### Sprint File Template

```markdown
# Sprint N: [Name]

**Status:** Not Started | In Progress | Complete
**Dates:** YYYY-MM-DD to YYYY-MM-DD
**Goal:** One sentence

## Pre-Launch Checklist
- [ ] Contracts defined in core
- [ ] Previous sprint quality gate passed
- [ ] Agent task files ready

## Agents

| Agent | Scope | Status | PR | Notes |
|-------|-------|--------|-----|-------|
| NA: name | crate | status | #N | — |

## Acceptance Criteria
(copied from PLAN.md, checked off as verified)

## Quality Gate Results
- [ ] cargo build clean
- [ ] cargo test all pass
- [ ] cargo clippy zero warnings
- [ ] E2E smoke test

## Decisions Made
| Date | Decision | Rationale | ADR |
|------|----------|-----------|-----|

## Blockers
(none)

## Retrospective
What went well? What to improve?
```

---

## 6. Memory Sync Protocol

Claude Code memory files bridge conversations. They must stay accurate.

### When to Update Memory

| Event | What to Update |
|-------|---------------|
| Sprint completes | `project_status.md` — new status, remaining work |
| Architecture changes | `project_architecture.md` — updated data flow, deps |
| User gives feedback | `feedback_*.md` — capture for future agents |
| New decision made | Reference the ADR in memory |
| Significant bug found | Note in `project_status.md` known issues |

### Memory File Hygiene
- Memory reflects **current truth**, not historical plans
- Delete or update memories that are now wrong
- Keep memory concise — it's loaded into every conversation
- Memory is for **cross-session context**, not in-session tracking (use tasks for that)

---

## 7. Definition of Done

### Per-Crate Done
- [ ] All planned public API functions implemented
- [ ] Tests pass: `cargo test -p <crate>`
- [ ] Clippy clean: `cargo clippy -p <crate>`
- [ ] Doc comments on all public items
- [ ] Error cases handled (return `CodeilusResult`, no panics)
- [ ] Agent report section filled in task file

### Per-Sprint Done
- [ ] All acceptance criteria from PLAN.md verified
- [ ] Full workspace compiles and tests pass
- [ ] CHANGELOG updated
- [ ] Sprint tracking file completed
- [ ] Memory files updated
- [ ] No known regressions from previous sprints

### Per-Feature Done (for frontend)
- [ ] Works in both light and dark theme
- [ ] Responsive on mobile viewport
- [ ] Loading states for async operations
- [ ] Error states for failed API calls
- [ ] Keyboard accessible

---

## 8. File Map — What Lives Where

```
codeilus/
├── CHANGELOG.md                      # What shipped, when
├── codeilus/
│   ├── CLAUDE.md                     # Agent shared context (READ FIRST)
│   ├── NORTH_STAR.md                 # Vision, architecture, full roadmap
│   │
│   ├── docs/
│   │   ├── DEV_TRACKING.md           # THIS FILE — process & tracking
│   │   ├── AGENT_PROMPTS.md          # Copy-paste prompts for agents
│   │   ├── user/
│   │   │   └── PERSONAS.md           # User personas & journeys
│   │   └── adr/
│   │       ├── 0001-*.md             # Architecture decisions
│   │       └── ...
│   │
│   ├── tasks/
│   │   ├── BOARD.md                  # Kanban status board
│   │   ├── wave1/*.md                # Per-agent task files
│   │   ├── wave2/*.md
│   │   └── ...
│   │
│   ├── sprints/
│   │   ├── sprint-0.md               # Complete
│   │   ├── sprint-1.md               # Ready
│   │   └── ...
│   │
│   └── crates/                       # The actual code
│
├── PLAN.md                           # Master plan (sprints, acceptance criteria)
├── AGENTS.md                         # Multi-agent coordination strategy
│
└── ~/.claude/projects/.../memory/    # AI cross-session memory
    ├── MEMORY.md                     # Memory index
    ├── project_status.md             # What's built, what's broken
    └── project_architecture.md       # Crate graph, data flow
```

---

## 9. Human ↔ AI Handoff Checklist

### Starting a New Session (AI reads)
1. Read `CLAUDE.md` — rules, style, build commands
2. Read memory files — where we left off, known issues
3. Read `sprints/sprint-N.md` — current sprint status
4. Read `tasks/BOARD.md` — what's in progress
5. Check `cargo test && cargo clippy` — is the base healthy?

### Ending a Session (AI writes)
1. Update memory files if status changed
2. Update sprint tracking if work was done
3. Update task report section if completing an agent task
4. Note any decisions that need ADRs
5. List what's next for the human to review/merge

### Human Between Sessions
1. Review any PRs or changes from agents
2. Run quality gates
3. Update `BOARD.md` and `CHANGELOG.md`
4. Resolve blockers noted by agents
5. Prepare contracts for next wave if ready
