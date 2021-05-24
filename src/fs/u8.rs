use std::iter;

use crate::fs::*;

#[derive(Clone, Debug)]
pub struct U8 {
    nodes: Vec<Node>,
}

impl U8 {
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

impl Parse for U8 {
    fn parse(input: &mut &[u8]) -> Result<U8, Error> {
        input
            .take::<u32>()
            .filter(|fourcc| *fourcc == u32::from_be_bytes(*b"U\xaa8-"))?;
        input
            .take::<u32>()
            .filter(|first_node_offset| *first_node_offset == 0x20)?;

        let fs_size = input.take::<u32>()? as usize;
        let mut file_data_offset = (input.take::<u32>()? as usize)
            .checked_sub(0x20)
            .ok_or(Error {})
            .filter(|file_data_offset| *file_data_offset >= fs_size)?;

        for _ in 0..4 {
            let _reserved = input.take::<u32>()?;
        }

        let root = input.clone().take::<RawNode>()?;
        let node_count = match root.content {
            RawNodeContent::Directory { next, .. } => next,
            _ => return Err(Error {}),
        };

        let (mut nodes_input, input) = input.try_split_at(0xc * node_count).ok_or(Error {})?;
        let node_iter = iter::repeat_with(|| nodes_input.take());

        let names_size = fs_size.checked_sub(0xc * node_count).ok_or(Error {})?;
        let (mut names_input, input) = input.try_split_at(names_size).ok_or(Error {})?;
        let name_iter =
            iter::repeat_with(|| names_input.take()).scan(0, |offset, name: Result<String, _>| {
                match name {
                    Ok(name) => {
                        let start_offset = *offset;
                        *offset += name.len() + 1;
                        Some(Ok((start_offset, name)))
                    }
                    Err(e) => Some(Err(e)),
                }
            });

        let (_padding, file_data) = input
            .try_split_at(file_data_offset - fs_size)
            .ok_or(Error {})?;
        file_data_offset += 0x20;

        let nodes = node_iter
            .zip(name_iter)
            .take(node_count)
            .try_fold::<_, _, _>(vec![], |mut nodes, (raw, name)| {
                let (name_offset, name) = name?;
                let node = Node::try_from_raw(
                    raw?,
                    name_offset,
                    name,
                    file_data_offset,
                    file_data,
                    &nodes,
                )?;
                nodes.push(node);
                Ok(nodes)
            })?;

        Ok(U8 { nodes }).filter(|_| nodes_input.is_empty() && names_input.is_empty())
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

impl Parse for RawNode {
    fn parse(input: &mut &[u8]) -> Result<RawNode, Error> {
        let val = input.take::<u32>()? as usize;
        let name_offset = val & 0xffffff;
        let content = match val >> 24 {
            0 => RawNodeContent::File {
                offset: input.take::<u32>()? as usize,
                size: input.take::<u32>()? as usize,
            },
            1 => RawNodeContent::Directory {
                parent: input.take::<u32>()? as usize,
                next: input.take::<u32>()? as usize,
            },
            _ => return Err(Error {}),
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
    BikePartsDispParam(BikePartsDispParam),
    Bsp(Bsp),
    DriverParam(DriverParam),
    KartParam(KartParam),
    Kcl(Kcl),
    Kmp(Kmp),
    Other,
}

impl File {
    fn parse(name: &str, mut input: &[u8]) -> Result<File, Error> {
        if name == "bikePartsDispParam.bin" {
            Ok(File::BikePartsDispParam(input.take()?))
        } else if name == "driverParam.bin" {
            Ok(File::DriverParam(input.take()?))
        } else if name == "kartParam.bin" {
            Ok(File::KartParam(input.take()?))
        } else if name.ends_with(".bsp") {
            Ok(File::Bsp(input.take()?))
        } else if name.ends_with(".kcl") {
            Ok(File::Kcl(input.take()?))
        } else if name.ends_with(".kmp") {
            Ok(File::Kmp(input.take()?))
        } else {
            Ok(File::Other)
        }
    }

    pub fn as_bike_parts_disp_param(&self) -> Option<&BikePartsDispParam> {
        match self {
            File::BikePartsDispParam(bike_parts_disp_param) => Some(bike_parts_disp_param),
            _ => None,
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

    pub fn as_kcl(&self) -> Option<&Kcl> {
        match self {
            File::Kcl(kcl) => Some(kcl),
            _ => None,
        }
    }

    pub fn as_kmp(&self) -> Option<&Kmp> {
        match self {
            File::Kmp(kmp) => Some(kmp),
            _ => None,
        }
    }
}
