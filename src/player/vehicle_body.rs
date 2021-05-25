use crate::fs::{BspHitbox, Kcl};
use crate::geom::{Hitbox, Vec3};
use crate::player::Physics;

#[derive(Clone, Debug)]
pub struct VehicleBody {
    bsp_hitboxes: Vec<BspHitbox>,
    floor_nor: Option<Vec3>,
}

impl VehicleBody {
    pub fn new(bsp_hitboxes: Vec<BspHitbox>) -> VehicleBody {
        VehicleBody {
            bsp_hitboxes,
            floor_nor: None,
        }
    }

    pub fn floor_nor(&self) -> Option<Vec3> {
        self.floor_nor
    }

    pub fn update(&mut self, physics: &mut Physics, kcl: &Kcl) {
        let mut count = 0;
        let (mut min, mut max) = (Vec3::ZERO, Vec3::ZERO);
        let mut floor_nor = Vec3::ZERO;
        let mut pos_rel = Vec3::ZERO;
        for bsp_hitbox in &self.bsp_hitboxes {
            if !bsp_hitbox.walls_only {
                let hitbox_pos_rel = physics.rot1.rotate(bsp_hitbox.pos);
                let pos = hitbox_pos_rel + physics.pos;
                let hitbox = Hitbox::new(pos, bsp_hitbox.radius);
                if let Some(collision) = kcl.check_collision(hitbox) {
                    count += 1;
                    min = min.min(collision.movement);
                    max = max.max(collision.movement);
                    floor_nor += collision.floor_nor;
                    let nor = collision.movement.normalize();
                    pos_rel = pos_rel + hitbox_pos_rel - bsp_hitbox.radius * nor;
                }
            }
        }

        if count > 0 {
            let movement = min + max;
            physics.pos += movement;

            let floor_nor = floor_nor.normalize();
            self.floor_nor = Some(floor_nor);

            let pos_rel = (1.0 / count as f32) * pos_rel;

            let rot_vec0 = physics.rot_factor * physics.rot_vec0;
            let pos_rel_r = physics.rot0.inv_rotate(pos_rel);
            let cross = rot_vec0.cross(pos_rel_r);
            let mut vel = physics.rot0.rotate(cross) + physics.vel0;
            if physics.vel1.y > 0.0 {
                vel.y += physics.vel1.y;
            }

            physics.apply_rigid_body_motion(pos_rel, vel, floor_nor);
        } else {
            self.floor_nor = None;
        }
    }
}
