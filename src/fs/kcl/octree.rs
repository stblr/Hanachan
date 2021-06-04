use std::iter;

use crate::fs::{kcl::Header, Error, Parse, SliceRefExt};
use crate::geom::Vec3;

#[derive(Clone, Debug)]
pub struct Octree {
    root_nodes: Vec<Node>,
    branches: Vec<[Node; 8]>,
    tri_lists: Vec<Vec<u16>>,
}

impl Octree {
    pub fn is_valid(&self, root_node_count: u32, tri_count: usize) -> bool {
        if self.root_nodes.len() != root_node_count as usize {
            return false;
        }

        self.tri_lists
            .iter()
            .flatten()
            .all(|idx| (*idx as usize) < tri_count)
    }

    pub fn find_tri_list(&self, header: &Header, pos: Vec3) -> Option<&Vec<u16>> {
        let x = (pos.x - header.origin.x) as u32;
        if x & header.x_mask != 0 {
            return None;
        }

        let y = (pos.y - header.origin.y) as u32;
        if y & header.y_mask != 0 {
            return None;
        }

        let z = (pos.z - header.origin.z) as u32;
        if z & header.z_mask != 0 {
            return None;
        }

        let mut shift = header.shift;
        let node_idx = (z >> shift) << header.z_shift | (y >> shift) << header.y_shift | x >> shift;
        let mut node = self.root_nodes[node_idx as usize];
        loop {
            match node {
                Node::Leaf { idx: tri_list_idx } => {
                    break self.tri_lists.get(tri_list_idx as usize)
                }
                Node::Branch { idx: branch_idx } => {
                    let branch = self.branches[branch_idx as usize];
                    shift -= 1;
                    let node_idx = (z >> shift & 1) << 2 | (y >> shift & 1) << 1 | (x >> shift & 1);
                    node = branch[node_idx as usize];
                }
            }
        }
    }
}

impl Parse for Octree {
    fn parse(mut input: &mut &[u8]) -> Result<Octree, Error> {
        let mut root_node_count = input.len() as u32 / 4;
        let mut nodes_size = 0;
        let mut tri_lists_offset = input.len() as u32;

        fn parse_node(
            input: &mut &[u8],
            nodes_size: &mut u32,
            tri_lists_offset: &mut u32,
            parent_offset: u32,
        ) -> Result<RawNode, Error> {
            let offset = input.take::<u32>()?;
            let is_leaf = offset & 0x80000000 != 0;
            let offset = offset & !0x80000000;
            let offset = offset + parent_offset;
            if is_leaf {
                let offset = offset + 0x2;
                *tri_lists_offset = (*tri_lists_offset).min(offset);
                Ok(RawNode::Leaf { offset })
            } else if offset % 0x4 == 0 {
                *nodes_size = (*nodes_size).max(offset + 0x20);
                Ok(RawNode::Branch { offset })
            } else {
                Err(Error {})
            }
        }

        let root_nodes: Vec<_> = (0..)
            .scan((), |(), root_node_idx| {
                if root_node_idx >= root_node_count {
                    return None;
                }

                let node = parse_node(&mut input, &mut nodes_size, &mut tri_lists_offset, 0);
                if let Ok(RawNode::Leaf { offset }) | Ok(RawNode::Branch { offset }) = node {
                    root_node_count = root_node_count.min(offset / 0x4);
                }
                Some(node)
            })
            .collect::<Result<_, _>>()?;

        let branches_offset = 0x4 * root_node_count;
        nodes_size = nodes_size.max(branches_offset);
        let branches: Vec<_> = (0..)
            .scan((), |(), branch_idx| {
                let branch_count = (nodes_size - branches_offset) / 0x20;
                if branch_idx >= branch_count {
                    return None;
                }

                let branch_offset = branches_offset + 0x20 * branch_idx;
                let mut nodes = [RawNode::Leaf { offset: 0 }; 8];
                for node in &mut nodes {
                    *node = match parse_node(
                        &mut input,
                        &mut nodes_size,
                        &mut tri_lists_offset,
                        branch_offset,
                    ) {
                        Ok(node) => node,
                        Err(e) => return Some(Err(e)),
                    };
                }
                Some(Ok(nodes))
            })
            .collect::<Result<_, _>>()?;

        if nodes_size != tri_lists_offset {
            return Err(Error {});
        }

        let mut tri_list_offset = tri_lists_offset;
        let mut tri_lists: Vec<_> = iter::from_fn(|| {
            if input.is_empty() {
                None
            } else {
                let tris = iter::from_fn(|| match input.take::<u16>() {
                    Ok(idx) => idx.checked_sub(1).map(|idx| Ok(idx)),
                    Err(e) => Some(Err(e)),
                })
                .collect::<Result<_, _>>();
                match tris {
                    Ok(tris) => {
                        let tri_list = RawTriList {
                            offset: tri_list_offset,
                            tris,
                        };
                        tri_list_offset += 0x2 * (tri_list.tris.len() as u32 + 1);
                        Some(Ok(tri_list))
                    }
                    Err(e) => Some(Err(e)),
                }
            }
        })
        .collect::<Result<_, _>>()?;
        tri_lists.push(RawTriList {
            offset: tri_list_offset - 0x2,
            tris: vec![],
        });

        let root_nodes = root_nodes
            .into_iter()
            .map(|raw| Node::try_from_raw(raw, branches_offset, &tri_lists))
            .collect::<Result<_, _>>()?;

        // TODO replace with into_iter
        let branches = branches
            .iter()
            .map(|raws| {
                let mut nodes = [Node::Leaf { idx: 0 }; 8];
                for (raw, node) in raws.into_iter().zip(nodes.iter_mut()) {
                    *node = Node::try_from_raw(*raw, branches_offset, &tri_lists)?;
                }
                Ok(nodes)
            })
            .collect::<Result<_, _>>()?;

        let tri_lists = tri_lists
            .into_iter()
            .map(|tri_list| tri_list.tris)
            .collect();

        Ok(Octree {
            root_nodes,
            branches,
            tri_lists,
        })
    }
}

#[derive(Clone, Copy, Debug)]
enum RawNode {
    Leaf { offset: u32 },
    Branch { offset: u32 },
}

#[derive(Clone, Copy, Debug)]
enum Node {
    Leaf { idx: u32 },
    Branch { idx: u32 },
}

impl Node {
    fn try_from_raw(
        raw: RawNode,
        branches_offset: u32,
        tri_lists: &Vec<RawTriList>,
    ) -> Result<Node, Error> {
        match raw {
            RawNode::Leaf { offset } => {
                let offset = offset;
                let idx = tri_lists
                    .iter()
                    .position(|tri_list| tri_list.offset == offset)
                    .ok_or(Error {})?;
                Ok(Node::Leaf { idx: idx as u32 })
            }
            RawNode::Branch { offset } => match offset.checked_sub(branches_offset) {
                Some(offset) if offset % 0x20 == 0 => Ok(Node::Branch { idx: offset / 0x20 }),
                _ => Err(Error {}),
            },
        }
    }
}

#[derive(Clone, Debug)]
struct RawTriList {
    offset: u32,
    tris: Vec<u16>,
}
