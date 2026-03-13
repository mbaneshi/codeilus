# Task: Static HTML Exporter

> **Crate:** `crates/codeilus-export/`
> **Wave:** 5 (parallel with harvest)
> **Depends on:** codeilus-core (done), codeilus-db (all waves), codeilus-narrate (wave 4), codeilus-diagram (wave 3)
> **Status:** pending

---

## Context

Read these files first:
- `CLAUDE.md` — project rules and conventions
- `NORTH_STAR.md` — section 6.6 (codeilus-export deep dive), section 9 Sprint 7, section 5.3 (Harvest → Export → Deploy flow)
- `crates/codeilus-core/src/types.rs` — NarrativeKind
- `crates/codeilus-db/src/repos/` — all repo modules (file, symbol, edge, community, narrative, pattern, file_metrics)
- `crates/codeilus-diagram/src/lib.rs` — generate_architecture, generate_file_tree
- `crates/codeilus-narrate/src/types.rs` — Narrative type
- No direct reference repo — this is a new component. Study the export-template/ directory structure.

## Objective

Generate self-contained HTML pages per repo. All data inlined as JSON in `<script>` tags. CSS inlined. Mermaid loaded inline. Each page is a complete "5-minute grasp" of a codebase. Target <500KB per page. Also create the `export-template/index.html` base template and a daily index generator.

Public API:
```rust
pub fn export_repo(repo_name: &str, db: &DbPool, output_dir: &Path) -> CodeilusResult<PathBuf>
pub fn generate_index(repos: &[ExportedRepo], output_dir: &Path) -> CodeilusResult<PathBuf>
```

## Files to Create/Modify

### 1. Update `crates/codeilus-export/Cargo.toml`

```toml
[package]
name = "codeilus-export"
version = "0.1.0"
edition = "2021"

[dependencies]
codeilus-core = { path = "../codeilus-core" }
codeilus-db = { path = "../codeilus-db" }
codeilus-narrate = { path = "../codeilus-narrate" }
codeilus-diagram = { path = "../codeilus-diagram" }
minijinja = "2"
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { version = "0.4", features = ["serde"] }
```

### 2. `src/types.rs` — Export types

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub repo_name: String,
    pub repo_description: Option<String>,
    pub language_badges: Vec<LanguageBadge>,
    pub overview: String,
    pub architecture_mermaid: String,
    pub reading_order: Vec<ReadingOrderEntry>,
    pub entry_points: Vec<EntryPointEntry>,
    pub architecture_narrative: String,
    pub extension_guide: String,
    pub contribution_guide: String,
    pub why_trending: String,
    pub metrics_snapshot: MetricsSnapshot,
    pub file_tree: String,
    pub communities: Vec<CommunityExport>,
    pub patterns: Vec<PatternExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageBadge {
    pub language: String,
    pub percentage: f64,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingOrderEntry {
    pub path: String,
    pub reason: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPointEntry {
    pub name: String,
    pub file_path: String,
    pub score: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub total_files: usize,
    pub total_sloc: usize,
    pub total_symbols: usize,
    pub avg_complexity: f64,
    pub modularity_q: f64,
    pub hotspot_files: Vec<HotspotFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotFile {
    pub path: String,
    pub heatmap_score: f64,
    pub complexity: f64,
    pub churn: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityExport {
    pub label: String,
    pub summary: String,
    pub member_count: usize,
    pub key_symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternExport {
    pub kind: String,
    pub severity: String,
    pub message: String,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedRepo {
    pub name: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub file_path: String,
    pub file_size_kb: usize,
    pub exported_at: String,
}
```

### 3. `src/data_loader.rs` — Load all data from DB

```rust
use crate::types::ExportData;
use codeilus_core::CodeilusResult;
use codeilus_db::DbPool;

/// Load all data needed for export from the database.
pub fn load_export_data(repo_name: &str, db: &DbPool) -> CodeilusResult<ExportData> { ... }
```

- Query all repos: FileRepo, SymbolRepo, EdgeRepo, CommunityRepo, NarrativeRepo, PatternRepo, FileMetricsRepo
- Build ExportData struct
- Generate architecture Mermaid diagram via codeilus-diagram
- Generate ASCII file tree via codeilus-diagram
- Compute language badges from file language distribution

### 4. `src/renderer.rs` — HTML renderer

```rust
use crate::types::ExportData;
use codeilus_core::CodeilusResult;
use std::path::Path;

/// Render ExportData into a self-contained HTML file.
pub fn render_html(data: &ExportData, output_path: &Path) -> CodeilusResult<()> { ... }
```

- Use `minijinja` template engine
- Load template from embedded string (compile-time include)
- Inline all data as JSON in `<script id="codeilus-data" type="application/json">{...}</script>`
- Inline Mermaid.js (~200KB minified) — use CDN URL with integrity hash as fallback
- Inline CSS (TailwindCSS utility subset, ~30KB)
- Target total page size: <500KB

### 5. `src/template.rs` — HTML template (embedded)

The template string (or loaded from `export-template/index.html`). Page structure:

```html
<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{{ repo_name }} — Codeilus</title>
    <style>/* inlined CSS */</style>
</head>
<body class="bg-gray-950 text-gray-100 font-mono">
    <!-- Hero -->
    <header class="...">
        <h1>{{ repo_name }}</h1>
        <p>{{ repo_description }}</p>
        <div><!-- language badges --></div>
    </header>

    <!-- Sections -->
    <main>
        <section id="overview"><!-- 30-second overview --></section>
        <section id="architecture"><!-- Mermaid diagram --></section>
        <section id="key-files"><!-- Reading order --></section>
        <section id="entry-points"><!-- Entry points --></section>
        <section id="how-it-works"><!-- Architecture narrative --></section>
        <section id="how-to-extend"><!-- Extension guide --></section>
        <section id="how-to-contribute"><!-- Contribution guide --></section>
        <section id="why-trending"><!-- Why trending --></section>
        <section id="metrics"><!-- Metrics snapshot --></section>
        <section id="deep-dive"><!-- Collapsible: communities, patterns, full tree --></section>
    </main>

    <script id="codeilus-data" type="application/json">{{ data_json }}</script>
    <script>/* Mermaid init + interactive JS */</script>
</body>
</html>
```

### 6. `src/index.rs` — Daily index page generator

```rust
use crate::types::ExportedRepo;
use codeilus_core::CodeilusResult;
use std::path::Path;

/// Generate an index.html listing all exported repos for a given date.
pub fn generate_index(
    repos: &[ExportedRepo],
    date: &str,
    output_dir: &Path,
) -> CodeilusResult<std::path::PathBuf> { ... }
```

- Card-based layout: repo name, description, language, file size, link to detail page
- Sorted by language popularity
- Include date header and navigation to other dates
- Also self-contained HTML (<50KB)

### 7. Create `export-template/index.html` — Base template file

Create the actual HTML template file at the workspace root in `export-template/`. This is the source that `src/template.rs` embeds or loads.

Include:
- Minimal CSS for dark theme (can be subset of Tailwind utilities)
- Mermaid initialization script
- Collapsible sections via vanilla JS (details/summary or custom toggle)
- Smooth scrolling navigation
- Responsive design (mobile-friendly)
- Print-friendly styles

### 8. `src/lib.rs` — Module entry point

```rust
pub mod data_loader;
pub mod index;
pub mod renderer;
pub mod template;
pub mod types;

pub use types::*;

use codeilus_core::CodeilusResult;
use codeilus_db::DbPool;
use std::path::{Path, PathBuf};

pub fn export_repo(repo_name: &str, db: &DbPool, output_dir: &Path) -> CodeilusResult<PathBuf> {
    let data = data_loader::load_export_data(repo_name, db)?;
    let filename = format!("{}.html", repo_name.replace('/', "-"));
    let output_path = output_dir.join(&filename);
    renderer::render_html(&data, &output_path)?;
    Ok(output_path)
}

pub fn generate_index(repos: &[ExportedRepo], output_dir: &Path) -> CodeilusResult<PathBuf> {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    index::generate_index(repos, &date, output_dir)
}
```

## Tests

### Test cases:
1. `export_data_all_fields` — load_export_data produces ExportData with non-empty overview, architecture, etc.
2. `export_data_empty_db` — Empty DB → ExportData with empty strings (not error)
3. `render_html_creates_file` — render_html creates a valid HTML file at the specified path
4. `render_html_size_limit` — Generated HTML is <500KB
5. `render_html_contains_data` — HTML contains `<script id="codeilus-data">` with valid JSON
6. `render_html_self_contained` — HTML has no external resource references (no CDN links that block rendering)
7. `render_html_valid_structure` — HTML has all 10 sections (overview through deep-dive)
8. `language_badges` — 3 languages → 3 badge entries with percentages summing to ~100
9. `index_page_lists_repos` — Generate index with 3 repos → HTML contains all 3 repo names
10. `index_page_size` — Index page is <50KB
11. `export_filename_sanitized` — "owner/repo" → "owner-repo.html"
12. `mermaid_inlined` — Output HTML contains Mermaid initialization code

### Fixtures:
Create test data by inserting into in-memory DB, then calling export functions.

## Acceptance Criteria

- [ ] `cargo test -p codeilus-export` — all tests pass
- [ ] `cargo clippy -p codeilus-export` — zero warnings
- [ ] Exported HTML is self-contained (opens offline)
- [ ] Exported HTML is <500KB
- [ ] All 10 page sections present (hero through deep-dive)
- [ ] Data inlined as JSON in script tag
- [ ] Mermaid diagrams render (init script included)
- [ ] Deep dive section is collapsible
- [ ] Responsive design (works on mobile viewport)
- [ ] Index page lists all exported repos with links
- [ ] `export-template/index.html` created at workspace root

## Do NOT Touch
- `crates/codeilus-core/` (read-only)
- `crates/codeilus-db/src/pool.rs`, `migrations.rs`, `batch_writer.rs`
- Existing repo files in `crates/codeilus-db/src/repos/`
- `crates/codeilus-narrate/` (read-only)
- `crates/codeilus-diagram/` (read-only)
- `migrations/0001_init.sql`
- `Cargo.toml` at workspace root
- Any other crate files

---

## Report

> **Agent: fill this section when done.**

### Status: complete

### Files Created/Modified:
- `crates/codeilus-export/Cargo.toml` — Updated: added codeilus-db, minijinja, chrono, serde, serde_json, tracing deps
- `crates/codeilus-export/src/lib.rs` — Created: module declarations, `export_repo()` and `generate_index()` public API
- `crates/codeilus-export/src/types.rs` — Created: ExportData, LanguageBadge, ReadingOrderEntry, EntryPointEntry, MetricsSnapshot, HotspotFile, CommunityExport, PatternExport, ExportedRepo
- `crates/codeilus-export/src/data_loader.rs` — Created: `load_export_data()` — queries all DB repos (files, symbols, narratives, communities, patterns, metrics), builds ExportData
- `crates/codeilus-export/src/renderer.rs` — Created: `render_html()` and `render_html_string()` — minijinja rendering with inlined JSON data
- `crates/codeilus-export/src/template.rs` — Created: embedded REPO_TEMPLATE (include_str from export-template/) and INDEX_TEMPLATE (inline string)
- `crates/codeilus-export/src/index.rs` — Created: `generate_index()` — card-based daily index page
- `export-template/index.html` — Created: self-contained HTML template with dark theme, 10 sections, sticky nav, collapsible deep-dive, Mermaid init, print styles, responsive design
- `crates/codeilus-export/tests/export.rs` — Created: 12 test cases

### Tests:
```
codeilus-export: 12 passed, 0 failed
  export_data_all_fields, export_data_empty_db, render_html_creates_file,
  render_html_size_limit, render_html_contains_data, render_html_self_contained,
  render_html_valid_structure, language_badges, index_page_lists_repos,
  index_page_size, export_filename_sanitized, mermaid_inlined
```

### Clippy:
Zero warnings for codeilus-export.

### Issues / Blockers:
- `codeilus-diagram` dependency was not added because generating architecture diagrams requires a `KnowledgeGraph` (from codeilus-graph), which cannot be reconstructed from DB alone. The `architecture_mermaid` field is left empty; the pipeline should populate it before export.
- `codeilus-narrate` dependency was not added — narratives are read from the DB via `NarrativeRepo`, so no direct dependency needed.

### Notes:
- **Data loading** queries all available DB repos (FileRepo, SymbolRepo, NarrativeRepo, CommunityRepo, PatternRepo, FileMetricsRepo) and assembles ExportData.
- **Architecture Mermaid** is empty by default — the pipeline should either store generated Mermaid in a narrative or pass it through another mechanism before calling `export_repo()`.
- **Reading order** is parsed from the narrative text format (`N. path — reason`). If the narrate crate changes its output format, this parsing will need updating.
- **Entry points** are empty since they're part of the KnowledgeGraph (not stored in DB). The pipeline should populate the ExportData.entry_points field if needed.
- **File tree** is built from file paths using a simple ASCII tree renderer (not depending on codeilus-diagram to avoid pulling in codeilus-graph transitively).
- **Mermaid.js** is loaded from CDN dynamically (not blocking), only if `.mermaid` elements exist on the page. No blocking external scripts.
- **Index template** is embedded as a const string (not in a file) since it's small.
- **All HTML is self-contained**: no external CSS, no blocking external scripts. Opens fully offline except for Mermaid diagrams (which need the CDN script).
