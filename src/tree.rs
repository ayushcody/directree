use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::detector::{FileRole, FileCategory};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub name: String,
    #[serde(skip)]
    pub path: PathBuf,
    pub kind: NodeKind,
    pub children: Vec<TreeNode>,
    #[serde(skip)]
    pub role: FileRole,
    pub ext: Option<String>,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeKind {
    Dir,
    File,
    Collapsed { count: usize, pattern: String },
}

impl TreeNode {
    pub fn new_dir(name: String, path: PathBuf, depth: usize) -> Self {
        Self { name, path, kind: NodeKind::Dir, children: vec![], role: FileRole::Unknown, ext: None, depth }
    }

    pub fn new_file(name: String, path: PathBuf, depth: usize, role: FileRole, ext: Option<String>) -> Self {
        Self { name, path, kind: NodeKind::File, children: vec![], role, ext, depth }
    }

    pub fn is_dir(&self) -> bool { matches!(self.kind, NodeKind::Dir) }
    pub fn is_empty_dir(&self) -> bool { self.is_dir() && self.children.is_empty() }

    pub fn flat_paths(&self, base: &std::path::Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        self.collect_flat(base, &mut out);
        out
    }

    fn collect_flat(&self, base: &std::path::Path, out: &mut Vec<PathBuf>) {
        match &self.kind {
            NodeKind::File => {
                if let Ok(rel) = self.path.strip_prefix(base) {
                    out.push(rel.to_path_buf());
                }
            }
            NodeKind::Dir => { for c in &self.children { c.collect_flat(base, out); } }
            _ => {}
        }
    }
}

#[derive(Debug, Default)]
pub struct TreeStats {
    pub total_files: usize,
    pub total_dirs: usize,
    pub by_ext: std::collections::BTreeMap<String, usize>,
    pub source_files: usize,
    pub config_files: usize,
    pub test_files: usize,
    pub asset_files: usize,
}

impl TreeStats {
    pub fn from_node(root: &TreeNode) -> Self {
        let mut s = Self::default();
        collect_stats(root, &mut s);
        s
    }
}

fn collect_stats(node: &TreeNode, s: &mut TreeStats) {
    match &node.kind {
        NodeKind::Dir => {
            s.total_dirs += 1;
            for c in &node.children { collect_stats(c, s); }
        }
        NodeKind::File => {
            s.total_files += 1;
            if let Some(ext) = &node.ext { *s.by_ext.entry(ext.clone()).or_insert(0) += 1; }
            match node.role.category() {
                FileCategory::Source => s.source_files += 1,
                FileCategory::Config => s.config_files += 1,
                FileCategory::Test   => s.test_files   += 1,
                FileCategory::Asset  => s.asset_files  += 1,
                FileCategory::Docs   => {}
            }
        }
        NodeKind::Collapsed { count, .. } => { s.total_files += count; }
    }
}
