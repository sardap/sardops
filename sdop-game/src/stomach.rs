use glam::{IVec2, Vec2};

use crate::{
    assets::{IMAGE_STOMACH_MASK, Image},
    display::{ComplexRender, ComplexRenderOption},
    geo::RectIVec2,
};

#[derive(Default)]
pub struct StomachRender {
    pub pos_center: Vec2,
    pub filled: f32,
}

impl StomachRender {
    pub fn size() -> Vec2 {
        IMAGE_STOMACH_MASK.size_vec2()
    }
}

impl ComplexRender for StomachRender {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        use crate::assets::{IMAGE_STOMACH, IMAGE_STOMACH_MASK};

        let x = self.pos_center.x as i32;
        let y = self.pos_center.y as i32;

        let filled_rect = RectIVec2::new_top_left(
            IVec2::new(
                x - (IMAGE_STOMACH_MASK.isize.x / 2),
                y - IMAGE_STOMACH_MASK.isize.y
                    + (IMAGE_STOMACH_MASK.isize.y
                        - (IMAGE_STOMACH_MASK.isize.y as f32 * self.filled) as i32),
            ),
            IVec2::new(
                IMAGE_STOMACH_MASK.isize.x,
                (IMAGE_STOMACH_MASK.isize.y as f32 * self.filled) as i32,
            ),
        );
        display.render_rect_solid(&filled_rect, true);
        display.render_image_complex(
            x - (IMAGE_STOMACH_MASK.isize.x / 2),
            y - IMAGE_STOMACH_MASK.isize.y,
            &IMAGE_STOMACH,
            ComplexRenderOption::new().with_white(),
        );
        display.render_image_complex(
            x - (IMAGE_STOMACH_MASK.isize.x / 2),
            y - IMAGE_STOMACH_MASK.isize.y,
            &IMAGE_STOMACH_MASK,
            ComplexRenderOption::new().with_flip().with_black(),
        );
    }
}
