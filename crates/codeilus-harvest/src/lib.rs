//! GitHub trending scraper, shallow clone queue, repo fingerprinting.

pub mod cloner;
pub mod fingerprint;
pub mod scraper;
pub mod types;

pub use types::*;

use codeilus_core::CodeilusResult;
use codeilus_db::DbPool;
use std::path::Path;
use tracing::{debug, info};

/// Harvest trending repos: scrape, clone, fingerprint, and skip already-analyzed.
///
/// The full pipeline:
/// 1. Scrape GitHub trending page
/// 2. Check fingerprints against DB — skip repos already analyzed
/// 3. Shallow-clone remaining repos (max 5 concurrent, 1s delay between starts)
/// 4. Compute fingerprints for cloned repos
pub async fn harvest_trending(
    config: HarvestConfig,
    db: Option<&DbPool>,
) -> CodeilusResult<Vec<HarvestedRepo>> {
    let mut repos = scraper::scrape_trending(&config).await?;
    info!(count = repos.len(), "scraped trending repos");

    // Skip already-analyzed repos if DB is available
    if let Some(db) = db {
        let harvest_repo = codeilus_db::HarvestRepoRepo::new(db.conn_arc());
        for repo in &mut repos {
            // Use a preliminary fingerprint based on owner/name (no commit hash yet)
            if let Ok(true) =
                fingerprint::is_already_analyzed(&repo.fingerprint, &harvest_repo)
            {
                debug!(repo = %repo.full_name, "skipping already-analyzed repo");
                repo.status = HarvestStatus::Skipped;
            }
        }
        let skipped = repos.iter().filter(|r| r.status == HarvestStatus::Skipped).count();
        if skipped > 0 {
            info!(skipped, "skipped already-analyzed repos");
        }
    }

    let queue = cloner::CloneQueue::new(config.clone_dir.clone(), config.concurrent_clones);
    let _paths = queue.clone_all(&mut repos).await?;

    // Compute fingerprints for successfully cloned repos
    for repo in &mut repos {
        if repo.status == HarvestStatus::Cloned {
            let clone_path = config
                .clone_dir
                .join(format!("{}-{}", repo.owner, repo.name));
            if let Ok(fp) = fingerprint::compute_fingerprint(&repo.owner, &repo.name, &clone_path)
            {
                repo.fingerprint = fp;
            }
        }
    }

    let cloned = repos.iter().filter(|r| r.status == HarvestStatus::Cloned).count();
    let failed = repos.iter().filter(|r| r.status == HarvestStatus::Failed).count();
    info!(cloned, failed, "harvest complete");

    Ok(repos)
}

/// Clone a single repo to a destination directory.
pub async fn clone_repo(repo: &HarvestedRepo, dest: &Path) -> CodeilusResult<std::path::PathBuf> {
    let queue = cloner::CloneQueue::new(dest.to_path_buf(), 1);
    queue.clone_repo(repo).await
}
