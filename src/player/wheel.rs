use crate::fs::BspWheel;
use crate::geom::{Hitbox, Mat33, Mat34, Vec3};
use crate::player::{Handle, Physics};
use crate::wii::F32Ext;

#[derive(Clone, Copy, Debug)]
pub struct Wheel {
    handle: Option<Handle>,
    bsp_wheel: BspWheel,
    axis_s: f32,
    pos: Vec3,
    last_pos_rel: Vec3,
    hitbox_radius: f32,
    pub floor_nor: Option<Vec3>,
}

impl Wheel {
    pub fn new(handle: Option<Handle>, bsp_wheel: BspWheel, player_pos: Vec3) -> Wheel {
        // The initial position of bikes' front wheel is incorrect because the game doesn't take
        // the handle into account.
        let axis_s = bsp_wheel.slack_y;
        let topmost_pos = bsp_wheel.topmost_pos + player_pos;
        let axis = Vec3::DOWN;
        let pos = topmost_pos + axis_s * axis;
        let last_pos_rel = pos - topmost_pos;
        let hitbox_radius = 10.0; // Another incorrectly initialized value
        Wheel {
            handle,
            bsp_wheel,
            axis_s,
            pos,
            last_pos_rel,
            hitbox_radius,
            floor_nor: None,
        }
    }

    pub fn update(&mut self, physics: &mut Physics) {
        let bsp_wheel = self.bsp_wheel;

        self.axis_s = (self.axis_s + 5.0).min(bsp_wheel.slack_y);
        let topmost_pos = self.mat(physics) * bsp_wheel.topmost_pos;
        let axis = Mat33::from(self.mat(physics)) * Vec3::DOWN;
        let last_pos = self.pos;
        self.pos = topmost_pos + self.axis_s * axis;

        let radius_diff = bsp_wheel.wheel_radius - bsp_wheel.hitbox_radius;
        let hitbox_pos = self.pos + radius_diff * axis;
        // TODO add turn_rot_z thing for bikes
        let hitbox = Hitbox::new(hitbox_pos, self.hitbox_radius);
        let floor_movement = hitbox.check_collision();
        if let Some(floor_movement) = floor_movement {
            self.pos += floor_movement;
        }
        self.hitbox_radius = bsp_wheel.hitbox_radius;

        self.axis_s = axis.dot(self.pos - topmost_pos).max(0.0);
        self.pos = topmost_pos + self.axis_s * axis;

        let pos_rel = self.pos - topmost_pos;
        if floor_movement.is_some() {
            let dist = bsp_wheel.slack_y - axis.dot(self.pos - topmost_pos);
            let dist_acceleration = -bsp_wheel.dist_suspension * dist;
            let speed = axis.dot(self.last_pos_rel - pos_rel);
            let speed_acceleration = -bsp_wheel.speed_suspension * speed;
            let acceleration = (dist_acceleration + speed_acceleration) * axis;
            if physics.vel0.y <= 5.0 {
                physics.normal_acceleration += acceleration.y;
            }

            let topmost_pos_rel = physics.rot1.inv_rotate(topmost_pos - physics.pos);
            let acceleration = physics.rot1.inv_rotate(acceleration);
            let mut cross = topmost_pos_rel.cross(acceleration);
            cross.y = 0.0;
            // TODO add wheelie checks
            physics.normal_rot_vec += cross;
        }
        self.last_pos_rel = pos_rel;

        self.floor_nor = floor_movement.map(|_| Vec3::UP); // TODO compute from Kcl
        if let Some(floor_nor) = self.floor_nor {
            let vel = self.pos - last_pos - physics.vel1;
            let dot = (vel + 10.0 * 1.3 * Vec3::DOWN).dot(floor_nor);
            if dot < 0.0 {
                let cross = floor_nor.cross(-vel).cross(floor_nor);
                if cross.sq_norm() > f32::EPSILON {
                    let mat = Mat34::from_quat_and_pos(physics.rot0, Vec3::ZERO);
                    let mat = Mat33::from(mat * physics.inv_inertia_tensor * mat.transpose());
                    let hitbox_pos_rel = hitbox_pos - physics.pos;
                    let other_cross = mat * hitbox_pos_rel.cross(floor_nor);
                    let other_cross = other_cross.cross(hitbox_pos_rel);
                    let val = -dot / (1.0 + floor_nor.dot(other_cross));
                    let cross = cross.normalize();
                    let cross = val * vel.dot(cross).min(0.0) / dot * cross;
                    let front = physics.rot1.rotate(Vec3::FRONT);
                    let proj = cross.proj_unit(front);
                    let proj_norm = proj.sq_norm().wii_sqrt();
                    let proj_norm = proj_norm.signum() * proj_norm.abs().min(0.1 * val.abs());
                    let proj = proj_norm * proj.normalize();
                    let rej = cross.rej_unit(front);
                    let rej_norm = rej.sq_norm().wii_sqrt();
                    // TODO 0.8 isn't a constant
                    let rej_norm = rej_norm.signum() * rej_norm.abs().min(0.8 * val.abs());
                    let rej = rej_norm * rej.normalize();
                    let sum = proj + rej;
                    let rej = sum.rej_unit(physics.dir);
                    physics.vel0 += rej;
                    // TODO add wheelie check
                    let mut cross = physics.rot0.inv_rotate(mat * hitbox_pos_rel.cross(sum));
                    cross.y = 0.0;
                    physics.rot_vec0 += cross;
                }
            }
        }
    }

    fn mat(&self, physics: &Physics) -> Mat34 {
        match self.handle {
            Some(handle) => {
                let handle_mat = Mat34::from_angles_and_pos(handle.angles(), handle.pos());
                physics.mat() * handle_mat
            }
            None => physics.mat(),
        }
    }
}
