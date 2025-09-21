use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use asefile::AsepriteFile;
use image::{GenericImage, RgbImage, RgbaImage, imageops};

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

fn place_plaents(rng: &mut fastrand::Rng, image: &mut RgbaImage, planet: &RgbaImage, count: usize) {
    for _ in 0..count {
        let x = rng.u32(0..WIDTH) as i64;
        let y = rng.u32(0..HEIGHT) as i64;
        imageops::overlay(image, planet, x, y);
    }
}

fn main() {
    let mut rng = fastrand::Rng::with_seed(0);

    let star_large = load_planet(&PathBuf::from_str("stars/star_large.aseprite").unwrap());
    let star_med = load_planet(&PathBuf::from_str("stars/star_med.aseprite").unwrap());
    let star_small = load_planet(&PathBuf::from_str("stars/star_small.aseprite").unwrap());
    let star_tiny = load_planet(&PathBuf::from_str("stars/star_tiny.aseprite").unwrap());

    let mut image: RgbaImage = RgbaImage::new(WIDTH, HEIGHT);
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            *image.get_pixel_mut(x, y) = image::Rgba([0, 0, 0, 255]);
        }
    }

    place_plaents(&mut rng, &mut image, &star_large, 150);
    place_plaents(&mut rng, &mut image, &star_med, 250);
    place_plaents(&mut rng, &mut image, &star_small, 450);
    place_plaents(&mut rng, &mut image, &star_tiny, 850);

    image.save("output.png").unwrap();
}
