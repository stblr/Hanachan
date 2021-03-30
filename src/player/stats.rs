use core::ops::Add;

use crate::take::{self, Take, TakeFromSlice};

#[derive(Clone, Copy, Debug)]
pub struct Stats {
    vehicle: VehicleStats,
    common: CommonStats,
}

impl Stats {
    pub fn merge_with(self, other: CommonStats) -> Stats {
        Stats {
            vehicle: self.vehicle,
            common: self.common + other,
        }
    }
}

impl TakeFromSlice for Stats {
    fn take_from_slice(slice: &mut &[u8]) -> Result<Stats, take::Error> {
        Ok(Stats {
            vehicle: slice.clone().take()?,
            common: slice.take()?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct VehicleStats {
    kind: VehicleKind,
}

impl TakeFromSlice for VehicleStats {
    fn take_from_slice(slice: &mut &[u8]) -> Result<VehicleStats, take::Error> {
        let kind = slice.take()?;
        slice.skip(0x18c - 0x8)?;

        Ok(VehicleStats { kind })
    }
}

#[derive(Clone, Copy, Debug)]
enum VehicleKind {
    OutsideDriftingKart,
    OutsideDriftingBike,
    InsideDriftingBike,
}

impl TakeFromSlice for VehicleKind {
    fn take_from_slice(slice: &mut &[u8]) -> Result<VehicleKind, take::Error> {
        match (slice.take::<u32>()?, slice.take::<u32>()?) {
            (0, 0) => Ok(VehicleKind::OutsideDriftingKart),
            (3, 0) => Ok(VehicleKind::OutsideDriftingKart),
            (1, 1) => Ok(VehicleKind::OutsideDriftingBike),
            (2, 1) => Ok(VehicleKind::OutsideDriftingBike),
            (1, 2) => Ok(VehicleKind::InsideDriftingBike),
            (2, 2) => Ok(VehicleKind::InsideDriftingBike),
            _ => Err(take::Error {}),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CommonStats {
    weight: f32,
    base_speed: f32,
    handling_speed_multiplier: f32,
    acceleration_ys: [f32; 4],
    acceleration_xs: [f32; 3],
    drift_acceleration_ys: [f32; 2],
    drift_acceleration_xs: [f32; 1],
    manual_handling_tightness: f32,
    automatic_handling_tightness: f32,
    handling_reactivity: f32,
    manual_drift_tightness: f32,
    automatic_drift_tightness: f32,
    drift_reactivity: f32,
}

impl TakeFromSlice for CommonStats {
    fn take_from_slice(slice: &mut &[u8]) -> Result<CommonStats, take::Error> {
        slice.skip(0x10)?;
        let weight = slice.take()?;
        slice.skip(0x4)?;
        let base_speed = slice.take()?;
        let handling_speed_multiplier = slice.take()?;
        slice.skip(0x4)?;
        let acceleration_ys = [slice.take()?, slice.take()?, slice.take()?, slice.take()?];
        let acceleration_xs = [slice.take()?, slice.take()?, slice.take()?];
        let drift_acceleration_ys = [slice.take()?, slice.take()?];
        let drift_acceleration_xs = [slice.take()?];
        let manual_handling_tightness = slice.take()?;
        let automatic_handling_tightness = slice.take()?;
        let handling_reactivity = slice.take()?;
        let manual_drift_tightness = slice.take()?;
        let automatic_drift_tightness = slice.take()?;
        let drift_reactivity = slice.take()?;
        slice.skip(0x18c - 0x64)?;

        Ok(CommonStats {
            weight,
            base_speed,
            handling_speed_multiplier,
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
