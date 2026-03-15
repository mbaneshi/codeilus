//! Shallow clone queue with concurrency control.

use codeilus_core::error::{CodeilusError, CodeilusResult};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, warn};

use crate::types::{HarvestStatus, HarvestedRepo};

/// Queue for shallow-cloning repositories with concurrency control.
pub struct CloneQueue {
    semaphore: Arc<Semaphore>,
    clone_dir: PathBuf,
}

impl CloneQueue {
    /// Create a new clone queue.
    pub fn new(clone_dir: PathBuf, max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            clone_dir,
        }
    }

    /// Clone a single repo. Returns the local path.
    pub async fn clone_repo(&self, repo: &HarvestedRepo) -> CodeilusResult<PathBuf> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| CodeilusError::Harvest(format!("Semaphore error: {}", e)))?;

        let dest = self.clone_dir.join(format!("{}-{}", repo.owner, repo.name));
        let clone_url = repo.clone_url.clone();

        debug!(repo = %repo.full_name, dest = %dest.display(), "cloning repo");

        // Use git2 for shallow clone
        shallow_clone(&clone_url, &dest)?;

        Ok(dest)
    }

    /// Clone all repos in parallel (respecting semaphore).
    pub async fn clone_all(&self, repos: &mut [HarvestedRepo]) -> CodeilusResult<Vec<PathBuf>> {
        // Collect clone tasks — extract data before spawning to avoid borrow issues
        let mut tasks: Vec<(usize, tokio::task::JoinHandle<Result<PathBuf, CodeilusError>>)> =
            Vec::new();

        for (i, repo) in repos.iter_mut().enumerate() {
            if repo.status == HarvestStatus::Skipped || repo.status == HarvestStatus::Failed {
                continue;
            }
            repo.status = HarvestStatus::Cloning;

            // Rate limit: 1s delay between clone task spawns
            if i > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }

            let semaphore = Arc::clone(&self.semaphore);
            let clone_dir = self.clone_dir.clone();
            let owner = repo.owner.clone();
            let name = repo.name.clone();
            let full_name = repo.full_name.clone();
            let clone_url = repo.clone_url.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore
                    .acquire()
                    .await
                    .map_err(|e| CodeilusError::Harvest(format!("Semaphore error: {}", e)))?;

                let dest = clone_dir.join(format!("{}-{}", owner, name));

                debug!(repo = %full_name, dest = %dest.display(), "cloning repo");

                match shallow_clone(&clone_url, &dest) {
                    Ok(()) => Ok(dest),
                    Err(e) => {
                        let _ = std::fs::remove_dir_all(&dest);
                        Err(e)
                    }
                }
            });

            tasks.push((i, handle));
        }

        let mut paths = Vec::new();

        for (i, handle) in tasks {
            match handle.await {
                Ok(Ok(path)) => {
                    repos[i].status = HarvestStatus::Cloned;
                    paths.push(path);
                }
                Ok(Err(e)) => {
                    warn!(error = %e, "clone failed");
                    repos[i].status = HarvestStatus::Failed;
                }
                Err(e) => {
                    warn!(error = %e, "clone task panicked");
                    repos[i].status = HarvestStatus::Failed;
                }
            }
        }

        Ok(paths)
    }
}

/// Perform a shallow clone (depth=1) using git2.
fn shallow_clone(url: &str, dest: &Path) -> CodeilusResult<()> {
    // If destination already exists, skip
    if dest.exists() {
        debug!(dest = %dest.display(), "clone destination already exists, skipping");
        return Ok(());
    }

    // Create parent directory if needed
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CodeilusError::Harvest(format!("Failed to create clone dir: {}", e)))?;
    }

    // git2 doesn't directly support --depth=1, so we use subprocess
    let output = std::process::Command::new("git")
        .args(["clone", "--depth=1", url, &dest.to_string_lossy()])
        .output()
        .map_err(|e| CodeilusError::Harvest(format!("Failed to run git clone: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CodeilusError::Harvest(format!(
            "git clone failed: {}",
            stderr.trim()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn clone_queue_semaphore() {
        // Verify the semaphore limits concurrency
        let dir = tempfile::tempdir().unwrap();
        let queue = CloneQueue::new(dir.path().to_path_buf(), 2);

        // The semaphore should have 2 permits
        assert_eq!(queue.semaphore.available_permits(), 2);

        // Acquire permits manually to verify they work
        let _p1 = queue.semaphore.acquire().await.unwrap();
        assert_eq!(queue.semaphore.available_permits(), 1);

        let _p2 = queue.semaphore.acquire().await.unwrap();
        assert_eq!(queue.semaphore.available_permits(), 0);

        // A third acquire would block (not tested here, just verifying the count)
        drop(_p1);
        assert_eq!(queue.semaphore.available_permits(), 1);
    }
}
