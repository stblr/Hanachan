use crate::fs::KclCollision;
use crate::geom::Vec3;
use crate::player::CommonStats;

#[derive(Clone, Debug)]
pub struct Collision {
    count: u8,
    floor_nor: Option<Vec3>,
    speed_factor: f32,
    rot_factor: f32,
    has_trickable: bool,
}

impl Collision {
    pub fn new() -> Collision {
        Collision {
            count: 0,
            floor_nor: None,
            speed_factor: 1.0,
            rot_factor: 0.0,
            has_trickable: false,
        }
    }

    pub fn count(&self) -> u8 {
        self.count
    }

    pub fn floor_nor(&self) -> Option<Vec3> {
        self.floor_nor
    }

    pub fn speed_factor(&self) -> Option<f32> {
        self.floor_nor.map(|_| self.speed_factor)
    }

    pub fn rot_factor(&self) -> Option<f32> {
        self.floor_nor.map(|_| self.rot_factor)
    }

    pub fn has_trickable(&self) -> bool {
        self.has_trickable
    }

    pub fn add(&mut self, stats: &CommonStats, kcl_collision: &KclCollision) {
        self.count += 1;

        *self.floor_nor.get_or_insert(Vec3::ZERO) += kcl_collision.floor_nor();

        if let Some(surface) = kcl_collision.find_closest(0x20e80fff) {
            if surface & 0x2000 != 0 {
                self.has_trickable = true;
            }

            let kind = (surface & 0x1f) as usize;
            self.speed_factor = self.speed_factor.min(stats.kcl_speed_factors[kind]);
            self.rot_factor += stats.kcl_rot_factors[kind];

            if let Some(_) = kcl_collision.find_closest(0x100) {
                self.has_trickable = true;
            }
        }
    }

    pub fn finalize(&mut self) {
        if let Some(floor_nor) = &mut self.floor_nor {
            *floor_nor = floor_nor.normalize();
        }

        if self.count > 0 {
            self.rot_factor /= self.count as f32;
        }
    }

    pub fn insert_floor_nor(&mut self, floor_nor: Vec3) {
        self.floor_nor = Some(floor_nor);
        self.rot_factor = 1.0;
    }
}
