use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct GlobalConfig {
    pub default_depth: Option<usize>,
    pub ignore: Option<Vec<String>>,
}

impl GlobalConfig {
    pub fn load() -> Self {
        let path = config_path();
        if !path.exists() { return Self::default(); }
        let Ok(content) = std::fs::read_to_string(&path) else { return Self::default(); };
        parse_rc(&content)
    }

    pub fn extra_ignores(&self) -> Vec<String> {
        self.ignore.clone().unwrap_or_default()
    }
}

fn config_path() -> PathBuf {
    // Try $HOME/.directreerc
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".directreerc");
    }
    #[cfg(windows)]
    if let Ok(home) = std::env::var("USERPROFILE") {
        return PathBuf::from(home).join(".directreerc");
    }
    PathBuf::from(".directreerc")
}

/// Parse a minimal TOML-like config without pulling in the toml crate
/// Supports:
///   default_depth = 4
///   ignore = ["*.log", "tmp/"]
fn parse_rc(content: &str) -> GlobalConfig {
    let mut cfg = GlobalConfig::default();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if let Some(rest) = line.strip_prefix("default_depth") {
            let rest = rest.trim_start_matches(|c| c == ' ' || c == '=').trim();
            cfg.default_depth = rest.parse().ok();
        } else if let Some(rest) = line.strip_prefix("ignore") {
            let rest = rest.trim_start_matches(|c| c == ' ' || c == '=').trim();
            // Parse ["a", "b", "c"]
            cfg.ignore = Some(parse_string_array(rest));
        }
    }
    cfg
}

fn parse_string_array(s: &str) -> Vec<String> {
    let s = s.trim_matches(|c| c == '[' || c == ']');
    s.split(',')
        .map(|item| item.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
