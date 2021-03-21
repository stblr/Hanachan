use std::fs;
use std::path::Path;

use crate::error;
use crate::take::{self, Take, TakeFromSlice, TakeIter};
use crate::yaz;

#[derive(Clone, Debug)]
pub struct U8 {
    nodes: Vec<Node>,
}

impl U8 {
    pub fn open_szs<P: AsRef<Path>>(path: P) -> Result<U8, error::Error> {
        let compressed = fs::read(path)?;
        let decompressed = yaz::decompress(&compressed)?;
        U8::parse(&decompressed).map_err(Into::into)
    }

    fn parse(mut input: &[u8]) -> Result<U8, Error> {
        let fourcc = input.take::<u32>()?;
        if fourcc != u32::from_be_bytes(*b"U\xaa8-") {
            return Err(Error {});
        }

        let first_node_offset = input.take::<u32>()? as usize;
        if first_node_offset != 0x20 {
            return Err(Error {});
        }

        let fs_size = input.take::<u32>()? as usize;
        let file_data_offset = (input.take::<u32>()? as usize)
            .checked_sub(0x20)
            .ok_or(Error {})?;
        if fs_size > file_data_offset {
            return Err(Error {});
        }

        for _ in 0..4 {
            let _reserved = input.take::<u32>()?;
        }

        let mut node_iter = TakeIter::new(input).peekable();
        let root: RawNode = *node_iter.peek().ok_or(Error {})?;
        let node_count = match root.content {
            RawNodeContent::Directory { next, .. } => next,
            _ => return Err(Error {}),
        };
        let node_iter = node_iter.take(node_count);

        let name_pool = input.get(0xc * node_count..fs_size).ok_or(Error {})?;
        let name_iter = TakeIter::new(name_pool).scan(0, |offset, name: String| {
            let start_offset = *offset;
            *offset += name.len() + 1;
            Some((start_offset, name))
        });

        let nodes = node_iter
            .zip(name_iter)
            .try_fold::<_, _, Result<Vec<Node>, Error>>(
                vec![],
                |mut nodes, (raw, (name_offset, name))| {
                    let node = Node::try_from_raw(raw, name_offset, name, &nodes)?;
                    nodes.push(node);
                    Ok(nodes)
                },
            )?;

        Ok(U8 { nodes })
    }
}

#[derive(Clone, Copy, Debug)]
struct RawNode {
    name_offset: usize,
    content: RawNodeContent,
}

#[derive(Clone, Copy, Debug)]
enum RawNodeContent {
    File { offset: usize, size: usize },
    Directory { parent: usize, next: usize },
}

impl TakeFromSlice for RawNode {
    fn take_from_slice(slice: &mut &[u8]) -> Result<RawNode, take::Error> {
        let val = slice.take::<u32>()? as usize;
        let name_offset = val & 0xffffff;
        let content = match val >> 24 {
            0 => RawNodeContent::File {
                offset: slice.take::<u32>()? as usize,
                size: slice.take::<u32>()? as usize,
            },
            1 => RawNodeContent::Directory {
                parent: slice.take::<u32>()? as usize,
                next: slice.take::<u32>()? as usize,
            },
            _ => return Err(take::Error {}),
        };
        Ok(RawNode {
            name_offset,
            content,
        })
    }
}

#[derive(Clone, Debug)]
struct Node {
    name: String,
    content: NodeContent,
}

#[derive(Clone, Debug)]
enum NodeContent {
    File,
    Directory { parent: usize, next: usize },
}

impl Node {
    fn try_from_raw(
        raw: RawNode,
        name_offset: usize,
        name: String,
        nodes: &Vec<Node>,
    ) -> Result<Node, Error> {
        let is_root = nodes.is_empty();

        if name_offset != raw.name_offset {
            return Err(Error {});
        }
        if is_root != name.is_empty() {
            return Err(Error {});
        }

        let content = match raw.content {
            RawNodeContent::File { .. } => NodeContent::File,
            RawNodeContent::Directory { parent, next } => {
                if !is_root {
                    match nodes.get(parent).ok_or(Error {})?.content {
                        NodeContent::File => return Err(Error {}),
                        NodeContent::Directory {
                            next: parent_next, ..
                        } => {
                            if next > parent_next {
                                return Err(Error {});
                            }
                        }
                    }
                }
                NodeContent::Directory { parent, next }
            }
        };

        Ok(Node { name, content })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Error {}

impl From<take::Error> for Error {
    fn from(_: take::Error) -> Error {
        Error {}
    }
}
