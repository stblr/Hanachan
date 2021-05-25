use crate::fs::{BspWheel, Kcl};
use crate::geom::{Hitbox, Mat33, Mat34, Vec3};
use crate::player::{Bike, Collision, CommonStats, Handle, Physics};
use crate::wii::F32Ext;

#[derive(Clone, Debug)]
pub struct Wheel {
    handle: Option<Handle>,
    bsp_wheel: BspWheel,
    axis: Vec3,
    axis_s: f32,
    topmost_pos: Vec3,
    pos: Vec3,
    last_pos: Vec3,
    last_pos_rel: Vec3,
    hitbox_radius: f32,
    hitbox_pos_rel: Vec3,
    collision: Option<Collision>,
}

impl Wheel {
    pub fn new(handle: Option<Handle>, bsp_wheel: BspWheel, player_pos: Vec3) -> Wheel {
        // The initial position of bikes' front wheel is incorrect because the game doesn't take
        // the handle into account.
        let axis = Vec3::DOWN;
        let axis_s = bsp_wheel.slack_y;
        let topmost_pos = bsp_wheel.topmost_pos + player_pos;
        let pos = topmost_pos + axis_s * axis;
        let last_pos_rel = pos - topmost_pos;
        let hitbox_radius = 10.0; // Another incorrectly initialized value
        Wheel {
            handle,
            bsp_wheel,
            axis,
            axis_s,
            topmost_pos,
            pos,
            last_pos: pos,
            last_pos_rel,
            hitbox_radius,
            hitbox_pos_rel: Vec3::ZERO,
            collision: None,
        }
    }

    pub fn collision(&self) -> Option<&Collision> {
        self.collision.as_ref()
    }

    pub fn update(
        &mut self,
        stats: &CommonStats,
        bike: Option<&Bike>,
        physics: &mut Physics,
        kcl: &Kcl,
    ) -> Vec3 {
        let bsp_wheel = self.bsp_wheel;

        self.axis_s = (self.axis_s + 5.0).min(bsp_wheel.slack_y);
        self.topmost_pos = self.mat(physics) * bsp_wheel.topmost_pos;
        self.axis = Mat33::from(self.mat(physics)) * Vec3::DOWN;
        self.last_pos = self.pos;
        self.pos = self.topmost_pos + self.axis_s * self.axis;

        let radius_diff = bsp_wheel.wheel_radius - bsp_wheel.hitbox_radius;
        let mut hitbox_pos = self.pos + radius_diff * self.axis;
        if let Some(bike) = bike {
            let right = Mat33::from(physics.mat()) * Vec3::RIGHT;
            hitbox_pos += bike.lean.rot() * bsp_wheel.hitbox_radius * 0.3 * right;
        }
        let hitbox = Hitbox::new(hitbox_pos, self.hitbox_radius);
        self.hitbox_pos_rel = hitbox_pos - physics.pos;
        let collision = kcl.check_collision(hitbox);
        if let Some(collision) = collision {
            self.pos += collision.movement;
        }
        self.hitbox_radius = bsp_wheel.hitbox_radius;

        self.collision = collision.map(|collision| Collision {
            floor_nor: collision.floor_nor,
            speed_factor: stats.kcl_speed_factors[collision.closest_kind as usize],
            rot_factor: stats.kcl_rot_factors[collision.closest_kind as usize],
            has_boost_panel: collision.all_kinds & 0x40 != 0,
        });

        self.axis_s = self.axis.dot(self.pos - self.topmost_pos);
        if self.axis_s < 0.0 {
            if let Some(collision) = collision {
                let vel = 10.0 * 1.3 * Vec3::DOWN;
                let floor_nor = collision.floor_nor.normalize();
                physics.apply_rigid_body_motion(self.hitbox_pos_rel, vel, floor_nor);
            }
            self.axis_s * self.axis
        } else {
            Vec3::ZERO
        }
    }

    pub fn apply_suspension(
        &mut self,
        bike: Option<&Bike>,
        physics: &mut Physics,
        vehicle_movement: Vec3,
    ) {
        let bsp_wheel = self.bsp_wheel;

        let topmost_pos = self.topmost_pos;
        self.topmost_pos = self.topmost_pos + vehicle_movement;
        self.axis_s = self.axis.dot(self.pos - self.topmost_pos).clamp(0.0, bsp_wheel.slack_y);
        self.pos = self.topmost_pos + self.axis_s * self.axis;

        if let Some(collision) = &self.collision {
            let pos_rel = self.pos - topmost_pos;
            let dist = bsp_wheel.slack_y - self.axis.dot(pos_rel).max(0.0);
            let dist_acceleration = -bsp_wheel.dist_suspension * dist;
            let speed = self.axis.dot(self.last_pos_rel - pos_rel);
            let speed_acceleration = -bsp_wheel.speed_suspension * speed;
            let acceleration = (dist_acceleration + speed_acceleration) * self.axis;
            if physics.vel0.y <= 5.0 {
                let normal_acceleration = Vec3::new(acceleration.x, 0.0, acceleration.z);
                let normal_acceleration = normal_acceleration.proj_unit(collision.floor_nor).y;
                let normal_acceleration = acceleration.y + normal_acceleration;
                physics.normal_acceleration += normal_acceleration;
            }

            let topmost_pos_rel = physics.rot1.inv_rotate(self.topmost_pos - physics.pos);
            let acceleration = physics.rot1.inv_rotate(acceleration);
            let mut cross = topmost_pos_rel.cross(acceleration);
            if bike.map(|bike| bike.wheelie.rot() > 0.0).unwrap_or(false) {
                cross.x = 0.0;
            }
            cross.y = 0.0;
            physics.normal_rot_vec += cross;
        }
        self.last_pos_rel = self.pos - self.topmost_pos;

        if let Some(collision) = &self.collision {
            let floor_nor = collision.floor_nor;
            let vel = self.pos - self.last_pos - physics.vel1;
            let dot = (vel + 10.0 * 1.3 * Vec3::DOWN).dot(floor_nor);
            if dot < 0.0 {
                let cross = floor_nor.cross(-vel).cross(floor_nor);
                if cross.sq_norm() > f32::EPSILON {
                    let mat = Mat34::from_quat_and_pos(physics.rot0, Vec3::ZERO);
                    let mat = Mat33::from(mat * physics.inv_inertia_tensor * mat.transpose());
                    let other_cross = mat * self.hitbox_pos_rel.cross(floor_nor);
                    let other_cross = other_cross.cross(self.hitbox_pos_rel);
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
                    if bike.map(|bike| bike.wheelie.rot() <= 0.0).unwrap_or(true) {
                        let cross = physics.rot0.inv_rotate(mat * self.hitbox_pos_rel.cross(sum));
                        let cross = Vec3::new(cross.x, 0.0, cross.z);
                        physics.rot_vec0 += cross;
                    }
                }
            }
        }
    }

    fn mat(&self, physics: &Physics) -> Mat34 {
        let player_mat = Mat34::from_quat_and_pos(physics.rot1, physics.pos);
        match self.handle {
            Some(handle) => {
                let handle_mat = Mat34::from_angles_and_pos(handle.angles(), handle.pos());
                player_mat * handle_mat
            }
            None => player_mat,
        }
    }
}
