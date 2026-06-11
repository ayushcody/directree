use std::path::Path;
use crate::detector::Framework;

const ALWAYS_IGNORE: &[&str] = &[
    ".git", ".svn", ".hg", "node_modules", ".DS_Store", "Thumbs.db",
    "desktop.ini", ".idea", ".vscode", ".cache", ".tmp", "tmp", "temp",
    ".turbo", ".parcel-cache", "coverage", ".nyc_output",
];

pub struct Ignorer {
    patterns: Vec<String>,
    pub rules: Vec<String>,
}

impl Ignorer {
    pub fn build(
        framework: &Framework,
        extra_patterns: &[String],
        no_ignore: bool,
        _show_hidden: bool,
    ) -> Self {
        let mut rules = Vec::new();
        let mut patterns = Vec::new();

        if !no_ignore {
            for p in ALWAYS_IGNORE {
                rules.push(format!("{} [built-in]", p));
                patterns.push(p.to_string());
            }
            for p in framework.extra_ignores() {
                rules.push(format!("{} [{}]", p, framework.name()));
                patterns.push(p.to_string());
            }
        }

        for p in extra_patterns {
            let p = p.trim();
            if p.is_empty() || p.starts_with('#') { continue; }
            rules.push(format!("{} [cli --ignore]", p));
            patterns.push(p.to_string());
        }

        Self { patterns, rules }
    }

    pub fn load_local_ignore(&mut self, root: &Path) {
        let ignore_file = root.join(".directreeignore");
        if !ignore_file.exists() { return; }
        let Ok(content) = std::fs::read_to_string(&ignore_file) else { return };
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            self.rules.push(format!("{} [.directreeignore]", line));
            self.patterns.push(line.to_string());
        }
    }

    pub fn is_ignored(&self, path: &Path, show_hidden: bool) -> bool {
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => return false,
        };

        if !show_hidden && name.starts_with('.') {
            return true;
        }

        for pat in &self.patterns {
            if glob_match(pat, name) {
                return true;
            }
            // Also match against path segments for patterns with /
            if pat.contains('/') {
                let path_str = path.to_string_lossy();
                if path_str.contains(pat.trim_end_matches('/')) {
                    return true;
                }
            }
        }
        false
    }
}

/// Minimal glob matching: supports * and ? wildcards
fn glob_match(pattern: &str, name: &str) -> bool {
    let pat = pattern.trim_end_matches('/');
    // Exact match
    if pat == name { return true; }
    // No wildcards - done
    if !pat.contains('*') && !pat.contains('?') { return false; }

    // Simple wildcard matching
    glob_match_inner(pat.as_bytes(), name.as_bytes())
}

fn glob_match_inner(pat: &[u8], s: &[u8]) -> bool {
    match (pat.first(), s.first()) {
        (None, None) => true,
        (Some(&b'*'), _) => {
            // * matches zero or more chars
            glob_match_inner(&pat[1..], s)
                || (!s.is_empty() && glob_match_inner(pat, &s[1..]))
        }
        (Some(&b'?'), Some(_)) => glob_match_inner(&pat[1..], &s[1..]),
        (Some(p), Some(c)) if p == c => glob_match_inner(&pat[1..], &s[1..]),
        _ => false,
    }
}
