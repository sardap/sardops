use glam::Vec2;

use crate::{
    assets::{self, Image},
    display::{ComplexRender, ComplexRenderOption},
    temperature::TemperatureLevel,
};

pub struct RenderThermometerMercury {
    pub pos: Vec2,
    pub temperature: f32,
}

impl RenderThermometerMercury {
    pub fn new(pos: Vec2, temperature: f32) -> Self {
        Self { pos, temperature }
    }

    pub fn size() -> Vec2 {
        assets::IMAGE_THERMOMETER_MERCURY.size_vec2()
    }
}

impl Default for RenderThermometerMercury {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            temperature: Default::default(),
        }
    }
}

impl ComplexRender for RenderThermometerMercury {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            &assets::IMAGE_THERMOMETER_MERCURY,
            ComplexRenderOption::new().with_white().with_center(),
        );

        let level = TemperatureLevel::from(self.temperature);

        let temp_y = match level {
            TemperatureLevel::VeryHot => 13.,
            TemperatureLevel::Hot => 10.,
            TemperatureLevel::Pleasant => 7.,
            TemperatureLevel::Cold => 4.,
            TemperatureLevel::VeryCold => 1.,
        };

        let start = Vec2::new(self.pos.x, self.pos.y + 5.);
        let end = Vec2::new(self.pos.x, self.pos.y + 5. - temp_y);

        display.render_line(start, end, true);
    }
}
