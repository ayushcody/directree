use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// A simple polling-based filesystem watcher.
/// Checks mtime of all files under root every `interval`.
/// Returns true when a change is detected.
pub struct Watcher {
    snapshots: HashMap<std::path::PathBuf, SystemTime>,
    interval: Duration,
}

impl Watcher {
    pub fn new(interval_ms: u64) -> Self {
        Self {
            snapshots: HashMap::new(),
            interval: Duration::from_millis(interval_ms),
        }
    }

    /// Take initial snapshot of the directory
    pub fn snapshot(&mut self, root: &Path) {
        self.snapshots.clear();
        walk_mtimes(root, &mut self.snapshots);
    }

    /// Block until a change is detected, then return
    pub fn wait_for_change(&mut self, root: &Path) {
        loop {
            std::thread::sleep(self.interval);

            let mut current: HashMap<std::path::PathBuf, SystemTime> = HashMap::new();
            walk_mtimes(root, &mut current);

            if current != self.snapshots {
                self.snapshots = current;
                return;
            }
        }
    }
}

fn walk_mtimes(dir: &Path, out: &mut HashMap<std::path::PathBuf, SystemTime>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip obvious noise dirs for faster polling
        if matches!(name, ".git" | "node_modules" | "target" | ".next" | "dist" | "build") {
            continue;
        }

        if let Ok(meta) = entry.metadata() {
            if let Ok(mtime) = meta.modified() {
                out.insert(path.clone(), mtime);
            }
            if meta.is_dir() {
                walk_mtimes(&path, out);
            }
        }
    }
}
