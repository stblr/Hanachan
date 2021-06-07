use crate::fs::Kcl;
use crate::geom::{Hitbox, Vec3};
use crate::player::{Physics, Wheel};

#[derive(Clone, Debug)]
pub struct StickyRoad {
    enabled: bool,
}

impl StickyRoad {
    pub fn new() -> StickyRoad {
        StickyRoad {
            enabled: false,
        }
    }

    pub fn update(&mut self, physics: &mut Physics, wheels: &Vec<Wheel>, kcl: &Kcl) {
        let has_sticky_road = wheels
            .iter()
            .filter_map(|wheel| wheel.collision())
            .any(|collision| collision.has_sticky_road);
        if has_sticky_road {
            self.enabled = true;
        }

        if !self.enabled {
            return;
        }

        let mut pos = physics.pos;
        let mut vel = physics.speed1 * physics.vel1_dir;
        for _ in 0..3 {
            let hitbox = Hitbox::new(pos + vel, false, 200.0, 0x400800);
            if let Some(collision) = kcl.check_collision(hitbox) {
                physics.vel1_dir = physics.vel1_dir.perp_in_plane(collision.floor_nor, true);
                return;
            }

            pos -= physics.mat * Vec3::new(0.0, 200.0, 0.0);
            vel = 0.5 * vel;
        }

        self.enabled = false;
    }
}
