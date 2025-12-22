use core::time::Duration;

use fixedstr::str_format;
use glam::{IVec2, Vec2};

use crate::{
    Button,
    anime::HasAnime,
    assets,
    clock::{AnalogueClockKind, AnalogueRenderClock},
    display::{
        CENTER_X, CENTER_X_I32, CENTER_Y, CENTER_Y_I32, ComplexRenderOption, GameDisplay,
        HEIGHT_F32, PostionMode, WIDTH_F32,
    },
    fonts::FONT_VARIABLE_SMALL,
    geo::RectVec2,
    pet::{definition::PetAnimationSet, render::PetRender},
    scene::{RenderArgs, Scene, SceneOutput, SceneTickArgs},
    sounds::{SONG_HEAL, SongPlayOptions},
    sprite::BasicAnimeSprite,
};

const HOPITAL_FLOOR_Y: i32 = CENTER_Y_I32 + 20;
const HOPITAL_FLOOR_Y_F32: f32 = HOPITAL_FLOOR_Y as f32;

enum State {
    Entering,
    DoctorQuestion,
    HealingScene,
    LeavingRoom,
}

pub struct HealScene {
    pet_render: PetRender,
    doctor_full: BasicAnimeSprite,
    hopstial_screen: BasicAnimeSprite,
    hopstial_screen_off: BasicAnimeSprite,
    clock: AnalogueRenderClock,
    state_elapsed: Duration,
    state: State,
    will_pay: bool,
    heal_time: Duration,
}

impl Default for HealScene {
    fn default() -> Self {
        Self::new()
    }
}

impl HealScene {
    pub fn new() -> Self {
        Self {
            pet_render: PetRender::default().with_pos_mode(PostionMode::Bottomleft),
            doctor_full: BasicAnimeSprite::new(
                Vec2::new(CENTER_X, CENTER_Y - 10.),
                &assets::FRAMES_DOCOR_FULL,
            ),
            hopstial_screen: BasicAnimeSprite::new(
                Vec2::new(10., HOPITAL_FLOOR_Y_F32),
                &assets::FRAMES_HOSPITAL_SCREEN,
            )
            .with_pos_mode(PostionMode::Bottomleft),
            hopstial_screen_off: BasicAnimeSprite::new(
                Vec2::new(10., HOPITAL_FLOOR_Y_F32),
                &assets::FRAMES_HOSPITAL_SCREEN_OFF,
            )
            .with_pos_mode(PostionMode::Bottomleft),
            clock: AnalogueRenderClock::new(
                AnalogueClockKind::Clock21,
                Vec2::new(45., 50.),
                Default::default(),
            ),
            state_elapsed: Duration::ZERO,
            state: State::Entering,
            will_pay: true,
            heal_time: Duration::ZERO,
        }
    }
}

impl Scene for HealScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.pet_render.set_def_id(args.game_ctx.pet.def_id);
        self.pet_render.set_animation(PetAnimationSet::Sad);
        self.pet_render.pos = Vec2::new(
            -(self.pet_render.anime.current_frame().size.x as f32 / 2.),
            HEIGHT_F32 - (self.pet_render.anime.current_frame().size.y as f32 / 2.),
        );
        self.pet_render.pos_mode = PostionMode::Center;
        self.heal_time = Duration::from_millis(args.game_ctx.rng.u64(30000..120000));
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.sound_system.clear_song();
    }

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        self.pet_render.tick(args.delta);
        self.state_elapsed += args.delta;

        match self.state {
            State::Entering => {
                const SPEED: f32 = 25.;

                self.pet_render.pos.x += SPEED * args.delta.as_secs_f32();
                if self.pet_render.pos.x > WIDTH_F32 - 46. {
                    self.pet_render.pos.y = HEIGHT_F32
                        - (self.pet_render.anime.current_frame().size.y as f32 / 2.)
                        - 2.;
                }

                if self.pet_render.pos.x
                    > WIDTH_F32 + self.pet_render.anime.current_frame().size.x as f32
                {
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::DoctorQuestion;
                }
            }
            State::DoctorQuestion => {
                self.doctor_full.anime().tick(args.delta);

                if args.input.pressed(Button::Left) || args.input.pressed(Button::Right) {
                    self.will_pay = !self.will_pay;
                }

                if args.input.pressed(Button::Middle) {
                    if args.game_ctx.pet.heal_cost() > args.game_ctx.money {
                        output.set_home();
                        return;
                    } else {
                        args.game_ctx.pet.cure();
                        args.game_ctx
                            .sound_system
                            .push_song(SONG_HEAL, SongPlayOptions::new().with_effect());
                        self.state_elapsed = Duration::ZERO;
                        self.state = State::HealingScene;
                    }
                }
            }
            State::HealingScene => {
                self.clock.update_time(&args.timestamp.inner().time());
                self.hopstial_screen.anime().tick(args.delta);
                self.pet_render.set_animation(PetAnimationSet::Sleeping);
                self.pet_render.pos = Vec2::new(
                    CENTER_X,
                    HOPITAL_FLOOR_Y_F32
                        - (self.pet_render.anime.current_frame().size.y as f32) / 2.,
                );

                if self.state_elapsed > self.heal_time {
                    self.state = State::LeavingRoom;
                    self.state_elapsed = Duration::ZERO;
                }
            }
            State::LeavingRoom => {
                self.clock.update_time(&args.timestamp.inner().time());
                self.pet_render.set_animation(PetAnimationSet::Happy);

                const SPEED: f32 = 10.;

                self.pet_render.pos.x -= SPEED * args.delta.as_secs_f32();
                if self.pet_render.pos.x < -(self.pet_render.anime.current_frame().size.x as f32) {
                    output.set_home();
                    return;
                }
            }
        }
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        match self.state {
            State::Entering => {
                display.render_image_complex(
                    WIDTH_F32 as i32 - assets::IMAGE_HOSPITAL_ENTRY.size.x as i32,
                    HEIGHT_F32 as i32 - assets::IMAGE_HOSPITAL_ENTRY.size.y as i32,
                    &assets::IMAGE_HOSPITAL_ENTRY,
                    ComplexRenderOption::new().with_white().with_black(),
                );

                display.render_sprite(&self.pet_render);
            }
            State::DoctorQuestion => {
                display.render_image_complex(
                    0,
                    0,
                    &assets::IMAGE_DOCOR_SIGN,
                    ComplexRenderOption::new().with_white().with_black(),
                );

                let mut current_y = self.doctor_full.pos.y as i32
                    + self.doctor_full.anime.current_frame().isize.y / 2
                    + 3;

                display.render_sprite(&self.doctor_full);

                if args.game_ctx.pet.heal_cost() > args.game_ctx.money {
                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, current_y),
                        "YOU NEED",
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    current_y += 7;

                    let str = str_format!(
                        fixedstr::str32,
                        "${}",
                        args.game_ctx.pet.heal_cost() - args.game_ctx.money
                    );
                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, current_y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    current_y += 7;

                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, current_y),
                        "MORE SO",
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    current_y += 7;

                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, current_y),
                        "YOU DON'T DIE",
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    current_y += 14;

                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, current_y),
                        "GO DIE",
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );

                    display.render_rect_outline(
                        RectVec2::new_center(
                            Vec2::new(CENTER_X, current_y as f32),
                            Vec2::new(30., 10.),
                        ),
                        true,
                    );
                } else {
                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, current_y),
                        "PAY ME OR",
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    current_y += 7;

                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, current_y),
                        "OR",
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    current_y += 7;

                    let str = str_format!(
                        fixedstr::str32,
                        "{} WILL DIE",
                        args.game_ctx.pet.name.trim()
                    );
                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, current_y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );

                    current_y += 7;

                    let str = str_format!(fixedstr::str32, "{}$", args.game_ctx.pet.heal_cost());
                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, current_y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    current_y += 14;

                    display.render_text_complex(
                        &IVec2::new(20, current_y),
                        "PAY",
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );

                    display.render_text_complex(
                        &IVec2::new(45, current_y),
                        "DIE",
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );

                    display.render_rect_outline(
                        RectVec2::new_center(
                            Vec2::new(if self.will_pay { 20. } else { 45. }, current_y as f32),
                            Vec2::new(18., 10.),
                        ),
                        true,
                    );
                }
            }
            State::HealingScene => {
                display.render_complex(&self.clock);
                display.render_sprite(&self.pet_render);
                display.render_sprite(&self.hopstial_screen);
                display.render_image_complex(
                    CENTER_X as i32,
                    30,
                    &assets::IMAGE_HOSPITAL_ICON,
                    ComplexRenderOption::new().with_white().with_center(),
                );
                display.render_line(
                    Vec2::new(0., HOPITAL_FLOOR_Y_F32),
                    Vec2::new(WIDTH_F32, HOPITAL_FLOOR_Y_F32),
                    true,
                );

                let mut current_y = HOPITAL_FLOOR_Y + 10;

                display.render_text_complex(
                    &IVec2::new(CENTER_X_I32, current_y),
                    "HEALING",
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL),
                );

                current_y += 7;

                let percent =
                    (self.state_elapsed.as_millis_f32() / self.heal_time.as_millis_f32()) * 100.;
                let str = str_format!(fixedstr::str12, "{:0>2}%", percent as i32);
                display.render_text_complex(
                    &IVec2::new(CENTER_X_I32, current_y),
                    &str,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL),
                );

                // current_y += 7.;
            }
            State::LeavingRoom => {
                display.render_complex(&self.clock);
                display.render_sprite(&self.pet_render);
                display.render_sprite(&self.hopstial_screen_off);
                display.render_image_complex(
                    CENTER_X as i32,
                    30,
                    &assets::IMAGE_HOSPITAL_ICON,
                    ComplexRenderOption::new().with_white().with_center(),
                );
                display.render_line(
                    Vec2::new(0., HOPITAL_FLOOR_Y_F32),
                    Vec2::new(WIDTH_F32, HOPITAL_FLOOR_Y_F32),
                    true,
                );
            }
        }
    }
}
