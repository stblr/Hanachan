use std::iter;

use crate::fs::{Error, Parse, ResultExt, SliceRefExt};
use crate::player::{Stats, Vehicle};

#[derive(Clone, Debug)]
pub struct KartParam {
    vehicles: Vec<Stats>,
}

impl KartParam {
    pub fn vehicle(&self, vehicle: Vehicle) -> &Stats {
        &self.vehicles[u8::from(vehicle) as usize]
    }
}

impl Parse for KartParam {
    fn parse(input: &mut &[u8]) -> Result<KartParam, Error> {
        input
            .take::<u32>()
            .filter(|vehicle_count| *vehicle_count == 36)?;
        let vehicles = iter::repeat_with(|| input.take())
            .take(36)
            .collect::<Result<_, _>>()?;

        Ok(KartParam { vehicles })
    }
}
