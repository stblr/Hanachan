use std::ops::Add;

use crate::fs::Bsp;
use crate::geom::{Mat33, Mat34, Quat, Vec3};
use crate::player::{Boost, Drift, Stats, Wheel};
use crate::race::{Race, Stage};

#[derive(Clone, Debug)]
pub struct Physics {
    pub inv_inertia_tensor: Mat34,
    pub rot_factor: f32,
    pub mat: Mat34,
    pub floor_nor: Vec3,
    pub dir: Vec3,
    pub dir_diff: Vec3,
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
    pub fn new(bsp: &Bsp, ktpt_pos: Vec3) -> Physics {
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

        let pos = ktpt_pos + Vec3::new(0.0, bsp.initial_pos_y, 0.0);

        Physics {
            inv_inertia_tensor,
            rot_factor: bsp.rot_factor,
            mat: Mat34::from_quat_and_pos(Quat::BACK, pos),
            floor_nor: Vec3::UP,
            dir: Vec3::BACK,
            dir_diff: Vec3::ZERO,
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
        ground: bool,
        is_landing: bool,
        is_hopping: bool,
        wheels: &Vec<Wheel>,
    ) {
        if !is_hopping {
            if ground {
                self.floor_nor = wheels
                    .iter()
                    .filter_map(|wheel| wheel.floor_nor)
                    .reduce(Add::add)
                    .map(|floor_nor| floor_nor.normalize())
                    .unwrap_or(Vec3::UP);
            }
        }

        self.stabilization_factor = if is_landing {
            self.dir_diff = self.dir.perp_in_plane(self.floor_nor, true);
            self.dir_diff = self.dir_diff.proj_unit(self.dir_diff);
            0.1
        } else if is_hopping {
            if is_inside_drift {
                0.22
            } else {
                0.5
            }
        } else {
            let front = Mat33::from(self.mat) * Vec3::FRONT;
            let dot = front.dot(self.floor_nor);
            if dot < -0.1 {
                0.1 + (0.5 * dot.abs().min(0.2))
            } else {
                0.1
            }
        }
    }

    pub fn update_dir(&mut self, airtime: u32, drift: &Drift) {
        if airtime > 5 {
            return;
        }
        
        let next_dir = drift.hop_dir().unwrap_or_else(|| {
            let right = self.rot0.rotate(Vec3::RIGHT);
            right.cross(self.floor_nor).normalize()
        });
        let angle = drift.outside_drift_angle().to_radians();
        let mat = Mat33::from(Mat34::from_axis_angle(self.floor_nor, angle));
        let next_dir = mat * next_dir;
        let next_dir = next_dir.perp_in_plane(self.floor_nor, true);
        let next_dir_diff = next_dir - self.dir;
        if next_dir_diff.sq_norm() <= f32::EPSILON {
            self.dir = next_dir;
            self.dir_diff = Vec3::ZERO;
        } else {
            let axis = self.dir.cross(next_dir);
            let next_dir_diff = self.dir_diff + 0.7 * next_dir_diff;
            self.dir = (self.dir + next_dir_diff).normalize();
            self.dir_diff = 0.1 * next_dir_diff;
            let next_axis = self.dir.cross(next_dir);
            if axis.dot(next_axis) < 0.0 {
                self.dir = next_dir;
                self.dir_diff = Vec3::ZERO;
            }
        }
    }

    pub fn update_vel1(
        &mut self,
        stats: &Stats,
        airtime: u32,
        is_drifting: bool,
        boost: &Boost,
        raw_turn: f32,
        is_wheelieing: bool,
        race: &Race,
    ) {
        if !is_drifting && race.stage() == Stage::Race {
            self.speed1 += self.speed1_adj;
        }

        let ground = airtime == 0;
        if !boost.is_boosting() && ground && !is_drifting {
            let t = stats.common.handling_speed_multiplier;
            self.speed1 *= t + (1.0 - t) * (1.0 - raw_turn.abs());
        }

        self.last_speed1 = self.speed1;
        if !ground {
            if airtime > 5 {
                self.speed1 *= 0.999;
            }
        } else if let Some(boost_acceleration) = boost.acceleration() {
            self.speed1 += boost_acceleration;
        } else if race.stage() == Stage::Race {
            let (ys, xs): (&[f32], &[f32]) = if is_drifting {
                (&stats.common.drift_acceleration_ys, &stats.common.drift_acceleration_xs)
            } else {
                (&stats.common.acceleration_ys, &stats.common.acceleration_xs)
            };
            self.speed1 += self.compute_acceleration(ys, xs);
        }

        let base_speed = stats.common.base_speed;
        let boost_factor = boost.factor();
        let wheelie_bonus = if is_wheelieing { 0.15 } else { 0.0 };
        let next_speed1_soft_limit = base_speed * (boost_factor + wheelie_bonus);
        self.speed1_soft_limit = (self.speed1_soft_limit - 3.0).max(next_speed1_soft_limit);
        self.speed1 = self.speed1.min(self.speed1_soft_limit);

        let vel1_dir = if airtime > 5 {
            self.dir
        } else {
            self.dir.perp_in_plane(self.floor_nor, true)
        };
        let right = self.floor_nor.cross(self.dir);
        let angle = if ground { 0.5_f32 } else { 0.2_f32 }.to_radians();
        let vel1_dir = Mat33::from(Mat34::from_axis_angle(right, angle)) * vel1_dir;
        self.vel1 = self.speed1 * vel1_dir;
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

    pub fn update(&mut self, is_bike: bool, race: &Race) {
        if race.stage() != Stage::Race {
            if is_bike {
                self.vel0 = self.vel0.rej_unit(self.floor_nor);
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
        self.vel = self.vel.norm() * self.vel.normalize();
        self.pos += self.vel;

        self.rot_vec0 = 0.98 * self.rot_vec0;
        let tmp0 = Mat33::from(self.inv_inertia_tensor) * self.normal_rot_vec;
        let tmp1 = Mat33::from(self.inv_inertia_tensor) * (self.normal_rot_vec + tmp0);
        let normal_rot_vec = 0.5 * (tmp0 + tmp1);
        self.rot_vec0 += normal_rot_vec;
        self.normal_rot_vec = Vec3::ZERO;
        if is_bike {
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
        self.stabilize(is_bike);
        self.rot0 = if self.rot0.sq_norm() >= f32::EPSILON {
            self.rot0.normalize()
        } else {
            Quat::IDENTITY
        };

        self.rot1 = self.rot0.normalize();
    }

    fn stabilize(&mut self, is_bike: bool) {
        let floor_nor = if is_bike {
            let front = self.rot0.rotate(Vec3::FRONT);
            let right = self.floor_nor.cross(front);
            let front = right.cross(self.floor_nor).normalize();
            let right = self.floor_nor.cross(front);
            front.cross(right).normalize()
        } else {
            self.floor_nor
        };
        let up = self.rot0.rotate(Vec3::UP);
        if floor_nor.dot(up).abs() < 0.9999 {
            let rot = Quat::from_vecs(up, floor_nor);
            self.rot0 = self.rot0.slerp_to(rot * self.rot0, self.stabilization_factor);
        }
    }

    pub fn update_mat(&mut self) {
        self.mat = Mat34::from_quat_and_pos(self.rot1, self.pos);
    }

}
