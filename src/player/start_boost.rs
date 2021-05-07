#[derive(Clone, Debug)]
pub struct StartBoost {
    pub charge: f32,
}

impl StartBoost {
    pub fn new() -> StartBoost {
        StartBoost { charge: 0.0 }
    }

    pub fn update(&mut self, accelerate: bool) {
        if accelerate {
            self.charge += 0.02 - (0.02 - 0.002) * self.charge;
        } else {
            self.charge *= 0.96;
        }
        self.charge = self.charge.clamp(0.0, 1.0);
    }

    pub fn boost_frames(&self) -> u16 {
        match self.charge {
            c if c <= 0.85 => 0,
            c if c <= 0.88 => 10,
            c if c <= 0.905 => 20,
            c if c <= 0.925 => 30,
            c if c <= 0.94 => 45,
            c if c <= 0.95 => 70,
            _ => 0, // TODO handle burnout
        }
    }
}
