use crate::hrsize::HumanReadableSize;
use std::fs::Metadata;
use std::path::PathBuf;

#[derive(Clone)]
pub enum NodeType {
    Directory,
    File,
    Symlink,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Node {
    pub path: PathBuf,
    pub depth: u16,
    pub size: HumanReadableSize,
    pub children: Vec<Node>,
    pub node_type: NodeType,
}
impl Node {
    pub fn new(path: PathBuf, size: u64, depth: u16, node_type: NodeType) -> Self {
        Node {
            children: Vec::new(),
            size: HumanReadableSize(size),
            path,
            depth,
            node_type,
        }
    }
}

impl From<&Metadata> for NodeType {
    fn from(md: &Metadata) -> Self {
        let ft = md.file_type();
        if ft.is_dir() {
            NodeType::Directory
        } else if ft.is_symlink() {
            NodeType::Symlink
        } else {
            NodeType::File
        }
    }
}
