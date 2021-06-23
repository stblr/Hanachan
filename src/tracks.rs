use std::path::PathBuf;

use crate::track::{Id as TrackId, Track};
use crate::Error;

pub enum Tracks {
    File {
        track: Track,
    },
    Dir {
        path: PathBuf,
        tracks: [Option<Track>; 32],
    },
}

impl Tracks {
    pub fn try_new(path: &str) -> Result<Tracks, Error> {
        let metadata = std::fs::metadata(path)?;
        if metadata.is_dir() {
            Ok(Tracks::Dir {
                path: PathBuf::from(path),
                tracks: Default::default(),
            })
        } else {
            Ok(Tracks::File {
                track: Track::load(path)?,
            })
        }
    }

    pub fn get(&mut self, id: TrackId) -> Result<&Track, Error> {
        match self {
            Tracks::File { track } => Ok(track),
            Tracks::Dir { path, tracks } => {
                let track = &mut tracks[id.id() as usize];
                match track {
                    Some(track) => Ok(track),
                    None => {
                        let mut path = path.clone();
                        path.push(id.filename());
                        path.set_extension("szs");
                        Ok(track.insert(Track::load(path)?))
                    }
                }
            }
        }
    }
}
