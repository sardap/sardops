use core::time::Duration;

use glam::{usize, U8Vec2, Vec2};

include!(concat!(env!("OUT_DIR"), "/dist_assets.rs"));

pub trait Image {
    fn texture<'a>(&'a self) -> &'a [u8];

    fn size<'a>(&'a self) -> &'a U8Vec2;

    fn size_vec2(&self) -> Vec2 {
        let size = self.size();
        Vec2::new(size.x as f32, size.y as f32)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    pub frame: &'static StaticImage,
    pub duration: Duration,
}

impl Frame {
    pub const fn new(frame: &'static StaticImage, duration: Duration) -> Self {
        Self { frame, duration }
    }
}

#[derive(Clone, Copy)]
pub struct MaskedFramesSet {
    pub frame: &'static [Frame],
    pub masked: &'static [Frame],
}

impl MaskedFramesSet {
    pub const fn new(frame: &'static [Frame], masked: &'static [Frame]) -> Self {
        Self { frame, masked }
    }
}
