use chrono::Datelike;
use fixedstr::{str32, str_format};
use glam::Vec2;

use crate::{
    assets::{self, Image},
    date_utils::DurationExt,
    display::{ComplexRenderOption, GameDisplay, CENTER_X},
    fonts,
    pet::{definition::PetAnimationSet, render::PetRender},
    scene::{home_scene::HomeScene, RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs},
    sprite::Sprite,
    Button,
};

#[derive(PartialEq, Eq, Clone, Copy)]
enum State {
    Main,
    Parents,
}

const STATE_ORDER: &[State] = &[State::Main, State::Parents];

pub struct PetInfoScene {
    state: State,
    pet_render: PetRender,
    parent_renders: [PetRender; 2],
}

impl Default for PetInfoScene {
    fn default() -> Self {
        Self::new()
    }
}

impl PetInfoScene {
    pub fn new() -> Self {
        Self {
            state: State::Main,
            pet_render: PetRender::default(),
            parent_renders: Default::default(),
        }
    }

    pub fn index(&self) -> usize {
        STATE_ORDER.iter().position(|i| i == &self.state).unwrap()
    }
}

const PLAYER_Y: f32 = 30.;

impl Scene for PetInfoScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.pet_render.set_def_id(args.game_ctx.pet.def_id);
        self.pet_render.pos = Vec2::new(CENTER_X, PLAYER_Y);

        if let Some(parents) = args.game_ctx.pet.parents {
            self.parent_renders[0].set_def_id(parents.values[0].def_id());
            self.parent_renders[1].set_def_id(parents.values[1].def_id());
        }

        for parent in &mut self.parent_renders {
            parent.set_animation(PetAnimationSet::Normal);
        }
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        self.pet_render.tick(args.delta);

        if self.state == State::Main && args.input.pressed(Button::Left) {
            return SceneOutput::new(SceneEnum::Home(HomeScene::new()));
        }

        for parent in &mut self.parent_renders {
            parent.anime.tick(args.delta);
        }

        if args.input.pressed(Button::Right) {
            let mut index = self.index() as isize;
            index += 1;
            if index as usize >= STATE_ORDER.len() {
                index = 0;
            }
            self.state = STATE_ORDER[index as usize];
        }

        if args.input.pressed(Button::Left) {
            let mut index = self.index() as isize;
            index -= 1;
            if index as usize >= STATE_ORDER.len() {
                index = STATE_ORDER.len() as isize - 1;
            }
            self.state = STATE_ORDER[index as usize];
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        const TEXT_X_OFFSET: f32 = 2.;

        match self.state {
            State::Main => {
                const Y_BUFFER: f32 = 10.;
                let pet = &args.game_ctx.pet;

                display.render_sprite(&self.pet_render);

                display.render_text_complex(
                    self.pet_render.pos - Vec2::new(0., self.pet_render.image().size_vec2().y + 2.),
                    &args.game_ctx.pet.name,
                    ComplexRenderOption::new().with_center().with_white(),
                );

                let mut current_y =
                    self.pet_render.pos.y + self.pet_render.image().size_vec2().y + 5.;

                {
                    let str = str_format!(str32, "PID:{:010X}", pet.upid);
                    display.render_text_complex(
                        Vec2::new(TEXT_X_OFFSET, current_y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&fonts::FONT_VARIABLE_SMALL),
                    );
                    current_y += Y_BUFFER;
                }

                {
                    let str = str_format!(str32, "{}", pet.definition().name);
                    display.render_text_complex(
                        Vec2::new(TEXT_X_OFFSET, current_y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&fonts::FONT_VARIABLE_SMALL),
                    );
                    current_y += Y_BUFFER;
                }

                {
                    let str = str_format!(
                        str32,
                        "WT:{:.0}g",
                        pet.definition().base_weight + pet.extra_weight
                    );
                    display.render_text_complex(
                        Vec2::new(TEXT_X_OFFSET, current_y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&fonts::FONT_VARIABLE_SMALL),
                    );
                    current_y += Y_BUFFER;
                }

                {
                    let str = fixedstr::str_format!(
                        fixedstr::str24,
                        "B{}/{:0>2}/{:0>2}",
                        args.game_ctx.pet.born.inner().year(),
                        args.game_ctx.pet.born.inner().month(),
                        args.game_ctx.pet.born.inner().day()
                    );
                    display.render_text_complex(
                        Vec2::new(TEXT_X_OFFSET, current_y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&fonts::FONT_VARIABLE_SMALL),
                    );
                    current_y += Y_BUFFER;
                }

                {
                    display.render_image_top_left(
                        TEXT_X_OFFSET as i32,
                        current_y as i32,
                        &assets::IMAGE_AGE_SYMBOL,
                    );
                    let hours = pet.age.as_hours() as i32;
                    let days = hours / 24;
                    let hours = hours % 24;
                    let str = str_format!(str32, ":{}d{}h", days, hours);
                    display.render_text_complex(
                        Vec2::new(
                            TEXT_X_OFFSET + assets::IMAGE_AGE_SYMBOL.size.x as f32,
                            current_y,
                        ),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&fonts::FONT_VARIABLE_SMALL),
                    );

                    // current_y += Y_BUFFER;
                }
            }
            State::Parents => {
                const Y_BUFFER: f32 = 6.;
                let mut current_y = Y_BUFFER;

                display.render_text_complex(
                    Vec2::new(CENTER_X, current_y),
                    "PARENTS",
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&fonts::FONT_VARIABLE_SMALL),
                );

                current_y += Y_BUFFER;

                if let Some(parents) = &args.game_ctx.pet.parents {
                    for (i, parent) in parents.values.iter().enumerate() {
                        let str = str_format!(fixedstr::str12, "#{} {}", i + 1, parent.name());

                        display.render_text_complex(
                            Vec2::new(TEXT_X_OFFSET, current_y),
                            &str,
                            ComplexRenderOption::new()
                                .with_white()
                                .with_font(&fonts::FONT_VARIABLE_SMALL),
                        );

                        current_y += Y_BUFFER;

                        display.render_image_complex(
                            CENTER_X as i32 - self.parent_renders[i].image().size().x as i32 / 2,
                            current_y as i32,
                            self.parent_renders[i].image(),
                            ComplexRenderOption::new().with_white(),
                        );

                        current_y += self.parent_renders[i].image().size().y as f32 + Y_BUFFER;

                        let record = args.game_ctx.pet_records.get_by_upid(parent.upid());

                        let str = match &record {
                            Some(record) => str_format!(
                                fixedstr::str24,
                                "B{}/{:0>2}/{:0>2}",
                                record.born.inner().year(),
                                record.born.inner().month(),
                                record.born.inner().day()
                            ),
                            None => {
                                str_format!(fixedstr::str24, "BIRTH UNKNOWN",)
                            }
                        };

                        display.render_text_complex(
                            Vec2::new(TEXT_X_OFFSET, current_y),
                            &str,
                            ComplexRenderOption::new()
                                .with_white()
                                .with_font(&fonts::FONT_VARIABLE_SMALL),
                        );

                        current_y += Y_BUFFER * 2.;
                    }
                } else {
                    display.render_text_complex(
                        Vec2::new(TEXT_X_OFFSET, current_y),
                        "NO PARENTS",
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&fonts::FONT_VARIABLE_SMALL),
                    );
                }
            }
        }
    }
}
