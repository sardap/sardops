use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use asefile::AsepriteFile;
use image::{RgbaImage, imageops};

const WIDTH: u32 = 365 * 10;
const HEIGHT: u32 = 128;

fn load_planet(path: &Path) -> RgbaImage {
    let ase = AsepriteFile::read_file(&path).unwrap();
    let rgba_image = ase.frame(0).image();
    RgbaImage::from_raw(
        rgba_image.width(),
        rgba_image.height(),
        rgba_image.into_raw(),
    )
    .unwrap()
}

fn place_planets(rng: &mut fastrand::Rng, image: &mut RgbaImage, planet: &RgbaImage, count: usize) {
    for _ in 0..count {
        let x = rng.u32(0..WIDTH) as i64;
        let y = rng.u32(0..HEIGHT) as i64;
        imageops::overlay(image, planet, x, y);
    }
}

fn main() {
    let star_large = load_planet(&PathBuf::from_str("stars/star_large.aseprite").unwrap());
    let star_med = load_planet(&PathBuf::from_str("stars/star_med.aseprite").unwrap());
    let star_small = load_planet(&PathBuf::from_str("stars/star_small.aseprite").unwrap());
    let star_tiny = load_planet(&PathBuf::from_str("stars/star_tiny.aseprite").unwrap());

    const LAYERS: usize = 3;
    const LARGE_STARS: usize = 15;
    const MED_STARS: usize = 250;
    const SMALL_STAR: usize = 450;
    const TINY_STAR: usize = 850;

    for i in 0..LAYERS {
        let mut rng = fastrand::Rng::with_seed(i as u64);

        let mut image: RgbaImage = RgbaImage::new(WIDTH, HEIGHT);
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                *image.get_pixel_mut(x, y) = image::Rgba([0, 0, 0, 255]);
            }
        }

        place_planets(&mut rng, &mut image, &star_large, LARGE_STARS / LAYERS);
        place_planets(&mut rng, &mut image, &star_med, MED_STARS / LAYERS);
        place_planets(&mut rng, &mut image, &star_small, SMALL_STAR / LAYERS);
        place_planets(&mut rng, &mut image, &star_tiny, TINY_STAR / LAYERS);

        image.save(format!("output_{}.png", i)).unwrap();
    }
}
