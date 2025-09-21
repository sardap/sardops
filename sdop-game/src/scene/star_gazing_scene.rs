use chrono::{Datelike, Timelike};
use glam::{U16Vec2, Vec2};

use crate::{
    HEIGHT, WIDTH,
    assets::{self, DynamicImage, StaticImage},
    date_utils::MoonRender,
    display::{CENTER_VEC, CENTER_X, ComplexRenderOption, GameDisplay, HEIGHT_F32, WIDTH_F32},
    fonts::FONT_VARIABLE_SMALL,
    game_consts::ALIEN_ODDS,
    pet::combine_pid,
    scene::{RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, home_scene::HomeScene},
    sprite::BasicSprite,
};

const NIGHT_SKY_SIZE: usize = 128 * 64 / 8;

type NightSky = DynamicImage<NIGHT_SKY_SIZE>;

pub struct StarGazingScene {
    moon_render: MoonRender,
    night_sky: NightSky,
    ufo_visible: bool,
    ufo_left: bool,
    ufo: BasicSprite,
}

impl StarGazingScene {
    pub fn new() -> Self {
        Self {
            moon_render: MoonRender::default(),
            night_sky: NightSky::default(),
            ufo_visible: true,
            ufo_left: false,
            ufo: BasicSprite::new(Vec2::new(-100., -100.), &assets::IMAGE_UFO_TINY),
        }
    }
}

impl Scene for StarGazingScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        let days_since_ce = args.timestamp.inner().num_days_from_ce();
        self.moon_render.pos = CENTER_VEC;
        self.moon_render.since_ce = days_since_ce;

        let x_offset = (days_since_ce % 365) as usize * 10;
        self.night_sky.size = U16Vec2::new(WIDTH as u16, HEIGHT as u16);
        const BASE_SKY: &'static StaticImage = &assets::IMAGE_NIGHT_SKY;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let src_x = (x + x_offset) % BASE_SKY.size.x as usize;
                let base_index = y * BASE_SKY.size.x as usize + src_x;
                let dst_index = y * WIDTH + x;

                let flag =
                    ((BASE_SKY.texture[base_index / 8] >> (base_index % 8)) & 1) << (dst_index % 8);
                self.night_sky.texture[dst_index / 8] =
                    (self.night_sky.texture[dst_index / 8] & !(1 << (dst_index % 8))) | flag;
            }
        }
        self.night_sky.used_length = NIGHT_SKY_SIZE;

        // let date_seed = args.timestamp.inner().date()

        let seed = combine_pid(args.game_ctx.pet.upid, args.timestamp.date_seed());
        let mut rng = fastrand::Rng::with_seed(seed);

        self.ufo_visible = rng.f32() < ALIEN_ODDS;

        if self.ufo_visible {
            self.ufo_left = args.game_ctx.rng.bool();
            self.ufo.pos = Vec2::new(
                if self.ufo_left { WIDTH_F32 + 20. } else { -20. },
                args.game_ctx.rng.i32(0..HEIGHT as i32) as f32,
            )
        }
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        if self.ufo_visible {
            args.game_ctx.pet.seen_alien = true;
        }
    }

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        if args.input.any_pressed() {
            return SceneOutput::new(SceneEnum::Home(HomeScene::new()));
        }

        const UFO_SPEED: f32 = 15.;

        if self.ufo_visible {
            let step_size = UFO_SPEED * args.delta.as_secs_f32();
            let step_size = if self.ufo_left { -step_size } else { step_size };
            self.ufo.pos.x += step_size;
            if (self.ufo_left && self.ufo.pos.x < -20.)
                || (!self.ufo_left && self.ufo.pos.x > WIDTH_F32 + 20.)
            {
                self.ufo_left = !self.ufo_left;
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        let time = args.timestamp.inner().time();

        if (time.hour() > 5 && time.minute() > 30) || (time.hour() < 17 && time.minute() < 30) {
            display.invert();

            display.render_text_complex(
                Vec2::new(CENTER_X, HEIGHT_F32 - 10.),
                "SUNNY",
                ComplexRenderOption::new()
                    .with_black()
                    .with_flip()
                    .with_center()
                    .with_font(&FONT_VARIABLE_SMALL),
            );
        } else {
            display.render_image_complex(
                0,
                0,
                &self.night_sky,
                ComplexRenderOption::new().with_white(),
            );

            display.render_sprite(&self.moon_render);

            display.render_sprite(&self.ufo);
        }
    }
}
