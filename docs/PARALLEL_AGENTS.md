# Parallel Agent Prompts — Wave Next

Copy-paste each section into a separate Claude Code tab. Each agent owns specific files and should NOT touch files owned by other agents.

---

## Agent 1: Fix Narrate Tests + Clippy (5 min)

```
cd /Users/bm/codeilus/codeilus

## Task: Fix codeilus-narrate test compilation + clippy warning

### Problem 1: Narrate tests don't compile
The tests in `crates/codeilus-narrate/tests/narrate.rs` fail to compile because `NarrativeGenerator::new()` is async but the tests call it without `.await` before chaining `.generate_all()`.

3 test functions are broken (lines ~236, ~251, ~294). Each does:
```rust
let narratives = gen
    .generate_all(&graph, &files, std::path::Path::new("/tmp"))
    .await
```
But `gen` is created with `NarrativeGenerator::new()` which is `pub async fn new() -> Self`.
So `gen` is a `Future`, not the generator itself. Need `.await` after `new()`.

Fix: read the test file, find all 3 broken tests, add `.await` after `NarrativeGenerator::new()` or `NarrativeGenerator::placeholder_only()` calls as appropriate.

### Problem 2: 1 clippy warning
In `crates/codeilus-api/src/routes/ask.rs` line 61, there's a `clippy::type_complexity` warning about a complex tuple type. Extract it into a type alias.

### YOUR FILES (only touch these):
- `crates/codeilus-narrate/tests/narrate.rs`
- `crates/codeilus-api/src/routes/ask.rs`

### Verify:
```bash
cargo test -p codeilus-narrate
cargo clippy --workspace 2>&1 | grep "warning:" | wc -l  # should be 0
```
```

---

## Agent 2: Enrich Learning Engine Backend (20 min)

```
cd /Users/bm/codeilus/codeilus

## Task: Make the learning engine generate real content, not just structure

### Context
`crates/codeilus-learn/` generates a curriculum (chapters from communities), but chapters are empty shells. The frontend `/learn` page shows chapter cards but clicking one leads nowhere useful.

### YOUR FILES (only touch these):
- `crates/codeilus-learn/src/curriculum.rs` (444 lines)
- `crates/codeilus-learn/src/types.rs` (190 lines)
- `crates/codeilus-learn/src/progress.rs` (396 lines)
- `crates/codeilus-learn/src/quiz.rs` (371 lines)
- `crates/codeilus-learn/src/lib.rs` (18 lines)
- `crates/codeilus-learn/tests/` (create if needed)

### What to do:

1. **Read all files** in `crates/codeilus-learn/src/` first to understand current state.

2. **Enrich curriculum.rs** — `generate_curriculum()` should produce chapters with real content:
   - Chapter 0: "The Big Picture" — overview, architecture diagram reference, entry points
   - Per-community chapters: ordered by dependency (imported-before-importing)
   - Each chapter section should have `content` populated:
     - `SectionKind::Overview` → list key symbols, their roles, how they connect
     - `SectionKind::CodeWalkthrough` → ordered list of symbols to read (most-imported first)
     - `SectionKind::Connections` → which other chapters/communities this one depends on or feeds into
   - Final chapter: "Putting It All Together" — cross-cutting concerns, data flow summary

3. **Enrich quiz.rs** — Generate questions from actual graph data:
   - "What does X call?" (from CALLS edges)
   - "Which module contains X?" (from community membership)
   - "How many direct dependencies does file Y have?" (from IMPORTS edges)
   - "True/False: X extends Y" (from heritage edges)
   - Generate 3-5 questions per chapter from the graph

4. **Progress tracking** — review `progress.rs`, make sure XP awards work:
   - +10 per section completed
   - +50 per chapter completed
   - +25 per quiz passed
   - Streak tracking (consecutive days)

5. **Add tests** in `crates/codeilus-learn/tests/learn.rs`:
   - Test curriculum generation with a small graph (3 communities, 10 symbols)
   - Test quiz generation produces valid questions
   - Test progress XP calculation

### Architecture rules:
- `codeilus-learn` depends on `codeilus-core` and `codeilus-graph` (for types)
- Use petgraph for graph traversal
- All IDs are i64 newtypes (SymbolId, CommunityId, etc.)
- Zero clippy warnings

### Verify:
```bash
cargo test -p codeilus-learn
cargo clippy -p codeilus-learn
```
```

---

## Agent 3: Learning Frontend + Chapter Pages (20 min)

```
cd /Users/bm/codeilus/codeilus

## Task: Build rich learning UI — chapter detail pages, progress tracking, quiz modal

### Context
The `/learn` page exists (160 lines) but is just a grid of chapter cards. There are no chapter detail pages, no quiz UI, no progress visualization. The API routes exist at `/api/v1/chapters`, `/api/v1/chapters/:id`, `/api/v1/learn/progress`, `/api/v1/learn/quiz/:chapter_id`.

### YOUR FILES (only touch these):
- `frontend/src/routes/learn/+page.svelte` (160 lines — enhance)
- `frontend/src/routes/learn/[id]/+page.svelte` (CREATE — chapter detail)
- `frontend/src/lib/api.ts` — add learn-specific API functions only (fetchProgress, submitQuiz, updateProgress, fetchQuiz)
- `frontend/src/lib/types.ts` — add learn-specific types only (Progress, QuizQuestion, QuizAttempt, LearnerStats, Badge)

### What to build:

1. **Read existing files first**: `frontend/src/routes/learn/+page.svelte`, `frontend/src/lib/api.ts`, `frontend/src/lib/types.ts`

2. **Types to add** in `types.ts` (append, don't modify existing):
```typescript
export interface Progress {
  chapter_id: number;
  section_id: string;
  completed: boolean;
  completed_at: string | null;
}

export interface QuizQuestion {
  id: number;
  chapter_id: number;
  question: string;
  options: string[];
  kind: string;
}

export interface LearnerStats {
  total_xp: number;
  streak_days: number;
  last_active: string;
  chapters_completed: number;
  badges: Badge[];
}

export interface Badge {
  id: number;
  name: string;
  description: string;
  icon: string;
  earned_at: string | null;
}
```

3. **API functions** to add in `api.ts` (append, don't modify existing):
```typescript
fetchProgress(): Promise<Progress[]>
fetchQuiz(chapterId: number): Promise<QuizQuestion[]>
submitQuizAnswer(questionId: number, answer: string): Promise<{ correct: boolean; xp: number }>
markSectionComplete(chapterId: number, sectionId: string): Promise<void>
fetchLearnerStats(): Promise<LearnerStats>
```

4. **Enhance `/learn` page**:
   - Add XP counter + streak display at top
   - Show progress bar per chapter card (% sections completed)
   - Badge shelf (earned badges as small icons)
   - Visual chapter dependency flow (Chapter 0 → Chapter 1 → etc.)
   - Difficulty badge on each card (beginner/intermediate/advanced)

5. **Create `/learn/[id]/+page.svelte`** — chapter detail page:
   - Header: chapter title, difficulty, progress bar
   - Tabbed sections: Overview | Code Walkthrough | Connections | Quiz
   - Overview tab: rendered markdown-like content, key symbols with `kindColor` badges
   - Code Walkthrough tab: ordered list of symbols to read, with "mark as read" checkboxes
   - Connections tab: links to prerequisite/downstream chapters
   - Quiz tab: multiple-choice questions, submit button, result feedback, XP award animation
   - Back button to /learn
   - "Next Chapter" button when 100% complete

### Tech stack:
- SvelteKit 5 with runes ($state, $derived, $effect)
- TailwindCSS 4 with `@reference "tailwindcss"` in <style>
- Use existing patterns from other pages (see graph/+page.svelte for overlay patterns)
- lucide-svelte for icons (BookOpen, Trophy, Star, ChevronRight, Check, etc.)
- Dark theme (bg-gray-900, text-gray-100, etc.)

### Verify:
```bash
cd frontend && pnpm build
```
```

---

## Agent 4: Harvest + Export Pipeline (20 min)

```
cd /Users/bm/codeilus/codeilus

## Task: Make harvest and export actually work end-to-end

### Context
`codeilus-harvest` scrapes GitHub trending and clones repos. `codeilus-export` generates static HTML pages. Both have real code (~500 and ~500 lines respectively) but need wiring and polish. The goal: `codeilus harvest --trending` → `codeilus export --all-harvested` produces self-contained HTML pages.

### YOUR FILES (only touch these):
- `crates/codeilus-harvest/src/` (all files)
- `crates/codeilus-export/src/` (all files)
- `crates/codeilus-export/tests/` (create if needed)
- `export-template/index.html` (the HTML template)

Do NOT touch: `crates/codeilus-app/src/main.rs`, any frontend files, any other crates.

### What to do:

1. **Read all files** in both crates first. Understand current state.

2. **codeilus-harvest** — ensure scraper works:
   - Read `scraper.rs` — it should parse GitHub trending page HTML
   - Read `cloner.rs` — shallow clone with `git clone --depth=1`
   - Read `fingerprint.rs` — skip already-analyzed repos
   - Make sure `harvest_trending()` in `lib.rs` orchestrates: scrape → filter → clone → return repos
   - Add rate limiting: max 5 concurrent clones, 1s delay between scrape pages
   - Add error handling: if a clone fails, skip it, don't abort the batch
   - The scraper should work with real GitHub HTML or fall back to the unofficial API at `https://github.com/trending?since=daily`

3. **codeilus-export** — produce real static HTML:
   - Read `data_loader.rs` — loads all data from DB (files, symbols, communities, narratives, metrics)
   - Read `renderer.rs` — renders data into HTML using template
   - Read `template.rs` — the HTML template system
   - The template at `export-template/index.html` should be a complete, self-contained HTML page
   - **Write the template** if it's just a stub. It should be a single HTML file with:
     - Inlined CSS (dark theme, responsive, professional)
     - Hero section: repo name, description, language badges, star count
     - 30-second overview (from narratives)
     - Architecture diagram placeholder (Mermaid SVG will be inlined)
     - Key files to read first (ranked list)
     - Entry points section
     - How it works (architecture prose)
     - Metrics snapshot (SLOC, complexity, file count)
     - Collapsible "Deep Dive" section with community details
     - Footer with "Generated by Codeilus" + timestamp
     - ALL data inlined as JSON in <script> tags (the renderer should inject it)
   - Target: <500KB, loads instantly, works offline, looks professional
   - `export_repo()` in `lib.rs` should: load data → render template → write HTML file

4. **Add tests**:
   - Test data_loader with in-memory DB
   - Test renderer produces valid HTML
   - Test template substitution works

### Architecture rules:
- `codeilus-harvest` depends on `codeilus-core` only
- `codeilus-export` depends on `codeilus-core`, `codeilus-db`, `codeilus-narrate`, `codeilus-diagram`
- Use `reqwest` for HTTP in harvest (check Cargo.toml for existing deps)
- Use `tokio` for async
- Zero clippy warnings

### Verify:
```bash
cargo test -p codeilus-harvest -p codeilus-export
cargo clippy -p codeilus-harvest -p codeilus-export
```
```

---

## Agent 5: E2E Validation + Integration Tests (15 min)

```
cd /Users/bm/codeilus/codeilus

## Task: Run real E2E analysis and add integration tests

### YOUR FILES (only touch these):
- `tests/` directory (create integration tests)
- `crates/codeilus-app/tests/` (create if needed)

Do NOT modify any source code in `src/` directories. Only create test files and report issues.

### What to do:

1. **Run a real analysis** of the codeilus repo itself:
```bash
cargo build && ./target/debug/codeilus analyze .
```
Capture and report the output. Specifically note:
- How many files, symbols, edges, communities?
- Did edge count improve from the recent changes? (should be 30-100+, not 10)
- Did community count decrease? (should be 3-10, not 26)
- Any errors or warnings?

2. **Start the server and test endpoints**:
```bash
./target/debug/codeilus serve &
sleep 2

# Test basic endpoints
curl -s http://localhost:4174/api/v1/health | head
curl -s http://localhost:4174/api/v1/files | python3 -m json.tool | head -20
curl -s http://localhost:4174/api/v1/graph | python3 -m json.tool | head -20
curl -s http://localhost:4174/api/v1/communities | python3 -m json.tool | head -20

# Test source endpoint (pick a file ID from the files response)
curl -s "http://localhost:4174/api/v1/files/1/source?start=1&end=10" | python3 -m json.tool

# Test chapters
curl -s http://localhost:4174/api/v1/chapters | python3 -m json.tool | head -20

kill %1
```

3. **Create integration tests** in `tests/integration_test.rs`:
   - Test: analyze a small fixture repo → verify files stored → verify symbols stored → verify edges exist → verify communities detected
   - Use the fixtures in `crates/codeilus-parse/tests/fixtures/` if they exist, or create a minimal test fixture
   - Test: source endpoint returns correct line ranges
   - Test: graph API returns nodes with community_ids assigned

4. **Report findings**: Create a file `tests/E2E_REPORT.md` documenting:
   - What works
   - What's broken
   - Edge/community count improvements
   - Any UI issues observed

### Verify:
```bash
cargo test --test integration_test
```
```

---

## Coordination Notes

| Agent | Owns | Does NOT touch |
|-------|------|----------------|
| 1 (Narrate fix) | `codeilus-narrate/tests/`, `codeilus-api/src/routes/ask.rs` | Everything else |
| 2 (Learn backend) | `codeilus-learn/src/`, `codeilus-learn/tests/` | Frontend, other crates |
| 3 (Learn frontend) | `frontend/src/routes/learn/`, types.ts (append), api.ts (append) | Rust code, other frontend routes |
| 4 (Harvest+Export) | `codeilus-harvest/src/`, `codeilus-export/src/`, `export-template/` | Everything else |
| 5 (E2E) | `tests/`, reports only | No source modifications |

**No file conflicts between agents.**

After all agents finish:
```bash
cargo build && cargo test --workspace && cargo clippy
cd frontend && pnpm build
```
