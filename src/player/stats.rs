use std::ops::Add;

use crate::fs::{Error, Parse, SliceRefExt};

#[derive(Clone, Copy, Debug)]
pub struct Stats {
    pub vehicle: VehicleStats,
    pub common: CommonStats,
}

impl Stats {
    pub fn merge_with(self, other: CommonStats) -> Stats {
        Stats {
            vehicle: self.vehicle,
            common: self.common + other,
        }
    }
}

impl Parse for Stats {
    fn parse(input: &mut &[u8]) -> Result<Stats, Error> {
        Ok(Stats {
            vehicle: input.clone().take()?,
            common: input.take()?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VehicleStats {
    pub kind: VehicleKind,
}

impl Parse for VehicleStats {
    fn parse(input: &mut &[u8]) -> Result<VehicleStats, Error> {
        let kind = input.take()?;
        input.skip(0x18c - 0x8)?;

        Ok(VehicleStats { kind })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum VehicleKind {
    OutsideDriftingFourWheeledKart,
    OutsideDriftingThreeWheeledKart,
    OutsideDriftingBike,
    InsideDriftingBike,
}

impl VehicleKind {
    pub fn is_bike(&self) -> bool {
        match self {
            VehicleKind::OutsideDriftingFourWheeledKart => false,
            VehicleKind::OutsideDriftingThreeWheeledKart => false,
            VehicleKind::OutsideDriftingBike => true,
            VehicleKind::InsideDriftingBike => true,
        }
    }

    pub fn wheel_count(&self) -> u8 {
        match self {
            VehicleKind::OutsideDriftingFourWheeledKart => 4,
            VehicleKind::OutsideDriftingThreeWheeledKart => 3,
            VehicleKind::OutsideDriftingBike => 2,
            VehicleKind::InsideDriftingBike => 2,
        }
    }
}

impl Parse for VehicleKind {
    fn parse(input: &mut &[u8]) -> Result<VehicleKind, Error> {
        match (input.take::<u32>()?, input.take::<u32>()?) {
            (0, 0) => Ok(VehicleKind::OutsideDriftingFourWheeledKart),
            (3, 0) => Ok(VehicleKind::OutsideDriftingThreeWheeledKart),
            (1, 1) => Ok(VehicleKind::OutsideDriftingBike),
            (2, 1) => Ok(VehicleKind::OutsideDriftingBike),
            (1, 2) => Ok(VehicleKind::InsideDriftingBike),
            (2, 2) => Ok(VehicleKind::InsideDriftingBike),
            _ => Err(Error {}),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CommonStats {
    weight: f32,
    pub base_speed: f32,
    pub handling_speed_multiplier: f32,
    pub tilt_factor: f32,
    pub acceleration_ys: [f32; 4],
    pub acceleration_xs: [f32; 4],
    drift_acceleration_ys: [f32; 2],
    drift_acceleration_xs: [f32; 2],
    pub manual_handling_tightness: f32,
    automatic_handling_tightness: f32,
    pub handling_reactivity: f32,
    manual_drift_tightness: f32,
    automatic_drift_tightness: f32,
    drift_reactivity: f32,
}

impl Parse for CommonStats {
    fn parse(input: &mut &[u8]) -> Result<CommonStats, Error> {
        input.skip(0x10)?;
        let weight = input.take()?;
        input.skip(0x4)?;
        let base_speed = input.take()?;
        let handling_speed_multiplier = input.take()?;
        let tilt_factor = input.take()?;
        let acceleration_ys = [input.take()?, input.take()?, input.take()?, input.take()?];
        let acceleration_xs = [0.0, input.take()?, input.take()?, input.take()?];
        let drift_acceleration_ys = [input.take()?, input.take()?];
        let drift_acceleration_xs = [0.0, input.take()?];
        let manual_handling_tightness = input.take()?;
        let automatic_handling_tightness = input.take()?;
        let handling_reactivity = input.take()?;
        let manual_drift_tightness = input.take()?;
        let automatic_drift_tightness = input.take()?;
        let drift_reactivity = input.take()?;
        input.skip(0x18c - 0x64)?;

        Ok(CommonStats {
            weight,
            base_speed,
            handling_speed_multiplier,
            tilt_factor,
            acceleration_ys,
            acceleration_xs,
            drift_acceleration_ys,
            drift_acceleration_xs,
            manual_handling_tightness,
            automatic_handling_tightness,
            handling_reactivity,
            manual_drift_tightness,
            automatic_drift_tightness,
            drift_reactivity,
        })
    }
}

impl Add for CommonStats {
    type Output = CommonStats;

    fn add(mut self, other: CommonStats) -> CommonStats {
        self.weight += other.weight;
        self.base_speed += other.base_speed;
        self.handling_speed_multiplier += other.handling_speed_multiplier;
        self.acceleration_ys[0] += other.acceleration_ys[0];
        self.acceleration_ys[1] += other.acceleration_ys[1];
        self.acceleration_ys[2] += other.acceleration_ys[2];
        self.acceleration_ys[3] += other.acceleration_ys[3];
        self.acceleration_xs[0] += other.acceleration_xs[0];
        self.acceleration_xs[1] += other.acceleration_xs[1];
        self.acceleration_xs[2] += other.acceleration_xs[2];
        self.drift_acceleration_ys[0] += other.drift_acceleration_ys[0];
        self.drift_acceleration_ys[1] += other.drift_acceleration_ys[1];
        self.drift_acceleration_xs[0] += other.drift_acceleration_xs[0];
        self.manual_handling_tightness += other.manual_handling_tightness;
        self.automatic_handling_tightness += other.automatic_handling_tightness;
        self.handling_reactivity += other.handling_reactivity;
        self.manual_drift_tightness += other.manual_drift_tightness;
        self.automatic_drift_tightness += other.automatic_drift_tightness;
        self.drift_reactivity += other.drift_reactivity;
        self
    }
}
