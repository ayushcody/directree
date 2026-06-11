use std::path::{Path, PathBuf};
use std::collections::{HashSet, HashMap};
use crate::cli::Args;
use crate::detector::{Framework, classify_file};
use crate::ignorer::Ignorer;
use crate::tree::{TreeNode, NodeKind};
use crate::importance::ImportanceScore;

pub struct Walker {
    root: PathBuf,
    framework: Framework,
    ignorer: Ignorer,
    max_depth: usize,
    show_hidden: bool,
    only_exts: Option<HashSet<String>>,
    collapse: bool,
    since_files: Option<HashSet<PathBuf>>,   // relative paths from git
    repo_root: Option<PathBuf>,              // for resolving git paths
    pub importance: HashMap<PathBuf, ImportanceScore>,
}

impl Walker {
    pub fn new(
        args: &Args,
        root: PathBuf,
        framework: Framework,
        mut ignorer: Ignorer,
        since_files: Option<HashSet<PathBuf>>,
        repo_root: Option<PathBuf>,
    ) -> Self {
        ignorer.load_local_ignore(&root);

        let only_exts = args.only.as_ref().map(|s| {
            s.split(',')
             .map(|e| e.trim().trim_start_matches('.').to_lowercase())
             .filter(|e| !e.is_empty())
             .collect::<HashSet<String>>()
        });

        Self {
            root,
            framework,
            ignorer,
            max_depth: args.depth,
            show_hidden: args.hidden,
            only_exts,
            collapse: args.collapse,
            since_files,
            repo_root,
            importance: HashMap::new(),
        }
    }

    pub fn walk(&mut self) -> TreeNode {
        let root_name = self.root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(".")
            .to_string();

        let mut root_node = TreeNode::new_dir(root_name, self.root.clone(), 0);
        self.walk_dir(&self.root.clone(), &mut root_node, 0);

        if self.collapse {
            collapse_repeated(&mut root_node);
        }

        root_node
    }

    fn walk_dir(&self, dir: &Path, node: &mut TreeNode, depth: usize) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        let mut dirs: Vec<(String, PathBuf)> = Vec::new();
        let mut files: Vec<(String, PathBuf)> = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            if self.ignorer.is_ignored(&path, self.show_hidden) {
                continue;
            }

            if path.is_dir() {
                dirs.push((name, path));
            } else {
                files.push((name, path));
            }
        }

        // Sort: dirs first, then files — both alphabetical
        dirs.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        files.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        let next_depth = depth + 1;
        let can_recurse = self.max_depth == 0 || next_depth <= self.max_depth;

        // Recurse into dirs
        for (name, path) in dirs {
            let mut dir_node = TreeNode::new_dir(name, path.clone(), next_depth);
            if can_recurse {
                self.walk_dir(&path, &mut dir_node, next_depth);
            } else {
                let count = count_entries_fast(&path);
                if count > 0 {
                    dir_node.children.push(TreeNode {
                        name: format!("… {} more", count),
                        path: path.clone(),
                        kind: NodeKind::Collapsed { count, pattern: "depth-limit".into() },
                        children: vec![],
                        role: crate::detector::FileRole::Unknown,
                        ext: None,
                        depth: next_depth + 1,
                    });
                }
            }
            // --since: only include dir if it has any matching children
            if self.since_files.is_some() && dir_node.children.is_empty() {
                continue;
            }
            node.children.push(dir_node);
        }

        // Add files
        for (name, path) in files {
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            // Extension filter
            if let Some(ref only) = self.only_exts {
                match &ext {
                    Some(e) if only.contains(e.as_str()) => {}
                    _ => continue,
                }
            }

            // --since filter: check if this file is in the changed set
            if let Some(ref changed) = self.since_files {
                let rel = self.relative_to_repo(&path);
                if !changed.contains(&rel) {
                    continue;
                }
            }

            let role = classify_file(&path, &self.framework);
            let file_node = TreeNode::new_file(name, path, next_depth, role, ext);
            node.children.push(file_node);
        }

        // Remove empty dirs when filtering is active
        if self.only_exts.is_some() || self.since_files.is_some() {
            node.children.retain(|c| !c.is_empty_dir());
        }
    }

    /// Resolve absolute path to a repo-relative path for --since comparison
    fn relative_to_repo(&self, abs: &Path) -> PathBuf {
        if let Some(ref repo) = self.repo_root {
            if let Ok(rel) = abs.strip_prefix(repo) {
                return rel.to_path_buf();
            }
        }
        // Fallback: strip scan root
        if let Ok(rel) = abs.strip_prefix(&self.root) {
            return rel.to_path_buf();
        }
        abs.to_path_buf()
    }
}

fn count_entries_fast(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|e| e.flatten().count())
        .unwrap_or(0)
}

fn collapse_repeated(node: &mut TreeNode) {
    for child in node.children.iter_mut() {
        if child.is_dir() {
            collapse_repeated(child);
        }
    }

    let mut shape_groups: std::collections::HashMap<String, Vec<usize>> = Default::default();
    for (i, child) in node.children.iter().enumerate() {
        if child.is_dir() && !child.children.is_empty() {
            shape_groups.entry(dir_shape(child)).or_default().push(i);
        }
    }

    let to_collapse: Vec<(Vec<usize>, String)> = shape_groups
        .into_iter()
        .filter(|(_, v)| v.len() >= 4)
        .map(|(k, v)| (v, k))
        .collect();

    if to_collapse.is_empty() { return; }

    let mut collapsed_indices: HashSet<usize> = HashSet::new();
    let mut collapse_nodes: Vec<(usize, TreeNode)> = Vec::new();

    for (indices, _) in &to_collapse {
        let first = indices[0];
        let count = indices.len();
        let pattern = node.children[first].name.clone();
        let collapsed = TreeNode {
            name: format!("{} ×{} (similar structure)", pattern, count),
            path: node.children[first].path.clone(),
            kind: NodeKind::Collapsed { count, pattern },
            children: vec![],
            role: crate::detector::FileRole::Unknown,
            ext: None,
            depth: node.children[first].depth,
        };
        for &idx in indices { collapsed_indices.insert(idx); }
        collapse_nodes.push((first, collapsed));
    }

    let old = std::mem::take(&mut node.children);
    let mut inserted: HashSet<usize> = HashSet::new();

    for (i, child) in old.into_iter().enumerate() {
        if collapsed_indices.contains(&i) {
            if let Some(pos) = collapse_nodes.iter().position(|(idx, _)| *idx == i) {
                if !inserted.contains(&pos) {
                    node.children.push(collapse_nodes[pos].1.clone());
                    inserted.insert(pos);
                }
            }
        } else {
            node.children.push(child);
        }
    }
}

fn dir_shape(node: &TreeNode) -> String {
    let mut exts: Vec<String> = node.children.iter()
        .filter_map(|c| c.ext.clone())
        .collect();
    exts.sort(); exts.dedup();
    exts.join(",")
}
