use chrono::Datelike;
use glam::{IVec2, Vec2};

use crate::{
    HEIGHT, WIDTH,
    assets::{self, DynamicImage},
    date_utils::{MoonRender, time_in_range},
    display::{
        CENTER_VEC, CENTER_X_I32, ComplexRenderOption, GameDisplay,
        HEIGHT_I32, WIDTH_F32,
    },
    fonts::FONT_VARIABLE_SMALL,
    game_consts::{ALIEN_ODDS, TELESCOPE_USE_RANGE},
    night_sky::generate_night_sky_image,
    pet::combine_pid,
    scene::{RenderArgs, Scene, SceneOutput, SceneTickArgs},
    sounds::{SONG_TWINKLE_TWINKLE_LITTLE_STAR, SongPlayOptions},
    sprite::BasicSprite,
};

type FullNightSky = DynamicImage<{ HEIGHT * WIDTH / 8 }>;

pub struct StarGazingScene {
    moon_render: MoonRender,
    night_sky: FullNightSky,
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
            night_sky: FullNightSky::default(),
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

        generate_night_sky_image::<HEIGHT>(&mut self.night_sky, days_since_ce);

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
        args.game_ctx.sound_system.clear_song();
        if self.ufo_visible {
            args.game_ctx.pet.seen_alien = true;
        }
    }

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        if !args.game_ctx.sound_system.get_playing() {
            args.game_ctx.sound_system.push_song(
                SONG_TWINKLE_TWINKLE_LITTLE_STAR,
                SongPlayOptions::new().with_music(),
            );
        }

        if args.input.any_pressed() {
            output.set_home();
            return;
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
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        if time_in_range(&args.timestamp.inner().time(), &TELESCOPE_USE_RANGE) {
            display.invert();

            const DAYTIME_RENDER_POS: IVec2 = IVec2::new(CENTER_X_I32, HEIGHT_I32 - 10);

            display.render_text_complex(
                &DAYTIME_RENDER_POS,
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
