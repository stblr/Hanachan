use std::convert::TryFrom;
use std::iter;

use crate::fs::{Error, Parse, SliceRefExt};
use crate::geom::Vec3;

#[derive(Clone, Debug)]
pub struct Kcl {
    thickness: f32,
    origin: Vec3,
    x_mask: u32,
    y_mask: u32,
    z_mask: u32,
    shift: u32,
    y_shift: u32,
    z_shift: u32,
    root_nodes: Vec<Node>,
    branches: Vec<[Node; 8]>,
}

impl Parse for Kcl {
    fn parse(input: &mut &[u8]) -> Result<Kcl, Error> {
        input.skip(0xc)?;
        let octree_offset: u32 = input.take()?;
        let thickness = input.take()?;
        let origin = input.take()?;
        let x_mask: u32 = input.take()?;
        let y_mask: u32 = input.take()?;
        let z_mask: u32 = input.take()?;
        let shift = input.take()?;
        let y_shift = input.take()?;
        let z_shift: u32 = input.take()?;

        input.skip(octree_offset as usize - 0x38)?;

        let root_bits_x = x_mask.trailing_zeros().checked_sub(shift).ok_or(Error {})?;
        if root_bits_x != y_shift {
            return Err(Error {});
        }
        let root_bits_y = y_mask.trailing_zeros().checked_sub(shift).ok_or(Error {})?;
        if root_bits_y != z_shift.checked_sub(y_shift).ok_or(Error {})? {
            return Err(Error {});
        }
        let root_bits_z = z_mask.trailing_zeros().checked_sub(shift).ok_or(Error {})?;
        let root_bits = root_bits_x + root_bits_y + root_bits_z;
        let root_node_count = 1u32.checked_shl(root_bits).ok_or(Error {})?;
        let root_node_offset = -i32::try_from(0x4 * root_node_count).map_err(|_| Error {})?;

        let mut branch_count = 0;
        let mut tri_lists_offset = input.len() as u32;

        fn parse_node(input: &mut &[u8], branch_count: &mut u32, tri_lists_offset: &mut u32, parent_offset: i32) -> Result<RawNode, Error> {
            let offset = input.take::<u32>()?;
            let is_leaf = offset & 0x80000000 != 0;
            let offset = offset & !0x80000000;
            let offset = i32::try_from(offset).map_err(|_| Error {})? + parent_offset;
            let offset = u32::try_from(offset).map_err(|_| Error {})?;
            if is_leaf {
                *tri_lists_offset = (*tri_lists_offset).min(offset);
                Ok(RawNode::Leaf { offset })
            } else if offset % 0x20 == 0 {
                let idx = offset / 0x20;
                *branch_count = (*branch_count).max(idx + 1);
                Ok(RawNode::Branch { idx })
            } else {
                Err(Error {})
            }
        }

        let root_nodes: Vec<_> = iter::repeat_with(|| {
            parse_node(input, &mut branch_count, &mut tri_lists_offset, root_node_offset)
        }).take(root_node_count as usize).collect::<Result<_, _>>()?;

        let branches: Vec<_> = (0..).scan((), |(), branch_idx| {
            if branch_idx >= branch_count {
                return None;
            }

            let branch_offset = match i32::try_from(0x20 * branch_idx).map_err(|_| Error {}) {
                Ok(branch_offset) => branch_offset,
                Err(e) => return Some(Err(e)),
            };
            let mut nodes = [RawNode::Leaf { offset: 0 }; 8];
            for node in &mut nodes {
                *node = match parse_node(input, &mut branch_count, &mut tri_lists_offset, branch_offset) {
                    Ok(node) => node,
                    Err(e) => return Some(Err(e)),
                };
            }
            Some(Ok(nodes))
        }).collect::<Result<_, _>>()?;

        let mut tri_list_offset = tri_lists_offset + 0x2;
        if branch_count * 0x20 != tri_list_offset {
            return Err(Error {});
        }

        let mut tri_lists: Vec<_> = iter::from_fn(|| {
            if input.is_empty() {
                None
            } else {
                let tris = iter::from_fn(|| {
                    match input.take::<u16>() {
                        Ok(idx) if idx != 0 => Some(Ok(idx)),
                        Ok(_) => None,
                        Err(e) => Some(Err(e)),
                    }
                }).collect::<Result<_, _>>();
                match tris {
                    Ok(tris) => {
                        let tri_list = RawTriList {
                            offset: tri_list_offset,
                            tris,
                        };
                        tri_list_offset += 0x2 * (tri_list.tris.len() as u32 + 1);
                        Some(Ok(tri_list))
                    },
                    Err(e) => Some(Err(e)),
                }
            }
        }).collect::<Result<_, _>>()?;
        tri_lists.push(RawTriList {
            offset: tri_list_offset - 0x2,
            tris: vec![],
        });

        let root_nodes = root_nodes.into_iter().map(|raw| Node::try_from_raw(raw, &tri_lists)).collect::<Result<_, _>>()?;

        // TODO replace with into_iter
        let branches = branches.iter().map(|raws| {
            let mut nodes = [Node::Leaf { idx: 0 }; 8];
            for (raw, node) in raws.into_iter().zip(nodes.iter_mut()) {
                *node = Node::try_from_raw(*raw, &tri_lists)?;
            }
            Ok(nodes)
        }).collect::<Result<_, _>>()?;

        Ok(Kcl {
            thickness,
            origin,
            x_mask,
            y_mask,
            z_mask,
            shift,
            y_shift,
            z_shift,
            root_nodes,
            branches,
        })
    }
}

#[derive(Clone, Debug)]
struct Triangle {
    altitude: f32,
}

#[derive(Clone, Copy, Debug)]
enum RawNode {
    Leaf {
        offset: u32,
    },
    Branch {
        idx: u32,
    },
}

#[derive(Clone, Copy, Debug)]
enum Node {
    Leaf {
        idx: usize,
    },
    Branch {
        idx: u32,
    },
}

impl Node {
    fn try_from_raw(raw: RawNode, tri_lists: &Vec<RawTriList>) -> Result<Node, Error> {
        match raw {
            RawNode::Leaf { offset } => {
                let offset = offset + 0x2;
                let idx = tri_lists.iter().position(|tri_list| tri_list.offset == offset).ok_or(Error {})?;
                Ok(Node::Leaf { idx })
            }
            RawNode::Branch { idx } => Ok(Node::Branch { idx }),
        }
    }
}

#[derive(Clone, Debug)]
struct RawTriList {
    offset: u32,
    tris: Vec<u16>,
}
