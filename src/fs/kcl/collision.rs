use crate::fs::kcl::tri::Collision as TriCollision;
use crate::geom::Vec3;

#[derive(Clone, Debug)]
pub struct Collision {
    min: Vec3,
    max: Vec3,
    floor_dist: f32,
    floor_nor: Vec3,
    hits: Vec<Hit>,
    surface_kinds: u32,
}

impl Collision {
    pub fn new() -> Collision {
        Collision {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
            floor_dist: 0.0,
            floor_nor: Vec3::ZERO,
            surface_kinds: 0,
            hits: Vec::new(),
        }
    }

    pub fn add(&mut self, tri_collision: TriCollision) {
        self.min = self.min.min(tri_collision.dist * tri_collision.nor);
        self.max = self.max.max(tri_collision.dist * tri_collision.nor);

        if tri_collision.dist > self.floor_dist {
            self.floor_dist = tri_collision.dist;
            self.floor_nor = tri_collision.nor;
        }

        self.surface_kinds |= 1 << (tri_collision.flags & 0x1f);

        if self.hits.len() < 64 {
            if self.hits.is_empty() {
                self.hits.reserve_exact(64);
            }

            let hit = Hit {
                surface: tri_collision.flags,
                dist: tri_collision.dist,
            };
            self.hits.push(hit);
        }
    }

    pub fn movement(&self) -> Vec3 {
        self.min + self.max
    }

    pub fn floor_nor(&self) -> Vec3 {
        self.floor_nor
    }

    pub fn surface_kinds(&self) -> u32 {
        self.surface_kinds
    }

    pub fn find_closest(&self, surface_kinds: u32) -> Option<u16> {
        self.hits
            .iter()
            .filter(|hit| (1 << (hit.surface & 0x1f)) & surface_kinds != 0)
            .max_by(|h0, h1| h0.dist.partial_cmp(&h1.dist).unwrap())
            .map(|hit| hit.surface)
    }
}

#[derive(Clone, Debug)]
struct Hit {
    surface: u16,
    dist: f32,
}
