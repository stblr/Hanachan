use crate::fs::{BspHitbox, Kcl};
use crate::geom::{Hitbox, Mat33, Mat34, Vec3};
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

            let dot = vel.dot(floor_nor);
            if dot < 0.0 {
                let mat = Mat34::from_quat_and_pos(physics.rot0, Vec3::ZERO);
                let mat = Mat33::from(mat * physics.inv_inertia_tensor * mat.transpose());
                let cross = mat * pos_rel.cross(floor_nor);
                let cross = cross.cross(pos_rel);
                let val = -dot / (1.0 + floor_nor.dot(cross));
                let cross = floor_nor.cross(-vel);
                let cross = cross.cross(floor_nor);
                let cross = cross.normalize();
                let other_val = val * vel.dot(cross) / dot;
                let other_val = other_val.signum() * other_val.abs().min(0.01 * val);
                let sum = val * floor_nor + other_val * cross;
                physics.vel0 += sum;
                let mut cross = physics.rot0.inv_rotate(mat * pos_rel.cross(sum));
                cross.y = 0.0;
                physics.rot_vec0 += cross;
            }
        } else {
            self.floor_nor = None;
        }
    }
}
