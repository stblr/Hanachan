use std::convert::TryInto;
use std::iter;

pub trait Parse: Sized {
    fn parse(input: &mut &[u8]) -> Result<Self, Error>;
}

impl Parse for u8 {
    fn parse(input: &mut &[u8]) -> Result<u8, Error> {
        let (head, tail) = input.split_first().ok_or(Error {})?;
        *input = tail;
        Ok(*head)
    }
}

impl Parse for u16 {
    fn parse(input: &mut &[u8]) -> Result<u16, Error> {
        let (head, tail) = input.try_split_at(2).ok_or(Error {})?;
        *input = tail;
        let head: [u8; 2] = head.try_into().unwrap();
        Ok(u16::from_be_bytes(head))
    }
}

impl Parse for u32 {
    fn parse(input: &mut &[u8]) -> Result<u32, Error> {
        let (head, tail) = input.try_split_at(4).ok_or(Error {})?;
        *input = tail;
        let head: [u8; 4] = head.try_into().unwrap();
        Ok(u32::from_be_bytes(head))
    }
}

impl Parse for u64 {
    fn parse(input: &mut &[u8]) -> Result<u64, Error> {
        let (head, tail) = input.try_split_at(8).ok_or(Error {})?;
        *input = tail;
        let head: [u8; 8] = head.try_into().unwrap();
        Ok(u64::from_be_bytes(head))
    }
}

impl Parse for f32 {
    fn parse(input: &mut &[u8]) -> Result<f32, Error> {
        input.take::<u32>().map(f32::from_bits)
    }
}

impl Parse for String {
    fn parse(input: &mut &[u8]) -> Result<String, Error> {
        let vec = iter::repeat_with(|| input.take::<u8>())
            .map(|c| c.filter(|c| *c < 0x80))
            .take_while(|c| c.map_or(true, |c| c != b'\0'))
            .collect::<Result<_, _>>()?;
        Ok(String::from_utf8(vec).unwrap())
    }
}

pub trait SliceExt {
    fn try_split_at(&self, mid: usize) -> Option<(&[u8], &[u8])>;
}

impl SliceExt for [u8] {
    fn try_split_at(&self, mid: usize) -> Option<(&[u8], &[u8])> {
        if mid > self.len() {
            None
        } else {
            Some(self.split_at(mid))
        }
    }
}

pub trait SliceRefExt {
    fn take<T: Parse>(&mut self) -> Result<T, Error>;
    fn skip(&mut self, size: usize) -> Result<(), Error>;
}

impl SliceRefExt for &[u8] {
    fn take<T: Parse>(&mut self) -> Result<T, Error> {
        <T>::parse(self)
    }

    fn skip(&mut self, size: usize) -> Result<(), Error> {
        *self = self.get(size..).ok_or(Error {})?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Bits<'a> {
    input: &'a [u8],
    leftover: Option<(u8, u8)>,
}

impl<'a> Bits<'a> {
    pub fn new(input: &[u8]) -> Bits {
        Bits {
            input,
            leftover: None,
        }
    }

    pub fn take_bool(&mut self) -> Result<bool, Error> {
        self.take_u8(1).map(|val| val != 0)
    }

    pub fn take_u8(&mut self, mut size: u8) -> Result<u8, Error> {
        assert!(size > 0 && size <= 8);

        let mut val = 0;
        if let Some((leftover_size, leftover_val)) = self.leftover {
            if size < leftover_size {
                let diff = leftover_size - size;
                size = 0;
                val = leftover_val >> diff;
                self.leftover = Some((diff, leftover_val & (1 << diff) - 1));
            } else {
                size -= leftover_size;
                val = leftover_val;
                self.leftover = None;
            }
        }

        if size != 0 {
            let next = self.input.take::<u8>()?;
            let leftover_size = 8 - size;
            val = val << size | next >> leftover_size;
            if size < 8 {
                self.leftover = Some((leftover_size, next & (1 << leftover_size) - 1));
            }
        }

        Ok(val)
    }

    pub fn take_u16(&mut self, size: u8) -> Result<u16, Error> {
        assert!(size > 8 && size <= 16);

        Ok((self.take_u8(8)? as u16) << size - 8 | self.take_u8(size - 8)? as u16)
    }

    pub fn try_into_inner(self) -> Result<&'a [u8], Error> {
        match self.leftover {
            Some(_) => Err(Error {}),
            None => Ok(self.input),
        }
    }
}

pub trait ResultExt<T> {
    fn filter<P>(self, predicate: P) -> Result<T, Error>
    where
        P: FnOnce(&T) -> bool;
}

impl<T> ResultExt<T> for Result<T, Error> {
    fn filter<P>(self, predicate: P) -> Result<T, Error>
    where
        P: FnOnce(&T) -> bool,
    {
        match self {
            Ok(val) if predicate(&val) => Ok(val),
            _ => Err(Error {}),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Error {}
