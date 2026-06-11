use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Returns the set of files changed since a git ref, relative to repo root.
/// Falls back gracefully if not in a git repo or git is unavailable.
pub fn changed_since(repo_root: &Path, git_ref: &str) -> Option<HashSet<PathBuf>> {
    // Verify we're in a git repo
    let status = Command::new("git")
        .args(["-C", &repo_root.to_string_lossy(), "rev-parse", "--git-dir"])
        .output()
        .ok()?;

    if !status.status.success() {
        eprintln!("directree: --since requires a git repository");
        return None;
    }

    // git diff --name-only <ref>
    let diff = Command::new("git")
        .args([
            "-C", &repo_root.to_string_lossy(),
            "diff", "--name-only", git_ref,
        ])
        .output()
        .ok()?;

    // Also get untracked/staged changes relative to ref
    let diff_cached = Command::new("git")
        .args([
            "-C", &repo_root.to_string_lossy(),
            "diff", "--name-only", "--cached", git_ref,
        ])
        .output()
        .ok()?;

    // Untracked new files since ref
    let new_files = Command::new("git")
        .args([
            "-C", &repo_root.to_string_lossy(),
            "ls-files", "--others", "--exclude-standard",
        ])
        .output()
        .ok()?;

    let mut files: HashSet<PathBuf> = HashSet::new();

    for output in [&diff, &diff_cached, &new_files] {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let line = line.trim();
            if !line.is_empty() {
                files.insert(PathBuf::from(line));
            }
        }
    }

    if files.is_empty() {
        eprintln!("directree: no changed files found since '{}'", git_ref);
    }

    Some(files)
}

/// Get the repo root (for resolving relative paths from git output)
pub fn repo_root(from: &Path) -> Option<PathBuf> {
    let out = Command::new("git")
        .args(["-C", &from.to_string_lossy(), "rev-parse", "--show-toplevel"])
        .output()
        .ok()?;

    if out.status.success() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        Some(PathBuf::from(s))
    } else {
        None
    }
}

/// Get last N commit messages for display in --since context
pub fn recent_commits(repo_root: &Path, n: usize) -> Vec<String> {
    let out = Command::new("git")
        .args([
            "-C", &repo_root.to_string_lossy(),
            "log", "--oneline", &format!("-{}", n),
        ])
        .output();

    match out {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .collect()
        }
        _ => vec![],
    }
}
