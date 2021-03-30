use core::iter;

use std::fs;
use std::path::Path;

use crate::bsp::{self, Bsp};
use crate::driver_param::DriverParam;
use crate::error;
use crate::kart_param::KartParam;
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
        let mut file_data_offset = (input.take::<u32>()? as usize)
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

        let file_data = input.get(file_data_offset..).ok_or(Error {})?;
        file_data_offset += 0x20;

        let nodes = node_iter
            .zip(name_iter)
            .try_fold::<_, _, Result<Vec<Node>, Error>>(
                vec![],
                |mut nodes, (raw, (name_offset, name))| {
                    let node = Node::try_from_raw(
                        raw,
                        name_offset,
                        name,
                        file_data_offset,
                        file_data,
                        &nodes,
                    )?;
                    nodes.push(node);
                    Ok(nodes)
                },
            )?;

        Ok(U8 { nodes })
    }

    pub fn get_node(&self, path: &str) -> Option<&Node> {
        let mut path_iter = path.split("/");
        let index =
            path_iter
                .by_ref()
                .try_fold(0, |index, name| match self.nodes[index].content {
                    NodeContent::File { .. } => None,
                    NodeContent::Directory { next, .. } => {
                        iter::successors(Some(index + 1), |index| {
                            match self.nodes.get(*index)?.content {
                                NodeContent::File { .. } => Some(index + 1),
                                NodeContent::Directory { next, .. } => Some(next),
                            }
                        })
                        .take_while(|index| *index < next)
                        .find(|index| self.nodes[*index].name == name)
                    }
                })?;
        match path_iter.next() {
            None => Some(&self.nodes[index]),
            Some(_) => None,
        }
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
pub struct Node {
    name: String,
    content: NodeContent,
}

impl Node {
    fn try_from_raw(
        raw: RawNode,
        name_offset: usize,
        name: String,
        file_data_offset: usize,
        file_data: &[u8],
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
            RawNodeContent::File { mut offset, size } => {
                offset -= file_data_offset;
                let input = file_data.get(offset..offset + size).ok_or(Error {})?;
                NodeContent::File(File::parse(&name, input)?)
            }
            RawNodeContent::Directory { parent, next } => {
                if !is_root {
                    match nodes.get(parent).ok_or(Error {})?.content {
                        NodeContent::File(_) => return Err(Error {}),
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

    pub fn content(&self) -> &NodeContent {
        &self.content
    }
}

#[derive(Clone, Debug)]
pub enum NodeContent {
    File(File),
    Directory { parent: usize, next: usize },
}

impl NodeContent {
    pub fn as_file(&self) -> Option<&File> {
        match self {
            NodeContent::File(file) => Some(file),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum File {
    Bsp(Bsp),
    DriverParam(DriverParam),
    KartParam(KartParam),
    Other,
}

impl File {
    fn parse(name: &str, mut input: &[u8]) -> Result<File, Error> {
        if name == "driverParam.bin" {
            Ok(File::DriverParam(input.take()?))
        } else if name == "kartParam.bin" {
            Ok(File::KartParam(input.take()?))
        } else if name.ends_with(".bsp") {
            Ok(File::Bsp(Bsp::parse(input)?))
        } else {
            Ok(File::Other)
        }
    }

    pub fn as_bsp(&self) -> Option<&Bsp> {
        match self {
            File::Bsp(bsp) => Some(bsp),
            _ => None,
        }
    }

    pub fn as_driver_param(&self) -> Option<&DriverParam> {
        match self {
            File::DriverParam(driver_param) => Some(driver_param),
            _ => None,
        }
    }

    pub fn as_kart_param(&self) -> Option<&KartParam> {
        match self {
            File::KartParam(kart_param) => Some(kart_param),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Error {}

impl From<take::Error> for Error {
    fn from(_: take::Error) -> Error {
        Error {}
    }
}

// TODO proper error handling
impl From<bsp::Error> for Error {
    fn from(_: bsp::Error) -> Error {
        Error {}
    }
}
