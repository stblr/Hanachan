#[derive(Clone, Copy, Debug)]
pub struct BoostRampVariant {
    id: u8,
}

impl BoostRampVariant {
    pub fn new(id: u8) -> BoostRampVariant {
        BoostRampVariant { id }
    }

    pub fn id(&self) -> u8 {
        self.id
    }
}
