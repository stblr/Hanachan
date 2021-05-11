use crate::fs::BspHitbox;
use crate::geom::{Hitbox, Mat33, Mat34, Vec3};
use crate::player::Physics;

#[derive(Clone, Debug)]
pub struct VehicleBody {
    bsp_hitboxes: Vec<BspHitbox>,
}

impl VehicleBody {
    pub fn new(bsp_hitboxes: Vec<BspHitbox>) -> VehicleBody {
        VehicleBody { bsp_hitboxes }
    }

    pub fn update(&mut self, physics: &mut Physics) {
        let mut count = 0;
        let mut floor_movement = Vec3::ZERO;
        let mut pos_rel = Vec3::ZERO;
        for bsp_hitbox in &self.bsp_hitboxes {
            if !bsp_hitbox.walls_only {
                let hitbox_pos_rel = physics.rot1.rotate(bsp_hitbox.pos);
                let pos = hitbox_pos_rel + physics.pos;
                let hitbox = Hitbox::new(pos, bsp_hitbox.radius);
                if let Some(hitbox_floor_movement) = hitbox.check_collision() {
                    count += 1;
                    floor_movement += hitbox_floor_movement;
                    let hitbox_floor_nor = Vec3::UP.normalize();
                    pos_rel = pos_rel + hitbox_pos_rel - bsp_hitbox.radius * hitbox_floor_nor;
                }
            }
        }

        if count > 0 {
            physics.pos += floor_movement;

            let pos_rel = (1.0 / count as f32) * pos_rel;

            let rot_vec0 = physics.rot_factor * physics.rot_vec0;
            let pos_rel_r = physics.rot0.inv_rotate(pos_rel);
            let cross = rot_vec0.cross(pos_rel_r);
            let vel = physics.rot0.rotate(cross) + physics.vel0;

            let floor_nor = Vec3::UP;
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
        }
    }
}
