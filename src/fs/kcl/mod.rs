mod header;
mod octree;
mod tri;

use std::iter;

use crate::fs::{Error, Parse, ResultExt, SliceExt, SliceRefExt};
use crate::geom::{Hitbox, Vec3};

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
    pub fn check_collision(&self, hitbox: Hitbox) -> Option<Collision> {
        let tri_list = self.octree.find_tri_list(&self.header, hitbox.pos)?;
        let mut found = false;
        let (mut min, mut max) = (Vec3::ZERO, Vec3::ZERO);
        let mut floor_dist = 0.0;
        let mut floor_nor = Vec3::ZERO;
        let mut closest_kind = 0;

        for tri_idx in tri_list.iter() {
            let tri = &self.tris[*tri_idx as usize];
            if let Some(collision) = tri.check_collision(self.header.thickness, hitbox) {
                found = true;
                min = min.min(collision.dist * collision.nor);
                max = max.max(collision.dist * collision.nor);
                if collision.dist > floor_dist {
                    floor_dist = collision.dist;
                    floor_nor = collision.nor;
                    closest_kind = (collision.flags & 0x1f) as u8;
                }
            }
        }

        if found {
            Some(Collision {
                movement: min + max,
                floor_dist,
                floor_nor,
                closest_kind,
            })
        } else {
            None
        }
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

#[derive(Clone, Copy, Debug)]
pub struct Collision {
    pub movement: Vec3,
    pub floor_dist: f32,
    pub floor_nor: Vec3,
    pub closest_kind: u8,
}
