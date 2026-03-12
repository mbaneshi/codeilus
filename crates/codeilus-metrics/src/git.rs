use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Compute git metrics: (path → (churn, contributors)).
///
/// Walks the last `max_commits` commits and counts:
/// - churn: number of commits touching each file
/// - contributors: unique author emails per file
///
/// Returns empty map if not a git repo or git2 fails.
pub fn compute_git_metrics(
    repo_path: &Path,
    max_commits: usize,
) -> HashMap<String, (usize, usize)> {
    let repo = match git2::Repository::discover(repo_path) {
        Ok(r) => r,
        Err(e) => {
            tracing::debug!("Not a git repo or git2 error: {e}");
            return HashMap::new();
        }
    };

    let mut revwalk = match repo.revwalk() {
        Ok(rw) => rw,
        Err(_) => return HashMap::new(),
    };

    if revwalk.push_head().is_err() {
        return HashMap::new();
    }

    revwalk.set_sorting(git2::Sort::TIME).ok();

    let mut file_churn: HashMap<String, usize> = HashMap::new();
    let mut file_authors: HashMap<String, HashSet<String>> = HashMap::new();
    let mut commit_count = 0;

    for oid in revwalk {
        if commit_count >= max_commits {
            break;
        }
        let oid = match oid {
            Ok(o) => o,
            Err(_) => continue,
        };
        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let author_email = commit
            .author()
            .email()
            .unwrap_or("unknown")
            .to_string();

        let tree = match commit.tree() {
            Ok(t) => t,
            Err(_) => continue,
        };

        // Compare with parent (if any)
        let parent_tree = commit
            .parent(0)
            .ok()
            .and_then(|p| p.tree().ok());

        let diff = match repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&tree),
            None,
        ) {
            Ok(d) => d,
            Err(_) => continue,
        };

        if let Ok(stats) = diff.stats() {
            let _ = stats; // We iterate deltas below
        }

        for delta in diff.deltas() {
            if let Some(path) = delta.new_file().path() {
                let path_str = path.to_string_lossy().to_string();
                *file_churn.entry(path_str.clone()).or_default() += 1;
                file_authors
                    .entry(path_str)
                    .or_default()
                    .insert(author_email.clone());
            }
        }

        commit_count += 1;
    }

    let mut result = HashMap::new();
    for (path, churn) in &file_churn {
        let contributors = file_authors
            .get(path)
            .map(|s| s.len())
            .unwrap_or(0);
        result.insert(path.clone(), (*churn, contributors));
    }

    result
}
