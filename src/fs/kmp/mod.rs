mod ckph;
mod ckpt;
mod enph;
mod enpt;
mod itph;
mod itpt;
mod ktpt;

pub use ktpt::Ktpt;

use std::iter;

use crate::fs::{Error, Parse, ResultExt, SliceExt, SliceRefExt};

use ckph::Ckph;
use ckpt::Ckpt;
use enph::Enph;
use enpt::Enpt;
use itph::Itph;
use itpt::Itpt;

#[derive(Clone, Debug)]
pub struct Kmp {
    pub ktpt: Section<Ktpt>,
    pub enpt: Section<Enpt>,
    pub enph: Section<Enph>,
    pub itpt: Section<Itpt>,
    pub itph: Section<Itph>,
    pub ckpt: Section<Ckpt>,
    pub ckph: Section<Ckph>,
}

impl Parse for Kmp {
    fn parse(input: &mut &[u8]) -> Result<Kmp, Error> {
        let input_len = input.len();
        input
            .take::<u32>()
            .filter(|fourcc| *fourcc == u32::from_be_bytes(*b"RKMD"))?;
        input
            .take::<u32>()
            .filter(|file_size| *file_size as usize == input_len)?;
        input
            .take::<u16>()
            .filter(|section_count| *section_count == 15)?;
        input
            .take::<u16>()
            .filter(|header_size| *header_size == 0x4c)?;
        input.take::<u32>().filter(|version| *version == 2520)?;

        let (mut section_offsets_input, mut input) =
            input.try_split_at(0x4 * 15).ok_or(Error {})?;
        let mut prev_offset = section_offsets_input
            .take::<u32>()
            .filter(|first_section_offset| *first_section_offset == 0)?;
        fn parse_section<T: Parse>(
            offset: u32,
            input: &mut &[u8],
            prev_offset: &mut u32,
        ) -> Result<T, Error> {
            let size = offset.checked_sub(*prev_offset).ok_or(Error {})?;
            *prev_offset = offset;
            let (mut head, tail) = input.try_split_at(size as usize).ok_or(Error {})?;
            *input = tail;
            head.take()
        }

        Ok(Kmp {
            ktpt: parse_section(section_offsets_input.take()?, &mut input, &mut prev_offset)?,
            enpt: parse_section(section_offsets_input.take()?, &mut input, &mut prev_offset)?,
            enph: parse_section(section_offsets_input.take()?, &mut input, &mut prev_offset)?,
            itpt: parse_section(section_offsets_input.take()?, &mut input, &mut prev_offset)?,
            itph: parse_section(section_offsets_input.take()?, &mut input, &mut prev_offset)?,
            ckpt: parse_section(section_offsets_input.take()?, &mut input, &mut prev_offset)?,
            ckph: parse_section(section_offsets_input.take()?, &mut input, &mut prev_offset)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Section<T: Entry> {
    pub entries: Vec<T>,
}

impl<T: Entry> Parse for Section<T> {
    fn parse(input: &mut &[u8]) -> Result<Section<T>, Error> {
        input
            .take::<u32>()
            .filter(|fourcc| *fourcc == u32::from_be_bytes(T::FOURCC))?;
        let entry_count = input.take::<u16>()?;
        let _metadata = input.skip(0x2)?;
        let entries = iter::repeat_with(|| input.take())
            .take(entry_count as usize)
            .collect::<Result<_, _>>()?;
        Ok(Section { entries }).filter(|_| input.is_empty())
    }
}

pub trait Entry: Parse {
    const FOURCC: [u8; 4];
}

struct GroupIdcs {
    group_idcs: Vec<u8>,
}

impl From<GroupIdcs> for Vec<u8> {
    fn from(group_idcs: GroupIdcs) -> Vec<u8> {
        group_idcs.group_idcs
    }
}

impl Parse for GroupIdcs {
    fn parse(input: &mut &[u8]) -> Result<GroupIdcs, Error> {
        iter::repeat_with(|| input.take())
            .take(6)
            .filter_map(|group_idx| match group_idx {
                Ok(255) => None,
                val => Some(val),
            })
            .collect::<Result<_, _>>()
            .map(|group_idcs| GroupIdcs { group_idcs })
    }
}
