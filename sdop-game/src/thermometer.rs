use glam::Vec2;

use crate::{
    assets::{self, Image},
    display::{ComplexRender, ComplexRenderOption},
    fonts::FONT_VARIABLE_SMALL,
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

pub struct RenderThermometerDigital {
    pub pos: Vec2,
    pub temperature: f32,
}

impl RenderThermometerDigital {
    pub fn new(pos: Vec2, temperature: f32) -> Self {
        Self { pos, temperature }
    }

    pub fn size() -> Vec2 {
        assets::IMAGE_THERMOMETER_DIGITAL.size_vec2()
    }
}

impl Default for RenderThermometerDigital {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            temperature: Default::default(),
        }
    }
}

impl ComplexRender for RenderThermometerDigital {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            &assets::IMAGE_THERMOMETER_DIGITAL,
            ComplexRenderOption::new().with_white().with_center(),
        );

        let temperature = libm::roundf(self.temperature) as i32;
        let abs_temp = temperature.abs();

        let first = if abs_temp > 0 {
            let pow = 10_i32.pow(libm::floorf(libm::log10f(abs_temp as f32)) as u32);
            abs_temp / pow
        } else {
            0
        };

        // remove first digit
        let remainder =
            abs_temp - first * 10_i32.pow(libm::floorf(libm::log10f(abs_temp as f32)) as u32);

        let second = if remainder > 0 || (abs_temp >= 10) {
            let pow = 10_i32.pow(libm::floorf(libm::log10f(remainder as f32)) as u32);
            Some(remainder / pow)
        } else {
            None
        };

        let left_x = self.pos.x - assets::IMAGE_THERMOMETER_DIGITAL.size.x as f32 / 2.;
        let left_y = self.pos.y - assets::IMAGE_THERMOMETER_DIGITAL.size.y as f32 / 2.;

        if self.temperature < 0. {
            display.render_point(left_x as i32 + 1, left_y as i32 + 4, true);
            display.render_point(left_x as i32 + 2, left_y as i32 + 4, true);
        }

        let first_digit = Vec2::new(left_x + 4., left_y + 2.);
        let second_digit = Vec2::new(left_x + 10., left_y + 2.);

        // This is stupid
        let num_map = |num| match num {
            0 => "0",
            1 => "1",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            9 => "9",
            _ => "0",
        };

        if let Some(second) = second {
            display.render_text_complex(
                first_digit,
                num_map(first),
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );
            display.render_text_complex(
                second_digit,
                num_map(second),
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );
        } else {
            display.render_text_complex(
                second_digit,
                num_map(first),
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );
        }
    }
}
