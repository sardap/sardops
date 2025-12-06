use glam::U16Vec2;

use crate::{
    assets::{DynamicImage, IMAGE_NIGHT_SKY_0, IMAGE_NIGHT_SKY_1, IMAGE_NIGHT_SKY_2},
    WIDTH,
};

pub fn generate_night_sky_image<const HEIGHT: usize>(
    night_sky: &mut DynamicImage<{ HEIGHT * WIDTH / 8 }>,
    days_since_ce: i32,
) {
    night_sky.size = U16Vec2::new(WIDTH as u16, HEIGHT as u16);
    night_sky.used_length = HEIGHT * WIDTH / 8;

    let days_since_ce = days_since_ce as f32;
    for (i, base_sky) in [
        (10.0, IMAGE_NIGHT_SKY_0),
        (3.0, IMAGE_NIGHT_SKY_1),
        (0.25, IMAGE_NIGHT_SKY_2),
    ] {
        let x_offset = libm::floorf(days_since_ce * i) as usize % 365;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let src_x = (x + x_offset) % base_sky.size.x as usize;
                let base_index = y * base_sky.size.x as usize + src_x;
                let dst_index = y * WIDTH + x;

                let bit_value = (base_sky.texture[base_index / 8] >> (base_index % 8)) & 1;
                let flag = bit_value << (dst_index % 8);
                night_sky.texture[dst_index / 8] |= flag;
            }
        }
    }
}
