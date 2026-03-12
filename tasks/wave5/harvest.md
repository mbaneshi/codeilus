# Task: GitHub Trending Harvester

> **Crate:** `crates/codeilus-harvest/`
> **Wave:** 5 (parallel with export)
> **Depends on:** codeilus-core (done), codeilus-db (wave 1+2+3+4)
> **Status:** pending

---

## Context

Read these files first:
- `CLAUDE.md` — project rules and conventions
- `NORTH_STAR.md` — section 9, Sprint 7 harvest deliverables, section 5.3 (Harvest → Export → Deploy flow)
- `crates/codeilus-core/src/error.rs` — CodeilusError::Harvest variant
- `crates/codeilus-core/src/events.rs` — CodeilusEvent::HarvestRepoFound variant
- No reference repos for harvesting — this is a new component. Use reqwest + scraper for HTML scraping.

## Objective

Scrape GitHub trending page for repos, shallow-clone them, fingerprint to skip already-analyzed repos, and queue them for analysis. This is the entry point for the automated daily publishing pipeline.

Public API:
```rust
pub async fn harvest_trending(config: HarvestConfig) -> CodeilusResult<Vec<HarvestedRepo>>
pub async fn clone_repo(repo: &HarvestedRepo, dest: &Path) -> CodeilusResult<PathBuf>
```

## Files to Create/Modify

### 1. Update `crates/codeilus-harvest/Cargo.toml`

```toml
[package]
name = "codeilus-harvest"
version = "0.1.0"
edition = "2021"

[dependencies]
codeilus-core = { path = "../codeilus-core" }
codeilus-db = { path = "../codeilus-db" }
reqwest = { version = "0.12", features = ["rustls-tls"] }
scraper = "0.21"
tokio = { workspace = true, features = ["process", "sync"] }
git2 = "0.19"
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
sha2 = "0.10"
```

### 2. `src/types.rs` — Harvest types

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct HarvestConfig {
    pub language: Option<String>,       // filter by language (e.g., "rust", "python")
    pub since: TrendingSince,           // daily, weekly, monthly
    pub max_repos: usize,              // limit number of repos (default 25)
    pub clone_dir: std::path::PathBuf, // directory for shallow clones
    pub concurrent_clones: usize,      // max parallel clones (default 5)
}

impl Default for HarvestConfig {
    fn default() -> Self {
        Self {
            language: None,
            since: TrendingSince::Daily,
            max_repos: 25,
            clone_dir: std::path::PathBuf::from("./clones"),
            concurrent_clones: 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendingSince {
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvestedRepo {
    pub owner: String,
    pub name: String,
    pub full_name: String,          // "owner/name"
    pub description: Option<String>,
    pub language: Option<String>,
    pub stars_today: Option<usize>,
    pub total_stars: Option<usize>,
    pub url: String,
    pub clone_url: String,
    pub fingerprint: String,        // SHA256 of "owner/name@latest_commit"
    pub status: HarvestStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HarvestStatus {
    Found,
    Cloning,
    Cloned,
    Analyzing,
    Complete,
    Failed,
    Skipped,    // already analyzed (fingerprint match)
}
```

### 3. `src/scraper.rs` — GitHub trending page scraper

- Fetch `https://github.com/trending` (or `https://github.com/trending/{language}?since={daily|weekly|monthly}`)
- Parse HTML with `scraper` crate
- Extract from each repo row (`article.Box-row`):
  - Owner + name from `h2 a` href
  - Description from `p.col-9`
  - Language from `span[itemprop="programmingLanguage"]`
  - Stars today from `.float-sm-right` text
  - Total stars from `a.Link--muted` with star SVG
- Build `Vec<HarvestedRepo>` (status = Found)
- Handle: rate limiting (retry with backoff), page structure changes (log warnings)

```rust
pub async fn scrape_trending(config: &HarvestConfig) -> CodeilusResult<Vec<HarvestedRepo>> { ... }
```

Note: GitHub trending HTML structure may change. Be defensive with CSS selectors and log warnings for missing fields rather than failing.

### 4. `src/cloner.rs` — Shallow clone queue

- Clone repos with `git clone --depth=1 <url> <dest>` via git2-rs or tokio subprocess
- Concurrency control: `tokio::sync::Semaphore` with `concurrent_clones` permits
- Clone to `{clone_dir}/{owner}-{name}/`
- Update repo status: Cloning → Cloned (or Failed)
- Clean up clone dir on failure

```rust
pub struct CloneQueue {
    semaphore: Arc<tokio::sync::Semaphore>,
    clone_dir: PathBuf,
}

impl CloneQueue {
    pub fn new(clone_dir: PathBuf, max_concurrent: usize) -> Self;

    /// Clone a single repo. Returns the local path.
    pub async fn clone_repo(&self, repo: &HarvestedRepo) -> CodeilusResult<PathBuf>;

    /// Clone all repos in parallel (respecting semaphore).
    pub async fn clone_all(&self, repos: &mut [HarvestedRepo]) -> CodeilusResult<Vec<PathBuf>>;
}
```

### 5. `src/fingerprint.rs` — Repo fingerprinting

- Generate fingerprint: SHA256 hash of `"{owner}/{name}@{latest_commit_hash}"`
- Get latest commit hash from the cloned repo via git2-rs: `repo.head()?.peel_to_commit()?.id()`
- Compare against stored fingerprints in DB
- If fingerprint matches → mark as Skipped (already analyzed)
- If new → proceed with analysis

```rust
pub fn compute_fingerprint(owner: &str, name: &str, repo_path: &Path) -> CodeilusResult<String> { ... }
pub fn is_already_analyzed(fingerprint: &str, db: &HarvestRepo) -> CodeilusResult<bool> { ... }
```

### 6. `src/lib.rs` — Module entry point

```rust
pub mod cloner;
pub mod fingerprint;
pub mod scraper;
pub mod types;

pub use types::*;

use codeilus_core::CodeilusResult;
use std::path::Path;

pub async fn harvest_trending(config: HarvestConfig) -> CodeilusResult<Vec<HarvestedRepo>> {
    let mut repos = scraper::scrape_trending(&config).await?;

    let queue = cloner::CloneQueue::new(config.clone_dir.clone(), config.concurrent_clones);

    // Fingerprint check and clone
    for repo in &mut repos {
        // Check fingerprint, skip if already analyzed
        // Clone if new
    }

    let _paths = queue.clone_all(&mut repos).await?;
    Ok(repos)
}

pub async fn clone_repo(repo: &HarvestedRepo, dest: &Path) -> CodeilusResult<std::path::PathBuf> {
    let queue = cloner::CloneQueue::new(dest.to_path_buf(), 1);
    queue.clone_repo(repo).await
}
```

### 7. Add to `crates/codeilus-db/src/repos/` — `harvest_repo.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvestRepoRow {
    pub id: i64,
    pub owner: String,
    pub name: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub stars_today: Option<i64>,
    pub total_stars: Option<i64>,
    pub url: String,
    pub fingerprint: String,
    pub status: String,
    pub harvested_at: String,
}

pub struct HarvestRepoRepo { conn: Arc<Mutex<Connection>> }

impl HarvestRepoRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self;
    pub fn insert(&self, repo: &HarvestRepoRow) -> CodeilusResult<i64>;
    pub fn insert_batch(&self, repos: &[HarvestRepoRow]) -> CodeilusResult<Vec<i64>>;
    pub fn get_by_fingerprint(&self, fingerprint: &str) -> CodeilusResult<Option<HarvestRepoRow>>;
    pub fn get_by_name(&self, owner: &str, name: &str) -> CodeilusResult<Option<HarvestRepoRow>>;
    pub fn list(&self) -> CodeilusResult<Vec<HarvestRepoRow>>;
    pub fn list_by_status(&self, status: &str) -> CodeilusResult<Vec<HarvestRepoRow>>;
    pub fn list_by_date(&self, date: &str) -> CodeilusResult<Vec<HarvestRepoRow>>;
    pub fn update_status(&self, id: i64, status: &str) -> CodeilusResult<()>;
    pub fn delete_all(&self) -> CodeilusResult<()>;
}
```

Update `crates/codeilus-db/src/repos/mod.rs` to include `harvest_repo`.

## Tests

### Test cases:
1. `scrape_parse_html` — Given sample GitHub trending HTML (stored as fixture), parse repos correctly
2. `scrape_missing_fields` — HTML with missing description/language → still parses, fields are None
3. `scrape_url_construction` — Config with language="rust" + since=Weekly → correct URL
4. `fingerprint_deterministic` — Same inputs → same hash
5. `fingerprint_different_commits` — Same repo, different commit → different hash
6. `clone_queue_semaphore` — Queue with max_concurrent=2, submit 5 → only 2 run at a time
7. `harvest_status_transitions` — Found → Cloning → Cloned → Analyzing → Complete
8. `skip_already_analyzed` — Repo with matching fingerprint → status = Skipped
9. `config_defaults` — Default config has sensible values (25 repos, 5 concurrent)

### Fixtures:
Create `crates/codeilus-harvest/tests/fixtures/trending.html` with a saved snapshot of GitHub's trending page (small, 3-5 repo entries) for parser testing.

### DB repo tests:
10. `harvest_repo_insert_and_get` — Insert repo, get by fingerprint
11. `harvest_repo_list_by_status` — Filter by "complete" status
12. `harvest_repo_update_status` — Update status from "found" to "cloned"

Note: Tests that actually scrape GitHub or clone repos should use `#[ignore]` and be treated as integration tests. Unit tests should use HTML fixtures and mock data.

## Acceptance Criteria

- [ ] `cargo test -p codeilus-harvest` — all unit tests pass
- [ ] `cargo clippy -p codeilus-harvest` — zero warnings
- [ ] `cargo test -p codeilus-db` — all tests pass (including harvest repo)
- [ ] HTML scraper extracts owner, name, description, language, stars from trending page
- [ ] Shallow clone via `git clone --depth=1` (git2-rs or subprocess)
- [ ] Semaphore limits concurrent clones to configured max
- [ ] Fingerprinting skips already-analyzed repos
- [ ] Status tracking through full lifecycle (Found → Complete)
- [ ] Graceful handling of network errors, missing fields, rate limits

## Do NOT Touch
- `crates/codeilus-core/` (read-only)
- All other crates (this is standalone except for core + db)
- `crates/codeilus-db/src/pool.rs`, `migrations.rs`, `batch_writer.rs`
- Existing repo files in `crates/codeilus-db/src/repos/`
- `migrations/0001_init.sql`
- Any files outside `crates/codeilus-harvest/` and the new DB repo file

---

## Report

> **Agent: fill this section when done.**

### Status: pending

### Files Created/Modified:
<!-- list all files you created/modified -->

### Tests:
<!-- paste `cargo test -p codeilus-harvest` output -->

### Clippy:
<!-- paste `cargo clippy -p codeilus-harvest` output -->

### Issues / Blockers:
<!-- any problems encountered -->

### Notes:
<!-- anything the next wave needs to know -->
