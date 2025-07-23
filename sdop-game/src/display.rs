use glam::Vec2;

use crate::{
    assets::{Image, get_char_image},
    bit_array::{BitArray, bytes_for_bits},
    geo::Rect,
    sprite::Sprite,
};

pub const WIDTH: usize = 64;
pub const WIDTH_F32: f32 = WIDTH as f32;
pub const HEIGHT: usize = 128;
pub const HEIGHT_F32: f32 = HEIGHT as f32;
pub const CENTER_X: f32 = WIDTH_F32 / 2.;
pub const CENTER_Y: f32 = HEIGHT_F32 / 2.;
pub const CENTER_VEC: Vec2 = Vec2::new(CENTER_X, CENTER_Y);
pub const DISPLAY_SIZE: usize = WIDTH as usize * HEIGHT as usize;

pub type DisplayArray = BitArray<DISPLAY_BYTES>;

const DISPLAY_BYTES: usize = bytes_for_bits(DISPLAY_SIZE);

pub struct GameDisplay {
    bits: DisplayArray,
}

impl Default for GameDisplay {
    fn default() -> Self {
        Self {
            bits: Default::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ComplexRenderOption {
    write_black: bool,
    write_white: bool,
    flip_colors: bool,
    pos_center: bool,
}

impl ComplexRenderOption {
    pub fn with_white(mut self) -> Self {
        self.write_white = true;
        self
    }

    pub fn with_black(mut self) -> Self {
        self.write_black = true;
        self
    }

    pub fn with_flip(mut self) -> Self {
        self.flip_colors = true;
        self
    }

    pub fn with_center(mut self) -> Self {
        self.pos_center = true;
        self
    }
}

impl Default for ComplexRenderOption {
    fn default() -> Self {
        Self {
            write_black: false,
            write_white: false,
            flip_colors: false,
            pos_center: false,
        }
    }
}

impl GameDisplay {
    pub fn array(&self) -> &DisplayArray {
        &self.bits
    }

    pub fn clear(&mut self) {
        self.bits.clear();
    }

    pub fn set_bit(&mut self, x: i32, y: i32, value: bool) {
        if !(x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32) {
            return;
        }
        let x = x as usize;
        let y = y as usize;
        let bit_index = y * WIDTH + x;
        self.bits.set_bit(bit_index, value);
    }

    pub fn render_image_complex<T: Image>(
        &mut self,
        x: i32,
        y: i32,
        image: &T,
        options: ComplexRenderOption,
    ) {
        if !options.write_black && !options.write_white {
            return;
        }

        let image_size = image.size();
        let texture = image.texture();

        let (x_plus, y_plus) = if options.pos_center {
            (x - (image_size.x as i32) / 2, y - (image_size.y as i32) / 2)
        } else {
            (x, y)
        };

        for iy in 0..image_size.y {
            for ix in 0..image_size.x {
                let pixel_index = (iy as usize) * (image_size.x as usize) + (ix as usize);
                let byte_index = pixel_index / 8;
                let bit_index = pixel_index % 8;
                let mut bit_set = (texture[byte_index] >> bit_index) & 1 != 0;
                if options.flip_colors {
                    bit_set = !bit_set;
                }

                if (options.write_white && !bit_set) || (options.write_black && bit_set) {
                    continue;
                }

                let dx = x_plus + ix as i32;
                let dy = y_plus + iy as i32;

                if dx >= 0 && dx < WIDTH as i32 && dy >= 0 && dy < HEIGHT as i32 {
                    self.set_bit(dx, dy, bit_set);
                }
            }
        }
    }

    pub fn render_image_center<T: Image>(&mut self, x: i32, y: i32, image: &T) {
        self.render_image_complex(
            x,
            y,
            image,
            ComplexRenderOption::default().with_white().with_center(),
        )
    }

    pub fn render_image_top_left<T: Image>(&mut self, x: i32, y: i32, image: &T) {
        self.render_image_complex(
            x,
            y,
            image,
            ComplexRenderOption {
                write_black: false,
                write_white: true,
                flip_colors: false,
                pos_center: false,
            },
        );
    }

    pub fn render_rect_solid(&mut self, rect: Rect, white: bool) {
        let top_left = rect.pos_top_left();
        for x in top_left.x as i32..(top_left.x + rect.size.x) as i32 {
            for y in top_left.y as i32..(top_left.y + rect.size.y) as i32 {
                self.set_bit(x, y, white);
            }
        }
    }

    pub fn render_rect_outline(&mut self, rect: Rect, white: bool) {
        let top_left = rect.pos_top_left();
        let bottom_right_x = top_left.x + rect.size.x - 1.;
        let bottom_right_y = top_left.y + rect.size.y - 1.;

        // Top and bottom borders
        for x in top_left.x as i32..=bottom_right_x as i32 {
            self.set_bit(x, top_left.y as i32, white); // Top
            self.set_bit(x, bottom_right_y as i32, white); // Bottom
        }

        // Left and right borders (excluding corners, already set above)
        for y in (top_left.y as i32 + 1)..(bottom_right_y as i32) {
            self.set_bit(top_left.x as i32, y, white); // Left
            self.set_bit(bottom_right_x as i32, y, white); // Right
        }
    }

    pub fn render_rect_outline_dashed(&mut self, rect: Rect, white: bool, dash_width: usize) {
        let top_left = rect.pos_top_left();
        let bottom_right_x = top_left.x + rect.size.x - 1.;
        let bottom_right_y = top_left.y + rect.size.y - 1.;

        let dash_width = dash_width.max(1); // avoid division by zero

        // Top and bottom borders
        for (i, x) in (top_left.x as i32..=bottom_right_x as i32).enumerate() {
            if (i / dash_width) % 2 == 0 {
                self.set_bit(x, top_left.y as i32, white); // Top
                self.set_bit(x, bottom_right_y as i32, white); // Bottom
            }
        }

        // Left and right borders
        for (i, y) in ((top_left.y as i32 + 1)..(bottom_right_y as i32)).enumerate() {
            if (i / dash_width) % 2 == 0 {
                self.set_bit(top_left.x as i32, y, white); // Left
                self.set_bit(bottom_right_x as i32, y, white); // Right
            }
        }
    }

    #[allow(dead_code)]
    pub fn render_line(&mut self, start: Vec2, end: Vec2, white: bool) {
        let (mut x0, mut y0) = (start.x as i32, start.y as i32);
        let (x1, y1) = (end.x as i32, end.y as i32);

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.set_bit(x0, y0, white);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    pub fn render_circle(&mut self, center: Vec2, radius: i32, white: bool) {
        if radius == 0 {
            return;
        }

        let cx = center.x as i32;
        let cy = center.y as i32;

        let mut x = radius;
        let mut y = 0;
        let mut err = 0;

        while x >= y {
            self.set_bit(cx + x, cy + y, white);
            self.set_bit(cx + y, cy + x, white);
            self.set_bit(cx - y, cy + x, white);
            self.set_bit(cx - x, cy + y, white);
            self.set_bit(cx - x, cy - y, white);
            self.set_bit(cx - y, cy - x, white);
            self.set_bit(cx + y, cy - x, white);
            self.set_bit(cx + x, cy - y, white);

            y += 1;
            if err <= 0 {
                err += 2 * y + 1;
            }
            if err > 0 {
                x -= 1;
                err -= 2 * x + 1;
            }
        }
    }

    pub fn render_sprite<T: Sprite>(&mut self, sprite: &T) {
        self.render_image_center(sprite.pos().x as i32, sprite.pos().y as i32, sprite.image());
    }

    pub fn render_sprites<T: Sprite>(&mut self, sprites: &[Option<T>]) {
        for sprite in sprites {
            if let Some(sprite) = sprite {
                self.render_sprite(sprite);
            }
        }
    }

    pub fn render_text(&mut self, top_left: Vec2, text: &str) {
        let mut x_offset = 0;
        for ch in text.chars() {
            let image = get_char_image(ch);
            self.render_image_top_left(top_left.x as i32 + x_offset, top_left.y as i32, image);
            x_offset += image.size.x as i32;
        }
    }

    pub fn render_text_complex(&mut self, pos: Vec2, text: &str, options: ComplexRenderOption) {
        let (x_start, y_start) = if options.pos_center {
            let width = (text.len() * 8) as f32;
            (pos.x - width / 2., pos.y - 4.)
        } else {
            (pos.x, pos.y)
        };

        let sub_complex_options = ComplexRenderOption {
            write_black: options.write_black,
            write_white: options.write_white,
            flip_colors: options.flip_colors,
            pos_center: false,
        };
        let mut x_offset = 0;
        for ch in text.chars() {
            let image = get_char_image(ch);
            self.render_image_complex(
                x_start as i32 + x_offset,
                y_start as i32,
                image,
                sub_complex_options,
            );
            x_offset += image.size.x as i32;
        }
    }

    pub fn render_stomach(&mut self, pos_center: Vec2, filled: f32) {
        use crate::assets::{IMAGE_STOMACH, IMAGE_STOMACH_MASK};

        let filled_rect = Rect::new_top_left(
            Vec2::new(
                pos_center.x - (IMAGE_STOMACH_MASK.size.x / 2) as f32,
                pos_center.y - IMAGE_STOMACH_MASK.size.y as f32
                    + (IMAGE_STOMACH_MASK.size.y as f32
                        - (IMAGE_STOMACH_MASK.size.y as f32 * filled)),
            ),
            Vec2::new(
                IMAGE_STOMACH_MASK.size.x as f32,
                IMAGE_STOMACH_MASK.size.y as f32 * filled,
            ),
        );
        self.render_rect_solid(filled_rect, true);
        self.render_image_complex(
            pos_center.x as i32 - (IMAGE_STOMACH_MASK.size.x / 2) as i32,
            pos_center.y as i32 - IMAGE_STOMACH_MASK.size.y as i32,
            &IMAGE_STOMACH,
            ComplexRenderOption::default().with_white(),
        );
        self.render_image_complex(
            pos_center.x as i32 - (IMAGE_STOMACH_MASK.size.x / 2) as i32,
            pos_center.y as i32 - IMAGE_STOMACH_MASK.size.y as i32,
            &IMAGE_STOMACH_MASK,
            ComplexRenderOption::default().with_flip().with_black(),
        );
    }

    pub fn invert(&mut self) {
        for i in self.bits.raw_mut().iter_mut() {
            *i = !*i;
        }
    }
}
