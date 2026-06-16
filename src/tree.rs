use crate::{error::AppError, hrsize::HumanReadableSize};

use std::os::unix::fs::MetadataExt;
use std::sync::RwLock;
use std::{fs, path::PathBuf};

#[derive(Clone)]
pub struct Node {
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
    pub show_percent: bool,
}

impl Node {
    pub fn new(parent: Option<usize>, path: PathBuf, depth: u8, is_dir: bool) -> Self {
        Node {
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

pub fn print_entries(entries: &mut Vec<Node>, total_size: u64, options: InfoOptions) {
    entries.sort_by(|a, b| {
        a.depth
            .cmp(&b.depth)
            .then_with(|| a.size.cmp(&b.size).reverse())
            .then_with(|| a.path.cmp(&b.path))
    });

    for entry in entries {
        // skip if directory only mode and entry is not directory
        if options.dir_only && !entry.is_dir {
            continue;
        }
        if entry.depth <= options.info_level {
            let size = &entry.size;
            let indent = (entry.depth * 4) as usize;
            let name = if entry.depth == 0 {
                entry.path.to_str().unwrap().to_string()
            } else {
                entry
                    .path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            };
            let display = if options.shorten {
                shorten_name(name, options.max_len)
            } else {
                name
            };
            let mut output = format!(
                "{:indent_width$}{:name_pad$} {}",
                "",
                display,
                size,
                indent_width = indent,
                name_pad = options.max_len as usize
            );
            if options.show_percent {
                let size_f64: f64 = size.into();
                let percent = (size_f64 / total_size as f64) * 100.0;
                output.push_str(&format!(" {:.5}%", percent));
            }
            println!("{}", output);
        }
    }
}
