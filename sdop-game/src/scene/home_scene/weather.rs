use core::time::Duration;

use crate::{
    assets::{self},
    display::{ComplexRender, GameDisplay, HEIGHT_F32, WIDTH_F32},
    sprite::Snowflake,
    temperature::TemperatureLevel,
};

#[derive(Clone, Copy)]
enum WeatherKind {
    None,
    Cold,
    Snow,
    Hot,
}

pub struct Weather {
    kind: WeatherKind,
    snow_flakes: [Snowflake; 30],
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            kind: WeatherKind::None,
            snow_flakes: Default::default(),
        }
    }
}

impl Weather {
    pub fn setup(&mut self, rng: &mut fastrand::Rng) {
        for flake in &mut self.snow_flakes {
            flake.reset(true, rng);
        }
    }

    pub fn tick<'a>(
        &mut self,
        delta: Duration,
        rng: &mut fastrand::Rng,
        temperature_level: TemperatureLevel,
    ) {
        self.kind = temperature_level.into();

        match self.kind {
            WeatherKind::None => {}
            WeatherKind::Cold => {}
            WeatherKind::Snow => {
                for flake in &mut self.snow_flakes {
                    flake.pos += flake.dir * delta.as_secs_f32();
                    if flake.pos.y > HEIGHT_F32 + assets::IMAGE_SNOWFLAKE.size.y as f32
                        || flake.pos.x > WIDTH_F32 + assets::IMAGE_SNOWFLAKE.size.x as f32
                        || flake.pos.x < -(assets::IMAGE_SNOWFLAKE.size.x as f32)
                    {
                        flake.reset(false, rng);
                    }
                }
            }
            WeatherKind::Hot => {}
        }
    }

    pub fn should_shake(&self) -> bool {
        match self.kind {
            WeatherKind::Cold | WeatherKind::Snow => true,
            _ => false,
        }
    }

    pub fn is_weather_none(&self) -> bool {
        matches!(self.kind, WeatherKind::None)
    }
}

impl From<TemperatureLevel> for WeatherKind {
    fn from(value: TemperatureLevel) -> Self {
        match value {
            TemperatureLevel::VeryHot | TemperatureLevel::Hot => Self::Hot,
            TemperatureLevel::Pleasant => Self::None,
            TemperatureLevel::Cold => Self::Cold,
            TemperatureLevel::VeryCold => Self::Snow,
        }
    }
}

impl ComplexRender for Weather {
    fn render(&self, display: &mut GameDisplay) {
        if matches!(self.kind, WeatherKind::Snow) {
            for flake in &self.snow_flakes {
                if flake.pos.y > -((assets::IMAGE_SNOWFLAKE.size.y / 2) as f32) {
                    display.render_sprite(flake);
                }
            }
        }
    }
}
