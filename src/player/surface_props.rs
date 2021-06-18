use crate::fs::{KclBoostRampVariant, KclCollision, KclJumpPadVariant};

#[derive(Clone, Debug)]
pub struct SurfaceProps {
    has_boost_panel: bool,
    has_boost_ramp: bool,
    boost_ramp: Option<KclBoostRampVariant>,
    jump_pad: Option<KclJumpPadVariant>,
    has_sticky_road: bool,
}

impl SurfaceProps {
    pub fn new() -> SurfaceProps {
        SurfaceProps {
            has_boost_panel: false,
            has_boost_ramp: false,
            boost_ramp: None,
            jump_pad: None,
            has_sticky_road: false,
        }
    }

    pub fn reset(&mut self) {
        self.has_boost_panel = false;
        self.has_boost_ramp = false;
        self.jump_pad = None;
        self.has_sticky_road = false;
    }

    pub fn has_boost_panel(&self) -> bool {
        self.has_boost_panel
    }

    pub fn has_boost_ramp(&self) -> bool {
        self.has_boost_ramp
    }

    pub fn boost_ramp(&self) -> Option<KclBoostRampVariant> {
        self.boost_ramp
    }

    pub fn jump_pad(&self) -> Option<KclJumpPadVariant> {
        self.jump_pad
    }

    pub fn has_sticky_road(&self) -> bool {
        self.has_sticky_road
    }

    pub fn add(&mut self, kcl_collision: &KclCollision, allow_boost_panels: bool) {
        if kcl_collision.find_closest(0x800).is_some() {
            self.has_sticky_road = true;
        }

        if let Some(_) = kcl_collision.find_closest(0x20e80fff) {
            if allow_boost_panels && kcl_collision.surface_kinds() & 0x40 != 0 {
                self.has_boost_panel = true;
            }

            if let Some(surface) = kcl_collision.find_closest(0x80) {
                self.has_boost_ramp = true;
                self.boost_ramp = Some(KclBoostRampVariant::new((surface >> 5 & 7) as u8));
            } else {
                self.has_boost_ramp = false;
            }

            if kcl_collision.surface_kinds() & 0x400000 != 0 {
                self.has_sticky_road = true;
            }

            if let Some(surface) = kcl_collision.find_closest(0x100) {
                self.jump_pad = Some(KclJumpPadVariant::new((surface >> 5 & 7) as u8));
            }
        }
    }
}
