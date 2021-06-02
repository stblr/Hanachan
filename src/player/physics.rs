use std::convert::identity;
use std::iter;
use std::ops::Add;

use crate::fs::{Bsp, Kcl};
use crate::geom::{Hitbox, Mat33, Mat34, Quat, Vec3};
use crate::player::{Boost, Drift, Stats, VehicleBody, Wheel};
use crate::race::{Race, Stage};

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
    pub pos: Vec3,
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
    pub rot0: Quat,
    pub rot1: Quat,
    pub stabilization_factor: f32,
}

impl Physics {
    pub fn new(bsp: &Bsp, ktpt_pos: Vec3, kcl: &Kcl) -> Physics {
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

        let diff0 = Vec3::new(-800.0, 0.0, 461.87988);
        let diff1 = Vec3::new(800.0, 0.0, -461.87991);
        let mut pos = ktpt_pos + diff0 + diff1;

        let hitbox = Hitbox::new(pos, 100.0);
        if let Some(collision) = kcl.check_collision(hitbox) {
            pos = pos + collision.movement - 100.0 * collision.floor_nor;
            pos += bsp.initial_pos_y * collision.floor_nor;
        }

        Physics {
            inv_inertia_tensor,
            rot_factor: bsp.rot_factor,
            mat: Mat34::from_quat_and_pos(Quat::BACK, pos),
            up: Vec3::UP,
            smoothed_up: Vec3::UP,
            dir: Vec3::BACK,
            dir_diff: Vec3::ZERO,
            vel1_dir: Vec3::BACK,
            pos,
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
            rot1: Quat::BACK,
            stabilization_factor: 0.0,
        }
    }

    pub fn mat(&self) -> Mat34 {
        self.mat
    }

    pub fn update_floor_nor(
        &mut self,
        is_inside_drift: bool,
        airtime: u32,
        is_landing: bool,
        is_hopping: bool,
        is_boosting: bool,
        is_wheelieing: bool,
        vehicle_body: &VehicleBody,
        wheels: &Vec<Wheel>,
    ) {
        let next_up = wheels
            .iter()
            .map(|wheel| wheel.collision())
            .chain(iter::once(vehicle_body.collision()))
            .filter_map(identity)
            .map(|collision| collision.floor_nor)
            .reduce(Add::add)
            .map(|floor_nor| floor_nor.normalize())
            .unwrap_or(Vec3::UP);

        self.stabilization_factor = 0.1;
        if is_landing {
            self.up = next_up;
            self.smoothed_up = self.up;
            self.dir_diff = self.dir.perp_in_plane(self.smoothed_up, true);
            self.dir_diff = self.dir_diff.proj_unit(self.dir_diff);
        } else if is_hopping {
            self.stabilization_factor = if is_inside_drift { 0.22 } else { 0.5 };
        } else if airtime > 20 {
            if self.up.y > 0.99 {
                self.up = Vec3::UP;
            } else {
                self.up += 0.03 * (Vec3::UP - self.up);
            }

            if self.smoothed_up.y > 0.99 {
                self.smoothed_up = Vec3::UP;
            } else {
                self.smoothed_up += 0.03 * (Vec3::UP - self.smoothed_up);
            }
        } else if airtime == 0 {
            self.up = next_up;

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
                self.stabilization_factor = 0.1 + (0.5 * dot.abs().min(0.2));
            }
        }
    }

    pub fn update_dir(&mut self, airtime: u32, kcl_rot_factor: f32, drift: &Drift) {
        if airtime > 5 {
            self.vel1_dir = self.dir;
            return;
        }

        let next_dir = drift.hop_dir().unwrap_or_else(|| {
            let right = self.rot0.rotate(Vec3::RIGHT);
            right.cross(self.smoothed_up).normalize()
        });
        let angle = drift.outside_drift_angle().to_radians();
        let mat = Mat33::from(Mat34::from_axis_angle(self.smoothed_up, angle));
        let next_dir = mat * next_dir;
        let next_dir = next_dir.perp_in_plane(self.smoothed_up, true);
        let next_dir_diff = next_dir - self.dir;
        if next_dir_diff.sq_norm() <= f32::EPSILON {
            self.dir = next_dir;
            self.dir_diff = Vec3::ZERO;
        } else {
            let axis = self.dir.cross(next_dir);
            let next_dir_diff = self.dir_diff + kcl_rot_factor * next_dir_diff;
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

    pub fn update_vel1(
        &mut self,
        stats: &Stats,
        airtime: u32,
        kcl_speed_factor: f32,
        is_drifting: bool,
        boost: &Boost,
        raw_turn: f32,
        is_wheelieing: bool,
        race: &Race,
    ) {
        let last_speed_ratio = (self.speed1 / stats.common.base_speed).min(1.0);

        if !is_drifting && race.stage() == Stage::Race {
            self.speed1 += self.speed1_adj;
        }

        let ground = airtime == 0;

        let mut acceleration = 0.0;
        if !ground {
            if airtime > 5 {
                self.speed1 *= 0.999;
            }
        } else if let Some(boost_acceleration) = boost.acceleration() {
            acceleration = boost_acceleration;
        } else if race.stage() == Stage::Race {
            let (ys, xs): (&[f32], &[f32]) = if is_drifting {
                (&stats.common.drift_acceleration_ys, &stats.common.drift_acceleration_xs)
            } else {
                (&stats.common.acceleration_ys, &stats.common.acceleration_xs)
            };
            acceleration = self.compute_acceleration(ys, xs);

            if !is_drifting {
                let t = stats.common.handling_speed_multiplier;
                self.speed1 *= t + (1.0 - t) * (1.0 - raw_turn.abs() * last_speed_ratio);
            }
        }

        self.last_speed1 = self.speed1;

        self.speed1 += acceleration;

        let base_speed = stats.common.base_speed;
        let boost_factor = boost.factor();
        let wheelie_bonus = if is_wheelieing { 0.15 } else { 0.0 };
        let mut next_soft_limit = (boost_factor + wheelie_bonus) * kcl_speed_factor * base_speed;
        if let Some(boost_limit) = boost.limit() {
            let boost_limit = boost_limit * kcl_speed_factor;
            next_soft_limit = next_soft_limit.max(boost_limit);
        }
        self.speed1_soft_limit = (self.speed1_soft_limit - 3.0).max(next_soft_limit);
        self.speed1_soft_limit = (self.speed1_soft_limit).min(120.0);
        self.speed1 = self.speed1.min(self.speed1_soft_limit);

        let right = self.smoothed_up.cross(self.dir);
        let angle = if ground { 0.5_f32 } else { 0.2_f32 }.to_radians();
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

    pub fn update(&mut self, stats: &Stats, race: &Race) {
        if race.stage() != Stage::Race {
            if stats.vehicle.drift_kind.is_bike() {
                self.vel0 = self.vel0.rej_unit(self.smoothed_up);
            } else {
                self.vel0.x = 0.0;
                self.vel0.z = 0.0;
            }
        }
        self.vel0.y += self.normal_acceleration - 1.3;
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

        self.rot1 = self.rot0.normalize();
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

    pub fn apply_rigid_body_motion(&mut self, pos_rel: Vec3, vel: Vec3, floor_nor: Vec3) {
        let dot = vel.dot(floor_nor);
        if dot >= 0.0 {
            return;
        }

        let mat = Mat34::from_quat_and_pos(self.rot0, Vec3::ZERO);
        let mat = Mat33::from(mat * self.inv_inertia_tensor * mat.transpose());
        let cross = mat * pos_rel.cross(floor_nor);
        let cross = cross.cross(pos_rel);
        let val = -dot / (1.0 + floor_nor.dot(cross));
        let cross = floor_nor.cross(-vel);
        let cross = cross.cross(floor_nor);
        let cross = cross.normalize();
        let other_val = val * vel.dot(cross) / dot;
        let other_val = other_val.signum() * other_val.abs().min(0.01 * val);
        let sum = val * floor_nor + other_val * cross;
        let last_vel0_y = self.vel0.y;
        self.vel0 += sum;
        if last_vel0_y < 0.0 && self.vel0.y > 0.0 && self.vel0.y < 10.0 {
            self.vel0.y = 0.0;
        }
        let mut cross = self.rot0.inv_rotate(mat * pos_rel.cross(sum));
        cross.y = 0.0;
        self.rot_vec0 += cross;
    }
}
