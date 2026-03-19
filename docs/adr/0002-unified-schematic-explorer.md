# ADR-0002: Unified Schematic Explorer (Replace Split Tree/Graph Views)

**Status:** Proposed
**Date:** 2026-03-19
**Decider:** Human (pending)

## Context

The schematic exploration feature currently has two separate pages:

- `/explore/schematic/tree` — Renders the file system as a right-flowing SVG tree diagram. Clicking a file opens a modal with symbols, narrative, and source preview. No community awareness, no learning links.
- `/explore/schematic/graph` — Renders communities as layered nodes, drills into symbols. No file context, no learning links, no connection to the tree.

Both pages share a layout library (`layout.ts`) but otherwise have no shared state, no shared components, and no shared data model. They were recently simplified by removing ELK.js and inlining canvas/modal/search components, leaving 159 lines of layout code and ~300 lines per page.

**Problems this creates:**

1. Users must choose between "where is it?" (tree) and "how does it connect?" (graph) — they can't see both.
2. Both pages load all data upfront. On large repos (2000+ symbols), initial load is slow and most data is never viewed.
3. The modal is a dead-end: no navigation to related symbols, no links to learning chapters, no way to follow call chains.
4. Search highlights nodes but can't navigate to them or show a result list.
5. Learning chapters (mapped to communities via `community_id`) are completely invisible in the schematic — a user exploring the graph has no idea that a curated learning path exists for the cluster they're looking at.

## Decision

Replace both schematic pages with a single **Unified Schematic Explorer** at `/explore/schematic/` that:

1. **Merges tree and graph** via three switchable modes (tree, graph, mixed) sharing one data store.
2. **Lazy-loads** via a new backend endpoint (`GET /api/v1/schematic`) that returns depth-limited results with on-demand expansion.
3. **Enriches every node** with community assignment, learning chapter link, and progress.
4. **Provides a rich detail panel** with tabs (explain, source, relations, learn) instead of a basic modal.
5. **Adds Cmd+K search** with navigation to results on the canvas.

Full design: `docs/SCHEMATIC_DESIGN.md`

## Alternatives Considered

### 1. Improve the two pages independently
Add community colors to the tree page, add file context to the graph page, add search to both.

**Rejected.** Duplicates effort, still no shared state. User still must choose one view. Lazy loading would need to be implemented twice.

### 2. Use a third-party graph library (Cytoscape, D3-force, vis.js)
Replace custom SVG rendering with a mature graph visualization library.

**Rejected for now.** These libraries are general-purpose and heavy (100-300KB). Our nodes have very specific rendering needs (community stripes, progress bars, kind badges). Custom SVG gives us full control. Could reconsider for Canvas rendering if SVG degrades past 1000 nodes.

### 3. Embed the graph inside the existing file tree page
Add an optional graph overlay to `/explore/tree` (the sidebar + code viewer page).

**Rejected.** The tree page is a code browser (sidebar + source). The schematic is a canvas-based diagram. Different interaction paradigms — merging them would compromise both.

### 4. Keep ELK.js for layout
Re-add ELK.js with lazy loading to get sophisticated layout quality.

**Deferred.** The custom layout algorithms work well for the current node count. ELK.js adds 1.5MB to the bundle. Can lazy-import it later if layout quality becomes a bottleneck on complex repos.

## Consequences

**Positive:**
- One page to learn instead of two — simpler mental model for users
- Lazy loading means fast initial paint on any repo size
- Community → learning path link makes the schematic a gateway to the learning experience (core product value)
- Shared components (canvas, detail panel, search) reduce total frontend code
- Server-side tree+community joining enables future features (community heatmap, progress map) without new endpoints

**Negative:**
- Requires a new backend endpoint and repository (schematic_repo)
- More complex frontend state management (single store with lazy-load actions)
- Mixed mode layout (tree + community grouping) is non-trivial to implement well
- Two old pages need redirects during migration period
- Risk of over-engineering if the mixed mode proves confusing to users

**Risks & Mitigations:**
- **Mixed mode too complex?** Ship tree and graph modes first. Add mixed mode behind a toggle. User-test before making it default.
- **Backend query performance?** The schematic endpoint joins 5 tables. Mitigate with depth limits, indexes (already added in `0010_add_indexes.sql`), and moka caching.
- **SVG performance at scale?** Add viewport culling (only render visible nodes). If insufficient, migrate rendering to Canvas in a follow-up.
