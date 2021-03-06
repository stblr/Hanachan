use std::ops::Add;

use crate::fs::Bsp;
use crate::geom::{Hitbox, Mat33, Mat34, Quat, Vec3};
use crate::player::{Boost, Drift, Floor, Stats, SurfaceProps};
use crate::race::{Stage, Timer};
use crate::track::Track;
use crate::wii::F32Ext;

#[derive(Clone, Debug)]
pub struct Physics {
    pub inv_inertia_tensor: Mat34,
    pub rot_factor: f32,
    pub mat: Mat34,
    pub up: Vec3,
    pub smoothed_up: Vec3,
    pub dir: Vec3,
    pub dir_diff: Vec3,
    pub vel1_dir: Vec3,
    pub landing_dir: Option<Vec3>,
    pub landing_angle: f32,
    pub pos: Vec3,
    pub gravity: f32,
    pub normal_acceleration: f32,
    pub vel0: Vec3,
    pub vel1: Vec3,
    pub last_speed1: f32,
    pub speed1: f32,
    pub speed1_adj: f32,
    pub speed1_soft_limit: f32,
    pub vel: Vec3,
    pub normal_rot_vec: Vec3,
    pub rot_vec0: Vec3,
    pub rot_vec2: Vec3,
    pub rot0: Quat, // TODO rename to main_rot
    pub non_conserved_special_rot: Quat,
    pub conserved_special_rot: Quat,
    pub rot1: Quat, // TODO rename to full_rot
    pub stabilization_factor: f32,
}

impl Physics {
    pub fn new(bsp: &Bsp, track: &Track) -> Physics {
        let masses = [1.0 / 12.0, 1.0];
        let inertia_tensor = masses
            .iter()
            .zip(bsp.cuboids.iter())
            .map(|(mass, cuboid)| {
                Vec3::new(
                    mass * (cuboid.y * cuboid.y + cuboid.z * cuboid.z),
                    mass * (cuboid.z * cuboid.z + cuboid.x * cuboid.x),
                    mass * (cuboid.x * cuboid.x + cuboid.y * cuboid.y),
                )
            })
            .reduce(Add::add)
            .unwrap();
        let det = inertia_tensor.x * inertia_tensor.y * inertia_tensor.z;
        let inv_inertia_tensor = Vec3::new(
            det.recip() * (inertia_tensor.y * inertia_tensor.z),
            det.recip() * (inertia_tensor.z * inertia_tensor.x),
            det.recip() * (inertia_tensor.x * inertia_tensor.y),
        );
        let inv_inertia_tensor = Mat34::from_diag(inv_inertia_tensor);

        let ktpt_pos = track.kmp().ktpt.entries[0].pos; // TODO bounds check
        let diff0 = Vec3::new(-800.0, 0.0, 461.87988);
        let diff1 = Vec3::new(800.0, 0.0, -461.87991);
        let mut pos = ktpt_pos + diff0 + diff1;

        let hitbox = Hitbox::new(pos, None, 100.0, 0x20e80fff);
        let collision = track.kcl().check_collision(hitbox);
        pos = pos + collision.movement() - 100.0 * collision.floor_nor();
        pos += bsp.initial_pos_y * collision.floor_nor();

        Physics {
            inv_inertia_tensor,
            rot_factor: bsp.rot_factor,
            mat: Mat34::from_quat_and_pos(Quat::BACK, pos),
            up: Vec3::UP,
            smoothed_up: Vec3::UP,
            dir: Vec3::BACK,
            dir_diff: Vec3::ZERO,
            vel1_dir: Vec3::BACK,
            landing_dir: None,
            landing_angle: 0.0,
            pos,
            gravity: -1.3,
            normal_acceleration: 0.0,
            vel0: Vec3::ZERO,
            vel1: Vec3::ZERO,
            last_speed1: 0.0,
            speed1: 0.0,
            speed1_adj: 0.0,
            speed1_soft_limit: 0.0,
            vel: Vec3::ZERO,
            normal_rot_vec: Vec3::ZERO,
            rot_vec0: Vec3::ZERO,
            rot_vec2: Vec3::ZERO,
            rot0: Quat::BACK,
            non_conserved_special_rot: Quat::IDENTITY,
            conserved_special_rot: Quat::IDENTITY,
            rot1: Quat::BACK,
            stabilization_factor: 0.0,
        }
    }

    pub fn mat(&self) -> Mat34 {
        self.mat
    }

    pub fn update_ups<'a>(
        &mut self,
        is_inside_drift: bool,
        floor: &Floor,
        has_hop_height: bool,
        is_boosting: bool,
        is_wheelieing: bool,
        has_boost_ramp: bool,
    ) {
        self.landing_dir = None;
        self.stabilization_factor = 0.1;
        if floor.is_landing() && floor.last_airtime() >= 3 {
            self.up = floor.nor().unwrap();
            self.smoothed_up = self.up;
            let landing_dir = self.dir.perp_in_plane(self.smoothed_up, true);
            self.dir_diff = landing_dir.proj_unit(landing_dir);
            self.landing_dir = Some(landing_dir);
        } else if has_hop_height {
            self.stabilization_factor = if is_inside_drift { 0.22 } else { 0.5 };
        } else if floor.airtime() > 20 {
            if self.up.y > 0.99 {
                self.up = Vec3::UP;
            } else {
                self.up += 0.03 * (Vec3::UP - self.up);
                self.up = self.up.normalize();
            }

            if self.smoothed_up.y > 0.99 {
                self.smoothed_up = Vec3::UP;
            } else {
                self.smoothed_up += 0.03 * (Vec3::UP - self.smoothed_up);
                self.smoothed_up = self.smoothed_up.normalize();
            }
        } else if floor.airtime() == 0 {
            self.up = floor.nor().unwrap();

            let smoothing_factor = if is_boosting || is_wheelieing {
                0.8
            } else {
                let front = Mat33::from(self.mat) * Vec3::FRONT;
                (0.8 - 6.0 * self.up.dot(front).abs()).clamp(0.3, 0.8)
            };
            self.smoothed_up += smoothing_factor * (self.up - self.smoothed_up);
            self.smoothed_up = self.smoothed_up.normalize();

            let front = Mat33::from(self.mat) * Vec3::FRONT;
            let dot = front.dot(self.smoothed_up);
            if dot < -0.1 {
                self.stabilization_factor += (0.5 * dot.abs()).min(0.2);
            }

            if has_boost_ramp {
                self.stabilization_factor = 0.4;
            }
        }
    }

    pub fn update_dirs(
        &mut self,
        floor: &Floor,
        floor_rot_factor: f32,
        drift: &Drift,
        boost_ramp_enabled: bool,
        jump_pad_enabled: bool,
        is_tricking: bool,
    ) {
        self.vel1_dir = self.dir;

        if floor.airtime() > 0 && boost_ramp_enabled {
            return;
        }

        if floor.airtime() > 5 || jump_pad_enabled {
            return;
        }

        if is_tricking {
            return;
        }

        let next_dir = drift.hop_dir().unwrap_or_else(|| {
            let right = self.rot0.rotate(Vec3::RIGHT);
            right.cross(self.smoothed_up).normalize()
        });
        let angle = self.landing_angle + drift.outside_drift_angle();
        let angle = angle.to_radians();
        let mat = Mat33::from(Mat34::from_axis_angle(self.smoothed_up, angle));
        let next_dir = mat * next_dir;
        let next_dir = next_dir.perp_in_plane(self.smoothed_up, true);
        let next_dir_diff = next_dir - self.dir;
        if next_dir_diff.sq_norm() <= f32::EPSILON {
            self.dir = next_dir;
            self.dir_diff = Vec3::ZERO;
        } else {
            let axis = self.dir.cross(next_dir);
            let next_dir_diff = self.dir_diff + floor_rot_factor * next_dir_diff;
            self.dir = (self.dir + next_dir_diff).normalize();
            self.dir_diff = 0.1 * next_dir_diff;
            let next_axis = self.dir.cross(next_dir);
            if axis.dot(next_axis) < 0.0 {
                self.dir = next_dir;
                self.dir_diff = Vec3::ZERO;
            }
        }
        self.vel1_dir = self.dir.perp_in_plane(self.smoothed_up, true);
    }

    pub fn update_landing_angle(&mut self) {
        if let Some(landing_dir) = self.landing_dir {
            let cross = self.dir.cross(landing_dir);
            let norm = cross.sq_norm().wii_sqrt();
            let dot = self.dir.dot(landing_dir);
            let angle = norm.wii_atan2(dot).abs().to_degrees();
            self.landing_angle += angle * cross.dot(self.smoothed_up).signum();
        }

        if self.landing_angle < 0.0 {
            self.landing_angle = (self.landing_angle + 2.0).min(0.0);
        } else {
            self.landing_angle = (self.landing_angle - 2.0).max(0.0);
        }
    }

    pub fn update_vel1<'a>(
        &mut self,
        stats: &Stats,
        accelerate: bool,
        brake: bool,
        last_accelerate: bool,
        last_brake: bool,
        airtime: u32,
        floor_speed_factor: f32,
        is_drifting: bool,
        boost: &Boost,
        raw_turn: f32,
        boost_ramp_enabled: bool,
        jump_pad_speed: Option<f32>,
        is_wheelieing: bool,
        surface_props: &SurfaceProps,
        timer: &Timer,
    ) {
        let last_speed_ratio = (self.speed1 / stats.common.base_speed).min(1.0);

        if !is_drifting && timer.stage() == Stage::Race {
            self.speed1 += self.speed1_adj;
        }

        let ground = airtime == 0;

        let mut acceleration = 0.0;
        if !ground {
            if boost_ramp_enabled && airtime < 4 {
                acceleration = 7.0;
            } else if airtime > 5 {
                self.speed1 *= 0.999;
            }
        } else if let Some(boost_acceleration) = boost.acceleration() {
            acceleration = boost_acceleration;
        } else if boost_ramp_enabled || jump_pad_speed.is_some() {
            acceleration = 7.0;
        } else {
            if timer.stage() == Stage::Race && accelerate {
                let (ys, xs): (&[f32], &[f32]) = if is_drifting {
                    (&stats.common.drift_acceleration_ys, &stats.common.drift_acceleration_xs)
                } else {
                    (&stats.common.acceleration_ys, &stats.common.acceleration_xs)
                };
                acceleration = self.compute_acceleration(ys, xs);
            } else if timer.stage() == Stage::Race && brake {
                if !last_accelerate && last_brake {
                    acceleration = -1.5;
                }
            } else {
                self.speed1 *= 0.98;
            }

            if !is_drifting {
                let t = stats.common.handling_speed_multiplier;
                self.speed1 *= t + (1.0 - t) * (1.0 - raw_turn.abs() * last_speed_ratio);
            }
        }

        self.last_speed1 = self.speed1;

        self.speed1 += acceleration;

        let base_speed = jump_pad_speed.unwrap_or(stats.common.base_speed);
        let boost_factor = boost.factor();
        let wheelie_bonus = if is_wheelieing { 0.15 } else { 0.0 };
        let mut next_soft_limit = (boost_factor + wheelie_bonus) * floor_speed_factor * base_speed;
        if let Some(boost_limit) = boost.limit() {
            if jump_pad_speed.is_none() {
                let boost_limit = boost_limit * floor_speed_factor;
                next_soft_limit = next_soft_limit.max(boost_limit);
            }
        }
        if boost_ramp_enabled {
            next_soft_limit = next_soft_limit.max(100.0);
        }
        self.speed1_soft_limit = (self.speed1_soft_limit - 3.0).max(next_soft_limit);
        self.speed1_soft_limit = (self.speed1_soft_limit).min(120.0);
        self.speed1 = self.speed1.min(self.speed1_soft_limit);

        if let Some(jump_pad_speed) = jump_pad_speed {
            self.speed1 = self.speed1.max(jump_pad_speed);
        }

        let right = self.smoothed_up.cross(self.dir);
        let angle: f32 = if surface_props.has_boost_ramp() {
            4.0
        } else if ground {
            0.5
        } else {
            0.2
        };
        let angle = angle.to_radians();
        self.vel1_dir = Mat33::from(Mat34::from_axis_angle(right, angle)) * self.vel1_dir;
        self.vel1 = self.speed1 * self.vel1_dir;
    }

    fn compute_acceleration(&self, ys: &[f32], xs: &[f32]) -> f32 {
        let t = self.speed1 / self.speed1_soft_limit;
        if t < 0.0 {
            1.0
        } else {
            match xs.iter().skip(1).position(|x| t < *x) {
                Some(i) => ys[i] + (ys[i + 1] - ys[i]) / (xs[i + 1] - xs[i]) * (t - xs[i]),
                None => ys[ys.len() - 1],
            }
        }
    }

    pub fn update(&mut self, stats: &Stats, timer: &Timer) {
        if timer.stage() != Stage::Race {
            if stats.vehicle.drift_kind.is_bike() {
                self.vel0 = self.vel0.rej_unit(self.smoothed_up);
            } else {
                self.vel0.x = 0.0;
                self.vel0.z = 0.0;
            }
        }
        self.vel0.y += self.normal_acceleration + self.gravity;
        self.normal_acceleration = 0.0;
        self.vel0 = 0.998 * self.vel0;

        let front = self.rot0.rotate(Vec3::FRONT);
        let front_xz = Vec3::new(front.x, 0.0, front.z);
        if front_xz.sq_norm() > f32::EPSILON {
            let front_xz = front_xz.normalize();
            let proj = self.vel0.proj_unit(front_xz);
            self.vel0 = self.vel0.rej_unit(front_xz);
            self.speed1_adj = front_xz.dot(proj).signum() * proj.norm() * front.dot(front_xz);
        }

        self.vel = self.vel0 + self.vel1;
        let speed = self.vel.norm().min(120.0);
        self.vel = speed * self.vel.normalize();
        self.pos += self.vel;

        self.rot_vec0 = 0.98 * self.rot_vec0;
        let tmp0 = Mat33::from(self.inv_inertia_tensor) * self.normal_rot_vec;
        let tmp1 = Mat33::from(self.inv_inertia_tensor) * (self.normal_rot_vec + tmp0);
        let normal_rot_vec = 0.5 * (tmp0 + tmp1);
        self.rot_vec0 += normal_rot_vec;
        self.normal_rot_vec = Vec3::ZERO;
        if stats.vehicle.drift_kind.is_bike() {
            self.rot_vec0.z = 0.0;
        }
        self.rot_vec0.x = self.rot_vec0.x.clamp(-0.4, 0.4);
        self.rot_vec0.y = self.rot_vec0.y.clamp(-0.4, 0.4);
        self.rot_vec0.z = self.rot_vec0.z.clamp(-0.8, 0.8);

        let rot_vec = self.rot_factor * self.rot_vec0 + self.rot_vec2;
        if rot_vec.sq_norm() > f32::EPSILON {
            self.rot0 += 0.5 * (self.rot0 * Quat::from(rot_vec));
            self.rot0 = if self.rot0.sq_norm() >= f32::EPSILON {
                self.rot0.normalize()
            } else {
                Quat::IDENTITY
            };
        }
        self.stabilize(stats);
        self.rot0 = if self.rot0.sq_norm() >= f32::EPSILON {
            self.rot0.normalize()
        } else {
            Quat::IDENTITY
        };

        let special_rot = self.non_conserved_special_rot * self.conserved_special_rot;
        self.rot1 = self.rot0 * special_rot;
        self.rot1 = self.rot1.normalize();
        self.non_conserved_special_rot = Quat::IDENTITY;
        self.conserved_special_rot = self.conserved_special_rot.slerp_to(Quat::IDENTITY, 0.1);
    }

    fn stabilize(&mut self, stats: &Stats) {
        let up = if stats.vehicle.drift_kind.is_bike() {
            let front = self.rot0.rotate(Vec3::FRONT);
            let right = self.up.cross(front);
            let front = right.cross(self.up).normalize();

            let speed_ratio = (self.speed1 / stats.common.base_speed).clamp(0.0, 1.0);
            let t = (2.0 * speed_ratio).min(1.0);
            let other_up = (1.0 - t) * Vec3::UP + t * self.up;
            let other_up = if other_up.sq_norm() > f32::EPSILON {
                other_up.normalize()
            } else {
                self.up
            };
            let right = other_up.cross(front);

            front.cross(right).normalize()
        } else {
            self.up
        };
        let rot0_up = self.rot0.rotate(Vec3::UP);
        if up.dot(rot0_up).abs() < 0.9999 {
            let rot = Quat::from_vecs(rot0_up, up);
            self.rot0 = self.rot0.slerp_to(rot * self.rot0, self.stabilization_factor);
        }
    }

    pub fn update_mat(&mut self) {
        self.mat = Mat34::from_quat_and_pos(self.rot1, self.pos);
    }

    pub fn apply_rigid_body_motion(
        &mut self,
        airtime: u32,
        is_boosting: bool,
        jump_pad_applied_dir: bool,
        mut pos_rel: Vec3,
        vel: Vec3,
        floor_nor: Vec3,
    ) {
        let floor_nor = floor_nor.normalize();
        let dot = vel.dot(floor_nor);
        if dot >= 0.0 {
            return;
        }

        if airtime > 20 && !is_boosting && self.vel.y < -50.0 {
            pos_rel.x = 0.0;
            pos_rel.z = 0.0;
        }

        let mat = Mat34::from_quat_and_pos(self.rot0, Vec3::ZERO);
        let mat = Mat33::from(mat * self.inv_inertia_tensor * mat.transpose());
        let cross = mat * pos_rel.cross(floor_nor);
        let cross = cross.cross(pos_rel);
        let s = if is_boosting { 1.0 } else { 1.0 + 0.05 };
        let val = (-dot * s) / (1.0 + floor_nor.dot(cross));
        let cross = floor_nor.cross(-vel);
        let cross = cross.cross(floor_nor);
        let cross = cross.normalize();
        let other_val = val * vel.dot(cross) / dot;
        let other_val = other_val.signum() * other_val.abs().min(0.01 * val);
        let sum = val * floor_nor + other_val * cross;
        if !jump_pad_applied_dir && self.vel1.y > 0.0 && self.vel0.y + self.vel1.y < 0.0 {
            self.vel0.y += self.vel1.y;
        }
        let last_vel0_y = self.vel0.y;
        self.vel0 += sum;
        if jump_pad_applied_dir {
            self.vel0.y = last_vel0_y;
        }
        if last_vel0_y < 0.0 && self.vel0.y > 0.0 && self.vel0.y < 10.0 {
            self.vel0.y = 0.0;
        }
        let mut cross = self.rot0.inv_rotate(mat * pos_rel.cross(sum));
        cross.y = 0.0;
        self.rot_vec0 += cross;
    }
}
