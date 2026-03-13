//! Repo fingerprinting for skip-if-already-analyzed logic.

use codeilus_core::error::{CodeilusError, CodeilusResult};
use codeilus_db::HarvestRepoRepo;
use sha2::{Digest, Sha256};
use std::path::Path;

/// Compute a fingerprint for a repo based on owner/name and latest commit hash.
///
/// The fingerprint is a SHA256 hex digest of `"{owner}/{name}@{commit_hash}"`.
pub fn compute_fingerprint(owner: &str, name: &str, repo_path: &Path) -> CodeilusResult<String> {
    let repo = git2::Repository::open(repo_path)
        .map_err(|e| CodeilusError::Harvest(format!("Failed to open git repo: {}", e)))?;

    let head = repo
        .head()
        .map_err(|e| CodeilusError::Harvest(format!("Failed to get HEAD: {}", e)))?;

    let commit = head
        .peel_to_commit()
        .map_err(|e| CodeilusError::Harvest(format!("Failed to peel to commit: {}", e)))?;

    let commit_hash = commit.id().to_string();
    compute_fingerprint_from_hash(owner, name, &commit_hash)
}

/// Compute fingerprint from explicit owner, name, and commit hash.
pub fn compute_fingerprint_from_hash(owner: &str, name: &str, commit_hash: &str) -> CodeilusResult<String> {
    let input = format!("{}/{}@{}", owner, name, commit_hash);
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Check if a repo with the given fingerprint has already been analyzed.
pub fn is_already_analyzed(fingerprint: &str, db: &HarvestRepoRepo) -> CodeilusResult<bool> {
    match db.get_by_fingerprint(fingerprint)? {
        Some(row) => Ok(row.status == "complete" || row.status == "skipped"),
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_deterministic() {
        let fp1 = compute_fingerprint_from_hash("owner", "repo", "abc123").unwrap();
        let fp2 = compute_fingerprint_from_hash("owner", "repo", "abc123").unwrap();
        assert_eq!(fp1, fp2, "Same inputs should produce same fingerprint");
        assert_eq!(fp1.len(), 64, "SHA256 hex digest should be 64 chars");
    }

    #[test]
    fn fingerprint_different_commits() {
        let fp1 = compute_fingerprint_from_hash("owner", "repo", "abc123").unwrap();
        let fp2 = compute_fingerprint_from_hash("owner", "repo", "def456").unwrap();
        assert_ne!(fp1, fp2, "Different commits should produce different fingerprints");
    }
}
