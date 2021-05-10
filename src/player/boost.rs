#[derive(Clone, Debug)]
pub struct Boost {
    durations: [u16; 3],
}

impl Boost {
    pub fn new() -> Boost {
        Boost { durations: [0; 3] }
    }

    fn kind(&self) -> Option<Kind> {
        if self.durations[Kind::Medium as usize] > 0 {
            Some(Kind::Medium)
        } else if self.durations[Kind::Strong as usize] > 0 {
            Some(Kind::Strong)
        } else if self.durations[Kind::Weak as usize] > 0 {
            Some(Kind::Weak)
        } else {
            None
        }
    }

    pub fn is_strong(&self) -> bool {
        match self.kind() {
            Some(Kind::Strong) => true,
            _ => false,
        }
    }

    pub fn is_boosting(&self) -> bool {
        self.kind().is_some()
    }

    pub fn factor(&self) -> f32 {
        match self.kind() {
            Some(Kind::Medium) => 1.3,
            Some(Kind::Strong) => 1.4,
            Some(Kind::Weak) => 1.2,
            None => 1.0,
        }
    }

    pub fn acceleration(&self) -> Option<f32> {
        match self.kind() {
            Some(Kind::Medium) => Some(6.0),
            Some(Kind::Strong) => Some(7.0),
            Some(Kind::Weak) => Some(3.0),
            None => None,
        }
    }

    pub fn update(&mut self) {
        for duration in &mut self.durations {
            *duration = duration.saturating_sub(1);
        }
    }

    pub fn activate(&mut self, kind: Kind, duration: u16) {
        self.durations[kind as usize] = (duration + 1).max(self.durations[kind as usize]);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Medium = 0, // trick, zipper - highest priority
    Strong = 1, // mushroom, boost panel
    Weak = 2,   // start boost, mt, ssmt, respawn boost
}
