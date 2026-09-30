use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use crate::error::AppError::{self, Fatal};
use crate::tree::node::{Node, NodeType};
use std::fs::{self, Metadata};
pub mod node;
pub mod printer;

// One inode on one device. Inode numbers are only unique per device,
// so a pair is needed for hard links.
pub type Inode = (u64, u64);

// Keeps a hard-linked inode from being
// counted more than once and stops a directory from being re-walked.
pub type Seen = HashSet<Inode>;

pub struct Tree {
    pub root: Node,
}

impl Tree {
    // Returns `None` when this inode was already visited, so a hard-linked
    // name or a second mount of a directory is ignored
    fn traverse(
        path: impl Into<PathBuf>,
        depth: u16,
        seen: &mut Seen,
    ) -> Result<Option<Node>, AppError> {
        let path = path.into();
        let md = fs::symlink_metadata(&path).map_err(|e| Fatal(e.to_string()))?;
        let key = inode_key(&md);

        if !seen.insert(key) {
            return Ok(None);
        }

        if !md.is_dir() {
            return Ok(Some(Node::new(
                path,
                allocated_bytes(&md),
                depth,
                NodeType::from(&md),
            )));
        }

        let mut total_size = allocated_bytes(&md);
        let mut children = Vec::new();

        let mut entries: Vec<PathBuf> = fs::read_dir(&path)?
            .inspect(|r| {
                if let Err(e) = r {
                    eprintln!("cannot read dir: {e}")
                }
            })
            .flatten()
            .map(|entry| entry.path())
            .collect();
        entries.sort();

        for entry_path in entries {
            match Self::traverse(entry_path.clone(), depth + 1, seen) {
                Ok(Some(n)) => {
                    total_size = n.size + total_size;
                    children.push(n);
                }
                Ok(None) => {
                    // already seen, do nothing
                }
                Err(e) => eprintln!("{}: {e}", entry_path.display()),
            }
        }

        let mut node = Node::new(path, total_size, depth, NodeType::Directory);
        node.children = children;
        Ok(Some(node))
    }
    pub fn new(path: PathBuf) -> Result<Self, AppError> {
        let mut seen = Seen::default();
        let node = Self::traverse(path, 0, &mut seen)?
            .ok_or_else(|| Fatal("root path was already visited".to_string()))?;
        Ok(Self { root: node })
    }
}

#[inline]
fn inode_key(metadata: &Metadata) -> Inode {
    (metadata.dev(), metadata.ino())
}

#[inline]
fn allocated_bytes(metadata: &Metadata) -> u64 {
    #[cfg(unix)]
    {
        // st_blocks returns 512-byte block units allocated on disk
        metadata.blocks() * 512
    }

    #[cfg(not(unix))]
    {
        // Fallback for non-Unix platforms where block allocation isn't directly exposed
        metadata.len()
    }
}
