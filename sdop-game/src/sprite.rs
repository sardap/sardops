use glam::Vec2;
use strum::IntoEnumIterator;

use crate::{
    anime::{Anime, HasAnime},
    assets::{self, Frame, Image, StaticImage},
    display::{PostionMode, Rotation, HEIGHT_F32, WIDTH_F32},
    geo::Rect,
};

pub trait Sprite {
    fn pos(&self) -> &Vec2;

    fn image(&self) -> &impl Image;

    fn rect(&self) -> Rect {
        Rect::new_center(*self.pos(), self.image().size_vec2())
    }

    fn x1(&self) -> f32 {
        self.pos().x - self.image().size().x as f32 / 2.
    }

    fn x2(&self) -> f32 {
        self.pos().x + self.image().size().x as f32 / 2.
    }

    fn y1(&self) -> f32 {
        self.pos().y - self.image().size().y as f32 / 2.
    }

    fn y2(&self) -> f32 {
        self.pos().y + self.image().size().y as f32 / 2.
    }

    #[allow(dead_code)]
    fn top_left(&self) -> Vec2 {
        Vec2::new(
            self.pos().x - (self.image().size().x / 2) as f32,
            self.pos().y - (self.image().size().y / 2) as f32,
        )
    }
}

pub trait SpriteMask {
    fn image_mask(&self) -> &impl Image;
}

pub trait SpritePostionMode {
    fn sprite_postion_mode(&self) -> PostionMode;
}

pub trait SpriteRotation {
    fn sprite_rotation(&self) -> Rotation;
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
        Self::new(Default::default(), &assets::IMAGE_ALPHABET_SPACE)
    }
}

#[derive(Copy, Clone)]
pub struct BasicMaskedSprite {
    pub pos: Vec2,
    pub image: &'static StaticImage,
    pub mask: &'static StaticImage,
}

impl BasicMaskedSprite {
    pub fn new(pos: Vec2, image: &'static StaticImage, mask: &'static StaticImage) -> Self {
        Self { pos, image, mask }
    }
}

impl Sprite for BasicMaskedSprite {
    fn pos(&self) -> &Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl Image {
        self.image
    }
}

impl SpriteMask for BasicMaskedSprite {
    fn image_mask(&self) -> &impl Image {
        self.mask
    }
}

impl Default for BasicMaskedSprite {
    fn default() -> Self {
        Self::new(
            Default::default(),
            &assets::IMAGE_ALPHABET_SPACE,
            &assets::IMAGE_ALPHABET_SPACE,
        )
    }
}

#[derive(Copy, Clone)]
pub struct BasicAnimeSprite {
    pub pos: Vec2,
    pub anime: Anime,
    pub pos_mode: PostionMode,
}

impl BasicAnimeSprite {
    pub fn new(pos: Vec2, frames: &'static [Frame]) -> Self {
        Self {
            pos,
            anime: Anime::new(frames),
            pos_mode: PostionMode::Center,
        }
    }

    pub fn with_pos_mode(mut self, pos_mode: PostionMode) -> Self {
        self.pos_mode = pos_mode;
        self
    }
}

impl Default for BasicAnimeSprite {
    fn default() -> Self {
        Self {
            pos: Vec2::default(),
            anime: Default::default(),
            pos_mode: PostionMode::Center,
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

impl SpritePostionMode for BasicAnimeSprite {
    fn sprite_postion_mode(&self) -> PostionMode {
        self.pos_mode
    }
}

#[derive(Copy, Clone)]
pub struct Snowflake {
    pub pos: Vec2,
    pub rotation: Rotation,
    pub dir: Vec2,
}

impl Snowflake {
    pub fn reset(&mut self, inital: bool, rng: &mut fastrand::Rng) {
        self.pos = Vec2::new(
            rng.i32(0..WIDTH_F32 as i32) as f32,
            rng.i32(if inital {
                (-HEIGHT_F32 as i32 + -20)..-5
            } else {
                -20..-5
            }) as f32,
        );
        self.rotation = rng.choice(Rotation::iter()).unwrap();
        self.dir = Vec2::new(rng.i32(-3..3) as f32, rng.i32(3..10) as f32);
    }

    pub const fn size() -> Vec2 {
        Vec2::new(
            assets::IMAGE_SNOWFLAKE.size.x as f32,
            assets::IMAGE_SNOWFLAKE.size.y as f32,
        )
    }
}

impl Sprite for Snowflake {
    fn pos(&self) -> &Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl Image {
        &assets::IMAGE_SNOWFLAKE
    }
}

impl SpriteRotation for Snowflake {
    fn sprite_rotation(&self) -> Rotation {
        self.rotation
    }
}

impl Default for Snowflake {
    fn default() -> Self {
        Self {
            pos: Vec2::new(-100., 0.),
            rotation: Default::default(),
            dir: Default::default(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct MusicNote {
    pub image: &'static StaticImage,
    pub pos: Vec2,
    pub dir: Vec2,
}

impl MusicNote {
    pub fn reset(&mut self, center: Vec2, rng: &mut fastrand::Rng) {
        const NOTES: &[&StaticImage] = &[
            &assets::IMAGE_MUSIC_NOTE_BEAM_NOTE,
            &assets::IMAGE_MUSIC_NOTE_CROTCHET,
            &assets::IMAGE_MUSIC_NOTE_QUAVER,
            &assets::IMAGE_MUSIC_NOTE_SEMI_QUAVER,
        ];

        let x_min = center.x as i32 - 5;
        let x_max = center.x as i32 + 5;

        self.pos = Vec2::new(rng.i32(x_min..x_max) as f32, center.y);
        self.image = rng.choice(NOTES.iter()).unwrap();
        let mut y_speed = rng.i32(10..25);
        if rng.bool() {
            y_speed = -y_speed;
        }
        let mut x_speed = rng.i32(10..25);
        if rng.bool() {
            x_speed = -x_speed;
        }
        self.dir = Vec2::new(x_speed as f32, y_speed as f32);
    }

    pub fn size(&self) -> Vec2 {
        self.image.size.as_vec2()
    }
}

impl Sprite for MusicNote {
    fn pos(&self) -> &Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl Image {
        self.image
    }
}

impl Default for MusicNote {
    fn default() -> Self {
        Self {
            pos: Vec2::new(-100., 0.),
            image: &assets::IMAGE_MUSIC_NOTE_BEAM_NOTE,
            dir: Default::default(),
        }
    }
}
