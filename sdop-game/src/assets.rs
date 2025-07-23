use core::time::Duration;

use glam::{U8Vec2, Vec2, usize};

include!(concat!(env!("OUT_DIR"), "/dist_assets.rs"));

pub trait Image {
    fn texture<'a>(&'a self) -> &'a [u8];

    fn size<'a>(&'a self) -> &'a U8Vec2;

    fn size_vec2(&self) -> Vec2 {
        let size = self.size();
        Vec2::new(size.x as f32, size.y as f32)
    }
}

#[derive(Clone, Copy)]
pub struct StaticImage {
    pub size: U8Vec2,
    pub texture: &'static [u8],
}

impl Image for StaticImage {
    fn texture<'a>(&'a self) -> &'a [u8] {
        &self.texture
    }

    fn size<'a>(&'a self) -> &'a U8Vec2 {
        &self.size
    }
}

impl StaticImage {
    pub const fn new(width: u8, height: u8, texture: &'static [u8]) -> Self {
        Self {
            size: U8Vec2::new(width, height),
            texture: texture,
        }
    }
}

pub struct DynamicImage<const T: usize> {
    pub size: U8Vec2,
    pub used_length: usize,
    pub texture: [u8; T],
}

impl<const T: usize> DynamicImage<T> {
    pub fn duplcaite<I: Image>(&mut self, other: &I) {
        self.size = *other.size();
        let target_texture = other.texture();
        self.used_length = (target_texture.len()).min(T);
        self.texture[..self.used_length].copy_from_slice(&target_texture[..self.used_length]);
    }
}

impl<const T: usize> Default for DynamicImage<T> {
    fn default() -> Self {
        Self {
            size: U8Vec2::default(),
            used_length: 0,
            texture: [0; T],
        }
    }
}

impl<const T: usize> Image for DynamicImage<T> {
    fn texture<'a>(&'a self) -> &'a [u8] {
        &self.texture[0..self.used_length]
    }

    fn size<'a>(&'a self) -> &'a U8Vec2 {
        &self.size
    }
}

pub struct Frame {
    pub frame: &'static StaticImage,
    pub duration: Duration,
}

impl Frame {
    pub const fn new(frame: &'static StaticImage, duration: Duration) -> Self {
        Self { frame, duration }
    }
}

pub fn get_char_image(val: char) -> &'static StaticImage {
    match val.to_ascii_uppercase() {
        ' ' => &IMAGE_ALPHABET_SPACE,
        '0' => &IMAGE_ALPHABET_0,
        '1' => &IMAGE_ALPHABET_1,
        '2' => &IMAGE_ALPHABET_2,
        '3' => &IMAGE_ALPHABET_3,
        '4' => &IMAGE_ALPHABET_4,
        '5' => &IMAGE_ALPHABET_5,
        '6' => &IMAGE_ALPHABET_6,
        '7' => &IMAGE_ALPHABET_7,
        '8' => &IMAGE_ALPHABET_8,
        '9' => &IMAGE_ALPHABET_9,
        'A' => &IMAGE_ALPHABET_A,
        'B' => &IMAGE_ALPHABET_B,
        'C' => &IMAGE_ALPHABET_C,
        'D' => &IMAGE_ALPHABET_D,
        'E' => &IMAGE_ALPHABET_E,
        'F' => &IMAGE_ALPHABET_F,
        'G' => &IMAGE_ALPHABET_G,
        'H' => &IMAGE_ALPHABET_H,
        'I' => &IMAGE_ALPHABET_I,
        'J' => &IMAGE_ALPHABET_J,
        'K' => &IMAGE_ALPHABET_K,
        'L' => &IMAGE_ALPHABET_L,
        'M' => &IMAGE_ALPHABET_M,
        'N' => &IMAGE_ALPHABET_N,
        'O' => &IMAGE_ALPHABET_O,
        'P' => &IMAGE_ALPHABET_P,
        'Q' => &IMAGE_ALPHABET_Q,
        'R' => &IMAGE_ALPHABET_R,
        'S' => &IMAGE_ALPHABET_S,
        'T' => &IMAGE_ALPHABET_T,
        'U' => &IMAGE_ALPHABET_U,
        'V' => &IMAGE_ALPHABET_V,
        'W' => &IMAGE_ALPHABET_W,
        'X' => &IMAGE_ALPHABET_X,
        'Y' => &IMAGE_ALPHABET_Y,
        'Z' => &IMAGE_ALPHABET_Z,
        '!' => &IMAGE_ALPHABET_EXCLAMATION_MARK,
        '"' => &IMAGE_ALPHABET_QUOTE,
        '#' => &IMAGE_ALPHABET_HASHTAG,
        '$' => &IMAGE_ALPHABET_DOP_BUCKS,
        '%' => &IMAGE_ALPHABET_PERCENT,
        '+' => &IMAGE_ALPHABET_PLUS,
        '/' => &IMAGE_ALPHABET_DIVIDE,
        ':' => &IMAGE_ALPHABET_COLON,
        '?' => &IMAGE_ALPHABET_QUESTION_MARK,
        '.' => &IMAGE_ALPHABET_DOT,
        _ => &IMAGE_ALPHABET_0,
    }
}
