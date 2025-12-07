use chrono::NaiveDateTime;
use glam::Vec2;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    assets::{self, Image},
    display::{ComplexRenderOption, GameDisplay, CENTER_X, HEIGHT_F32},
    fonts::FONT_VARIABLE_SMALL,
    geo::Rect,
    scene::{
        enter_date_scene::{self, EnterDateScene},
        home_scene::HomeScene,
        RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs,
    },
    sounds::SoundOptions,
    Button, Timestamp,
};

enum State {
    Selecting,
    Sounds,
    GettingTime,
    GotTime,
}

#[derive(Debug, EnumIter, PartialEq, Eq)]
enum Option {
    Sound,
    Time,
    Back,
}

impl Option {
    pub fn text(&self) -> &'static str {
        match self {
            Option::Sound => "SOUND",
            Option::Time => "TIME",
            Option::Back => "BACK",
        }
    }
}

#[derive(Debug, EnumIter, PartialEq, Eq)]
enum SoundSelection {
    Music,
    Effect,
    Essential,
    Back,
}

impl SoundSelection {
    pub fn text(&self) -> &'static str {
        match self {
            SoundSelection::Music => "MUSIC",
            SoundSelection::Effect => "EFFECT",
            SoundSelection::Essential => "ESSENTIAL",
            SoundSelection::Back => "BACK",
        }
    }

    pub fn enabled(&self, options: &SoundOptions) -> bool {
        match self {
            SoundSelection::Music => options.play_music,
            SoundSelection::Effect => options.play_effect,
            SoundSelection::Essential => options.play_essential,
            SoundSelection::Back => false,
        }
    }
}

pub struct SettingsScene {
    option: Option,
    sound_selected: SoundSelection,
    reset_selected: bool,
    state: State,
}

impl Default for SettingsScene {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsScene {
    pub fn new() -> Self {
        Self {
            option: Option::Sound,
            sound_selected: SoundSelection::Music,
            reset_selected: false,
            state: State::Selecting,
        }
    }
}

impl Scene for SettingsScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        match self.state {
            State::Selecting => {
                if args.input.pressed(Button::Left) {
                    self.option = Option::iter()
                        .rev()
                        .skip_while(|o| *o != self.option)
                        .nth(1)
                        .unwrap_or(Option::Back);
                }

                if args.input.pressed(Button::Right) {
                    self.option = Option::iter()
                        .skip_while(|o| *o != self.option)
                        .nth(1)
                        .unwrap_or(Option::Sound);
                }

                if args.input.pressed(Button::Middle) {
                    match self.option {
                        Option::Sound => {
                            self.state = State::Sounds;
                        }
                        Option::Time => {
                            self.state = State::GettingTime;
                        }
                        Option::Back => return SceneOutput::new(SceneEnum::Home(HomeScene::new())),
                    }
                }
            }
            State::Sounds => {
                if args.input.pressed(Button::Left) {
                    self.sound_selected = SoundSelection::iter()
                        .rev()
                        .skip_while(|o| *o != self.sound_selected)
                        .nth(1)
                        .unwrap_or(SoundSelection::Back);
                }

                if args.input.pressed(Button::Right) {
                    self.sound_selected = SoundSelection::iter()
                        .skip_while(|o| *o != self.sound_selected)
                        .nth(1)
                        .unwrap_or(SoundSelection::Music);
                }

                if args.input.pressed(Button::Middle) {
                    let options = args.game_ctx.sound_system.sound_options_mut();
                    match self.sound_selected {
                        SoundSelection::Music => {
                            options.play_music = !options.play_music;
                        }
                        SoundSelection::Effect => {
                            options.play_effect = !options.play_effect;
                        }
                        SoundSelection::Essential => {
                            options.play_essential = !options.play_essential;
                        }
                        SoundSelection::Back => {
                            self.state = State::Selecting;
                        }
                    }
                }
            }
            State::GettingTime => {
                self.state = State::GotTime;
                return SceneOutput::new(SceneEnum::EnterDate(
                    EnterDateScene::new(
                        enter_date_scene::Required::DateTime,
                        fixedstr::str_format!(fixedstr::str12, "WHEN IS IT?"),
                    )
                    .with_date(args.timestamp.inner().date())
                    .with_time(args.timestamp.inner().time()),
                ));
            }
            State::GotTime => {
                self.state = State::Selecting;
                args.game_ctx.set_timestamp = Some(Timestamp::new(NaiveDateTime::new(
                    args.game_ctx.shared_out.date_out,
                    args.game_ctx.shared_out.time_out,
                )));
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        display.render_image_complex(
            CENTER_X as i32,
            17,
            &assets::IMAGE_SETTINGS_TITLE,
            ComplexRenderOption::new().with_white().with_center(),
        );

        match self.state {
            State::Selecting => {
                for (i, option) in Option::iter().enumerate() {
                    if option == Option::Back {
                        break;
                    }

                    let y = if option == Option::iter().last().unwrap() {
                        110.
                    } else {
                        (40 + i * 17) as f32
                    };

                    let width = display.render_text_complex(
                        Vec2::new(CENTER_X, y),
                        option.text(),
                        ComplexRenderOption::new().with_white().with_center(),
                    );

                    if self.option == option {
                        display.render_rect_solid(
                            Rect::new_center(Vec2::new(CENTER_X, y + 7.), Vec2::new(width, 1.)),
                            true,
                        );
                    }
                }

                display.render_image_complex(
                    CENTER_X as i32,
                    (HEIGHT_F32 - 20.) as i32,
                    &assets::IMAGE_BACK_SYMBOL,
                    ComplexRenderOption::new().with_white().with_center(),
                );

                if self.option == Option::Back {
                    let rect: Rect = Rect::new_center(
                        Vec2::new(CENTER_X, HEIGHT_F32 - 20.),
                        assets::IMAGE_BACK_SYMBOL.size_vec2(),
                    )
                    .grow(6.);
                    display.render_rect_outline(rect, true);
                }
            }
            State::Sounds => {
                for (i, option) in [
                    SoundSelection::Music,
                    SoundSelection::Effect,
                    SoundSelection::Essential,
                ]
                .into_iter()
                .enumerate()
                {
                    let y = 50. + (i * 13) as f32;
                    display.render_text_complex(
                        Vec2::new(20., y),
                        option.text(),
                        ComplexRenderOption::new()
                            .with_white()
                            .with_bottom_left()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );

                    let rect = Rect::new_bottom_left(Vec2::new(5., y), Vec2::new(10., 10.));
                    if self.sound_selected == option {
                        display.render_rect_outline_dashed(rect, true, 1);
                    } else {
                        display.render_rect_outline(rect, true);
                    }

                    if option.enabled(args.game_ctx.sound_system.sound_options()) {
                        display.render_rect_solid(rect.shrink(4.), true);
                    }
                }

                display.render_image_complex(
                    CENTER_X as i32,
                    (HEIGHT_F32 - 20.) as i32,
                    &assets::IMAGE_BACK_SYMBOL,
                    ComplexRenderOption::new().with_white().with_center(),
                );

                if self.sound_selected == SoundSelection::Back {
                    let rect: Rect = Rect::new_center(
                        Vec2::new(CENTER_X, HEIGHT_F32 - 20.),
                        assets::IMAGE_BACK_SYMBOL.size_vec2(),
                    )
                    .grow(6.);
                    display.render_rect_outline(rect, true);
                }
            }
            State::GettingTime => {}
            State::GotTime => {}
        }
    }
}
