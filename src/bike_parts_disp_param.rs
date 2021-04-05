use std::iter;

use crate::player::{Handle, Vehicle};
use crate::take::{self, Take, TakeFromSlice};

#[derive(Clone, Debug)]
pub struct BikePartsDispParam {
    vehicles: Vec<Handle>,
}

impl BikePartsDispParam {
    pub fn vehicle(&self, vehicle: Vehicle) -> Option<Handle> {
        u8::from(vehicle)
            .checked_sub(18)
            .map(|id| self.vehicles[id as usize])
    }
}

impl TakeFromSlice for BikePartsDispParam {
    fn take_from_slice(slice: &mut &[u8]) -> Result<BikePartsDispParam, take::Error> {
        let vehicle_count = slice.take::<u32>()?;
        if vehicle_count != 18 {
            return Err(take::Error {});
        }
        let vehicles = iter::repeat_with(|| {
            slice.skip(0xc)?;
            let handle = slice.take()?;
            slice.skip(0xb0 - 0x24)?;
            Ok(handle)
        })
        .take(18)
        .collect::<Result<_, _>>()?;

        Ok(BikePartsDispParam { vehicles })
    }
}
