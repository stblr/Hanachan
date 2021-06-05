use crate::fs::{Error, Parse, SliceRefExt};
use crate::geom::{Hitbox, Vec3};
use crate::wii::F32Ext;

#[derive(Clone, Debug)]
pub struct RawTri {
    altitude: f32,
    pos_idx: u16,
    plane_nor_idx: u16,
    ca_nor_idx: u16,
    ab_nor_idx: u16,
    bc_nor_idx: u16,
    flags: u16,
}

impl Parse for RawTri {
    fn parse(input: &mut &[u8]) -> Result<RawTri, Error> {
        Ok(RawTri {
            altitude: input.take()?,
            pos_idx: input.take()?,
            plane_nor_idx: input.take()?,
            ca_nor_idx: input.take()?,
            ab_nor_idx: input.take()?,
            bc_nor_idx: input.take()?,
            flags: input.take()?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Tri {
    altitude: f32,
    pos: Vec3,
    plane_nor: Vec3,
    ca_nor: Vec3,
    ab_nor: Vec3,
    bc_nor: Vec3,
    flags: u16,
}

impl Tri {
    pub fn try_from_raw(raw: RawTri, poss: &Vec<Vec3>, nors: &Vec<Vec3>) -> Result<Tri, Error> {
        Ok(Tri {
            altitude: raw.altitude,
            pos: *poss.get(raw.pos_idx as usize).ok_or(Error {})?,
            plane_nor: *nors.get(raw.plane_nor_idx as usize).ok_or(Error {})?,
            ca_nor: *nors.get(raw.ca_nor_idx as usize).ok_or(Error {})?,
            ab_nor: *nors.get(raw.ab_nor_idx as usize).ok_or(Error {})?,
            bc_nor: *nors.get(raw.bc_nor_idx as usize).ok_or(Error {})?,
            flags: raw.flags,
        })
    }

    pub fn check_collision(&self, thickness: f32, hitbox: Hitbox) -> Option<Collision> {
        fn ps_dot(v0: Vec3, v1: Vec3) -> f32 {
            let y = v0.y * v1.y;
            let xy = ((v0.x as f64 * v1.x as f64) + y as f64) as f32;
            xy + v0.z * v1.z
        }

        if 1 << (self.flags & 0x1f) & 0x20e80fff == 0 {
            return None;
        }

        let pos = hitbox.pos - self.pos;
        let radius = hitbox.radius;

        let ca_dist = ps_dot(pos, self.ca_nor);
        if ca_dist >= radius {
            return None;
        }

        let ab_dist = ps_dot(pos, self.ab_nor);
        if ab_dist >= radius {
            return None;
        }

        let bc_dist = ps_dot(pos, self.bc_nor) - self.altitude;
        if bc_dist >= radius {
            return None;
        }

        let plane_dist = ps_dot(pos, self.plane_nor);
        let dist_in_plane = radius - plane_dist;
        if dist_in_plane <= 0.0 || dist_in_plane >= thickness {
            return None;
        }

        if ca_dist <= 0.0 && ab_dist <= 0.0 && bc_dist <= 0.0 {
            return Some(Collision {
                dist: dist_in_plane,
                nor: self.plane_nor,
                flags: self.flags,
            });
        }

        let (edge_nor, edge_dist);
        let (other_edge_nor, other_edge_dist);
        if ab_dist >= ca_dist && ab_dist > bc_dist {
            edge_nor = self.ab_nor;
            edge_dist = ab_dist;
            if ca_dist >= bc_dist {
                other_edge_nor = self.ca_nor;
                other_edge_dist = ca_dist;
            } else {
                other_edge_nor = self.bc_nor;
                other_edge_dist = bc_dist;
            }
        } else if bc_dist >= ca_dist {
            edge_nor = self.bc_nor;
            edge_dist = bc_dist;
            if ab_dist >= ca_dist {
                other_edge_nor = self.ab_nor;
                other_edge_dist = ab_dist;
            } else {
                other_edge_nor = self.ca_nor;
                other_edge_dist = ca_dist;
            }
        } else {
            edge_nor = self.ca_nor;
            edge_dist = ca_dist;
            if bc_dist >= ab_dist {
                other_edge_nor = self.bc_nor;
                other_edge_dist = bc_dist;
            } else {
                other_edge_nor = self.ab_nor;
                other_edge_dist = ab_dist;
            }
        }

        let cos = ps_dot(edge_nor, other_edge_nor);
        let sq_dist = if cos * edge_dist > other_edge_dist {
            radius * radius - edge_dist * edge_dist
        } else {
            let t = (cos * edge_dist - other_edge_dist) / (cos * cos - 1.0);
            let s = edge_dist - t * cos;
            let corner_pos = s * edge_nor + t * other_edge_nor;
            radius * radius - corner_pos.sq_norm()
        };

        if sq_dist < plane_dist * plane_dist || sq_dist < 0.0 {
            return None;
        }

        let dist = sq_dist.wii_sqrt() - plane_dist;
        if dist <= 0.0 {
            return None;
        }

        Some(Collision {
            dist,
            nor: self.plane_nor,
            flags: self.flags,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Collision {
    pub dist: f32,
    pub nor: Vec3,
    pub flags: u16,
}
