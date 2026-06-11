use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Importance score for a file — higher = more important
#[derive(Debug, Default, Clone)]
pub struct ImportanceScore {
    pub total: f32,
    pub reasons: Vec<String>,
}

impl ImportanceScore {
    pub fn is_important(&self) -> bool {
        self.total >= 3.0
    }

    pub fn label(&self) -> Option<&'static str> {
        if self.total >= 8.0 { Some("★★★") }
        else if self.total >= 5.0 { Some("★★") }
        else if self.total >= 3.0 { Some("★") }
        else { None }
    }
}

/// Compute importance scores for all files in a tree
pub fn score_files(root: &Path, files: &[PathBuf]) -> HashMap<PathBuf, ImportanceScore> {
    let mut scores: HashMap<PathBuf, ImportanceScore> = HashMap::new();

    // 1. Git recency scores (files changed in last 7 days get boost)
    let recency = git_recency_scores(root);

    // 2. File size as proxy for "has content"
    // 3. Role-based scoring via filename heuristics
    // 4. Import graph (grep-based: how many other files reference this one)
    let import_counts = grep_import_counts(root, files);

    for file in files {
        let mut score = ImportanceScore::default();

        // Role-based scoring
        let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        let stem = file.file_stem().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        let path_str = file.to_string_lossy().to_lowercase().replace('\\', "/");

        // Entry points — highest weight
        if matches!(stem.as_str(), "main" | "index" | "app" | "server" | "mod") {
            score.total += 4.0;
            score.reasons.push("entry-point".into());
        }
        // Layout/page roots
        if matches!(stem.as_str(), "layout" | "page" | "_app" | "_document") {
            score.total += 3.0;
            score.reasons.push("route-root".into());
        }
        // Config files that matter a lot
        if matches!(name.as_str(),
            "package.json" | "cargo.toml" | "go.mod" | "pyproject.toml"
            | "next.config.js" | "next.config.ts" | "next.config.mjs"
            | "vite.config.ts" | "tailwind.config.ts" | "tsconfig.json"
            | "docker-compose.yml" | "dockerfile" | ".env.example"
        ) {
            score.total += 3.0;
            score.reasons.push("key-config".into());
        }
        // DB / auth / core infra
        if name.contains("supabase") || name.contains("prisma") || name.contains("database")
            || name.contains("auth") || name.contains("middleware")
        {
            score.total += 2.0;
            score.reasons.push("core-infra".into());
        }
        // Path depth bonus — shallower = more important
        let depth = file.components().count();
        if depth <= 2 { score.total += 2.0; }
        else if depth <= 4 { score.total += 1.0; }

        // Git recency bonus
        if let Some(days_ago) = recency.get(file) {
            if *days_ago <= 1 { score.total += 3.0; score.reasons.push("changed-today".into()); }
            else if *days_ago <= 7 { score.total += 2.0; score.reasons.push("changed-this-week".into()); }
        }

        // Import frequency bonus
        if let Some(&count) = import_counts.get(file) {
            if count >= 10 { score.total += 4.0; score.reasons.push(format!("imported-{}x", count)); }
            else if count >= 5 { score.total += 2.5; score.reasons.push(format!("imported-{}x", count)); }
            else if count >= 2 { score.total += 1.0; score.reasons.push(format!("imported-{}x", count)); }
        }

        // Type definitions centrally located
        if (name.ends_with(".d.ts") || path_str.contains("/types/"))
            && depth <= 4
        {
            score.total += 1.5;
            score.reasons.push("types".into());
        }

        if score.total > 0.0 {
            scores.insert(file.clone(), score);
        }
    }

    scores
}

/// Use git log to find how recently each file was modified (days ago)
fn git_recency_scores(root: &Path) -> HashMap<PathBuf, u32> {
    let mut map = HashMap::new();

    let out = Command::new("git")
        .args([
            "-C", &root.to_string_lossy(),
            "log", "--name-only", "--format=%ct", "--since=30 days ago",
        ])
        .output();

    let Ok(out) = out else { return map };
    if !out.status.success() { return map; }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let text = String::from_utf8_lossy(&out.stdout);
    let mut current_ts: Option<u64> = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }

        if let Ok(ts) = line.parse::<u64>() {
            current_ts = Some(ts);
        } else if let Some(ts) = current_ts {
            let days_ago = ((now.saturating_sub(ts)) / 86400) as u32;
            let path = PathBuf::from(line);
            // Keep the most recent (smallest days_ago) for each file
            map.entry(path)
               .and_modify(|e| { if days_ago < *e { *e = days_ago; } })
               .or_insert(days_ago);
        }
    }

    map
}

/// Count how many files import/require each file (grep-based, fast)
fn grep_import_counts(root: &Path, files: &[PathBuf]) -> HashMap<PathBuf, usize> {
    let mut counts: HashMap<PathBuf, usize> = HashMap::new();

    for file in files {
        let stem = match file.file_stem().and_then(|n| n.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };

        // Skip very short stems that would cause false positives
        if stem.len() < 3 { continue; }

        // grep -rl "stem" in root, count matching files
        let out = Command::new("grep")
            .args([
                "-rl",
                "--include=*.ts", "--include=*.tsx", "--include=*.js",
                "--include=*.jsx", "--include=*.rs", "--include=*.py",
                "--include=*.go",
                &stem,
                &root.to_string_lossy(),
            ])
            .output();

        if let Ok(o) = out {
            if o.status.success() {
                let count = String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter(|l| !l.is_empty())
                    .count();
                if count > 0 {
                    counts.insert(file.clone(), count);
                }
            }
        }
    }

    counts
}
