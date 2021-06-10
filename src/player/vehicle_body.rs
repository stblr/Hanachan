use crate::fs::{BspHitbox, Kcl};
use crate::geom::{Hitbox, Vec3};
use crate::player::{Collision, CommonStats, Physics};

#[derive(Clone, Debug)]
pub struct VehicleBody {
    bsp_hitboxes: Vec<BspHitbox>,
    hitboxes: Vec<Hitbox>,
    collision: Collision,
    has_floor_collision: bool,
}

impl VehicleBody {
    pub fn new(bsp_hitboxes: Vec<BspHitbox>, physics: &Physics) -> VehicleBody {
        let hitboxes = bsp_hitboxes
            .iter()
            .map(|bsp_hitbox| Hitbox {
                pos: Vec3::ZERO,
                last_pos: Some(physics.mat * bsp_hitbox.pos),
                radius: bsp_hitbox.radius,
                flags: 0x20e80fff,
            })
            .collect();

        VehicleBody {
            bsp_hitboxes,
            hitboxes,
            collision: Collision::new(),
            has_floor_collision: false,
        }
    }

    pub fn collision(&self) -> &Collision {
        &self.collision
    }

    pub fn has_floor_collision(&self) -> bool {
        self.has_floor_collision
    }

    pub fn update(
        &mut self,
        stats: &CommonStats,
        is_boosting: bool,
        physics: &mut Physics,
        kcl: &Kcl,
    ) {
        let (mut min, mut max) = (Vec3::ZERO, Vec3::ZERO);
        let mut pos_rel = Vec3::ZERO;
        self.collision = Collision::new();
        for (bsp_hitbox, hitbox) in self.bsp_hitboxes.iter().zip(self.hitboxes.iter_mut()) {
            if !bsp_hitbox.walls_only {
                let hitbox_pos_rel = physics.rot1.rotate(bsp_hitbox.pos);
                let pos = hitbox_pos_rel + physics.pos;
                hitbox.update_pos(pos);

                let kcl_collision = kcl.check_collision(*hitbox);

                if kcl_collision.surface_kinds() & 0x20e80fff != 0 {
                    min = min.min(kcl_collision.movement());
                    max = max.max(kcl_collision.movement());

                    let nor = kcl_collision.movement().normalize();
                    pos_rel = pos_rel + hitbox_pos_rel - bsp_hitbox.radius * nor;

                    self.collision.add(stats, kcl_collision);
                }
            }
        }

        let count = self.collision.count();
        self.has_floor_collision = count > 0;

        if count > 0 {
            let movement = min + max;
            physics.pos += movement;

            self.collision.finalize();
            self.collision.disable_boost_panels();

            let pos_rel = (1.0 / count as f32) * pos_rel;

            let rot_vec0 = physics.rot_factor * physics.rot_vec0;
            let pos_rel_r = physics.rot0.inv_rotate(pos_rel);
            let cross = rot_vec0.cross(pos_rel_r);
            let mut vel = physics.rot0.rotate(cross) + physics.vel0;
            if physics.vel1.y > 0.0 {
                vel.y += physics.vel1.y;
            }

            if let Some(floor_nor) = self.collision.floor_nor() {
                physics.apply_rigid_body_motion(is_boosting, pos_rel, vel, floor_nor);
            }
        }
    }

    pub fn insert_floor_nor(&mut self, floor_nor: Vec3) {
        self.collision.insert_floor_nor(floor_nor);
    }
}
