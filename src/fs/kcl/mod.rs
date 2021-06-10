mod collision;
mod header;
mod octree;
mod tri;

pub use collision::Collision;

use std::iter;

use crate::fs::{Error, Parse, ResultExt, SliceExt, SliceRefExt};
use crate::geom::Hitbox;

use header::Header;
use octree::Octree;
use tri::Tri;

#[derive(Clone, Debug)]
pub struct Kcl {
    header: Header,
    tris: Vec<Tri>,
    octree: Octree,
}

impl Kcl {
    pub fn check_collision(&self, hitbox: Hitbox) -> Collision {
        let mut collision = Collision::new();

        let tri_list = match self.octree.find_tri_list(&self.header, hitbox.pos) {
            Some(tri_list) => tri_list,
            None => return collision,
        };

        for tri_idx in tri_list.iter() {
            let tri = &self.tris[*tri_idx as usize];
            if let Some(tri_collision) = tri.check_collision(self.header.thickness, hitbox) {
                collision.add(tri_collision);
            }
        }

        collision
    }
}

impl Parse for Kcl {
    fn parse(input: &mut &[u8]) -> Result<Kcl, Error> {
        let header = input.take::<Header>()?;

        fn parse_section<T: Parse>(
            input: &mut &[u8],
            offset: u32,
            next_offset: u32,
        ) -> Result<Vec<T>, Error> {
            let size = next_offset.checked_sub(offset).ok_or(Error {})?;
            let (mut head, tail) = input.try_split_at(size as usize).ok_or(Error {})?;
            *input = tail;
            iter::from_fn(|| (!head.is_empty()).then(|| head.take())).collect()
        }

        let poss = parse_section(input, header.poss_offset, header.nors_offset)?;
        let nors = parse_section(input, header.nors_offset, header.tris_offset)?;

        let tris = parse_section(input, header.tris_offset, header.octree_offset)?;
        let tris: Vec<_> = tris
            .into_iter()
            .map(|raw| Tri::try_from_raw(raw, &poss, &nors))
            .collect::<Result<_, _>>()?;

        let octree = input
            .take::<Octree>()
            .filter(|octree| octree.is_valid(header.root_node_count, tris.len()))?;

        Ok(Kcl {
            header,
            tris,
            octree,
        })
    }
}
