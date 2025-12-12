use chrono::Datelike;
use fixedstr::{str_format, str32};
use glam::Vec2;
use strum::EnumCount;

use crate::{
    Button,
    assets::{self, Image},
    date_utils::DurationExt,
    display::{CENTER_X, CENTER_X_I32, ComplexRenderOption, GameDisplay},
    fonts,
    pet::{LifeStage, definition::PetAnimationSet, render::PetRender},
    scene::{RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, home_scene::HomeScene},
    sprite::Sprite,
};

#[derive(PartialEq, Eq, Clone, Copy)]
enum State {
    Main,
    LifeStages,
    Parents,
}

const STATE_ORDER: &[State] = &[State::Main, State::LifeStages, State::Parents];

pub struct PetInfoScene {
    state: State,
    pet_render: PetRender,
    parent_renders: [PetRender; 2],
    life_stages_renders: [Option<PetRender>; LifeStage::COUNT],
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
            life_stages_renders: Default::default(),
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

        for (i, entry) in args
            .game_ctx
            .pet
            .life_stage_history
            .inner()
            .iter()
            .enumerate()
        {
            if let Some(entry) = entry {
                let mut render = PetRender::new(entry.def_id);
                render.anime.set_random_frame(&mut args.game_ctx.rng);
                self.life_stages_renders[i] = Some(render);
            } else {
                break;
            }
        }
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        self.pet_render.tick(args.delta);

        if self.state == State::Main && args.input.pressed(Button::Left) {
            output.set_home();
            return;
        }

        for parent in &mut self.parent_renders {
            parent.tick(args.delta);
        }

        for life_stage_render in self.life_stages_renders.iter_mut().flatten() {
            life_stage_render.tick(args.delta);
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
            State::LifeStages => {
                const Y_BUFFER: f32 = 6.;
                let mut current_y = 1.;

                for (index, pet_render) in self
                    .life_stages_renders
                    .iter()
                    .filter_map(|i| *i)
                    .enumerate()
                {
                    let life_stage_history =
                        args.game_ctx.pet.life_stage_history.inner()[index].unwrap();
                    let life_stage = LifeStage::from_index(index);
                    let str = str_format!(
                        fixedstr::str32,
                        "{}: {}/{:0>2}/{:0>2}",
                        life_stage.name(),
                        life_stage_history.when.inner().year() - 2000,
                        life_stage_history.when.inner().month(),
                        life_stage_history.when.inner().day()
                    );
                    display.render_text_complex(
                        Vec2::new(2., current_y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&fonts::FONT_VARIABLE_SMALL),
                    );
                    current_y += Y_BUFFER + 1. + pet_render.image().size_vec2().y / 2.;
                    display.render_image_complex(
                        CENTER_X_I32,
                        current_y as i32,
                        pet_render.image(),
                        ComplexRenderOption::new().with_white().with_center(),
                    );
                    current_y += pet_render.image().size_vec2().y / 2. + Y_BUFFER;
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
