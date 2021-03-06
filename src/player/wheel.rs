use crate::fs::{BspWheel, Kcl};
use crate::geom::{Hitbox, Mat33, Mat34, Vec3};
use crate::player::{Bike, Collision, CommonStats, Handle, Physics, SurfaceProps};
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
    hitbox: Hitbox,
    hitbox_pos_rel: Vec3,
    collision: Collision,
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

        let hitbox_pos = pos;
        let hitbox_last_pos = Some(player_pos);
        let hitbox_radius = 10.0; // Another incorrectly initialized value
        let hitbox = Hitbox {
            pos: hitbox_pos,
            last_pos: hitbox_last_pos,
            radius: hitbox_radius,
            flags: 0x20e80fff,
        };

        Wheel {
            handle,
            bsp_wheel,
            axis,
            axis_s,
            topmost_pos,
            pos,
            last_pos: pos,
            last_pos_rel,
            hitbox,
            hitbox_pos_rel: Vec3::ZERO,
            collision: Collision::new(),
        }
    }

    pub fn hitbox_pos_rel(&self) -> Vec3 {
        self.hitbox_pos_rel
    }

    pub fn collision(&self) -> &Collision {
        &self.collision
    }

    pub fn update(
        &mut self,
        stats: &CommonStats,
        bike: Option<&Bike>,
        physics: &mut Physics,
        surface_props: &mut SurfaceProps,
        kcl: &Kcl,
    ) -> Option<Vec3> {
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
        self.hitbox.update_pos(hitbox_pos);
        self.hitbox_pos_rel = hitbox_pos - physics.pos;
        let kcl_collision = kcl.check_collision(self.hitbox);
        self.pos += kcl_collision.movement();
        self.hitbox.radius = bsp_wheel.hitbox_radius;

        self.collision = Collision::new();
        if kcl_collision.surface_kinds() & 0x20e80fff != 0 {
            self.collision.add(stats, &kcl_collision);
            surface_props.add(&kcl_collision, true);
        }

        self.axis_s = self.axis.dot(self.pos - self.topmost_pos);
        if self.axis_s < 0.0 {
            Some(self.axis_s * self.axis)
        } else {
            None
        }
    }

    pub fn apply_suspension(
        &mut self,
        max_normal_acceleration: f32,
        jump_pad_applied_dir: bool,
        bike: Option<&Bike>,
        physics: &mut Physics,
        vehicle_movement: Vec3,
    ) {
        let bsp_wheel = self.bsp_wheel;

        let topmost_pos = self.topmost_pos;
        self.topmost_pos = self.topmost_pos + vehicle_movement;
        self.axis_s = self.axis.dot(self.pos - self.topmost_pos).clamp(0.0, bsp_wheel.slack_y);
        self.pos = self.topmost_pos + self.axis_s * self.axis;

        if let Some(floor_nor) = self.collision.floor_nor() {
            let pos_rel = self.pos - topmost_pos;
            let dist = bsp_wheel.slack_y - self.axis.dot(pos_rel).max(0.0);
            let dist_acceleration = -bsp_wheel.dist_suspension * dist;
            let speed = self.axis.dot(self.last_pos_rel - pos_rel);
            let speed_acceleration = -bsp_wheel.speed_suspension * speed;
            let acceleration = (dist_acceleration + speed_acceleration) * self.axis;

            if !jump_pad_applied_dir && physics.vel0.y <= 5.0 {
                let normal_acceleration = Vec3::new(acceleration.x, 0.0, acceleration.z);
                let normal_acceleration = normal_acceleration.proj_unit(floor_nor).y;
                let normal_acceleration = acceleration.y + normal_acceleration;
                let normal_acceleration = normal_acceleration.min(max_normal_acceleration);
                physics.normal_acceleration += normal_acceleration;
            }

            let topmost_pos_rel = physics.rot1.inv_rotate(topmost_pos - physics.pos);
            let acceleration = physics.rot1.inv_rotate(acceleration);
            let mut cross = topmost_pos_rel.cross(acceleration);
            if bike.map(|bike| bike.wheelie.rot() > 0.0).unwrap_or(false) {
                cross.x = 0.0;
            }
            cross.y = 0.0;
            physics.normal_rot_vec += cross;
        }
        self.last_pos_rel = self.pos - self.topmost_pos;

        if let Some(floor_nor) = self.collision.floor_nor() {
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
