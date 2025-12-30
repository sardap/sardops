use core::time::Duration;

use glam::Vec2;

use crate::{
    assets::{Frame, StaticImage},
    display::PostionMode,
    sprite::{Sprite, SpriteMask, SpritePostionMode},
};

const DEFAULT_FRAMES: [Frame; 1] = [Frame::new(
    &crate::assets::IMAGE_FOOD_BISCUIT,
    Duration::from_secs(1),
)];

#[derive(Copy, Clone)]
pub struct Anime {
    frames: &'static [Frame],
    masked_frames: Option<&'static [Frame]>,
    elapsed: Duration,
    current_index: usize,
}

impl Default for Anime {
    fn default() -> Self {
        Self {
            frames: &DEFAULT_FRAMES,
            masked_frames: None,
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

    pub fn with_mask(mut self, frames: &'static [Frame]) -> Self {
        self.masked_frames = Some(frames);
        self
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
            rng.u64(0..self.frames[self.current_index].duration.as_millis() as u64),
        )
    }

    pub fn frames(&self) -> &'static [Frame] {
        self.frames
    }

    pub const fn current_frame(&self) -> &'static StaticImage {
        self.frames[self.current_index].frame
    }

    pub fn last_frame(&self) -> &'static StaticImage {
        self.frames[self.frames.len() - 1].frame
    }

    pub fn current_frame_mask(&self) -> Option<&'static StaticImage> {
        if let Some(mask) = &self.masked_frames {
            return Some(mask[self.current_index].frame);
        }

        None
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
    for anime in animes.iter_mut().flatten() {
        anime.anime().tick(delta);
    }
}

#[derive(Copy, Clone)]
pub struct MaskedAnimeSprite {
    pub pos: Vec2,
    pub anime: Anime,
    pub pos_mode: PostionMode,
}

impl MaskedAnimeSprite {
    pub fn new(pos: Vec2, frames: &'static [Frame], masked_frames: &'static [Frame]) -> Self {
        Self {
            pos,
            anime: Anime::new(frames).with_mask(masked_frames),
            pos_mode: PostionMode::Center,
        }
    }
}

impl Default for MaskedAnimeSprite {
    fn default() -> Self {
        Self {
            pos: Vec2::default(),
            anime: Default::default(),
            pos_mode: PostionMode::Center,
        }
    }
}

impl HasAnime for MaskedAnimeSprite {
    fn anime(&mut self) -> &mut Anime {
        &mut self.anime
    }
}

impl Sprite for MaskedAnimeSprite {
    fn pos(&self) -> &Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl crate::assets::Image {
        self.anime.current_frame()
    }

    fn size_x(&self) -> i32 {
        self.anime.current_frame().isize.x
    }

    fn size_y(&self) -> i32 {
        self.anime.current_frame().isize.y
    }
}

impl SpriteMask for MaskedAnimeSprite {
    fn image_mask(&self) -> &impl crate::assets::Image {
        self.anime.current_frame_mask().unwrap()
    }
}

impl SpritePostionMode for MaskedAnimeSprite {
    fn sprite_postion_mode(&self) -> PostionMode {
        self.pos_mode
    }
}
