use crate::player::{CommonStats, Drift, Physics};

#[derive(Clone, Debug)]
pub struct Turn {
    raw: f32,
    drift: f32,
}

impl Turn {
    pub fn new() -> Turn {
        Turn {
            raw: 0.0,
            drift: 0.0,
        }
    }

    pub fn raw(&self) -> f32 {
        self.raw
    }

    pub fn update(&mut self, stats: &CommonStats, airtime: u32, stick_x: f32, drift: &Drift) {
        let stick_x = match drift.hop_stick_x() {
            Some(hop_stick_x) => hop_stick_x,
            None if airtime > 20 => 0.01 * stick_x,
            None => stick_x,
        };
        let reactivity = if drift.is_drifting() {
            stats.drift_reactivity
        } else {
            stats.handling_reactivity
        };
        self.raw = (1.0 - reactivity) * self.raw + reactivity * -stick_x;

        self.drift = if let Some(drift_stick_x) = drift.drift_stick_x() {
            let drift_turn = 0.5 * (self.raw - drift_stick_x);
            (0.8 * drift_turn - 0.2 * drift_stick_x).clamp(-1.0, 1.0)
        } else {
            self.raw
        };
    }

    pub fn update_rot(
        &self,
        stats: &CommonStats,
        drift: &Drift,
        is_wheelieing: bool,
        physics: &mut Physics,
    ) {
        physics.rot_vec2.y += if drift.is_drifting() {
            self.drift * (stats.manual_drift_tightness + drift.outside_drift_turn_bonus())
        } else {
            let rot = self.drift * stats.manual_handling_tightness;

            let hop_factor = if drift.is_hopping() { 1.4 } else { 1.0 };
            let rot = rot * hop_factor;

            let rot = if physics.speed1.abs() < 1.0 {
                0.0
            } else if physics.speed1 < 20.0 {
                0.4 * rot + (physics.speed1 / 20.0) * (rot * 0.6)
            } else if physics.speed1 < 70.0 {
                0.5 * rot + (1.0 - (physics.speed1 - 20.0) / (70.0 - 20.0)) * (rot * 0.5)
            } else {
                0.5 * rot
            };

            let wheelie_factor = if is_wheelieing { 0.2 } else { 1.0 };
            rot * wheelie_factor
        };
    }
}
