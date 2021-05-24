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
    pub wheel_count: u8,
    pub has_handle: bool,
    pub drift_kind: DriftKind,
}

impl Parse for VehicleStats {
    fn parse(input: &mut &[u8]) -> Result<VehicleStats, Error> {
        let (wheel_count, has_handle) = match input.take::<u32>()? {
            0 => (4, false),
            1 => (2, true),
            2 => (2, false),
            3 => (3, false),
            _ => return Err(Error {}),
        };

        let drift_kind = input.take()?;

        input.skip(0x18c - 0x8)?;

        Ok(VehicleStats {
            wheel_count,
            has_handle,
            drift_kind,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DriftKind {
    KartOutsideDrift,
    BikeOutsideDrift,
    BikeInsideDrift,
}

impl DriftKind {
    pub fn is_bike(&self) -> bool {
        match self {
            DriftKind::KartOutsideDrift => false,
            DriftKind::BikeOutsideDrift => true,
            DriftKind::BikeInsideDrift => true,
        }
    }

    pub fn is_inside(&self) -> bool {
        match self {
            DriftKind::KartOutsideDrift => false,
            DriftKind::BikeOutsideDrift => false,
            DriftKind::BikeInsideDrift => true,
        }
    }
}

impl Parse for DriftKind {
    fn parse(input: &mut &[u8]) -> Result<DriftKind, Error> {
        match input.take::<u32>()? {
            0 => Ok(DriftKind::KartOutsideDrift),
            1 => Ok(DriftKind::BikeOutsideDrift),
            2 => Ok(DriftKind::BikeInsideDrift),
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
    pub drift_acceleration_ys: [f32; 2],
    pub drift_acceleration_xs: [f32; 2],
    pub manual_handling_tightness: f32,
    automatic_handling_tightness: f32,
    pub handling_reactivity: f32,
    pub manual_drift_tightness: f32,
    automatic_drift_tightness: f32,
    pub drift_reactivity: f32,
    pub outside_drift_target_angle: f32,
    pub outside_drift_dec: f32,
    pub mt_duration: u32,
    pub kcl_speed_factors: [f32; 32],
    pub kcl_rot_factors: [f32; 32],
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
        let outside_drift_target_angle = input.take()?;
        let outside_drift_dec = input.take()?;
        let mt_duration = input.take()?;
        let mut kcl_speed_factors = [0.0; 32];
        for kcl_speed_factor in &mut kcl_speed_factors {
            *kcl_speed_factor = input.take()?;
        }
        let mut kcl_rot_factors = [0.0; 32];
        for kcl_rot_factor in &mut kcl_rot_factors {
            *kcl_rot_factor = input.take()?;
        }
        input.skip(0x18c - 0x170)?;

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
            outside_drift_target_angle,
            outside_drift_dec,
            mt_duration,
            kcl_speed_factors,
            kcl_rot_factors,
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
        self.mt_duration += other.mt_duration;
        for i in 0..32 {
            self.kcl_speed_factors[i] += other.kcl_speed_factors[i];
            self.kcl_rot_factors[i] += other.kcl_rot_factors[i];
        }
        self
    }
}
