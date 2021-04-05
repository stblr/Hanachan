pub struct Race {
    frame: u32,
}

impl Race {
    pub fn new() -> Race {
        Race { frame: 0 }
    }

    pub fn update(&mut self) {
        self.frame += 1;
    }

    pub fn frame(&self) -> u32 {
        self.frame
    }

    pub fn stage(&self) -> Stage {
        match self.frame {
            0..=171 => Stage::Pan,
            172..=410 => Stage::Countdown,
            _ => Stage::Race,
        }
    }
}

#[derive(PartialEq)]
pub enum Stage {
    Pan,
    Countdown,
    Race,
}
