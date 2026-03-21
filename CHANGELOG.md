# Changelog

All notable changes to Codeilus are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.2.0] - 2026-03-21

### Added

#### Schematic Explorer (`/explore/schematic`)
- Unified tree + graph viewer with mode toggle on a single page
- Tree mode: expandable directory hierarchy with lazy loading via `/api/v1/schematic`
- Graph mode: community grid overview with drill-down into symbol layouts
- 6 interaction components: Tooltip, DetailTabs, SourcePopup, ContextMenu, KeyboardOverlay, Minimap
- Hover: tooltips with node metadata, edge highlighting, ghost-node dimming
- Single click: expand/collapse directories, open 5-tab detail panel (Overview, Source, Relations, Learn, Notes)
- Double click: deep dive (file → tree view, symbol → source popup, community → learning chapter)
- Right-click context menu per node type (View source, Ask AI, Add annotation, Focus, Hide, etc.)
- Keyboard shortcuts: 1/2 modes, F fit-to-view, ? help overlay, Esc close
- Legend panel with edge colors, node types, community swatches, interaction reference
- Minimap with viewport indicator and click-to-pan
- Auto fit-to-view on load, mode switch, and community drill-down
- Search highlighting across all loaded nodes

#### Schematic Backend API
- `GET /api/v1/schematic` — depth-limited tree with community + learning enrichment
- `GET /api/v1/schematic/expand` — lazy-load children of a directory or file
- `GET /api/v1/schematic/detail` — narrative, callers/callees, chapter link for any node
- `SchematicRepo` with community-filtered symbols/edges, dominant community for directories
- Common-prefix path normalization (works with relative and absolute file paths)

#### Theme System
- Light/dark mode toggle with `localStorage` persistence
- CSS variable architecture: `:root:not(.light)` for dark, `.light` for light
- Inline `<script>` in `app.html` prevents flash of wrong theme
- 3D graph canvas background reacts to theme toggle in real-time
- Shiki syntax highlighting switches between `github-dark` and `github-light`
- `theme-change` custom event for components that need to react to toggles

#### E2E Tests
- 31 Playwright tests for schematic explorer (API, tree mode, graph mode, interactions, TDD behaviors)
- Tests are codebase-agnostic (work with any analyzed repository)

#### LLM
- Max subscription routing via `CODEILUS_USE_MAX_SUBSCRIPTION=true` env var
- Sets `CLAUDE_CODE_ENTRYPOINT=sdk-max`, `CLAUDE_USE_SUBSCRIPTION=true`, `CLAUDE_BYPASS_BALANCE_CHECK=true`

### Fixed
- Source code endpoint handles absolute file paths (was returning "path traversal" error)
- Narrative `("name")` prefix stripped from module summaries in all rendering locations
- 3D graph: tooltips, loading screens, community hover cards use CSS variables (light mode support)
- Layered layout wraps rows at 1200px (prevents single-line overflow for large communities)
- Community drill-down filters symbols and edges by community (was loading all 5000+ symbols)
- `$derived` vs `$derived.by` fix in SchematicContextMenu

### Changed
- Explore hub shows "Schematic Explorer" card (replaces separate Tree/Graph Schematic cards)
- Grid layout for community overview (was single horizontal row)
- `computeFitToView` utility in layout engine for viewport calculations

---

## [0.1.0] - 2026-03-17

### Added

#### Parsing & Analysis (`codeilus-parse`, `codeilus-graph`, `codeilus-metrics`, `codeilus-analyze`)
- Tree-sitter parsing for 12 languages with symbol extraction (functions, classes, imports)
- Language-aware import resolution for Rust, TypeScript, and Python
- Incremental parsing with mtime tracking (skip unchanged files)
- Call graph construction, Louvain community detection, entry point identification, process detection
- Semantic community labels derived from module contents
- SLOC, cyclomatic complexity, fan-in/fan-out, and modularity metrics
- Anti-pattern detection: god classes, circular dependencies, and more

#### Diagrams & Narratives (`codeilus-diagram`, `codeilus-narrate`, `codeilus-llm`)
- Mermaid architecture diagrams and flowchart generation
- 8 narrative types with batched module summaries
- Rich data-driven placeholder content (no LLM required for basic narratives)
- Trait-based LLM provider abstraction with Claude Code CLI integration
- Streaming LLM responses with rate limiting and graceful error handling

#### Learning (`codeilus-learn`)
- Curriculum generation from knowledge graph (capped at 15 chapters)
- Quiz generation with multiple question types
- XP, badges, and streak tracking for gamified learning

#### Data & Export (`codeilus-db`, `codeilus-harvest`, `codeilus-export`)
- 13 repository structs (FileRepo, SymbolRepo, ChapterRepo, QuizRepo, etc.)
- Full migration suite including quiz columns and FTS5 search indexes
- BatchWriter with crossbeam channels (50-event / 2-second flush)
- r2d2 connection pool replacing raw DbPool (76 call sites migrated)
- Moka in-memory cache with 10-minute TTL
- GitHub trending scraper (`codeilus-harvest`)
- Static HTML export (`codeilus-export`)

#### MCP Server (`codeilus-mcp`)
- 16-tool MCP server with structured JSON output

#### API (`codeilus-api`)
- 50+ REST endpoints covering files, symbols, graphs, metrics, learning, and harvest
- SSE streaming for long-running analysis and LLM responses
- CORS configuration for local development
- 14 new API integration tests

#### Frontend (SvelteKit 5)
- Graph explorer with 3-level zoom (communities, module, symbol)
- Learning path view with chapter navigation
- Ask AI page with streaming responses
- Metrics dashboard and settings page
- Markdown rendering component
- 6 utility components: OnboardingBanner, HelpTooltip, SystemStatus, Breadcrumbs, CommandPalette, annotations
- Graph rendering fix (force-directed layout, WebGL fallback, proper dimensions)

#### Infrastructure
- `cargo xtask` commands: check, clean, build-frontend, migrate
- Pipeline checkpoint/resume with `--force` flag for re-analysis
- Structured logging with JSON format support
- 14 proptest graph invariants
- Documentation site with mkdocs-material and GitHub Pages deploy

### Changed
- Replaced heuristic regex parsers (Sprint 0) with tree-sitter grammars
- Replaced raw DbPool with r2d2 connection pooling
- Refactored LLM module from concrete implementation to trait-based provider
- Rewrote placeholder system to produce rich data-driven content without LLM calls

### Fixed
- Foreign key constraint error in `clear_analysis_data`
- Graph rendering issues (layout algorithm, WebGL fallback, container dimensions)
- NULL SLOC bug in metrics calculation
- JavaScript parser clippy warnings
- Flaky Louvain community detection test
- EventBus::new signature mismatch from Sprint 0

### Removed
- Heuristic line-by-line regex parsers (replaced by tree-sitter)
- 8 clippy warnings from Sprint 0 parse crate

## [0.0.1] - 2026-03-05

### Added
- 16-crate Rust workspace with shared dependencies
- `codeilus-core`: EventBus (tokio broadcast), 18 event types, 12 error variants, 5 typed ID wrappers, Language/SymbolKind/EdgeKind enums
- `codeilus-db`: DbPool with SQLite WAL mode, Migrator, BatchWriter
- `codeilus-api`: Axum HTTP server with CORS, WebSocket event streaming, rust-embed SPA fallback
- `codeilus-app`: clap CLI with analyze/serve/harvest/export/deploy/mcp subcommands
- `codeilus-parse`: File walker (gitignore-aware), language detection, basic heuristic parsers for 6 languages
- `migrations/0001_init.sql`: 20-table schema (files, symbols, edges, communities, metrics, learning, harvest, events)
- 12 stub crates ready for Wave 1+
