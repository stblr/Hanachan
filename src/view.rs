use std::convert::TryInto;

pub struct View<'a> {
    inner: &'a [u8]
}

impl View<'_> {
    pub fn new(inner: &[u8]) -> View {
        View {
            inner,
        }
    }

    pub fn inner(&self) -> &[u8] {
        &self.inner
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn get_u8(&self, offset: usize) -> Option<u8> {
        Some(*self.inner.get(offset)?)
    }

    pub fn get_u16(&self, offset: usize) -> Option<u16> {
        let slice = self.inner.get(offset..offset + 2)?;
        let array: [u8; 2] = slice.try_into().unwrap();
        Some(u16::from_be_bytes(array))
    }

    pub fn get_u32(&self, offset: usize) -> Option<u32> {
        let slice = self.inner.get(offset..offset + 4)?;
        let array: [u8; 4] = slice.try_into().unwrap();
        Some(u32::from_be_bytes(array))
    }

    pub fn index_u8(&self, offset: usize) -> u8 {
        self.get_u8(offset).unwrap()
    }

    pub fn index_u16(&self, offset: usize) -> u16 {
        self.get_u16(offset).unwrap()
    }

    pub fn index_u32(&self, offset: usize) -> u32 {
        self.get_u32(offset).unwrap()
    }

    pub fn stream(&self) -> Stream {
        Stream {
            view: &self,
            index: 0,
        }
    }
}

pub struct Stream<'a> {
    view: &'a View<'a>,
    index: usize,
}

impl Stream<'_> {
    pub fn next_u8(&mut self) -> Option<u8> {
        let val = self.view.get_u8(self.index);
        self.index += 1;
        val
    }

    pub fn next_u16(&mut self) -> Option<u16> {
        let val = self.view.get_u16(self.index);
        self.index += 2;
        val
    }
}
