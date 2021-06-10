#[derive(Clone, Copy, Debug)]
pub struct JumpPadVariant {
    id: u8,
}

impl JumpPadVariant {
    pub fn new(id: u8) -> JumpPadVariant {
        JumpPadVariant { id }
    }

    pub fn speed(&self) -> f32 {
        match self.id {
            0 => 50.0,
            1 => 50.0,
            2 => 59.0,
            3 => 73.0,
            4 => 73.0,
            5 => 56.0,
            6 => 55.0,
            7 => 56.0,
            _ => unreachable!(),
        }
    }

    pub fn vel_y(&self) -> f32 {
        match self.id {
            0 => 35.0,
            1 => 47.0,
            2 => 30.0,
            3 => 45.0,
            4 => 53.0,
            5 => 50.0,
            6 => 35.0,
            7 => 50.0,
            _ => unreachable!(),
        }
    }
}
