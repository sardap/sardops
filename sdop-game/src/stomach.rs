use glam::Vec2;

use crate::{
    assets::{Image, IMAGE_STOMACH_MASK},
    display::{ComplexRender, ComplexRenderOption},
    geo::Rect,
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

        let filled_rect = Rect::new_top_left(
            Vec2::new(
                self.pos_center.x - (IMAGE_STOMACH_MASK.size.x / 2) as f32,
                self.pos_center.y - IMAGE_STOMACH_MASK.size.y as f32
                    + (IMAGE_STOMACH_MASK.size.y as f32
                        - (IMAGE_STOMACH_MASK.size.y as f32 * self.filled)),
            ),
            Vec2::new(
                IMAGE_STOMACH_MASK.size.x as f32,
                IMAGE_STOMACH_MASK.size.y as f32 * self.filled,
            ),
        );
        display.render_rect_solid(filled_rect, true);
        display.render_image_complex(
            self.pos_center.x as i32 - (IMAGE_STOMACH_MASK.size.x / 2) as i32,
            self.pos_center.y as i32 - IMAGE_STOMACH_MASK.size.y as i32,
            &IMAGE_STOMACH,
            ComplexRenderOption::new().with_white(),
        );
        display.render_image_complex(
            self.pos_center.x as i32 - (IMAGE_STOMACH_MASK.size.x / 2) as i32,
            self.pos_center.y as i32 - IMAGE_STOMACH_MASK.size.y as i32,
            &IMAGE_STOMACH_MASK,
            ComplexRenderOption::new().with_flip().with_black(),
        );
    }
}
