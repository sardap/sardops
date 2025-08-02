use glam::Vec2;

use crate::{
    anime::{Anime, HasAnime},
    assets::{self, Frame, Image, StaticImage},
    geo::Rect,
};

pub trait Sprite {
    fn pos(&self) -> &Vec2;

    fn image(&self) -> &impl Image;

    fn rect(&self) -> Rect {
        Rect::new_center(*self.pos(), self.image().size_vec2())
    }

    #[allow(dead_code)]
    fn top_left(&self) -> Vec2 {
        Vec2::new(
            self.pos().x - (self.image().size().x / 2) as f32,
            self.pos().y - (self.image().size().y / 2) as f32,
        )
    }
}

#[derive(Copy, Clone)]
pub struct BasicSprite {
    pub pos: Vec2,
    pub image: &'static StaticImage,
}

impl BasicSprite {
    pub fn new(pos: Vec2, image: &'static StaticImage) -> Self {
        Self { pos, image }
    }
}

impl Sprite for BasicSprite {
    fn pos(&self) -> &Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl Image {
        self.image
    }
}

impl Default for BasicSprite {
    fn default() -> Self {
        Self {
            pos: Vec2::default(),
            image: &assets::IMAGE_ALPHABET_SPACE,
        }
    }
}

#[derive(Copy, Clone)]
pub struct BasicAnimeSprite {
    pub pos: Vec2,
    pub anime: Anime,
}

impl BasicAnimeSprite {
    pub fn new(pos: Vec2, frames: &'static [Frame]) -> Self {
        Self {
            pos,
            anime: Anime::new(frames),
        }
    }
}

impl Default for BasicAnimeSprite {
    fn default() -> Self {
        Self {
            pos: Vec2::default(),
            anime: Default::default(),
        }
    }
}

impl HasAnime for BasicAnimeSprite {
    fn anime(&mut self) -> &mut Anime {
        &mut self.anime
    }
}

impl Sprite for BasicAnimeSprite {
    fn pos(&self) -> &Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl Image {
        self.anime.current_frame()
    }
}
