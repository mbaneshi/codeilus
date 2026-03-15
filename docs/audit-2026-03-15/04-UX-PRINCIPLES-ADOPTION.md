# UX Principles Adoption Proposal

**Date:** 2026-03-15
**Scope:** Codeilus interactive UI (SvelteKit 5 frontend)
**Framework:** Nielsen's Heuristics + Progressive Disclosure + Gamification Psychology

---

## Executive Summary

Codeilus transforms codebases into learning experiences — this is inherently a UX product, not just a developer tool. The current frontend has the right pages but lacks the interaction design that makes learning feel effortless. Below are 8 principle-driven proposals ranked by user impact, each with concrete implementation guidance.

---

## Principle 1: Progressive Disclosure — Don't Overwhelm on First Load

### Problem

The home page (`+page.svelte`, 14,567 lines) loads an onboarding modal, stats cards, quick-start links, and a full sidebar simultaneously. A first-time user who just ran `codeilus ./repo` sees everything at once with no sense of "what do I do first?"

### Proposal: Guided First-Run Experience

**Phase 1 — Empty State (No Analysis)**
```
┌─────────────────────────────────────────────────┐
│                                                 │
│          Welcome to Codeilus                    │
│                                                 │
│    Analyze a repository to get started:         │
│                                                 │
│    $ codeilus analyze ./path/to/repo            │
│                                                 │
│    [Open Documentation]                         │
│                                                 │
└─────────────────────────────────────────────────┘
```

**Phase 2 — Analysis Running (Real-Time Progress)**
```
┌─────────────────────────────────────────────────┐
│  Analyzing: my-project                          │
│                                                 │
│  ✓ Parsing files         [342/342]              │
│  ✓ Building graph        [done]                 │
│  ● Computing metrics     [67%]                  │
│  ○ Generating diagrams                          │
│  ○ Writing narratives                           │
│  ○ Building curriculum                          │
│                                                 │
│  [This takes ~30s for most repos]               │
└─────────────────────────────────────────────────┘
```

**Phase 3 — Analysis Complete (Guided Onboarding)**
```
┌─────────────────────────────────────────────────┐
│  ✓ my-project analyzed!                         │
│                                                 │
│  342 files · 1,247 symbols · 4 languages        │
│  12 communities · 3 entry points                │
│                                                 │
│  ┌──────────────┐  Start here:                  │
│  │ 30-Second    │  A quick overview of what     │
│  │ Overview     │  this project does and how    │
│  │    →         │  it's structured.             │
│  └──────────────┘                               │
│                                                 │
│  Or explore:                                    │
│  [Learning Path]  [Graph]  [Metrics]  [Ask AI]  │
└─────────────────────────────────────────────────┘
```

**Implementation:**
- Add a `ui_state` store in SvelteKit: `{ phase: 'empty' | 'analyzing' | 'ready' | 'learning' }`
- Use WebSocket events to drive phase transitions
- Collapse sidebar to minimal on first load; expand after user has explored 2+ pages
- Store `has_seen_onboarding` in localStorage (or a `/api/v1/preferences` endpoint)

---

## Principle 2: Visibility of System Status — Real-Time Feedback Everywhere

### Problem

The analysis pipeline runs 8 steps that can take 30-120 seconds. Currently there's no progress feedback during analysis. The Q&A page (`/ask`) streams responses but shows no typing indicator before the first token arrives.

### Proposal: Unified Status System

**2a: Analysis Progress Bar (WebSocket-driven)**

Wire the existing EventBus → WebSocket pipeline to the frontend:

```svelte
<!-- AnalysisProgress.svelte -->
<script>
  import { onMount } from 'svelte';

  let steps = [
    { name: 'Parsing', status: 'pending', progress: null },
    { name: 'Storing', status: 'pending', progress: null },
    { name: 'Graph Building', status: 'pending', progress: null },
    { name: 'Metrics', status: 'pending', progress: null },
    { name: 'Analysis', status: 'pending', progress: null },
    { name: 'Diagrams', status: 'pending', progress: null },
    { name: 'Narratives', status: 'pending', progress: null },
    { name: 'Curriculum', status: 'pending', progress: null },
  ];

  onMount(() => {
    const ws = new WebSocket(`ws://${location.host}/ws`);
    ws.onmessage = (e) => {
      const event = JSON.parse(e.data);
      // Update steps based on event type
      if (event.type === 'ParsingProgress') {
        steps[0].status = 'active';
        steps[0].progress = `${event.files_done}/${event.files_total}`;
      }
      // ... map all 18 event types to step updates
    };
  });
</script>
```

**2b: LLM Thinking Indicator**

Before streaming tokens arrive (can be 1-3 seconds for Claude):

```svelte
{#if isWaitingForLlm}
  <div class="flex items-center gap-2 text-gray-400">
    <div class="flex gap-1">
      <span class="animate-bounce delay-0">·</span>
      <span class="animate-bounce delay-100">·</span>
      <span class="animate-bounce delay-200">·</span>
    </div>
    Claude is thinking...
  </div>
{/if}
```

**2c: Stale Data Indicator**

If the repo has been modified since last analysis:

```svelte
{#if repoModifiedSinceAnalysis}
  <div class="bg-amber-900/20 text-amber-300 p-2 rounded text-sm">
    Repository has changed since last analysis.
    <button on:click={reanalyze}>Re-analyze</button>
  </div>
{/if}
```

---

## Principle 3: Recognition Over Recall — Navigation Should Be Spatial

### Problem

The sidebar has text-only navigation links: Learn, Explore (with sub-pages: Tree, Graph, Metrics, Diagrams), Ask, Settings. Users must remember what each section contains.

### Proposal: Visual Navigation with Context Hints

**3a: Sidebar with Mini-Previews**

```
┌──────────────────────┐
│ 📖 Learn            │
│   ├ Ch.0: Big Picture│ ← current chapter highlighted
│   ├ Ch.1: Core       │
│   └ +4 more...       │
│                      │
│ 🔍 Explore          │
│   ├ 📊 Metrics [3!] │ ← "3!" = 3 anti-patterns found
│   ├ 🌐 Graph        │
│   ├ 🌳 File Tree    │
│   └ 📐 Diagrams     │
│                      │
│ 💬 Ask AI           │
│   └ 2 conversations │
│                      │
│ ──────────────────── │
│ 📈 Progress: 23%    │ ← persistent progress indicator
│ ⚡ 150 XP           │
│ 🔥 3-day streak     │
└──────────────────────┘
```

**3b: Breadcrumb Trail**

Add breadcrumbs to every page so users always know where they are:

```
Home > Learn > Chapter 3: API Layer > Section 2: Route Handlers
```

**3c: Command Palette (Ctrl+K)**

Power users need fast navigation:

```svelte
<!-- CommandPalette.svelte -->
<dialog class="...">
  <input placeholder="Search symbols, chapters, pages..." />
  <div class="results">
    <!-- Mix: symbol search + page navigation + chapter jump -->
    <div class="result-group">Pages</div>
    <div class="result">📊 Metrics Dashboard</div>
    <div class="result-group">Symbols</div>
    <div class="result">fn parse_repository()</div>
    <div class="result-group">Chapters</div>
    <div class="result">Chapter 3: API Layer</div>
  </div>
</dialog>
```

---

## Principle 4: Aesthetic and Minimalist Design — Information Density Control

### Problem

The Graph Explorer page loads ALL nodes and edges into a 3D force-directed graph. For large codebases (1000+ symbols), this creates a visual mess — a hairball that provides no insight.

### Proposal: Layered Graph Exploration

**Level 1: Community Overview (default)**
```
┌─────────────────────────────────────────────────┐
│  [Community View]  [Symbol View]  [File View]   │
│                                                 │
│        ┌───────┐     ┌───────┐                  │
│        │ Core  │────→│ Parse │                  │
│        │ (24)  │     │ (18)  │                  │
│        └───┬───┘     └───────┘                  │
│            │                                    │
│        ┌───┴───┐     ┌───────┐                  │
│        │  DB   │←────│  API  │                  │
│        │ (12)  │     │ (31)  │                  │
│        └───────┘     └───────┘                  │
│                                                 │
│  Click a community to explore its symbols       │
└─────────────────────────────────────────────────┘
```

**Level 2: Community Detail (on click)**
```
┌─────────────────────────────────────────────────┐
│  ← Back to Overview    Community: API Layer      │
│                                                 │
│  31 symbols · 47 internal edges · 12 external   │
│                                                 │
│     [3D graph of just this community's symbols] │
│                                                 │
│  Entry Points:                                  │
│  • fn app() → Router setup                      │
│  • fn health_check() → GET /health              │
│                                                 │
│  Key Metrics:                                   │
│  Fan-in: 8 (heavily depended on)                │
│  Complexity: Medium (avg 4.2)                   │
└─────────────────────────────────────────────────┘
```

**Level 3: Symbol Focus (on node click)**
```
┌─────────────────────────────────────────────────┐
│  fn parse_repository()                          │
│  codeilus-parse/src/lib.rs:42-181               │
│                                                 │
│  Callers (3):  run_pipeline() → analyze_cmd()   │
│  Callees (5):  walk_files() · detect_lang() ·   │
│                parse_file() · extract_symbols()  │
│                                                 │
│  [View Source]  [Explain (AI)]  [Show in Graph] │
│                                                 │
│  ┌─ Source Preview ─────────────────────────┐   │
│  │ 42 │ pub fn parse_repository(             │   │
│  │ 43 │     config: &ParseConfig,            │   │
│  │ 44 │     bus: Option<&EventBus>,          │   │
│  │ 45 │ ) -> Result<Vec<ParsedFile>> {       │   │
│  └───────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

**Implementation:**
- Default to Level 1 (community clusters only) — max 20 nodes
- On click, transition to Level 2 with animation
- Use `3d-force-graph` zoom-to-fit for level transitions
- Add a "Complexity Filter" slider: show only symbols above complexity N

---

## Principle 5: User Control and Freedom — Undo and Escape Hatches

### Problem

The learning path is auto-generated with a fixed chapter order. Users who are experienced with part of the codebase must still start from Chapter 0. There's no way to skip, reorder, or customize the path.

### Proposal: Flexible Learning Path

**5a: Skip/Test-Out Button per Chapter**

```
┌─────────────────────────────────────────────────┐
│  Chapter 3: Database Layer                      │
│                                                 │
│  Already familiar with this module?             │
│  [Take the Quiz to Skip →]                      │
│                                                 │
│  Or start learning:                             │
│  Section 1: Overview ........................ ○  │
│  Section 2: Schema Design .................. ○  │
│  Section 3: Repository Pattern ............. ○  │
│  Section 4: Migrations ..................... ○  │
│  Quiz ..................................... ○  │
└─────────────────────────────────────────────────┘
```

If the user passes the quiz without reading sections, award full XP and mark chapter complete.

**5b: Custom Learning Path**

Allow drag-and-drop reordering of chapters, with a warning if dependencies are skipped:

```
⚠️ Chapter 5 (Graph) depends on Chapter 2 (Core Types).
   Are you sure you want to skip Chapter 2?
   [Skip Anyway]  [Add Chapter 2 First]
```

**5c: Reset Progress**

Settings page should have:
```
Learning Progress
├ Reset Chapter Progress  [Reset]  ← clears progress, keeps XP
├ Reset All Data          [Reset]  ← clears everything
└ Re-analyze Repository   [Run]    ← re-runs pipeline
```

---

## Principle 6: Error Prevention + Recovery — Helpful Error States

### Problem

When LLM is unavailable, narrative pages show empty content. When search returns no results, the page is blank. When the DB has no data (no analysis run), pages throw JavaScript errors.

### Proposal: Designed Empty & Error States

**6a: No-Data State (Before Analysis)**

```svelte
<!-- EmptyState.svelte -->
{#if !hasAnalysisData}
  <div class="flex flex-col items-center justify-center h-96 text-gray-400">
    <svg><!-- illustration --></svg>
    <h2 class="text-xl font-semibold mt-4">No analysis data yet</h2>
    <p class="mt-2">Run the analysis pipeline to populate this page:</p>
    <code class="mt-4 bg-gray-800 p-3 rounded">codeilus analyze ./your-repo</code>
  </div>
{/if}
```

**6b: No-LLM State (Narratives)**

```svelte
{#if narrative.content === '' && !llmAvailable}
  <div class="border border-amber-500/30 bg-amber-900/10 p-4 rounded-lg">
    <h3 class="font-semibold text-amber-300">AI narrative unavailable</h3>
    <p class="text-sm mt-1">Install Claude Code for AI-generated explanations.</p>
    <p class="text-sm mt-2">Here's what we know from static analysis:</p>
    <ul class="mt-2 text-sm">
      <li>This module has {symbolCount} symbols across {fileCount} files</li>
      <li>Key concepts: {keywords.join(', ')}</li>
      <li>Connected to: {connectedCommunities.join(', ')}</li>
    </ul>
  </div>
{/if}
```

**6c: Search No-Results**

```svelte
{#if searchResults.length === 0 && query.length > 0}
  <div class="text-center py-8 text-gray-400">
    <p>No results for "{query}"</p>
    <p class="text-sm mt-2">Try searching for:</p>
    <div class="flex gap-2 mt-3 justify-center">
      {#each suggestedTerms as term}
        <button class="px-3 py-1 bg-gray-800 rounded hover:bg-gray-700"
                on:click={() => search(term)}>
          {term}
        </button>
      {/each}
    </div>
  </div>
{/if}
```

---

## Principle 7: Consistency and Standards — Design System

### Problem

The frontend uses TailwindCSS utilities directly everywhere. Colors, spacing, and component patterns vary across pages. There's no shared component library.

### Proposal: Minimal Design System

**7a: Token-Based Color System**

```css
/* app.css */
:root {
  --color-bg-primary: #0f172a;
  --color-bg-secondary: #1e293b;
  --color-bg-tertiary: #334155;
  --color-text-primary: #f8fafc;
  --color-text-secondary: #94a3b8;
  --color-text-muted: #64748b;
  --color-accent: #6366f1;
  --color-success: #22c55e;
  --color-warning: #f59e0b;
  --color-danger: #ef4444;

  /* Spacing */
  --space-page: 2rem;
  --space-section: 1.5rem;
  --space-element: 1rem;

  /* Community colors (consistent across graph, sidebar, chapters) */
  --community-0: #6366f1;
  --community-1: #ec4899;
  --community-2: #14b8a6;
  --community-3: #f59e0b;
  --community-4: #8b5cf6;
  --community-5: #06b6d4;
}
```

**7b: Shared Components**

Create `src/lib/components/`:

```
components/
├── Card.svelte           ← consistent card container
├── Badge.svelte          ← XP, badges, status indicators
├── ProgressBar.svelte    ← used in chapters, overview, sidebar
├── CodeBlock.svelte      ← syntax-highlighted code with line numbers
├── EmptyState.svelte     ← reusable empty state with illustration
├── ErrorBanner.svelte    ← consistent error display
├── LoadingSpinner.svelte ← consistent loading state
├── Tooltip.svelte        ← hover info (symbol details, metrics)
└── Modal.svelte          ← consistent modal (onboarding, quiz, etc.)
```

---

## Principle 8: Help Users Recognize, Diagnose, and Recover — Contextual Help

### Problem

The app has powerful features (graph explorer, metrics dashboard, Q&A) but no in-context help. Users must leave the app to read documentation.

### Proposal: In-App Contextual Help

**8a: Feature Tooltips (First Visit)**

```svelte
{#if !hasSeenGraphHelp}
  <div class="absolute top-4 right-4 bg-indigo-900/90 p-4 rounded-lg max-w-xs shadow-xl">
    <h4 class="font-semibold">Graph Explorer</h4>
    <p class="text-sm mt-1">
      Each dot is a function or class. Lines show how they connect.
      Colors group related code into modules.
    </p>
    <div class="flex gap-2 mt-3">
      <button on:click={dismissHelp}>Got it</button>
      <button on:click={showAllTips}>Show me around</button>
    </div>
  </div>
{/if}
```

**8b: Metrics Explanation Popovers**

When hovering over "Cyclomatic Complexity: 12":

```
┌─────────────────────────────────────────────┐
│ Cyclomatic Complexity: 12                   │
│                                             │
│ Measures the number of independent paths    │
│ through the code. Higher = harder to test.  │
│                                             │
│ 1-10: Simple    11-20: Moderate    20+: High│
│ This function is moderately complex.        │
│                                             │
│ Tip: Consider extracting helper functions   │
│ to reduce complexity below 10.              │
└─────────────────────────────────────────────┘
```

**8c: "What does this mean?" Links**

Every data visualization (heatmap, graph, metrics table) should have a small `(?)` icon that opens an explanation overlay:

```svelte
<span class="inline-flex items-center gap-1">
  Fan-in: {metric.fan_in}
  <button class="text-gray-500 hover:text-gray-300" on:click={() => showExplanation('fan-in')}>
    <HelpCircle size={14} />
  </button>
</span>
```

---

## Implementation Roadmap

### Phase 1: Foundations (Week 1)

| Task | Effort | Principle |
|------|--------|-----------|
| Empty state components | 2h | P6 |
| Design tokens (CSS variables) | 1h | P7 |
| Shared component library (6 components) | 4h | P7 |
| Breadcrumbs | 1h | P3 |

### Phase 2: First-Run Experience (Week 2)

| Task | Effort | Principle |
|------|--------|-----------|
| Analysis progress page (WebSocket) | 4h | P1, P2 |
| Post-analysis guided landing | 3h | P1 |
| First-visit tooltips (3 pages) | 2h | P8 |

### Phase 3: Graph Overhaul (Week 3)

| Task | Effort | Principle |
|------|--------|-----------|
| Layered graph (3 levels) | 8h | P4 |
| Community-first default view | 2h | P4 |
| Symbol focus panel | 3h | P4 |

### Phase 4: Learning Polish (Week 4)

| Task | Effort | Principle |
|------|--------|-----------|
| Skip/test-out per chapter | 3h | P5 |
| Persistent progress in sidebar | 2h | P3 |
| Command palette (Ctrl+K) | 4h | P3 |
| Metrics explanation popovers | 2h | P8 |

---

## Success Metrics

| Metric | Current (Estimated) | Target |
|--------|---------------------|--------|
| Time to first insight | >60s (find correct page) | <15s (guided to overview) |
| Graph page bounce rate | High (hairball confusion) | Low (community view is clear) |
| Learning path completion | <5% (no progress tracking UX) | >30% (gamification + skip option) |
| Q&A usage | Low (unclear entry point) | High (contextual "Ask about this" buttons) |
| Error encounters | Frequent (blank pages) | Rare (designed empty states) |

---

## Quick Wins (< 1 hour each)

1. Add `loading` spinners to all API-backed pages
2. Add `error` banners when API calls fail
3. Show community color dots next to symbols in sidebar
4. Add "Copy to clipboard" on code blocks
5. Show symbol kind icons (fn, class, struct) in search results
6. Add keyboard shortcuts: `j/k` to navigate chapters, `?` for help
7. Show "Last analyzed: 2 hours ago" in the sidebar footer
