# Schematic Explorer — Design Document

> Unified, lazy-loaded, interactive schematic view that merges file tree, community graph, and learning curriculum into one explorable canvas.

**Status:** Draft
**Date:** 2026-03-19
**Relates to:** NORTH_STAR.md (Mode 1: Interactive Local Server)

---

## 1. Problem Statement

The current implementation has **three disconnected exploration surfaces**:

| View | Route | Shows | Missing |
|------|-------|-------|---------|
| File tree | `/explore/tree` | Directory hierarchy + source | No communities, no learning links |
| Tree schematic | `/explore/schematic/tree` | File tree as SVG diagram | No symbols, no communities |
| Symbol graph | `/explore/schematic/graph` | Communities → symbols (layered) | No file context, no learning links |

**Pain points:**

1. **No unified model.** Each page fetches independently, builds its own layout, shares no state. Navigating between them loses all context.
2. **Eager loading.** Both schematic pages load ALL data upfront (`fetchGraph()` = up to 2000 nodes, `fetchFiles()` = everything). Slow on large repos.
3. **Modal is a dead-end.** Click a node → spinner → text. No way to navigate to related symbols, no "Start learning this" link, no outgoing paths.
4. **Search is inline-only.** Highlights matching nodes but doesn't provide a result list, can't navigate/zoom to a result, no keyboard navigation.
5. **Learning is disconnected.** Chapters live at `/learn`, mapped to communities via `community_id`, but the schematic has zero awareness of learning progress or chapter links.
6. **Layout quality.** Heuristic node sizing (`label.length * 7.5 + 32`), straight-line edges, no edge bundling, no community grouping in tree view.

---

## 2. Design Goals

1. **One page, multiple lenses.** Tree, graph, and mixed modes on a single route with shared state.
2. **Lazy everything.** Initial load is lightweight (top dirs + communities). Children, symbols, narratives, and source load on expand/click.
3. **Community-aware tree.** Files and directories are color-coded by dominant community. Community badges on nodes.
4. **Learning-integrated.** Every community node shows chapter progress. Detail panel links directly to the relevant chapter.
5. **Real search.** Cmd+K modal with results across files, symbols, and communities. Click result → navigate + zoom to node.
6. **Smooth interactions.** Bezier edges, animated expand/collapse, breadcrumb navigation, keyboard shortcuts.

---

## 3. Data Model

### 3.1 SchematicNode

The unified node type that represents directories, files, symbols, and communities in a single tree:

```typescript
interface SchematicNode {
  id: string;                          // "dir:src/lib", "file:42", "sym:108", "comm:5"
  type: "directory" | "file" | "symbol" | "community";
  label: string;                       // Display name
  parent_id: string | null;            // Tree parent (null for root)

  // File/symbol enrichment
  file_id?: number;                    // For file and symbol nodes
  symbol_id?: number;                  // For symbol nodes
  language?: string;                   // For file nodes
  sloc?: number;                       // For file nodes
  kind?: string;                       // For symbol nodes: function, class, struct, etc.
  signature?: string;                  // For symbol nodes

  // Community linkage
  community_id?: number;               // Which community this belongs to
  community_label?: string;            // Human-readable community name
  community_color?: string;            // Hex color for visual grouping

  // Learning linkage
  chapter_id?: number;                 // Linked learning chapter
  chapter_title?: string;              // Chapter display name
  difficulty?: string;                 // beginner, intermediate, advanced
  progress?: {                         // Learning completion
    completed_sections: number;
    total_sections: number;
  };

  // Layout state (client-side only, not persisted)
  collapsed?: boolean;                 // Whether children are hidden
  x?: number;                          // Computed by layout engine
  y?: number;
  width?: number;
  height?: number;
}
```

### 3.2 SchematicEdge

```typescript
interface SchematicEdge {
  id: string;                          // "e:108-209"
  source: string;                      // SchematicNode.id
  target: string;                      // SchematicNode.id
  type: "contains" | "calls" | "imports" | "extends" | "implements";
  confidence?: number;                 // 0.0-1.0
  label?: string;                      // Optional display label
}
```

### 3.3 SchematicGraph (the full client-side model)

```typescript
interface SchematicGraph {
  nodes: Map<string, SchematicNode>;   // Fast lookup by id
  edges: SchematicEdge[];              // All visible edges
  root_ids: string[];                  // Top-level node ids
  communities: CommunityInfo[];        // Community metadata
}

interface CommunityInfo {
  id: number;
  label: string;
  color: string;                       // Generated from HSL space
  cohesion: number;
  member_count: number;
  chapter_id?: number;
  chapter_title?: string;
  progress?: { completed: number; total: number };
}
```

### 3.4 How Existing Data Maps to This Model

```
Database                    →  SchematicNode
─────────────────────────────────────────────
files.path (split by /)     →  type:"directory" (intermediate segments)
files row                   →  type:"file", file_id, language, sloc
symbols row                 →  type:"symbol", symbol_id, kind, signature
communities row             →  type:"community", community_id, cohesion

community_members           →  symbol.community_id, file.community_id (dominant)
chapters                    →  community.chapter_id, chapter_title, difficulty
progress                    →  community.progress.{completed,total}
edges                       →  SchematicEdge with type mapping
```

---

## 4. Backend API

### 4.1 New Endpoint: `GET /api/v1/schematic`

Single endpoint that returns a depth-limited, expandable schematic tree with community and learning enrichment.

**Query parameters:**

| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `depth` | `u32` | `2` | How many directory levels deep to return |
| `expand` | `string` | `""` | Comma-separated node IDs to expand beyond depth limit |
| `community_id` | `i64` | — | Filter: only show files/symbols in this community |
| `include_symbols` | `bool` | `false` | Whether to include symbol nodes inside files |
| `include_edges` | `bool` | `false` | Whether to include cross-reference edges |

**Response:**

```json
{
  "nodes": [
    {
      "id": "dir:src",
      "type": "directory",
      "label": "src",
      "parent_id": "dir:.",
      "has_children": true,
      "child_count": 12,
      "dominant_community_id": 3,
      "dominant_community_color": "#6366f1"
    },
    {
      "id": "file:42",
      "type": "file",
      "label": "parser.rs",
      "parent_id": "dir:src",
      "file_id": 42,
      "language": "rust",
      "sloc": 340,
      "community_id": 3,
      "community_label": "parser_module",
      "community_color": "#6366f1",
      "symbol_count": 15,
      "has_children": true
    },
    {
      "id": "sym:108",
      "type": "symbol",
      "label": "parse_file",
      "parent_id": "file:42",
      "symbol_id": 108,
      "file_id": 42,
      "kind": "function",
      "signature": "pub fn parse_file(path: &Path) -> Result<ParsedFile>",
      "community_id": 3,
      "community_color": "#6366f1",
      "chapter_id": 3,
      "difficulty": "intermediate"
    }
  ],
  "edges": [
    {
      "id": "e:108-209",
      "source": "sym:108",
      "target": "sym:209",
      "type": "calls",
      "confidence": 0.95
    }
  ],
  "communities": [
    {
      "id": 3,
      "label": "parser_module",
      "color": "#6366f1",
      "cohesion": 0.87,
      "member_count": 15,
      "chapter_id": 3,
      "chapter_title": "Chapter 3: The Parser",
      "difficulty": "intermediate",
      "progress": { "completed": 2, "total": 5 }
    }
  ],
  "meta": {
    "total_files": 342,
    "total_symbols": 2847,
    "total_communities": 12,
    "depth_returned": 2
  }
}
```

### 4.2 Lazy Expansion: `GET /api/v1/schematic/expand`

Fetch children of a specific node (used when user clicks to expand).

**Query parameters:**

| Param | Type | Description |
|-------|------|-------------|
| `node_id` | `string` | The node to expand (e.g., `dir:src/lib`, `file:42`) |
| `include_symbols` | `bool` | For directory expansion: also load symbols |
| `include_edges` | `bool` | Include edges between returned nodes |

**Response:** Same shape as `/schematic` but only the children + their edges.

### 4.3 Detail Fetch: `GET /api/v1/schematic/detail`

Fetch rich detail for the detail panel (narrative, source, callers/callees, learning link).

**Query parameters:**

| Param | Type | Description |
|-------|------|-------------|
| `node_id` | `string` | e.g., `file:42` or `sym:108` |
| `include_source` | `bool` | Include source code lines |
| `source_start` | `u32` | Start line for source |
| `source_end` | `u32` | End line for source |

**Response:**

```json
{
  "node_id": "sym:108",
  "narrative": "This function is the main entry point for parsing...",
  "narrative_kind": "symbol_explanation",
  "source": {
    "path": "src/parser.rs",
    "language": "rust",
    "lines": [{ "number": 42, "content": "pub fn parse_file..." }],
    "total_lines": 340
  },
  "callers": [
    { "id": "sym:55", "name": "run_pipeline", "kind": "function", "file_path": "src/main.rs" }
  ],
  "callees": [
    { "id": "sym:209", "name": "extract_symbols", "kind": "function", "file_path": "src/extract.rs" }
  ],
  "chapter": {
    "id": 3,
    "title": "Chapter 3: The Parser",
    "difficulty": "intermediate",
    "progress": { "completed": 2, "total": 5 },
    "next_section_id": 12
  },
  "annotations": [
    { "id": 1, "content": "Key entry point — understand this first", "flagged": true }
  ]
}
```

### 4.4 Existing Endpoints (Unchanged)

These remain for backward compatibility and use by other pages:

- `GET /api/v1/graph` — Raw symbol graph (used by `/explore/graph`)
- `GET /api/v1/graph/communities` — Community-level graph
- `GET /api/v1/communities` — Community list
- `GET /api/v1/files`, `/files/:id/source`, `/files/:id/symbols` — File data
- `GET /api/v1/symbols/search?q=...` — Symbol search
- `GET /api/v1/chapters`, `/chapters/:id` — Learning chapters

---

## 5. Frontend Architecture

### 5.1 Route Structure

```
/explore/schematic/             →  Unified Schematic Explorer (NEW)
/explore/schematic/tree/        →  DEPRECATED (redirect to /explore/schematic?mode=tree)
/explore/schematic/graph/       →  DEPRECATED (redirect to /explore/schematic?mode=graph)
```

### 5.2 Component Hierarchy

```
SchematicExplorer (+page.svelte)
├── SchematicToolbar
│   ├── Breadcrumb trail
│   ├── Mode toggle: [Tree] [Graph] [Mixed]
│   ├── Edge type filters: [Calls] [Imports] [Extends] [Implements]
│   ├── Node count + community count
│   └── Search trigger (Cmd+K)
│
├── SchematicSidebar
│   ├── CommunityList
│   │   ├── Community card (color, label, member count, cohesion bar)
│   │   ├── Chapter progress bar
│   │   └── Click → filter canvas to this community
│   ├── LearnerProgress (XP, streak, badges)
│   └── CollapsibleTreeNav (mini file tree for quick nav)
│
├── SchematicCanvas (main area)
│   ├── SVG layer: edges (bezier curves, colored by type)
│   ├── SVG layer: nodes (positioned by layout engine)
│   │   ├── DirectoryNode — folder icon, expand/collapse chevron
│   │   ├── FileNode — language color dot, sloc badge, community stripe
│   │   ├── SymbolNode — kind badge (FUN/CLS/STR), community color border
│   │   └── CommunityNode — large card with cohesion, member count, progress
│   ├── Pan/zoom/drag handler
│   └── Viewport culling (only render visible nodes)
│
├── SchematicDetail (slide-in panel, right side)
│   ├── Header: node name, type badge, file path
│   ├── Tab: Explain — narrative content
│   ├── Tab: Source — syntax-highlighted code (lazy-loaded, line range)
│   ├── Tab: Relations — callers, callees, imports (clickable → navigate)
│   ├── Tab: Learn — chapter link, section list, "Start Learning" button
│   └── Annotations section (add/edit/flag)
│
└── SchematicSearch (Cmd+K modal overlay)
    ├── Input with debounced search (calls /symbols/search + local filter)
    ├── Result groups: [Files] [Symbols] [Communities]
    ├── Keyboard navigation (↑↓ to select, Enter to go)
    └── Action: navigate canvas to node + open detail panel
```

### 5.3 State Management

Single Svelte `$state` store, shared across all components:

```typescript
// schematic-store.svelte.ts

interface SchematicStore {
  // Data
  graph: SchematicGraph;                   // All loaded nodes + edges
  communities: CommunityInfo[];            // Community metadata

  // View state
  mode: "tree" | "graph" | "mixed";        // Layout mode
  selectedNodeId: string | null;           // Currently selected node
  expandedNodes: Set<string>;              // Which nodes are expanded
  visibleEdgeTypes: Set<string>;           // Which edge types to show
  filteredCommunityId: number | null;      // Community filter (null = all)

  // Layout output
  layoutNodes: LayoutNode[];               // Positioned nodes (from layout engine)
  layoutEdges: LayoutEdge[];               // Positioned edges
  canvasWidth: number;
  canvasHeight: number;

  // Viewport
  panX: number;
  panY: number;
  scale: number;

  // Detail panel
  detailOpen: boolean;
  detailData: DetailData | null;
  detailLoading: boolean;

  // Search
  searchOpen: boolean;
  searchQuery: string;
  searchResults: SearchResult[];

  // Loading
  initialLoading: boolean;
  expandLoading: Set<string>;              // Nodes currently being expanded
}
```

### 5.4 Layout Modes

#### Tree Mode
- Right-flowing tree layout (like current `layoutTree`)
- Directories → files → (symbols if expanded)
- Community color as left border stripe on each node
- `contains` edges shown as tree lines (bezier curves)
- Cross-reference edges (calls/imports) shown as dashed arcs

#### Graph Mode
- Community-centric layout
- Top level: community nodes (large cards)
- Drill into community: symbols laid out with topological layering
- All edge types visible, colored by type
- No file/directory structure shown

#### Mixed Mode (the key innovation)
- Tree layout as the backbone (directory → file → symbol)
- Community grouping overlaid: files in the same community get a shared background region (convex hull)
- Cross-reference edges drawn as arcs between symbols across communities
- Best of both worlds: you see WHERE things are (tree) AND how they relate (graph)

### 5.5 Lazy Loading Flow

```
Page load
  │
  ├─ GET /api/v1/schematic?depth=2
  │   Returns: top dirs, files at depth ≤ 2, communities, chapter progress
  │   Cost: ~50-100 nodes (lightweight)
  │
  ├─ Run layout engine → render initial canvas
  │
  User clicks directory node "src/lib" (expand)
  │
  ├─ GET /api/v1/schematic/expand?node_id=dir:src/lib
  │   Returns: children of src/lib (files + subdirs)
  │   Merge into graph store → re-layout → animate expansion
  │
  User clicks file node "parser.rs" (expand)
  │
  ├─ GET /api/v1/schematic/expand?node_id=file:42&include_symbols=true&include_edges=true
  │   Returns: symbols inside parser.rs + edges between them
  │   Merge into graph store → re-layout → animate expansion
  │
  User clicks symbol node "parse_file" (select)
  │
  ├─ GET /api/v1/schematic/detail?node_id=sym:108&include_source=true
  │   Returns: narrative, source, callers, callees, chapter link
  │   Open detail panel → show tabs
  │
  User clicks caller "run_pipeline" in detail panel
  │
  ├─ Ensure sym:55 exists in graph (expand its file if needed)
  ├─ Pan/zoom canvas to sym:55
  └─ Open detail for sym:55
```

### 5.6 Search Flow

```
User presses Cmd+K
  │
  ├─ Open search modal with focus on input
  │
  User types "parse"
  │
  ├─ Debounce 200ms
  ├─ Local filter: search loaded nodes in graph store
  ├─ Remote: GET /api/v1/symbols/search?q=parse (if local results < 5)
  ├─ Combine + deduplicate → show grouped results
  │
  User selects "parse_file" with ↓ + Enter
  │
  ├─ Close search modal
  ├─ If node not in graph → expand its parent chain (lazy load)
  ├─ Pan/zoom canvas to center on node
  ├─ Highlight node with pulse animation
  └─ Open detail panel for node
```

---

## 6. Visual Design

### 6.1 Node Styles

```
Directory Node:
┌─────────────────┐
│ 📂 src/lib      │  ← muted border, expandable chevron
│    12 files      │
└─────────────────┘

File Node:
┌─────────────────┐
│● parser.rs      │  ← ● = language color dot
│  rust · 340 loc │     left border = community color
│  ■■■■■□□ 5/7   │  ← optional: chapter progress mini-bar
└─────────────────┘

Symbol Node:
┌─────────────────┐
│ FUN parse_file  │  ← kind badge + name
│  L42-L98        │     border color = community
└─────────────────┘

Community Node (graph mode):
┌───────────────────────┐
│  ● parser_module      │  ← ● = community color
│  15 symbols           │
│  cohesion: 0.87       │
│  ■■□□□ 2/5 learned   │  ← chapter progress
│  [→ Start Chapter 3]  │  ← learning link
└───────────────────────┘
```

### 6.2 Edge Styles

| Type | Color | Style | Arrowhead |
|------|-------|-------|-----------|
| contains | `var(--c-border)` | Solid, thin | None |
| calls | `#6366f1` (indigo) | Solid | Triangle |
| imports | `#14b8a6` (teal) | Dashed | Triangle |
| extends | `#f59e0b` (amber) | Solid, thick | Diamond |
| implements | `#ec4899` (pink) | Dotted | Circle |

### 6.3 Community Colors

Generated from HSL space to ensure distinctness:

```typescript
function communityColor(id: number, total: number): string {
  const hue = (id * 360 / Math.max(total, 1)) % 360;
  return `hsl(${hue}, 65%, 55%)`;
}
```

### 6.4 Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Cmd+K` / `Ctrl+K` | Open search |
| `Escape` | Close search / detail panel |
| `1` / `2` / `3` | Switch mode: tree / graph / mixed |
| `+` / `-` | Zoom in / out |
| `0` | Fit all nodes in viewport |
| `←` / `→` | Collapse / expand selected node |
| `Tab` | Cycle through detail panel tabs |
| `Enter` | Open detail for selected node |

---

## 7. Implementation Plan

### Phase 1: Backend — Unified Data Endpoint

**Files to create/modify:**

| File | Action |
|------|--------|
| `crates/codeilus-api/src/routes/schematic.rs` | NEW — `/schematic`, `/schematic/expand`, `/schematic/detail` |
| `crates/codeilus-api/src/routes/mod.rs` | Add schematic routes |
| `crates/codeilus-db/src/repos/schematic_repo.rs` | NEW — Queries joining files, symbols, communities, chapters, progress |
| `crates/codeilus-db/src/repos/mod.rs` | Add schematic repo |
| `crates/codeilus-core/src/types.rs` | Add `SchematicNode`, `SchematicEdge`, `SchematicResponse` types |

**Key implementation details:**

- Directory tree built by splitting file paths on `/` and grouping — same as frontend `buildDirTree()` but server-side
- `depth` parameter controls how many levels to return — BFS with depth counter
- `dominant_community_id` for directories: most common community among child files' symbols
- Community colors assigned once, returned in response so all clients see the same colors
- Chapter progress computed by joining `chapters` → `chapter_sections` → `progress`

**Estimated scope:** ~400 lines Rust

### Phase 2: Frontend — Core Components

**Files to create/modify:**

| File | Action |
|------|--------|
| `frontend/src/lib/schematic/types.ts` | NEW — SchematicNode, SchematicEdge, SchematicGraph interfaces |
| `frontend/src/lib/schematic/store.svelte.ts` | NEW — Centralized state with lazy-load actions |
| `frontend/src/lib/schematic/layout.ts` | REWRITE — Add mixed mode, community grouping, bezier edges |
| `frontend/src/lib/schematic/SchematicCanvas.svelte` | NEW — SVG canvas with pan/zoom, viewport culling |
| `frontend/src/lib/schematic/SchematicToolbar.svelte` | NEW — Mode toggle, edge filters, breadcrumb |
| `frontend/src/lib/schematic/SchematicSidebar.svelte` | NEW — Community list, progress, mini-tree |
| `frontend/src/lib/schematic/SchematicDetail.svelte` | NEW — Tabbed detail panel (explain, source, relations, learn) |
| `frontend/src/lib/schematic/SchematicSearch.svelte` | NEW — Cmd+K search modal |
| `frontend/src/lib/schematic/nodes/DirectoryNode.svelte` | NEW — Directory node renderer |
| `frontend/src/lib/schematic/nodes/FileNode.svelte` | NEW — File node renderer |
| `frontend/src/lib/schematic/nodes/SymbolNode.svelte` | NEW — Symbol node renderer |
| `frontend/src/lib/schematic/nodes/CommunityNode.svelte` | NEW — Community node renderer |
| `frontend/src/routes/explore/schematic/+page.svelte` | NEW — Unified explorer page |
| `frontend/src/lib/api.ts` | ADD — `fetchSchematic()`, `fetchSchematicExpand()`, `fetchSchematicDetail()` |
| `frontend/src/lib/types.ts` | ADD — Schematic-specific types |

**Estimated scope:** ~1200 lines Svelte/TS

### Phase 3: Polish

| Feature | Details |
|---------|---------|
| Animated expand/collapse | CSS transitions on node position changes |
| Viewport culling | Only render nodes within visible SVG bounds |
| Edge bundling | Group parallel edges between same communities |
| Breadcrumb trail | Track navigation path with clickable segments |
| Keyboard navigation | Full keyboard support per section 6.4 |
| URL state sync | `?mode=mixed&community=3&selected=sym:108` in URL |
| Responsive layout | Sidebar collapses on narrow screens |
| Loading skeletons | Placeholder nodes while expanding |

---

## 8. Migration Strategy

1. Build the new `/explore/schematic` page alongside existing pages
2. Add redirect from `/explore/schematic/tree` and `/explore/schematic/graph` to new page
3. Remove old pages and dead `layout.ts` exports once stable
4. Keep existing `/explore/graph` (force-directed 3D graph) unchanged — different use case

---

## 9. Performance Considerations

### Backend
- **Depth-limited queries** prevent full-table scans
- **Community assignment** cached in moka (already exists)
- **Progress aggregation** via SQL `GROUP BY` instead of N+1
- **Response size**: depth=2 on a 342-file repo returns ~80 nodes (~5KB JSON)

### Frontend
- **Viewport culling**: Only render SVG elements within the visible bounds + 200px margin
- **Layout caching**: Re-layout only when nodes are added/removed, not on pan/zoom
- **Debounced search**: 200ms debounce on keystroke, local-first then remote
- **Edge rendering**: Skip edges where both endpoints are outside viewport
- **Node pooling**: Reuse SVG `<g>` elements during re-layout instead of destroying/recreating

### Scaling Targets
| Metric | Target |
|--------|--------|
| Initial load (depth=2) | < 200ms |
| Expand a directory | < 100ms (fetch + layout + render) |
| Open detail panel | < 300ms (narrative + source fetch) |
| Pan/zoom at 500 nodes | 60fps |
| Search results | < 150ms |

---

## 10. Open Questions

1. **Canvas vs SVG for large graphs?** SVG is simpler but degrades past ~1000 nodes. Could use Canvas for rendering + invisible SVG for hit testing. Decide after measuring real-world repos.

2. **Community convex hull in mixed mode?** Drawing a shaded region around files in the same community looks great but is computationally expensive (convex hull + SVG path). Could simplify to a colored background band instead.

3. **How to handle files in multiple communities?** A file may contain symbols from different communities. Options: (a) assign file to dominant community, (b) show multiple color stripes, (c) split file node into symbol-level nodes.

4. **Offline narrative generation?** Detail panel calls `/narratives/:kind/:target_id` which may return a placeholder if LLM hasn't generated content yet. Should we show a "Generate now" button or queue it automatically?

5. **Process flows integration?** The `processes` table has execution flows (BFS from entry points). Could overlay these as "guided paths" on the schematic — "Follow the request lifecycle: route → handler → service → db". Defer to Phase 3+.
