use core::time::Duration;

use glam::{IVec2, Vec2};

use crate::{
    Button, assets,
    display::{CENTER_X, CENTER_X_I32, ComplexRenderOption, GameDisplay, Rotation, WIDTH_F32},
    fonts::FONT_VARIABLE_SMALL,
    pet::{ParentInfo, definition::PetAnimationSet, render::PetRender},
    scene::{RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, breed_scene::BreedScene},
    suiter::Suiter,
};

const FALLING_SPEED: f32 = 15.;
const Y_STOP: f32 = 45.;

enum State {
    Intro,
    Waiting,
    Question,
}

pub struct SuitersScene {
    state: State,
    state_elapsed: Duration,
    suiter_render: PetRender,
    suiter: Suiter,
    flower_left: Vec2,
    flower_right: Vec2,
    embrace: bool,
}

impl SuitersScene {
    pub fn new(suiter: Suiter) -> Self {
        Self {
            state: State::Intro,
            state_elapsed: Duration::ZERO,
            suiter_render: PetRender::new(suiter.pet_def_id),
            suiter,
            flower_left: Vec2::new(-15., Y_STOP + 10.),
            flower_right: Vec2::new(WIDTH_F32 + 15., Y_STOP + 10.),
            embrace: true,
        }
    }
}

impl Scene for SuitersScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {
        self.suiter_render.pos = Vec2::new(CENTER_X, -30.);
        self.suiter_render.set_animation(PetAnimationSet::Happy);
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.suiter_system.clear_suiter();
    }

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        self.state_elapsed += args.delta;
        self.suiter_render.tick(args.delta);

        match self.state {
            State::Intro => {
                self.suiter_render.pos.y += FALLING_SPEED * args.delta.as_secs_f32();
                self.flower_left.x += FALLING_SPEED * 0.4 * args.delta.as_secs_f32();
                self.flower_right.x -= FALLING_SPEED * 0.4 * args.delta.as_secs_f32();

                if self.suiter_render.pos.y > Y_STOP {
                    self.suiter_render.pos.y = Y_STOP;
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::Waiting;
                }
            }
            State::Waiting => {
                if self.state_elapsed > Duration::from_secs(2) {
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::Question;
                }
            }
            State::Question => {
                if args.input.pressed(Button::Left) || args.input.pressed(Button::Right) {
                    self.embrace = !self.embrace;
                }

                if args.input.pressed(Button::Middle) {
                    if self.embrace {
                        output.set(SceneEnum::Breed(BreedScene::new(
                            ParentInfo::new(
                                args.game_ctx.pet.upid,
                                args.game_ctx.pet.def_id,
                                args.game_ctx.pet.name,
                            ),
                            ParentInfo::new(
                                self.suiter.upid,
                                self.suiter.pet_def_id,
                                self.suiter.name,
                            ),
                        )));
                        return;
                    } else {
                        output.set_home();
                        return;
                    }
                }
            }
        }
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        display.render_sprite(&self.suiter_render);
        display.render_image_complex(
            self.flower_left.x as i32,
            self.flower_left.y as i32,
            &assets::IMAGE_FLOWER,
            ComplexRenderOption::new().with_center().with_white(),
        );
        display.render_image_complex(
            self.flower_right.x as i32,
            self.flower_right.y as i32,
            &assets::IMAGE_FLOWER,
            ComplexRenderOption::new().with_center().with_white(),
        );

        match self.state {
            State::Intro => {}
            State::Waiting => {}
            State::Question => {
                let mut y = self.suiter_render.pos.y as i32 + 30;

                display.render_text_complex(
                    &IVec2::new(CENTER_X_I32, y),
                    "ENGAGE",
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL),
                );

                y += 6;

                let str = fixedstr::str_format!(fixedstr::str12, "{}?", self.suiter.name.as_str());

                display.render_text_complex(
                    &IVec2::new(CENTER_X_I32, y),
                    &str,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL),
                );

                y += 20;

                let embrace_y = y;

                display.render_text_complex(
                    &IVec2::new(20, embrace_y),
                    "EMBRACE",
                    ComplexRenderOption::new()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );

                y += 10;

                let leave_y = y;

                display.render_text_complex(
                    &IVec2::new(20, leave_y),
                    "LEAVE",
                    ComplexRenderOption::new()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );

                display.render_image_complex(
                    10,
                    if self.embrace { embrace_y } else { leave_y } as i32 + 2,
                    &assets::IMAGE_NAME_ARROW,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_rotation(Rotation::R270),
                );
            }
        }
    }
}
