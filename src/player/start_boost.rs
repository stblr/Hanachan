#[derive(Clone, Copy, Debug)]
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
}
