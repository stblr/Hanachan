#[derive(Clone, Debug)]
pub struct Timer {
    frame_idx: u32,
}

impl Timer {
    pub fn new() -> Timer {
        Timer { frame_idx: 0 }
    }

    pub fn update(&mut self) {
        self.frame_idx += 1;
    }

    pub fn frame_idx(&self) -> u32 {
        self.frame_idx
    }

    pub fn stage(&self) -> Stage {
        match self.frame_idx {
            0..=171 => Stage::Pan,
            172..=410 => Stage::Countdown,
            _ => Stage::Race,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stage {
    Pan,
    Countdown,
    Race,
}
