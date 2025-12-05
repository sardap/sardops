use chrono::Datelike;
use glam::{U16Vec2, Vec2};

use crate::{
    assets::{
        self, DynamicImage, IMAGE_NIGHT_SKY_0, IMAGE_NIGHT_SKY_1, IMAGE_NIGHT_SKY_2,
    },
    date_utils::MoonRender,
    display::{ComplexRenderOption, GameDisplay, CENTER_VEC, CENTER_X, HEIGHT_F32, WIDTH_F32},
    fonts::FONT_VARIABLE_SMALL,
    game_consts::{ALIEN_ODDS, TELESCOPE_RANGE},
    pet::combine_pid,
    scene::{home_scene::HomeScene, RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs},
    sounds::{SongPlayOptions, SONG_TWINKLE_TWINKLE_LITTLE_STAR},
    sprite::BasicSprite,
    HEIGHT, WIDTH,
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

impl Default for StarGazingScene {
    fn default() -> Self {
        Self::new()
    }
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

        self.night_sky.size = U16Vec2::new(WIDTH as u16, HEIGHT as u16);

        self.night_sky.used_length = NIGHT_SKY_SIZE;

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
                    self.night_sky.texture[dst_index / 8] |= flag;
                }
            }
        }
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.sound_system.clear_song();
        if self.ufo_visible {
            args.game_ctx.pet.seen_alien = true;
        }
    }

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        if !args.game_ctx.sound_system.get_playing() {
            args.game_ctx.sound_system.push_song(
                SONG_TWINKLE_TWINKLE_LITTLE_STAR,
                SongPlayOptions::new().with_music(),
            );
        }

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

        if time.ge(&TELESCOPE_RANGE.start) && time.le(&TELESCOPE_RANGE.end) {
            display.invert();

            display.render_text_complex(
                Vec2::new(CENTER_X, HEIGHT_F32 - 10.),
                "DAYTIME",
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
