use crate::fs::{BspHitbox, Kcl};
use crate::geom::{Hitbox, Vec3};
use crate::player::{Collision, CommonStats, Physics};

#[derive(Clone, Debug)]
pub struct VehicleBody {
    bsp_hitboxes: Vec<BspHitbox>,
    collision: Option<Collision>,
    has_floor_collision: bool,
}

impl VehicleBody {
    pub fn new(bsp_hitboxes: Vec<BspHitbox>) -> VehicleBody {
        VehicleBody {
            bsp_hitboxes,
            collision: None,
            has_floor_collision: false,
        }
    }

    pub fn collision(&self) -> Option<&Collision> {
        self.collision.as_ref()
    }

    pub fn override_collision(&mut self, collision: Collision) {
        self.collision = Some(collision);
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
        let mut count = 0;
        let (mut min, mut max) = (Vec3::ZERO, Vec3::ZERO);
        let mut floor_nor = Vec3::ZERO;
        let mut speed_factor: f32 = 1.0;
        let mut rot_factor = 0.0;
        let mut has_boost_panel = false;
        let mut pos_rel = Vec3::ZERO;
        for bsp_hitbox in &self.bsp_hitboxes {
            if !bsp_hitbox.walls_only {
                let hitbox_pos_rel = physics.rot1.rotate(bsp_hitbox.pos);
                let pos = hitbox_pos_rel + physics.pos;
                let hitbox = Hitbox::new(pos, bsp_hitbox.radius);
                if let Some(collision) = kcl.check_collision(hitbox) {
                    count += 1;

                    min = min.min(collision.min);
                    max = max.max(collision.max);

                    floor_nor += collision.floor_nor;
                    let closest_kind = collision.closest_kind as usize;
                    let hitbox_speed_factor = stats.kcl_speed_factors[closest_kind];
                    speed_factor = speed_factor.min(hitbox_speed_factor);
                    let hitbox_rot_factor = stats.kcl_rot_factors[closest_kind];
                    rot_factor += hitbox_rot_factor;
                    has_boost_panel = has_boost_panel || collision.all_kinds & 0x40 != 0;

                    let nor = (collision.min + collision.max).normalize();
                    pos_rel = pos_rel + hitbox_pos_rel - bsp_hitbox.radius * nor;
                }
            }
        }

        self.has_floor_collision = count > 0;

        if count > 0 {
            let movement = min + max;
            physics.pos += movement;

            let floor_nor = floor_nor.normalize();
            self.collision = Some(Collision {
                floor_nor,
                speed_factor,
                rot_factor: rot_factor / count as f32,
                has_boost_panel: false,
            });

            let pos_rel = (1.0 / count as f32) * pos_rel;

            let rot_vec0 = physics.rot_factor * physics.rot_vec0;
            let pos_rel_r = physics.rot0.inv_rotate(pos_rel);
            let cross = rot_vec0.cross(pos_rel_r);
            let mut vel = physics.rot0.rotate(cross) + physics.vel0;
            if physics.vel1.y > 0.0 {
                vel.y += physics.vel1.y;
            }

            physics.apply_rigid_body_motion(is_boosting, pos_rel, vel, floor_nor);
        } else {
            self.collision = None;
        }
    }
}
