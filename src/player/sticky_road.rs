use crate::fs::Kcl;
use crate::geom::{Hitbox, Vec3};
use crate::player::{Collision, Physics};

#[derive(Clone, Debug)]
pub struct StickyRoad {
    enabled: bool,
}

impl StickyRoad {
    pub fn new() -> StickyRoad {
        StickyRoad { enabled: false }
    }

    pub fn update<'a>(
        &mut self,
        physics: &mut Physics,
        mut collisions: impl Iterator<Item = &'a Collision>,
        kcl: &Kcl,
    ) {
        let has_sticky_road = collisions.any(Collision::has_sticky_road);
        if has_sticky_road {
            self.enabled = true;
        }

        if !self.enabled {
            return;
        }

        let mut pos = physics.pos;
        let mut vel = physics.speed1 * physics.vel1_dir;
        for _ in 0..3 {
            let hitbox = Hitbox::new(pos + vel, None, 200.0, 0x400800);

            let kcl_collision = kcl.check_collision(hitbox);

            if kcl_collision.surface_kinds() & 0x400800 != 0 {
                let floor_nor = kcl_collision.floor_nor();
                physics.vel1_dir = physics.vel1_dir.perp_in_plane(floor_nor, true);
                return;
            }

            pos -= physics.mat * Vec3::new(0.0, 200.0, 0.0);
            vel = 0.5 * vel;
        }

        self.enabled = false;
    }
}
