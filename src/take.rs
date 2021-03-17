use core::convert::TryInto;
use core::iter;
use core::marker::PhantomData;

pub trait TakeFromSlice: Sized {
    fn take_from_slice(slice: &mut &[u8]) -> Result<Self, Error>;
}

impl TakeFromSlice for u8 {
    fn take_from_slice(slice: &mut &[u8]) -> Result<u8, Error> {
        let (head, tail) = slice.split_first().ok_or(Error {})?;
        *slice = tail;
        Ok(*head)
    }
}

impl TakeFromSlice for u16 {
    fn take_from_slice(slice: &mut &[u8]) -> Result<u16, Error> {
        if slice.len() < 2 {
            return Err(Error {});
        }
        let (head, tail) = slice.split_at(2);
        *slice = tail;
        let head: [u8; 2] = head.try_into().unwrap();
        Ok(u16::from_be_bytes(head))
    }
}

impl TakeFromSlice for u32 {
    fn take_from_slice(slice: &mut &[u8]) -> Result<u32, Error> {
        if slice.len() < 4 {
            return Err(Error {});
        }
        let (head, tail) = slice.split_at(4);
        *slice = tail;
        let head: [u8; 4] = head.try_into().unwrap();
        Ok(u32::from_be_bytes(head))
    }
}

impl TakeFromSlice for String {
    fn take_from_slice(slice: &mut &[u8]) -> Result<String, Error> {
        let vec = iter::repeat_with(|| slice.take::<u8>())
            .map(|c| c.and_then(|c| if c < 0x80 { Ok(c) } else { Err(Error {}) }))
            .take_while(|c| c.map_or(true, |c| c != b'\0'))
            .collect::<Result<Vec<u8>, Error>>()?;
        Ok(String::from_utf8(vec).unwrap())
    }
}

pub trait Take {
    fn take<T: TakeFromSlice>(&mut self) -> Result<T, Error>;
}

impl Take for &[u8] {
    fn take<T: TakeFromSlice>(&mut self) -> Result<T, Error> {
        <T>::take_from_slice(self)
    }
}

#[derive(Clone, Debug)]
pub struct TakeIter<'a, T: TakeFromSlice> {
    slice: &'a [u8],
    phantom: PhantomData<T>,
}

impl<T: TakeFromSlice> TakeIter<'_, T> {
    pub fn new(slice: &[u8]) -> TakeIter<T> {
        TakeIter {
            slice,
            phantom: PhantomData,
        }
    }
}

impl<T: TakeFromSlice> Iterator for TakeIter<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.slice.take::<T>().ok()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Error {}
