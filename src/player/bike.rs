use crate::player::{Lean, Wheelie};

#[derive(Clone, Debug)]
pub struct Bike {
    pub lean: Lean,
    pub wheelie: Wheelie,
}

impl Bike {
    pub fn new(is_inside_drift: bool) -> Bike {
        Bike {
            lean: Lean::new(is_inside_drift),
            wheelie: Wheelie::new(),
        }
    }
}
