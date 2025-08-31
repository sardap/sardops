use glam::Vec2;

use crate::{
    assets::{self, Image},
    display::{ComplexRender, ComplexRenderOption, Rotation},
    furniture::HomeFurnitureLocation,
    math::norm_tau,
};

pub struct InvetroLightRender {
    pub pos: Vec2,
    pub length: i32,
    start: f32,
    end: f32,
    rotation: Rotation,
}

impl InvetroLightRender {
    pub const fn new(pos: Vec2, length: i32, location: HomeFurnitureLocation) -> Self {
        let width = core::f32::consts::FRAC_PI_2;

        let center = match location {
            HomeFurnitureLocation::Top => core::f32::consts::FRAC_PI_2,
            HomeFurnitureLocation::Left => core::f32::consts::PI,
            HomeFurnitureLocation::Right => 0.0,
        };

        let start = norm_tau(center - width / 2.0);
        let mut end = norm_tau(center + width / 2.0);

        if end < start {
            end += core::f32::consts::TAU;
        }

        Self {
            pos,
            length,
            start: start,
            end: end,
            rotation: match location {
                HomeFurnitureLocation::Top => Rotation::R0,
                HomeFurnitureLocation::Left => Rotation::R90,
                HomeFurnitureLocation::Right => Rotation::R270,
            },
        }
    }

    pub fn size() -> Vec2 {
        assets::IMAGE_INVETRO_LIGHT.size_vec2()
    }
}

impl ComplexRender for InvetroLightRender {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        display.invert_cone(
            self.pos + Vec2::new(0., 0.),
            self.length,
            self.start,
            self.end,
        );
        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            &assets::IMAGE_INVETRO_LIGHT,
            ComplexRenderOption::new()
                .with_white()
                .with_center()
                .with_rotation(self.rotation),
        );
        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            &assets::IMAGE_INVETRO_LIGHT_MASK,
            ComplexRenderOption::new()
                .with_black()
                .with_center()
                .with_rotation(self.rotation),
        );
    }
}
