use core::{ops::Sub, time::Duration};

use fixedstr::str_format;
use glam::{IVec2, Vec2};

use crate::{
    Button,
    assets::{self, Image},
    display::{
        CENTER_VEC, CENTER_X, CENTER_X_I32, CENTER_Y, ComplexRenderOption, GameDisplay, Rotation,
        WIDTH_F32,
    },
    fonts::FONT_VARIABLE_SMALL,
    geo::RectVec2,
    input::{ALL_BUTTONS, random_button},
    pet::{
        definition::{PetAnimationSet, PetDefinitionId},
        render::PetRender,
    },
    scene::{RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, mg_fanfare::MgFanFareScene},
    sprite::{BasicMaskedSprite, Sprite},
};

enum State {
    EnterLeft,
    Hyping,
    Lifting,
    Dropping,
    Shaking,
}

const PET_Y_CENTER: f32 = CENTER_Y;
const HOLD_TIME: Duration = Duration::from_secs(3);
const LIFTING_TIME: Duration = Duration::from_secs(10);

pub struct MgWeightLift {
    pet_render: PetRender,
    state: State,
    state_elapsed: Duration,
    pet_def_id: PetDefinitionId,
    barbell: BasicMaskedSprite,
    current_button: Button,
    amount_lifted: f32,
    shake_button_left: Duration,
    until_button_change: Duration,
    since_presed: Duration,
    hold_time: Duration,
    lifting_time: Duration,
    won: bool,
}

impl MgWeightLift {
    pub fn new(pet_def_id: PetDefinitionId) -> Self {
        Self {
            pet_render: PetRender::default(),
            state: State::EnterLeft,
            state_elapsed: Duration::ZERO,
            barbell: BasicMaskedSprite::new(
                CENTER_VEC,
                &assets::IMAGE_BARBELL_FULL,
                &assets::IMAGE_BARBELL_FULL_MASK,
            ),
            pet_def_id,
            current_button: Button::Left,
            amount_lifted: 0.,
            shake_button_left: Duration::ZERO,
            until_button_change: Duration::ZERO,
            since_presed: Duration::ZERO,
            hold_time: Duration::ZERO,
            lifting_time: Duration::ZERO,
            won: false,
        }
    }

    fn target_y(&self) -> f32 {
        self.pet_render.image().size().y as f32 - 5.
    }
}

impl Scene for MgWeightLift {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.pet_render.set_def_id(self.pet_def_id);
        self.pet_render.pos.y = PET_Y_CENTER;
        self.pet_render.pos.x = -self.pet_render.image().size_vec2().x;

        self.barbell.pos.y = self.pet_render.y2() - self.barbell.image().size().y as f32 / 2.;

        self.current_button = random_button(&mut args.game_ctx.rng);
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        self.state_elapsed += args.delta;

        match self.state {
            State::EnterLeft => {
                self.pet_render.tick(args.delta);
                const SPEED_X: f32 = 10.;

                self.pet_render.pos.x += SPEED_X * args.delta.as_secs_f32();

                self.pet_render.pos.y = if self.state_elapsed.subsec_millis() < 500 {
                    PET_Y_CENTER - 1.
                } else {
                    PET_Y_CENTER
                };

                if self.pet_render.pos.x > CENTER_X {
                    self.pet_render.pos.y = PET_Y_CENTER;
                    self.pet_render.pos.x = CENTER_X;
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::Hyping;
                }
            }
            State::Hyping => {
                self.pet_render.tick(args.delta);
                self.pet_render.set_animation(PetAnimationSet::Eat);

                if self.state_elapsed > Duration::from_secs(2) {
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::Lifting;
                }
            }
            State::Lifting => {
                self.pet_render.set_animation(PetAnimationSet::Normal);

                self.since_presed += args.delta;

                self.until_button_change = self
                    .until_button_change
                    .checked_sub(args.delta)
                    .unwrap_or_default();

                self.shake_button_left = self
                    .shake_button_left
                    .checked_sub(args.delta)
                    .unwrap_or_default();

                if self.amount_lifted > self.target_y() {
                    self.hold_time += args.delta;

                    if self.hold_time > HOLD_TIME {
                        self.won = true;
                        self.state_elapsed = Duration::ZERO;
                        self.state = State::Dropping
                    }
                } else {
                    self.hold_time = Duration::ZERO;
                }

                if self.lifting_time > Duration::ZERO {
                    self.lifting_time += args.delta;
                    if self.lifting_time > LIFTING_TIME && self.amount_lifted < self.target_y() {
                        self.state_elapsed = Duration::ZERO;
                        self.state = State::Dropping;
                    }
                }

                let strength_per_100_grams: f32 = if self.amount_lifted < self.target_y() {
                    0.5
                } else {
                    0.3
                };

                if args.input.pressed(self.current_button) {
                    self.shake_button_left = Duration::from_millis(200);
                    self.since_presed = Duration::ZERO;

                    if self.amount_lifted < self.target_y() + 5. {
                        self.amount_lifted +=
                            args.game_ctx.pet.weight() / 100. * strength_per_100_grams;
                    }
                }
                for button in ALL_BUTTONS {
                    if button != self.current_button && args.input.pressed(button) {
                        self.amount_lifted -=
                            args.game_ctx.pet.weight() / 100. * (strength_per_100_grams * 0.75);
                    }
                }

                if self.since_presed > Duration::from_millis(500) && self.amount_lifted > 1. {
                    self.amount_lifted -=
                        args.game_ctx.pet.weight() / 100. * strength_per_100_grams / 2.;
                }

                if self.amount_lifted > 1. {
                    self.lifting_time += Duration::from_micros(1);
                    self.pet_render.pos.x = if self.state_elapsed.subsec_millis() % 500 < 250 {
                        CENTER_X - 1.
                    } else {
                        CENTER_X
                    };
                }

                self.amount_lifted = self.amount_lifted.max(0.);

                if self.until_button_change <= Duration::ZERO {
                    self.until_button_change =
                        Duration::from_millis(args.game_ctx.rng.u64(500..2500));
                    self.current_button = random_button(&mut args.game_ctx.rng);
                }

                self.barbell.pos.y = self.pet_render.y2()
                    - self.barbell.image().size().y as f32 / 2.
                    - self.amount_lifted;
            }
            State::Dropping => {
                self.pet_render.tick(args.delta);

                if self.won {
                    self.pet_render.set_animation(PetAnimationSet::Happy);
                } else {
                    self.pet_render.set_animation(PetAnimationSet::Sad);
                }

                if self.amount_lifted > 0. {
                    self.amount_lifted = self
                        .amount_lifted
                        .sub(args.game_ctx.pet.weight() / 100. * 0.6)
                        .max(0.);
                }

                if self.amount_lifted <= 0. {
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::Shaking;
                }

                self.barbell.pos.y = self.pet_render.y2()
                    - self.barbell.image().size().y as f32 / 2.
                    - self.amount_lifted;
            }
            State::Shaking => {
                self.pet_render.tick(args.delta);

                if self.state_elapsed > Duration::from_secs(3) {
                    self.state_elapsed = Duration::ZERO;
                    output.set(SceneEnum::MgFanFare(MgFanFareScene::new(
                        self.won,
                        1000,
                        args.game_ctx.pet.def_id,
                    )));
                    return;
                }
            }
        }
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        let mut floor_y = 0;
        display.render_sprite(&self.pet_render);
        display.render_sprite(&self.barbell);

        match self.state {
            State::EnterLeft => {}
            State::Hyping => {}
            State::Lifting => {
                let x = if self.shake_button_left > Duration::from_millis(100) {
                    CENTER_X_I32 - 1
                } else {
                    CENTER_X_I32
                };

                if self.lifting_time > Duration::ZERO {
                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, 5),
                        "REMAINING",
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL)
                            .with_center(),
                    );
                    let str = str_format!(
                        fixedstr::str32,
                        "{}",
                        (LIFTING_TIME
                            .checked_sub(self.lifting_time)
                            .unwrap_or_default())
                        .as_millis()
                    );
                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, 15),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL)
                            .with_center(),
                    );
                }

                let lift_line = RectVec2::new_top_left(
                    Vec2::new(2., self.pet_render.y2() - self.target_y() - 5.),
                    Vec2::new(WIDTH_F32 - 4., 1.),
                );

                display.render_rect_outline_dashed(lift_line, true, 2);

                let mut y =
                    self.pet_render.pos.y as i32 + self.pet_render.static_image().isize.y / 2 + 15;

                if self.hold_time > Duration::ZERO {
                    let str = str_format!(
                        fixedstr::str32,
                        "HOLD FOR {}",
                        (HOLD_TIME.checked_sub(self.hold_time).unwrap_or_default()).as_millis()
                    );
                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL)
                            .with_center(),
                    );
                }

                y += 20;

                display.render_image_complex(
                    x,
                    y,
                    &assets::IMAGE_BUTTON_UP,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_rotation(match self.current_button {
                            Button::Left => Rotation::R90,
                            Button::Middle => Rotation::R0,
                            Button::Right => Rotation::R270,
                        }),
                );

                display.render_image_complex(
                    x,
                    y,
                    &assets::IMAGE_BUTTON_UP_MASK,
                    ComplexRenderOption::new()
                        .with_black()
                        .with_center()
                        .with_rotation(match self.current_button {
                            Button::Left => Rotation::R90,
                            Button::Middle => Rotation::R0,
                            Button::Right => Rotation::R270,
                        }),
                );
            }
            State::Dropping => {}
            State::Shaking => {
                floor_y = if self.state_elapsed < Duration::from_millis(500)
                    && self.state_elapsed.subsec_millis() % 50 > 25
                {
                    -1
                } else {
                    0
                };
            }
        }

        display.render_image_complex(
            0,
            (PET_Y_CENTER + self.pet_render.image().size().y as f32 / 2.0) as i32 + floor_y,
            &assets::IMAGE_MG_WEIGHT_LIFTING_FLOOR,
            ComplexRenderOption::new().with_white(),
        );
    }
}
