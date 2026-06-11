use std::path::Path;
use crate::tree::{TreeNode, NodeKind, TreeStats};
use crate::detector::{FileRole, FileCategory, Framework};
use crate::cli::Args;

// ── ANSI color codes (no external crate needed) ───────────────────────────────
struct Color;
impl Color {
    fn bold_blue(s: &str)     -> String { format!("\x1b[1;34m{}\x1b[0m", s) }
    fn bright_white(s: &str)  -> String { format!("\x1b[97m{}\x1b[0m", s) }
    fn yellow(s: &str)        -> String { format!("\x1b[33m{}\x1b[0m", s) }
    fn magenta(s: &str)       -> String { format!("\x1b[35m{}\x1b[0m", s) }
    fn cyan(s: &str)          -> String { format!("\x1b[36m{}\x1b[0m", s) }
    fn dim(s: &str)           -> String { format!("\x1b[2m{}\x1b[0m", s) }
    fn bright_purple(s: &str) -> String { format!("\x1b[95m{}\x1b[0m", s) }
    fn bright_green(s: &str)  -> String { format!("\x1b[92m{}\x1b[0m", s) }
}

fn use_color(args: &Args) -> bool {
    if args.no_color { return false; }
    is_tty()
}

fn is_tty() -> bool {
    #[cfg(unix)]
    unsafe {
        extern "C" { fn isatty(fd: i32) -> i32; }
        isatty(1) != 0
    }
    #[cfg(not(unix))]
    { true }
}

fn c_dir(s: &str, col: bool)    -> String { if col { Color::bold_blue(s)    } else { s.to_string() } }
fn c_file(s: &str, role: &FileRole, col: bool) -> String {
    if !col { return s.to_string(); }
    match role.category() {
        FileCategory::Config => Color::yellow(s),
        FileCategory::Test   => Color::magenta(s),
        FileCategory::Asset  => Color::cyan(s),
        _                    => Color::bright_white(s),
    }
}
fn c_dim(s: &str, col: bool)   -> String { if col { Color::dim(s)           } else { s.to_string() } }
fn c_ai(s: &str, col: bool)    -> String { if col { Color::bright_purple(s) } else { s.to_string() } }
fn c_green(s: &str, col: bool) -> String { if col { Color::bright_green(s)  } else { s.to_string() } }

// ── Tree ──────────────────────────────────────────────────────────────────────

pub fn render_tree(root: &TreeNode, args: &Args) -> String {
    let col = use_color(args);
    let mut out = String::new();
    out.push_str(&c_dir(&format!("{}/", root.name), col));
    out.push('\n');
    render_children(&root.children, "", col, &mut out);
    out
}

fn render_children(children: &[TreeNode], prefix: &str, col: bool, out: &mut String) {
    let last = children.len().saturating_sub(1);
    for (i, node) in children.iter().enumerate() {
        let is_last   = i == last;
        let connector = if is_last { "└── " } else { "├── " };
        let next_pfx  = format!("{}{}   ", prefix, if is_last { " " } else { "│" });
        match &node.kind {
            NodeKind::File => {
                out.push_str(&format!("{}{}{}\n", prefix, connector,
                    c_file(&node.name, &node.role, col)));
            }
            NodeKind::Dir => {
                out.push_str(&format!("{}{}{}\n", prefix, connector,
                    c_dir(&format!("{}/", node.name), col)));
                if !node.children.is_empty() {
                    render_children(&node.children, &next_pfx, col, out);
                }
            }
            NodeKind::Collapsed { count, .. } => {
                out.push_str(&format!("{}{}{}\n", prefix, connector,
                    c_dim(&format!("… {} more", count), col)));
            }
        }
    }
}

// ── AI ────────────────────────────────────────────────────────────────────────

pub fn render_ai(root: &TreeNode, framework: &Framework, args: &Args) -> String {
    let col = use_color(args);
    let stats = TreeStats::from_node(root);
    let tokens = estimate_tokens(root);

    let tag = format!(
        "<project_structure lang=\"{}\" framework=\"{}\" files=\"{}\" est_tokens=\"{}\">",
        framework.lang(), framework.name(), stats.total_files, tokens
    );

    let mut out = String::new();
    out.push_str(&c_ai(&tag, col));
    out.push('\n');
    out.push_str(&c_dir(&format!("{}/", root.name), col));
    out.push('\n');
    render_ai_children(&root.children, "", col, &mut out);
    out.push_str(&c_ai("</project_structure>", col));
    out.push('\n');
    out.push('\n');
    out.push_str(&c_dim(
        &format!("tokens: ~{}  ·  files: {}  ·  ignored: node_modules .git dist build",
                 tokens, stats.total_files), col));
    out.push('\n');
    out
}

fn render_ai_children(children: &[TreeNode], prefix: &str, col: bool, out: &mut String) {
    let last = children.len().saturating_sub(1);
    for (i, node) in children.iter().enumerate() {
        let is_last   = i == last;
        let connector = if is_last { "└── " } else { "├── " };
        let next_pfx  = format!("{}{}   ", prefix, if is_last { " " } else { "│" });
        match &node.kind {
            NodeKind::File => {
                let ann = node.role.annotation()
                    .map(|a| c_dim(&format!("  [{}]", a), col))
                    .unwrap_or_default();
                out.push_str(&format!("{}{}{}{}\n", prefix, connector,
                    c_file(&node.name, &node.role, col), ann));
            }
            NodeKind::Dir => {
                let ann = dir_annotation(&node.name)
                    .map(|a| c_dim(&format!("  [{}]", a), col))
                    .unwrap_or_default();
                out.push_str(&format!("{}{}{}{}\n", prefix, connector,
                    c_dir(&format!("{}/", node.name), col), ann));
                if !node.children.is_empty() {
                    render_ai_children(&node.children, &next_pfx, col, out);
                }
            }
            NodeKind::Collapsed { count, .. } => {
                out.push_str(&format!("{}{}{}\n", prefix, connector,
                    c_dim(&format!("… {} more", count), col)));
            }
        }
    }
}

fn dir_annotation(name: &str) -> Option<&'static str> {
    match name.to_lowercase().as_str() {
        "components"              => Some("ui-components"),
        "hooks"                   => Some("react-hooks"),
        "lib" | "libs"            => Some("lib"),
        "utils" | "helpers"       => Some("utils"),
        "api"                     => Some("api-routes"),
        "app"                     => Some("app-router"),
        "pages"                   => Some("pages-router"),
        "store" | "stores"        => Some("state"),
        "context"                 => Some("context"),
        "types" | "typings"       => Some("type-defs"),
        "styles" | "css"          => Some("styles"),
        "public"                  => Some("static-assets"),
        "assets"                  => Some("assets"),
        "scripts"                 => Some("scripts"),
        "config"                  => Some("config"),
        "migrations"              => Some("db-migrations"),
        "tests" | "__tests__" | "spec" => Some("tests"),
        "middleware"              => Some("middleware"),
        "models"                  => Some("models"),
        "services"                => Some("services"),
        "controllers"             => Some("controllers"),
        "routes"                  => Some("routes"),
        "db" | "database"         => Some("db"),
        "bin"                     => Some("binaries"),
        "src"                     => Some("source-root"),
        _                         => None,
    }
}

// ── Flat ──────────────────────────────────────────────────────────────────────

pub fn render_flat(root: &TreeNode, base: &Path, args: &Args) -> String {
    let col = use_color(args);
    let paths = root.flat_paths(base);
    let mut out = String::new();
    out.push_str(&c_dim(&format!("# directree --flat  ({} paths)\n", paths.len()), col));
    for p in &paths {
        out.push_str(&p.display().to_string());
        out.push('\n');
    }
    let tok = paths.iter().map(|p| p.display().to_string().len()).sum::<usize>() / 4 + paths.len() * 2;
    out.push('\n');
    out.push_str(&c_dim(&format!("{} paths  ·  ~{} tokens", paths.len(), tok), col));
    out.push('\n');
    out
}

// ── Group ─────────────────────────────────────────────────────────────────────

pub fn render_group(root: &TreeNode, base: &Path, args: &Args) -> String {
    use std::collections::BTreeMap;
    let col = use_color(args);
    let mut groups: BTreeMap<u8, Vec<(std::path::PathBuf, FileRole)>> = BTreeMap::new();

    fn cat_order(cat: &FileCategory) -> u8 {
        match cat {
            FileCategory::Source => 0,
            FileCategory::Config => 1,
            FileCategory::Test   => 2,
            FileCategory::Asset  => 3,
            FileCategory::Docs   => 4,
        }
    }

    collect_by_cat(root, base, &mut groups, cat_order);

    let mut out = String::new();
    let labels = ["source", "config", "tests", "assets", "docs"];
    for (order, items) in &groups {
        if items.is_empty() { continue; }
        let label = labels.get(*order as usize).unwrap_or(&"other");
        let header = format!("── {} ", label);
        let pad = "─".repeat(44usize.saturating_sub(header.len()));
        out.push_str(&c_dim(&format!("{}{}\n", header, pad), col));
        for (path, role) in items {
            let s = path.display().to_string();
            let colored = match *order {
                1 => if col { Color::yellow(&s) } else { s.clone() },
                2 => if col { Color::magenta(&s) } else { s.clone() },
                3 => if col { Color::cyan(&s) } else { s.clone() },
                _ => if col { Color::bright_white(&s) } else { s.clone() },
            };
            let ann = role.annotation()
                .map(|a| c_dim(&format!("  [{}]", a), col))
                .unwrap_or_default();
            out.push_str(&format!("{}{}\n", colored, ann));
        }
        out.push('\n');
    }
    out
}

fn collect_by_cat(
    node: &TreeNode,
    base: &Path,
    groups: &mut std::collections::BTreeMap<u8, Vec<(std::path::PathBuf, FileRole)>>,
    cat_order: fn(&FileCategory) -> u8,
) {
    match &node.kind {
        NodeKind::File => {
            if let Ok(rel) = node.path.strip_prefix(base) {
                let ord = cat_order(&node.role.category());
                groups.entry(ord).or_default().push((rel.to_path_buf(), node.role.clone()));
            }
        }
        NodeKind::Dir => {
            for c in &node.children { collect_by_cat(c, base, groups, cat_order); }
        }
        _ => {}
    }
}

// ── JSON ──────────────────────────────────────────────────────────────────────

pub fn render_json(root: &TreeNode) -> String {
    serde_json::to_string_pretty(&node_to_json(root)).unwrap_or_else(|_| "{}".into())
}

fn node_to_json(node: &TreeNode) -> serde_json::Value {
    use serde_json::{json, Value};
    match &node.kind {
        NodeKind::File => {
            let mut obj = json!({ "type": "file", "name": node.name });
            if let Some(ext) = &node.ext { obj["ext"] = Value::String(ext.clone()); }
            if let Some(a) = node.role.annotation() { obj["role"] = Value::String(a.into()); }
            obj
        }
        NodeKind::Dir => json!({
            "type": "dir",
            "name": node.name,
            "children": node.children.iter().map(node_to_json).collect::<Vec<_>>(),
        }),
        NodeKind::Collapsed { count, pattern } => json!({
            "type": "collapsed", "pattern": pattern, "count": count,
        }),
    }
}

// ── Stats ─────────────────────────────────────────────────────────────────────

pub fn render_stats(root: &TreeNode, args: &Args) -> String {
    let col = use_color(args);
    let stats = TreeStats::from_node(root);
    let tokens = estimate_tokens(root);
    let mut out = String::new();
    out.push('\n');
    out.push_str(&c_dim("── stats ─────────────────────────────────────\n", col));
    out.push_str(&format!(
        "  {} {}   {} {}   {} {}   {} {}\n",
        c_dim("files",  col), c_green(&stats.total_files.to_string(), col),
        c_dim("dirs",   col), c_green(&stats.total_dirs.to_string(),  col),
        c_dim("src",    col), c_green(&stats.source_files.to_string(), col),
        c_dim("tests",  col), c_green(&stats.test_files.to_string(),  col),
    ));
    if !stats.by_ext.is_empty() {
        let mut exts: Vec<_> = stats.by_ext.iter().collect();
        exts.sort_by(|a, b| b.1.cmp(a.1));
        let top: Vec<String> = exts.iter().take(8).map(|(e, n)| format!(".{} ({})", e, n)).collect();
        out.push_str(&format!("  {} {}\n", c_dim("exts:", col), c_dim(&top.join("  "), col)));
    }
    out.push_str(&format!("  {} {}\n",
        c_dim("est. tokens:", col), c_green(&format!("~{}", tokens), col)));
    out.push_str(&c_dim("──────────────────────────────────────────────\n", col));
    out
}

// ── Token estimator ───────────────────────────────────────────────────────────

pub fn estimate_tokens(root: &TreeNode) -> usize {
    fn chars(n: &TreeNode) -> usize {
        n.name.len() + 6 + n.children.iter().map(chars).sum::<usize>()
    }
    ((chars(root) as f64 * 1.3) as usize / 4).max(1)
}
