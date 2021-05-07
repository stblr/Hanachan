use crate::player::{Lean, Wheelie};

#[derive(Clone, Debug)]
pub struct Bike {
    pub lean: Lean,
    pub wheelie: Wheelie,
}

impl Bike {
    pub fn new() -> Bike {
        Bike {
            lean: Lean::new(),
            wheelie: Wheelie::new(),
        }
    }
}
