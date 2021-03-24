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

impl TakeFromSlice for f32 {
    fn take_from_slice(slice: &mut &[u8]) -> Result<f32, Error> {
        slice.take::<u32>().map(f32::from_bits)
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
    fn skip(&mut self, size: usize) -> Result<(), Error>;
}

impl Take for &[u8] {
    fn take<T: TakeFromSlice>(&mut self) -> Result<T, Error> {
        <T>::take_from_slice(self)
    }

    fn skip(&mut self, size: usize) -> Result<(), Error> {
        *self = self.get(size..).ok_or(Error {})?;
        Ok(())
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

#[derive(Clone, Debug)]
pub struct Bits<'a> {
    slice: &'a [u8],
    leftover: Option<(u8, u8)>,
}

impl<'a> Bits<'a> {
    pub fn new(slice: &[u8]) -> Bits {
        Bits {
            slice,
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
            let next = self.slice.take::<u8>()?;
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
            None => Ok(self.slice),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Error {}
