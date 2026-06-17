use crate::{error::AppError, hrsize::HumanReadableSize};

use std::os::unix::fs::MetadataExt;
use std::sync::RwLock;
use std::{fs, path::PathBuf};

macro_rules! sub_min_from_max {
    ($a:expr, $b:expr) => {{
        let a = $a;
        let b = $b;
        let max = a.max(b);
        let min = a.min(b);
        max - min
    }};
}

#[derive(Clone)]
pub struct Node {
    id: usize,
    parent: Option<usize>,
    path: PathBuf,
    depth: u8,
    is_dir: bool,
    size: HumanReadableSize,
    children: Vec<usize>,
}

pub struct Tree {
    root: Option<usize>,
    nodes: RwLock<Vec<Node>>,
    pub total_size: HumanReadableSize,
}

pub struct InfoOptions {
    pub info_level: u8,
    pub shorten: bool,
    pub max_len: u16,
    pub dir_only: bool,
    pub show_percent_only: bool,
    pub show_size_only: bool,
}

impl Node {
    pub fn new(id: usize, parent: Option<usize>, path: PathBuf, depth: u8, is_dir: bool) -> Self {
        Node {
            id: id,
            parent: parent,
            children: Vec::new(),
            size: HumanReadableSize(0),
            path: path,
            depth: depth,
            is_dir: is_dir,
        }
    }
}

impl Tree {
    pub fn new(root: Option<Node>) -> Self {
        if let Some(n) = root {
            Tree {
                root: Some(0),
                nodes: RwLock::from(vec![n]),
                total_size: HumanReadableSize(0),
            }
        } else {
            Tree {
                root: None,
                nodes: RwLock::from(Vec::new()),
                total_size: HumanReadableSize(0),
            }
        }
    }
    pub fn nodes(&self) -> Vec<Node> {
        self.nodes.read().unwrap().clone()
    }

    pub fn build(&mut self) -> Result<(), AppError> {
        if let Some(root) = self.root {
            let root_node = {
                let nodes_read = self.nodes.read().unwrap();
                nodes_read[root].clone()
            };
            match fs::symlink_metadata(&root_node.path) {
                Ok(meta) => {
                    if meta.is_dir() {
                        fs::read_dir(root_node.path).map_err(|e| {
                            if e.kind() == std::io::ErrorKind::PermissionDenied {
                                AppError::AccessDenied
                            } else {
                                AppError::Fatal(e.to_string())
                            }
                        })?;
                    }
                    self.total_size = HumanReadableSize(self.traverse()?);
                    return Ok(());
                }
                Err(e) => {
                    return Err(match e.kind() {
                        std::io::ErrorKind::NotFound => AppError::NotFound,
                        _ => AppError::Fatal(e.to_string()),
                    });
                }
            }
        }
        Ok(())
    }

    fn traverse(&mut self) -> Result<u64, AppError> {
        if let Some(root_id) = self.root {
            let mut total_size = 0;
            let mut stack: Vec<usize> = vec![root_id];

            while let Some(node_id) = stack.pop() {
                let node = self.nodes.read().unwrap()[node_id].clone();
                let metadata = match fs::symlink_metadata(&node.path) {
                    Ok(data) => data,
                    Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                        println!("permission denied: {}", node.path.display());
                        continue;
                    }
                    Err(e) => return Err(AppError::Fatal(e.to_string())),
                };

                let physical_size = metadata.blocks() * 512;
                total_size += physical_size;
                self.bubble_up_size(node_id, physical_size);

                if metadata.is_dir() {
                    if let Ok(entries) = fs::read_dir(&node.path) {
                        let mut idx = self.nodes.read().unwrap().len();
                        let mut new_children = Vec::new();
                        let mut child_indices = Vec::new();
                        let current_depth = node.depth;

                        for entry in entries.flatten() {
                            new_children.push(Node::new(
                                idx,
                                Some(node_id),
                                entry.path(),
                                current_depth + 1,
                                entry.metadata().unwrap().is_dir(),
                            ));
                            child_indices.push(idx);
                            stack.push(idx);
                            idx += 1;
                        }

                        if !new_children.is_empty() {
                            let mut write_nodes = self.nodes.write().unwrap();
                            write_nodes.extend(new_children);
                            write_nodes[node_id].children.extend(child_indices);
                        }
                    }
                }
            }
            let mut nodes = self.nodes.write().unwrap();
            for i in 0..nodes.len() {
                let mut children = nodes[i].children.clone();
                children.sort_by(|&a, &b| {
                    nodes[a]
                        .children
                        .len()
                        .cmp(&nodes[b].children.len())
                        .reverse()
                        .then_with(|| nodes[b].size.cmp(&nodes[a].size))
                        .then_with(|| nodes[a].path.cmp(&nodes[b].path))
                });
                nodes[i].children = children;
            }
            Ok(total_size)
        } else {
            Ok(0)
        }
    }

    //     fn post_process(&self) -> Result<(),AppError>{

    // }

    fn bubble_up_size(&mut self, node_id: usize, size: u64) {
        let mut current_id = node_id;

        loop {
            if let Some(node) = self.nodes.write().unwrap().get_mut(current_id) {
                node.size += size;
            }

            let parent_id = self.nodes.read().unwrap().get(current_id).unwrap().parent;

            if let Some(pid) = parent_id {
                current_id = pid;
            } else {
                break;
            }
        }
    }
}

fn shorten_name(name: String, max_len: u16) -> String {
    let max = max_len as usize;
    let chars: Vec<char> = name.chars().collect();
    if chars.len() <= max {
        return name;
    }
    let half = (max - 1) / 2;
    let left: String = chars[..half].iter().collect();
    let right: String = chars[chars.len() - (max - half - 1)..].iter().collect();
    format!("{}…{}", left, right)
}

struct PrintState {
    node_idx: usize,
    is_last_child: bool,
    ancestor_is_last: Vec<bool>,
}

pub fn print_entries(entries: &mut Vec<Node>, total_size: u64, options: InfoOptions) {
    let mut stack: Vec<PrintState> = Vec::new();

    let root_indices: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.depth == 0)
        .map(|(idx, _)| idx)
        .collect();

    for &idx in root_indices.iter().rev() {
        stack.push(PrintState {
            node_idx: idx,
            is_last_child: true,
            ancestor_is_last: Vec::new(),
        });
    }

    while let Some(state) = stack.pop() {
        let entry = &entries[state.node_idx];

        if options.dir_only && !entry.is_dir {
            continue;
        }

        if entry.depth <= options.info_level {
            print_node(
                entry,
                total_size,
                &options,
                &state.ancestor_is_last,
                state.is_last_child,
            );
        }

        if entry.depth >= options.info_level {
            continue;
        }

        let mut child_ancestors = state.ancestor_is_last.clone();
        if entry.depth > 0 {
            child_ancestors.push(state.is_last_child);
        }

        let children_count = entry.children.len();
        for (idx, &child_idx) in entry.children.iter().enumerate().rev() {
            let child_is_last = idx == children_count - 1;

            stack.push(PrintState {
                node_idx: child_idx,
                is_last_child: child_is_last,
                ancestor_is_last: child_ancestors.clone(),
            });
        }
    }
}

pub fn print_node(
    n: &Node,
    total_size: u64,
    options: &InfoOptions,
    ancestor_is_last: &[bool],
    is_last_child: bool,
) {
    let size = &n.size;

    let mut prefix = String::new();
    for &parent_was_last in ancestor_is_last {
        if parent_was_last {
            prefix.push_str("    ");
        } else {
            prefix.push_str("│   ");
        }
    }

    let connector = if n.depth == 0 {
        ""
    } else if is_last_child {
        "└── "
    } else {
        "├── "
    };

    let name = if n.depth == 0 {
        let path = if n.id == 0 {
            &fs::canonicalize(&n.path).unwrap()
        } else {
            &n.path
        };
        path.to_str().unwrap().to_string()
    } else {
        n.path.file_name().unwrap().to_str().unwrap().to_string()
    };

    let display = if options.shorten {
        shorten_name(name, options.max_len)
    } else {
        name
    };

    let mut output = format!(
        "{}{}{:<left_space$}",
        prefix,
        connector,
        display,
        left_space = sub_min_from_max!(5, display.len())
    );

    // check if we need to show size
    if !options.show_percent_only {
        output.push_str(&format!(" {}", size));
    }

    // check if we need to show percent also
    if !options.show_size_only {
        let size_f64: f64 = size.clone().into();
        let percent = (size_f64 / total_size as f64) * 100.0;
        let percent_str = format!("{:.5}", percent);

        let mut trimmed = percent_str.trim_end_matches('0');
        trimmed = trimmed.trim_end_matches('.');
        if options.show_percent_only {
            output.push_str(&format!(" {}%", trimmed));
        } else {
            output.push_str(&format!(" ({}%)", trimmed));
        }
    }

    println!("{}", output);
}
