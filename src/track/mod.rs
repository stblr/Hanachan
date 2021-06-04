use std::fs;
use std::path::Path;

use crate::fs::{yaz, Kcl, Kmp, SliceRefExt, U8};
use crate::Error;

#[derive(Clone, Debug)]
pub struct Track {
    kmp: Kmp,
    kcl: Kcl,
}

impl Track {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Track, Error> {
        let compressed = fs::read(path)?;
        let mut decompressed: &[u8] = &yaz::decompress(&compressed)?;
        let archive: U8 = decompressed.take()?;

        let kmp = archive
            .get_file("./course.kmp")
            .and_then(|file| file.as_kmp())
            .ok_or(Error::Parsing)?
            .clone();
        let kcl = archive
            .get_file("./course.kcl")
            .and_then(|file| file.as_kcl())
            .ok_or(Error::Parsing)?
            .clone();

        Ok(Track { kmp, kcl })
    }

    pub fn kmp(&self) -> &Kmp {
        &self.kmp
    }

    pub fn kcl(&self) -> &Kcl {
        &self.kcl
    }
}
