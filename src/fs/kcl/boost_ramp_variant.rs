#[derive(Clone, Debug)]
pub struct BoostRampVariant {
    id: u8,
}

impl BoostRampVariant {
    pub fn new(id: u8) -> BoostRampVariant {
        BoostRampVariant { id }
    }
}
