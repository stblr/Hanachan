use core::iter;

use crate::player::{Character, CommonStats};
use crate::take::{self, Take, TakeFromSlice};

#[derive(Clone, Debug)]
pub struct DriverParam {
    characters: Vec<CommonStats>,
}

impl DriverParam {
    pub fn character(&self, character: Character) -> &CommonStats {
        &self.characters[u8::from(character) as usize]
    }
}

impl TakeFromSlice for DriverParam {
    fn take_from_slice(slice: &mut &[u8]) -> Result<DriverParam, take::Error> {
        let character_count = slice.take::<u32>()?;
        if character_count != 27 {
            return Err(take::Error {});
        }
        let characters = iter::repeat_with(|| slice.take())
            .take(27)
            .collect::<Result<_, _>>()?;

        Ok(DriverParam { characters })
    }
}
