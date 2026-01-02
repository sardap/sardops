use core::{i32, ops::Range, time::Duration};

use chrono::Datelike;
use fixedstr::str_format;
use glam::{IVec2, Vec2};
use sdop_common::MelodyEntry;

use crate::{
    Song, Timestamp,
    anime::{Anime, AnimeTick},
    assets::{self, FRAMES_SPARKLER, FRAMES_SPARKLER_START},
    display::{CENTER_X, ComplexRenderOption, GameDisplay, HEIGHT_F32, HEIGHT_I32, WIDTH_I32},
    firework::Firework,
    fonts::FONT_MONOSPACE_TALL,
    pet::{
        definition::{PetAnimationSet, PetDefinitionId},
        render::PetRender,
    },
    scene::{RenderArgs, Scene, SceneOutput, SceneTickArgs},
    sounds::{SONG_GREENSLEEVES, SongPlayOptions},
};

const SPAWN_TIME_RANGE: Range<Duration> = Duration::from_millis(250)..Duration::from_millis(1250);

enum State {
    CountingDown,
    Fireworks,
    Complete,
}

pub struct NyeScene {
    pet_render: PetRender,
    fireworks: [Option<Firework>; 20],
    spawn_left: Duration,
    start: Timestamp,
    state_elapsed: Duration,
    state: State,
    said_number: u32,
    sparklers: [Anime; 2],
}

fn create_firework(rng: &mut fastrand::Rng) -> Firework {
    Firework::new(
        Vec2::new(
            rng.i32(5..(WIDTH_I32 - 5)) as f32,
            rng.i32(HEIGHT_I32..(HEIGHT_I32 + 3)) as f32,
        ),
        Vec2::new(0., rng.i32(-80..-50) as f32),
        rng.i32(10..40) as f32,
    )
}

impl NyeScene {
    pub fn new(def_id: PetDefinitionId, start: Timestamp) -> Self {
        Self {
            pet_render: PetRender::new(def_id).with_anime(PetAnimationSet::Happy),
            fireworks: Default::default(),
            spawn_left: SPAWN_TIME_RANGE.start,
            start,
            state_elapsed: Duration::ZERO,
            state: State::CountingDown,
            said_number: u32::MAX,
            sparklers: [
                Anime::new(&FRAMES_SPARKLER_START),
                Anime::new(&FRAMES_SPARKLER_START),
            ],
        }
    }
}

impl Scene for NyeScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {
        self.pet_render.pos = Vec2::new(
            CENTER_X,
            HEIGHT_F32 - (self.pet_render.anime.current_frame().size.y / 2) as f32 - 5.,
        );
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        self.state_elapsed += args.delta;

        match self.state {
            State::CountingDown => {
                self.pet_render.tick(args.delta / 2);
                let left = self.start - args.timestamp;
                if left.as_secs() <= 10 {
                    if self.said_number != left.as_secs() as u32 {
                        self.pet_render.set_animation(PetAnimationSet::Eat);
                        self.said_number = left.as_secs() as u32;

                        const SONGS: &[Song] = &[
                            Song::new(&[MelodyEntry::new(sdop_common::Note::E3, 8)], 80), // 10
                            Song::new(&[MelodyEntry::new(sdop_common::Note::F3, 8)], 80), // 9
                            Song::new(&[MelodyEntry::new(sdop_common::Note::G3, 8)], 80), // 8
                            Song::new(&[MelodyEntry::new(sdop_common::Note::A3, 8)], 80), // 7
                            Song::new(&[MelodyEntry::new(sdop_common::Note::B3, 8)], 80), // 6
                            Song::new(&[MelodyEntry::new(sdop_common::Note::C4, 8)], 80), // 5
                            Song::new(&[MelodyEntry::new(sdop_common::Note::D4, 8)], 80), // 4
                            Song::new(&[MelodyEntry::new(sdop_common::Note::E4, 8)], 80), // 3
                            Song::new(&[MelodyEntry::new(sdop_common::Note::F4, 8)], 80), // 2
                            Song::new(&[MelodyEntry::new(sdop_common::Note::G4, 8)], 80), // 1
                        ];

                        args.game_ctx.sound_system.push_song(
                            *SONGS.get(10 - left.as_secs() as usize).unwrap_or(&SONGS[0]),
                            SongPlayOptions::new().with_music(),
                        );
                    }
                } else {
                    self.pet_render.set_animation(PetAnimationSet::Normal);
                    self.pet_render.tick(args.delta);
                }

                if args.timestamp > self.start {
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::Fireworks;

                    for i in 0..5 {
                        self.fireworks[i] = Some(create_firework(&mut args.game_ctx.rng));
                    }
                }
            }
            State::Fireworks => {
                self.pet_render.set_animation(PetAnimationSet::Happy);
                self.pet_render.tick(args.delta);

                for sparkler in &mut self.sparklers {
                    if sparkler.frames() == FRAMES_SPARKLER {
                        sparkler.tick(args.delta);
                    } else {
                        if sparkler.tick(args.delta) == AnimeTick::Looped {
                            *sparkler = Anime::new(&FRAMES_SPARKLER);
                        }
                    }
                }

                self.spawn_left = self
                    .spawn_left
                    .checked_sub(args.delta)
                    .unwrap_or(Duration::ZERO);

                if !args.game_ctx.sound_system.get_playing() {
                    args.game_ctx
                        .sound_system
                        .push_song(SONG_GREENSLEEVES, SongPlayOptions::new().with_music());
                }

                const FIREWORKS_DURATION: Duration = Duration::from_secs(30);

                if self.spawn_left <= Duration::ZERO
                    && self.state_elapsed < (FIREWORKS_DURATION - Duration::from_secs(5))
                {
                    let mut target = 0;
                    for (index, firework) in self.fireworks.iter().enumerate() {
                        if firework.is_none() {
                            target = index;
                            break;
                        }
                    }

                    let rng = &mut args.game_ctx.rng;

                    self.fireworks[target] = Some(create_firework(rng));

                    self.spawn_left = Duration::from_millis(rng.u64(
                        (SPAWN_TIME_RANGE.start.as_millis() as u64)
                            ..SPAWN_TIME_RANGE.end.as_millis() as u64,
                    ));
                }

                for firework in &mut self.fireworks {
                    let mut should_drop = false;
                    if let Some(firework) = firework {
                        firework.tick(args.delta);
                        should_drop = firework.done();
                    }

                    if should_drop {
                        *firework = None;
                    }
                }

                if self.state_elapsed > FIREWORKS_DURATION {
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::Complete;
                    args.game_ctx.sound_system.clear_song();

                    for sparkler in &mut self.sparklers {
                        *sparkler = Anime::new(&FRAMES_SPARKLER_START);
                    }
                }
            }
            State::Complete => {
                self.pet_render.set_animation(PetAnimationSet::Normal);
                self.pet_render.tick(args.delta);

                if args.input.any_pressed() {
                    output.set_home();
                    return;
                }
            }
        }
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        const X_SIGN_OFFSET: i32 = 6;
        const Y_SIGN_OFFSET: i32 = 35;

        display.render_sprite(&self.pet_render);

        if matches!(self.state, State::Fireworks) {
            for firework in &self.fireworks {
                if let Some(firework) = firework {
                    display.render_sprite(firework);
                }
            }
        }

        if matches!(self.state, State::Fireworks | State::Complete) {
            display.render_image_complex(
                self.pet_render.pos.x as i32 + self.pet_render.static_image().isize.x / 2 + 3,
                self.pet_render.pos.y as i32,
                self.sparklers[0].current_frame(),
                ComplexRenderOption::new().with_white().with_center(),
            );

            display.render_image_complex(
                self.pet_render.pos.x as i32 - self.pet_render.static_image().isize.x / 2 - 3,
                self.pet_render.pos.y as i32,
                self.sparklers[1].current_frame(),
                ComplexRenderOption::new().with_white().with_center(),
            );
        }

        let mut y = Y_SIGN_OFFSET;

        display.render_image_complex(
            X_SIGN_OFFSET + 3,
            y,
            &assets::IMAGE_NYE_YEAR_BOX,
            ComplexRenderOption::new().with_white(),
        );

        {
            let str = str_format!(fixedstr::str6, "{:0>4}", self.start.inner().date().year());
            display.render_text_complex(
                &IVec2::new(X_SIGN_OFFSET + 5, y + 3),
                &str,
                ComplexRenderOption::new()
                    .with_font(&FONT_MONOSPACE_TALL)
                    .with_white(),
            );
        }

        y += 31;

        display.render_image_complex(
            X_SIGN_OFFSET,
            y,
            &assets::IMAGE_NYE_COUNTDOWN_BOX,
            ComplexRenderOption::new().with_white(),
        );

        {
            let str = if matches!(self.state, State::CountingDown) {
                let total_secs = (self.start - args.timestamp).as_secs() as i32;
                let mins = (total_secs % 3600) / 60;
                let seconds = total_secs % 60;
                str_format!(fixedstr::str6, "{:0>2}:{:0>2}", mins, seconds)
            } else {
                str_format!(fixedstr::str6, "{:0>2}:{:0>2}", 0, 0)
            };
            display.render_text_complex(
                &IVec2::new(X_SIGN_OFFSET + 3, y + 3),
                &str,
                ComplexRenderOption::new()
                    .with_font(&FONT_MONOSPACE_TALL)
                    .with_white(),
            );
        }
    }
}
