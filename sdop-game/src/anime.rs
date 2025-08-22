use core::time::Duration;

use glam::Vec2;

use crate::{
    assets::{Frame, StaticImage},
    display::{ComplexRender, ComplexRenderOption, PostionMode},
};

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

    pub fn set_random_frame(&mut self, rng: &mut fastrand::Rng) {
        self.current_index = rng.usize(0..self.frames.len());
        self.elapsed = Duration::from_millis(
            rng.u64(0..self.frames[self.current_index].duration.as_millis() as u64) as u64,
        )
    }

    pub fn frames(&self) -> &'static [Frame] {
        self.frames
    }

    pub fn current_frame(&self) -> &'static StaticImage {
        &self.frames[self.current_index].frame
    }

    pub fn current_frame_index(&self) -> usize {
        self.current_index
    }

    pub fn total_duration(&self) -> Duration {
        self.frames.iter().map(|i| i.duration).sum()
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

#[derive(Copy, Clone)]
pub struct MaskedAnimeRender {
    pub pos: Vec2,
    pub anime: Anime,
    pub masked: &'static [Frame],
    pub pos_mode: PostionMode,
}

impl MaskedAnimeRender {
    pub fn new(pos: Vec2, frames: &'static [Frame], masked_frames: &'static [Frame]) -> Self {
        Self {
            pos,
            anime: Anime::new(frames),
            masked: masked_frames,
            pos_mode: PostionMode::Center,
        }
    }
}

impl Default for MaskedAnimeRender {
    fn default() -> Self {
        Self {
            pos: Vec2::default(),
            anime: Default::default(),
            masked: Default::default(),
            pos_mode: PostionMode::Center,
        }
    }
}

impl HasAnime for MaskedAnimeRender {
    fn anime(&mut self) -> &mut Anime {
        &mut self.anime
    }
}

impl ComplexRender for MaskedAnimeRender {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            self.anime.current_frame(),
            ComplexRenderOption::new()
                .with_pos_mode(self.pos_mode)
                .with_white(),
        );
        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            self.masked[self.anime.current_frame_index()].frame,
            ComplexRenderOption::new()
                .with_pos_mode(self.pos_mode)
                .with_black(),
        );
    }
}
