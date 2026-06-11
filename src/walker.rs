use std::path::{Path, PathBuf};
use std::collections::HashSet;
use crate::cli::Args;
use crate::detector::{Framework, classify_file};
use crate::ignorer::Ignorer;
use crate::tree::{TreeNode, NodeKind};

pub struct Walker {
    root: PathBuf,
    framework: Framework,
    ignorer: Ignorer,
    max_depth: usize,          // 0 = unlimited
    show_hidden: bool,
    only_exts: Option<HashSet<String>>,
    collapse: bool,
}

impl Walker {
    pub fn new(args: &Args, root: PathBuf, framework: Framework, mut ignorer: Ignorer) -> Self {
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
        }
    }

    pub fn walk(&self) -> TreeNode {
        let root_name = self.root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(".")
            .to_string();

        let mut root_node = TreeNode::new_dir(root_name, self.root.clone(), 0);
        self.walk_dir(&self.root, &mut root_node, 0);

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

            // Apply ignore rules
            if self.ignorer.is_ignored(&path, self.show_hidden) {
                continue;
            }

            if path.is_dir() {
                dirs.push((name, path));
            } else {
                files.push((name, path));
            }
        }

        // Sort: dirs first, then files — both alphabetically
        dirs.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        files.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        // Recurse into dirs
        let next_depth = depth + 1;
        let can_recurse = self.max_depth == 0 || next_depth <= self.max_depth;

        for (name, path) in dirs {
            let mut dir_node = TreeNode::new_dir(name, path.clone(), next_depth);
            if can_recurse {
                self.walk_dir(&path, &mut dir_node, next_depth);
            } else {
                // Mark as truncated with child count hint
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
            // Only add dir if it has children OR we want empty dirs
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

            let role = classify_file(&path, &self.framework);
            let file_node = TreeNode::new_file(name, path, next_depth, role, ext);
            node.children.push(file_node);
        }

        // Remove empty dirs when extension filter is active
        if self.only_exts.is_some() {
            node.children.retain(|c| {
                !c.is_empty_dir()
            });
        }
    }
}

/// Count direct entries in a dir quickly (for depth-limit hint)
fn count_entries_fast(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| entries.flatten().count())
        .unwrap_or(0)
}

/// Detect and collapse repeated sibling patterns in a dir node
fn collapse_repeated(node: &mut TreeNode) {
    // Recurse first
    for child in node.children.iter_mut() {
        if child.is_dir() {
            collapse_repeated(child);
        }
    }

    // Collect dir children that share the same "shape"
    // Shape = sorted set of child extensions
    let mut shape_groups: std::collections::HashMap<String, Vec<usize>> = Default::default();

    for (i, child) in node.children.iter().enumerate() {
        if child.is_dir() && !child.children.is_empty() {
            let shape = dir_shape(child);
            shape_groups.entry(shape).or_default().push(i);
        }
    }

    // Find groups of 4+ identical-shape siblings → collapse
    let mut to_collapse: Vec<(Vec<usize>, String)> = Vec::new();
    for (shape, indices) in &shape_groups {
        if indices.len() >= 4 {
            to_collapse.push((indices.clone(), shape.clone()));
        }
    }

    if to_collapse.is_empty() { return; }

    // Build new children list
    let mut collapsed_indices: HashSet<usize> = HashSet::new();
    let mut collapse_nodes: Vec<(usize, TreeNode)> = Vec::new(); // (insertion_pos, node)

    for (indices, _shape) in &to_collapse {
        let first_idx = indices[0];
        let count = indices.len();
        let pattern = node.children[first_idx].name.clone();

        let collapsed = TreeNode {
            name: format!("{} ×{} (similar structure)", pattern, count),
            path: node.children[first_idx].path.clone(),
            kind: NodeKind::Collapsed { count, pattern: pattern.clone() },
            children: vec![],
            role: crate::detector::FileRole::Unknown,
            ext: None,
            depth: node.children[first_idx].depth,
        };

        for &idx in indices {
            collapsed_indices.insert(idx);
        }
        collapse_nodes.push((first_idx, collapsed));
    }

    // Rebuild children
    let old_children = std::mem::take(&mut node.children);
    let mut new_children: Vec<TreeNode> = Vec::new();
    let mut inserted: HashSet<usize> = HashSet::new();

    for (i, child) in old_children.into_iter().enumerate() {
        if collapsed_indices.contains(&i) {
            if let Some(pos) = collapse_nodes.iter().position(|(idx, _)| *idx == i) {
                if !inserted.contains(&pos) {
                    new_children.push(collapse_nodes[pos].1.clone());
                    inserted.insert(pos);
                }
            }
        } else {
            new_children.push(child);
        }
    }

    node.children = new_children;
}

/// Compute a "shape fingerprint" for a directory node
fn dir_shape(node: &TreeNode) -> String {
    let mut exts: Vec<String> = node.children.iter()
        .filter_map(|c| c.ext.clone())
        .collect();
    exts.sort();
    exts.dedup();
    exts.join(",")
}
