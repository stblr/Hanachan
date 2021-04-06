use std::iter;

use crate::fs::{Error, Parse, ResultExt, SliceRefExt};
use crate::player::{Handle, Vehicle};

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

impl Parse for BikePartsDispParam {
    fn parse(input: &mut &[u8]) -> Result<BikePartsDispParam, Error> {
        input
            .take::<u32>()
            .filter(|vehicle_count| *vehicle_count == 18)?;
        let vehicles = iter::repeat_with(|| {
            input.skip(0xc)?;
            let handle = input.take()?;
            input.skip(0xb0 - 0x24)?;
            Ok(handle)
        })
        .take(18)
        .collect::<Result<_, _>>()?;

        Ok(BikePartsDispParam { vehicles })
    }
}
