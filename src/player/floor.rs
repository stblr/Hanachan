use std::iter;
use std::ops::Add;

use crate::geom::Vec3;
use crate::player::{Collision, VehicleBody, Wheel};

#[derive(Clone, Debug)]
pub struct Floor {
    nor: Option<Vec3>,
    airtime: u32,
    last_airtime: u32,
    has_trickable: bool,
    trickable_timer: u8,
}

impl Floor {
    pub fn new() -> Floor {
        Floor {
            nor: None,
            last_airtime: 0,
            airtime: 0,
            has_trickable: false,
            trickable_timer: 0,
        }
    }

    pub fn nor(&self) -> Option<Vec3> {
        self.nor
    }

    pub fn airtime(&self) -> u32 {
        self.airtime
    }

    pub fn last_airtime(&self) -> u32 {
        self.last_airtime
    }

    pub fn is_airborne(&self) -> bool {
        self.airtime > 0
    }

    pub fn is_landing(&self) -> bool {
        self.airtime == 0 && self.last_airtime != 0
    }

    pub fn has_trickable(&self) -> bool {
        self.has_trickable
    }

    pub fn update(&mut self, wheels: &Vec<Wheel>, vehicle_body: &VehicleBody) {
        self.nor = wheels
            .iter()
            .map(|wheel| wheel.collision())
            .chain(iter::once(vehicle_body.collision()))
            .filter_map(Collision::floor_nor)
            .reduce(Add::add)
            .map(|sum| sum.normalize());

        self.last_airtime = self.airtime;
        if self.nor.is_some() {
            self.airtime = 0;
        } else {
            self.airtime += 1;
        }

        self.trickable_timer = self.trickable_timer.saturating_sub(1);
        if !self.is_airborne() {
            let has_trickable = wheels
                .iter()
                .map(|wheel| wheel.collision())
                .chain(iter::once(vehicle_body.collision()))
                .any(Collision::has_trickable);
            if has_trickable {
                self.trickable_timer = 3;
            }

            self.has_trickable = self.trickable_timer > 0;
        }
    }
}
