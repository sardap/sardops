use core::time::Duration;

use glam::IVec2;

use crate::{
    Button, Timestamp, assets,
    date_utils::DurationExt,
    display::{CENTER_X_I32, ComplexRenderOption, GameDisplay, Rotation, WIDTH_I32},
    explore::{Location, LocationHistoryIter, get_location},
    fonts::FONT_VARIABLE_SMALL,
    pet::{definition::PetAnimationSet, render::PetRender},
    scene::{
        RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs,
        exploring_post_scene::ExploringPostScene,
    },
    sounds::{self, SongPlayOptions},
    sprite::Sprite,
};

const SIGN_SHAKE_DURATION: Duration = Duration::from_millis(200);

enum State {
    Cooldown,
    Selecting,
}

pub struct ExploreSelectScene {
    selected_location: usize,
    sign_shake_remaining: Duration,
    next_unlocked: bool,
    next_explore_time: Timestamp,
    state: State,
    pet_render: PetRender,
}

impl ExploreSelectScene {
    pub fn new() -> Self {
        Self {
            selected_location: 0,
            sign_shake_remaining: Duration::ZERO,
            next_unlocked: false,
            next_explore_time: Timestamp::default(),
            state: State::Selecting,
            pet_render: PetRender::new(0),
        }
    }

    pub fn location(&self) -> &'static Location {
        get_location(self.selected_location)
    }
}

impl Scene for ExploreSelectScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        for (i, history) in args
            .game_ctx
            .pet
            .explore
            .location_history
            .iter()
            .enumerate()
        {
            let next_run_time = history.last_ran + get_location(i).cooldown;
            if next_run_time > self.next_explore_time {
                self.next_explore_time = next_run_time;
                self.selected_location = i;
            }
        }

        if args.timestamp < self.next_explore_time {
            self.state = State::Cooldown;
        } else {
            let mut iter =
                LocationHistoryIter::new(0, &args.game_ctx.pet.explore, &args.game_ctx.inventory);
            self.selected_location = iter.first().unwrap_or(0);
            self.next_unlocked = iter.next().is_some();
        }

        self.pet_render.set_def_id(args.game_ctx.pet.def_id);
        self.pet_render.set_animation(PetAnimationSet::Sad);
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        match self.state {
            State::Cooldown => {
                self.pet_render.tick(args.delta);

                if args.timestamp > self.next_explore_time {
                    self.state = State::Selecting;
                }

                if args.input.any_pressed() {
                    if args.input.pressed(Button::Right) {
                        output.set(SceneEnum::ExploringPost(ExploringPostScene::new()));
                    } else {
                        output.set_home();
                    }
                    return;
                }
            }
            State::Selecting => {
                self.sign_shake_remaining = self
                    .sign_shake_remaining
                    .checked_sub(args.delta)
                    .unwrap_or_default();

                if args.input.pressed(Button::Right) {
                    self.sign_shake_remaining = SIGN_SHAKE_DURATION;
                    let mut iter = LocationHistoryIter::new(
                        self.selected_location,
                        &args.game_ctx.pet.explore,
                        &args.game_ctx.inventory,
                    );
                    (self.selected_location, self.next_unlocked) = match iter.next() {
                        Some(index) => (index, iter.next().is_some()),
                        None => (
                            0,
                            LocationHistoryIter::new(
                                0,
                                &args.game_ctx.pet.explore,
                                &args.game_ctx.inventory,
                            )
                            .next()
                            .is_some(),
                        ),
                    };
                }

                if args.input.pressed(Button::Left) {
                    self.sign_shake_remaining = SIGN_SHAKE_DURATION;

                    if self.selected_location == 0 {
                        output.set_home();
                        return;
                    }

                    let mut iter = LocationHistoryIter::new(
                        self.selected_location,
                        &args.game_ctx.pet.explore,
                        &args.game_ctx.inventory,
                    );
                    self.selected_location = iter.next_back().unwrap_or(1);
                    self.next_unlocked = true;
                }

                if args.input.pressed(Button::Middle) {
                    if args
                        .game_ctx
                        .inventory
                        .has_item(get_location(self.selected_location).item)
                        && (args.game_ctx.pet.definition().life_stage.bitmask()
                            & get_location(self.selected_location).ls_mask)
                            > 0
                        && !args
                            .game_ctx
                            .pet
                            .should_be_sleeping(&(args.timestamp + self.location().length))
                        && !args.game_ctx.pet.is_ill()
                    {
                        args.game_ctx
                            .explore_system
                            .start_exploring(self.selected_location);
                        output.set_home();
                        return;
                    } else {
                        args.game_ctx
                            .sound_system
                            .push_song(sounds::SONG_ERROR, SongPlayOptions::new().with_effect());
                        self.sign_shake_remaining = SIGN_SHAKE_DURATION;
                    }
                }
            }
        }
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        match self.state {
            State::Cooldown => {
                let mut y = 3;

                display.render_text_complex(
                    &IVec2::new(CENTER_X_I32, y),
                    "COOLING OFF",
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL),
                );

                y += 6;

                display.render_text_complex(
                    &IVec2::new(CENTER_X_I32, y),
                    "FROM GOING TO",
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL),
                );

                y += 6;

                display.render_image_complex(
                    0,
                    y,
                    &self.location().cover,
                    ComplexRenderOption::new().with_black().with_white(),
                );

                y += self.location().cover.isize.y + 3;

                let total_secs = (self.next_explore_time - args.timestamp).as_secs() as i32;
                let hours = total_secs / 3600;
                let mins = (total_secs % 3600) / 60;
                let seconds = total_secs % 60;
                let str =
                    fixedstr::str_format!(fixedstr::str24, "NEED {}h{}m{}s", hours, mins, seconds);
                display.render_text_complex(
                    &IVec2::new(CENTER_X_I32, y),
                    &str,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL),
                );

                y += 6;

                display.render_image_complex(
                    CENTER_X_I32 - (self.pet_render.anime.current_frame().isize.x / 2),
                    y,
                    self.pet_render.image(),
                    ComplexRenderOption::new().with_white(),
                );
            }
            State::Selecting => {
                let mut y = 0;
                let location = get_location(self.selected_location);
                let right_life_stage =
                    (args.game_ctx.pet.definition().life_stage.bitmask() & location.ls_mask) > 0;
                let bed_soon = args
                    .game_ctx
                    .pet
                    .should_be_sleeping(&(args.timestamp + self.location().length));
                let is_ill = args.game_ctx.pet.is_ill();
                let unlocked = args.game_ctx.inventory.has_item(location.item)
                    && right_life_stage
                    && !bed_soon
                    && !is_ill;

                // Gotta handle cooldown here
                display.render_image_complex(
                    0,
                    y,
                    &location.cover,
                    ComplexRenderOption::new().with_black().with_white(),
                );

                y += location.cover.isize.y + 2;

                if unlocked {
                    const SKILL_X_OFFSET: i32 = 2;
                    const TEXT_X_OFFSET: i32 = 35;

                    display.render_image_complex(
                        SKILL_X_OFFSET,
                        y,
                        &assets::IMAGE_LENGTH_SYMBOL,
                        ComplexRenderOption::new().with_black().with_white(),
                    );
                    let mins = self.location().length.as_mins() as i32;
                    let hours = mins / 60;
                    let mins = mins % 60;
                    let str = fixedstr::str_format!(fixedstr::str24, "{}h{}m", hours, mins);
                    display.render_text_complex(
                        &IVec2::new(TEXT_X_OFFSET, y - 1),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    y += assets::IMAGE_LENGTH_SYMBOL.isize.y + 1;

                    {
                        let delta = args.game_ctx.pet.explore_skill() - location.difficulty;
                        display.render_image_complex(
                            SKILL_X_OFFSET,
                            y,
                            &assets::IMAGE_SYMBOL_SKILL_GAP,
                            ComplexRenderOption::new().with_black().with_white(),
                        );
                        let str = fixedstr::str_format!(
                            fixedstr::str12,
                            "{}{}",
                            if delta > 0 { "+" } else { "" },
                            delta
                        );
                        display.render_text_complex(
                            &IVec2::new(TEXT_X_OFFSET, y),
                            &str,
                            ComplexRenderOption::new()
                                .with_white()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        y += assets::IMAGE_SKILL_SYMBOL.isize.y + 1;
                    }

                    display.render_image_complex(
                        SKILL_X_OFFSET,
                        y,
                        &assets::IMAGE_COOLDOWN_SYMBOL,
                        ComplexRenderOption::new().with_black().with_white(),
                    );
                    let mins = self.location().cooldown.as_mins() as i32;
                    let hours = mins / 60;
                    let mins = mins % 60;
                    let str = fixedstr::str_format!(fixedstr::str24, "{}h{}m", hours, mins);
                    display.render_text_complex(
                        &IVec2::new(TEXT_X_OFFSET, y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    y += assets::IMAGE_SKILL_SYMBOL.isize.y + 1;

                    {
                        let history = args.game_ctx.pet.explore.get_by_id(self.selected_location);
                        let percent = if history.runs == 0 || history.successful == 0 {
                            0
                        } else {
                            libm::floorf((history.successful as f32 / history.runs as f32) * 100.)
                                as i32
                        };
                        let str = fixedstr::str_format!(
                            fixedstr::str32,
                            "RUNS:{} WIN:{}%",
                            history.runs,
                            percent
                        );
                        display.render_text_complex(
                            &IVec2::new(CENTER_X_I32, y + 3),
                            &str,
                            ComplexRenderOption::new()
                                .with_white()
                                .with_center()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        y += assets::IMAGE_SKILL_SYMBOL.isize.y + 1;
                    }

                    y += 5;
                } else {
                    if !right_life_stage {
                        let text_area = display.render_text_complex(
                            &IVec2::new(CENTER_X_I32, y + 5),
                            "NOT RIGHT LIFE STAGE",
                            ComplexRenderOption::new()
                                .with_white()
                                .with_center()
                                .with_font_wrapping_x(WIDTH_I32 - 2)
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        y += (text_area.y - y) + 10;
                    } else if bed_soon {
                        let text_area = display.render_text_complex(
                            &IVec2::new(CENTER_X_I32, y + 5),
                            "BEDTIME BEFORE BACK",
                            ComplexRenderOption::new()
                                .with_white()
                                .with_center()
                                .with_font_wrapping_x(WIDTH_I32 - 2)
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        y += (text_area.y - y) + 10;
                    } else {
                        let text_area = display.render_text_complex(
                            &IVec2::new(CENTER_X_I32, y + 5),
                            "DOP IS SICK",
                            ComplexRenderOption::new()
                                .with_white()
                                .with_center()
                                .with_font_wrapping_x(WIDTH_I32 - 2)
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        y += (text_area.y - y) + 10;
                    }
                }

                display.render_image_complex(
                    WIDTH_I32 / 2 - assets::IMAGE_GO_EXPLORE_SYMBOL.size.x as i32 / 2
                        + if self.sign_shake_remaining > Duration::ZERO {
                            args.game_ctx.rng.i32(-2..=2)
                        } else {
                            0
                        },
                    y,
                    if unlocked {
                        &assets::IMAGE_GO_EXPLORE_SYMBOL
                    } else {
                        &assets::IMAGE_GO_EXPLORE_LOCKED_SYMBOL
                    },
                    ComplexRenderOption::new().with_black().with_white(),
                );

                if self.selected_location > 0 {
                    display.render_image_complex(
                        1,
                        y,
                        &assets::IMAGE_GO_EXPLORE_ARROW,
                        ComplexRenderOption::new().with_black().with_white(),
                    );
                }

                if self.next_unlocked {
                    display.render_image_complex(
                        WIDTH_I32 - 9,
                        y,
                        &assets::IMAGE_GO_EXPLORE_ARROW,
                        ComplexRenderOption::new()
                            .with_black()
                            .with_white()
                            .with_rotation(Rotation::R180),
                    );
                }
            }
        }
    }
}
