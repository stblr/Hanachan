use crate::geom::Vec3;
use crate::player::{CommonStats, Drift, Physics};
use crate::wii::F32Ext;

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
        airtime: u32,
        drift: &Drift,
        is_wheelieing: bool,
        physics: &mut Physics,
    ) {
        let mut rot = if drift.is_drifting() {
            self.drift * (stats.manual_drift_tightness + drift.outside_drift_turn_bonus())
        } else {
            self.drift * stats.manual_handling_tightness
        };

        if drift.has_hop_height() {
            rot *= 1.4;
        }

        if !drift.is_drifting() {
            rot = if physics.speed1.abs() < 1.0 {
                0.0
            } else if physics.speed1 < 20.0 {
                0.4 * rot + (physics.speed1 / 20.0) * (rot * 0.6)
            } else if physics.speed1 < 70.0 {
                0.5 * rot + (1.0 - (physics.speed1 - 20.0) / (70.0 - 20.0)) * (rot * 0.5)
            } else {
                0.5 * rot
            };
        }

        rot = match airtime {
            0..=29 => rot,
            30..=70 => ((1.0 - 0.025 * (airtime - 30) as f32) * rot).max(0.0),
            _ => 0.0,
        };

        let front = physics.rot0.rotate(Vec3::FRONT);
        let cross = front.cross(physics.dir);
        let norm = cross.sq_norm().wii_sqrt();
        let dot = front.dot(physics.dir);
        let angle = norm.wii_atan2(dot).abs().to_degrees();
        if angle > 60.0 {
            rot *= (1.0 - (angle - 60.0) / (100.0 - 60.0)).max(0.0);
        }

        if is_wheelieing {
            rot *= 0.2;
        }

        physics.rot_vec2.y += rot;
    }
}
