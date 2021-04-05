use std::iter;

use crate::player::{Stats, Vehicle};
use crate::take::{self, Take, TakeFromSlice};

#[derive(Clone, Debug)]
pub struct KartParam {
    vehicles: Vec<Stats>,
}

impl KartParam {
    pub fn vehicle(&self, vehicle: Vehicle) -> &Stats {
        &self.vehicles[u8::from(vehicle) as usize]
    }
}

impl TakeFromSlice for KartParam {
    fn take_from_slice(slice: &mut &[u8]) -> Result<KartParam, take::Error> {
        let vehicle_count = slice.take::<u32>()?;
        if vehicle_count != 36 {
            return Err(take::Error {});
        }
        let vehicles = iter::repeat_with(|| slice.take())
            .take(36)
            .collect::<Result<_, _>>()?;

        Ok(KartParam { vehicles })
    }
}
