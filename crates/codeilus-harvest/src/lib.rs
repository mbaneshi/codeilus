//! GitHub trending scraper, shallow clone queue, repo fingerprinting.

pub mod cloner;
pub mod fingerprint;
pub mod scraper;
pub mod types;

pub use types::*;

use codeilus_core::CodeilusResult;
use std::path::Path;

/// Harvest trending repos: scrape, clone, and fingerprint.
pub async fn harvest_trending(config: HarvestConfig) -> CodeilusResult<Vec<HarvestedRepo>> {
    let mut repos = scraper::scrape_trending(&config).await?;

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

    Ok(repos)
}

/// Clone a single repo to a destination directory.
pub async fn clone_repo(repo: &HarvestedRepo, dest: &Path) -> CodeilusResult<std::path::PathBuf> {
    let queue = cloner::CloneQueue::new(dest.to_path_buf(), 1);
    queue.clone_repo(repo).await
}
