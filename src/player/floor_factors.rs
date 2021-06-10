use std::ops::Add;

use crate::player::{Collision, CommonStats, VehicleBody, Wheel};

#[derive(Clone, Debug)]
pub struct FloorFactors {
    speed_factor: f32,
    rot_factor: f32,
    invicibility: u16,
}

impl FloorFactors {
    pub fn new() -> FloorFactors {
        FloorFactors {
            speed_factor: 1.0,
            rot_factor: 1.0,
            invicibility: 0,
        }
    }

    pub fn speed_factor(&self) -> f32 {
        self.speed_factor
    }

    pub fn rot_factor(&self) -> f32 {
        self.rot_factor
    }

    pub fn update_factors<'a>(
        &mut self,
        stats: &CommonStats,
        vehicle_body: &VehicleBody,
        wheels: &Vec<Wheel>,
        collisions: impl Iterator<Item = &'a Collision> + Clone,
    ) {
        let speed_factor_min = collisions
            .clone()
            .filter_map(Collision::speed_factor)
            .reduce(|sf0, sf1| sf0.min(sf1));
        if self.invicibility > 0 {
            self.speed_factor = stats.kcl_speed_factors[0];
        } else if let Some(speed_factor_min) = speed_factor_min {
            self.speed_factor = speed_factor_min;
        }

        let rot_factor_sum = collisions
            .filter_map(Collision::rot_factor)
            .reduce(Add::add);
        if self.invicibility > 0 {
            self.rot_factor = stats.kcl_rot_factors[0];
        } else if let Some(rot_factor_sum) = rot_factor_sum {
            let wheel_collision_count = wheels
                .iter()
                .filter(|wheel| wheel.collision().floor_nor().is_some())
                .count();
            let mut collision_count = wheel_collision_count;
            if wheel_collision_count > 0 && vehicle_body.has_floor_collision() {
                // This is a bug in the game
                collision_count += 1;
            }
            if vehicle_body.collision().floor_nor().is_some() {
                collision_count += 1;
            }
            self.rot_factor = rot_factor_sum / collision_count as f32;
        }
    }

    pub fn activate_invicibility(&mut self, duration: u16) {
        self.invicibility = self.invicibility.max(duration);
    }

    pub fn update_invicibility(&mut self) {
        self.invicibility = self.invicibility.saturating_sub(1);
    }
}
