mod timer;

pub use timer::{Stage, Timer};

use crate::player::Player;
use crate::track::Track;

pub struct Race<'a> {
    track: &'a Track,
    player: Player,
    timer: Timer,
}

impl Race<'_> {
    pub fn new(track: &Track, player: Player) -> Race {
        Race {
            track,
            player,
            timer: Timer::new(),
        }
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn frame_idx(&self) -> u32 {
        self.timer.frame_idx()
    }

    pub fn update(&mut self) {
        self.player.update(self.track.kcl(), &self.timer);
        self.timer.update();
    }
}
