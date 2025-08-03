use core::time::Duration;

use crate::assets::{Frame, StaticImage};

const DEFAULT_FRAMES: [Frame; 1] = [Frame::new(
    &crate::assets::IMAGE_FOOD_BISCUIT,
    Duration::from_secs(1),
)];

#[derive(Copy, Clone)]
pub struct Anime {
    frames: &'static [Frame],
    elapsed: Duration,
    current_index: usize,
}

impl Default for Anime {
    fn default() -> Self {
        Self {
            frames: &DEFAULT_FRAMES,
            elapsed: Duration::ZERO,
            current_index: 0,
        }
    }
}

impl Anime {
    pub fn new(frames: &'static [Frame]) -> Self {
        Self {
            frames,
            ..Default::default()
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        self.elapsed += delta;
        if self.elapsed > self.frames[self.current_index].duration {
            if self.current_index + 1 >= self.frames.len() {
                self.current_index = 0;
            } else {
                self.current_index += 1;
            }
            self.elapsed = Duration::ZERO;
        }
    }

    pub fn set_frame(&mut self, index: usize) {
        self.current_index = index;
    }

    pub fn frames(&self) -> &'static [Frame] {
        self.frames
    }

    pub fn current_frame(&self) -> &'static StaticImage {
        &self.frames[self.current_index].frame
    }
}

pub trait HasAnime {
    fn anime(&mut self) -> &mut Anime;
}

pub fn tick_all_anime<T: HasAnime>(animes: &mut [Option<T>], delta: Duration) {
    for i in animes {
        if let Some(anime) = i {
            anime.anime().tick(delta);
        }
    }
}
