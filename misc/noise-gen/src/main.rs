use image::{imageops, RgbaImage};

const WIDTH: u32 = 30;
const HEIGHT: u32 = 22;

fn main() {
    let mut rng = fastrand::Rng::with_seed(19951220);

    for i in 5..=20 {
        let mut image: RgbaImage = RgbaImage::new(WIDTH, HEIGHT);

        for _ in 0..(WIDTH * HEIGHT * 100) {
            let x = rng.u32(0..WIDTH);
            let y = rng.u32(0..HEIGHT);
            *image.get_pixel_mut(x, y) = if rng.i32(0..3) == 0 {
                image::Rgba([0xFF, 0xFF, 0xFF, 0xFF])
            } else {
                image::Rgba([0, 0, 0, 0xFF])
            };
        }

        image.save(format!("output_{}.png", i)).unwrap();
    }
}
