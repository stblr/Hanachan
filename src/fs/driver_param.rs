use std::iter;

use crate::fs::{Error, Parse, ResultExt, SliceRefExt};
use crate::player::{Character, CommonStats};

#[derive(Clone, Debug)]
pub struct DriverParam {
    characters: Vec<CommonStats>,
}

impl DriverParam {
    pub fn character(&self, character: Character) -> &CommonStats {
        &self.characters[u8::from(character) as usize]
    }
}

impl Parse for DriverParam {
    fn parse(input: &mut &[u8]) -> Result<DriverParam, Error> {
        input
            .take::<u32>()
            .filter(|character_count| *character_count == 27)?;
        let characters = iter::repeat_with(|| input.take())
            .take(27)
            .collect::<Result<_, _>>()?;

        Ok(DriverParam { characters })
    }
}
