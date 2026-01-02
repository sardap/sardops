use core::time::Duration;

use glam::Vec2;

use crate::{
    assets::{self},
    display::{ComplexRender, GameDisplay, HEIGHT_F32, Rotation, WIDTH_F32},
    geo::RectVec2,
    particle_system::{ParticleSystem, ParticleTemplate, SpawnTrigger, Spawner, TemplateCullTatic},
    temperature::TemperatureLevel,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum WeatherKind {
    None,
    Cold,
    Snow,
    Hot,
}

pub struct Weather {
    kind: WeatherKind,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            kind: WeatherKind::None,
        }
    }
}

const SNOW_SPAWNER: Spawner = Spawner::new(
    "snow",
    SpawnTrigger::timer_range(Duration::from_millis(500)..Duration::from_millis(2000)),
    |particles, args| {
        const TEMPLATE: ParticleTemplate = ParticleTemplate::new(
            TemplateCullTatic::OutsideRect(RectVec2::new_top_left(
                Vec2::new(-10., -10.),
                Vec2::new(WIDTH_F32 + 20., HEIGHT_F32 + 20.),
            )),
            RectVec2::new_top_left(Vec2::new(0., -5.), Vec2::new(WIDTH_F32, 2.)),
            Vec2::new(-3., 3.)..Vec2::new(3., 10.),
            &[&assets::IMAGE_SNOWFLAKE],
        )
        .with_rotation(&[Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270]);
        particles.add(TEMPLATE.instantiate(&mut args.rng));
    },
);

impl Weather {
    pub fn setup(&mut self, rng: &mut fastrand::Rng) {}

    pub fn tick<'a, const MAX_PARTICLES: usize, const MAX_SPAWN_FUNCS: usize>(
        &mut self,
        delta: Duration,
        rng: &mut fastrand::Rng,
        temperature_level: TemperatureLevel,
        particle_system: &mut ParticleSystem<MAX_PARTICLES, MAX_SPAWN_FUNCS>,
    ) {
        let next_kind = temperature_level.into();

        if self.kind != next_kind {
            // Teardown
            match self.kind {
                WeatherKind::None => {}
                WeatherKind::Cold => {}
                WeatherKind::Snow => {
                    particle_system.remove_spawner(SNOW_SPAWNER.name);
                }
                WeatherKind::Hot => {}
            }

            // setup
            match next_kind {
                WeatherKind::None => {}
                WeatherKind::Cold => {}
                WeatherKind::Snow => {
                    particle_system.add_spawner(SNOW_SPAWNER);
                }
                WeatherKind::Hot => {}
            }

            self.kind = next_kind;
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
        // if matches!(self.kind, WeatherKind::Snow) {
        //     for flake in &self.snow_flakes {
        //         if flake.pos.y > -((assets::IMAGE_SNOWFLAKE.size.y / 2) as f32) {
        //             display.render_sprite(flake);
        //         }
        //     }
        // }
    }
}
