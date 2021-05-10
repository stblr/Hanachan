use crate::player::{CommonStats, Drift, Physics};

#[derive(Clone, Debug)]
pub struct Turn {
    manual_handling_tightness: f32,
    handling_reactivity: f32,
    manual_drift_tightness: f32,
    drift_reactivity: f32,
    raw: f32,
    drift: f32,
}

impl Turn {
    pub fn new(stats: &CommonStats) -> Turn {
        Turn {
            manual_handling_tightness: stats.manual_handling_tightness,
            handling_reactivity: stats.handling_reactivity,
            manual_drift_tightness: stats.manual_drift_tightness,
            drift_reactivity: stats.drift_reactivity,
            raw: 0.0,
            drift: 0.0,
        }
    }

    pub fn raw(&self) -> f32 {
        self.raw
    }

    pub fn update(&mut self, stick_x: f32, drift: &Drift) {
        let stick_x = drift.hop_stick_x().unwrap_or(stick_x);
        let reactivity = if drift.is_drifting() {
            self.drift_reactivity
        } else {
            self.handling_reactivity
        };
        self.raw = (1.0 - reactivity) * self.raw + reactivity * -stick_x;

        self.drift = if let Some(drift_stick_x) = drift.drift_stick_x() {
            let drift_turn = 0.5 * (self.raw - drift_stick_x);
            (0.8 * drift_turn - 0.2 * drift_stick_x).clamp(-1.0, 1.0)
        } else {
            self.raw
        };
    }

    pub fn update_rot(&self, drift: &Drift, is_wheelieing: bool, physics: &mut Physics) {
        physics.rot_vec2.y += if drift.is_drifting() {
            self.drift * self.manual_drift_tightness
        } else {
            let rot = self.drift * self.manual_handling_tightness;

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
