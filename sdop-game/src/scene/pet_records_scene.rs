use chrono::Datelike;
use fixedstr::str_format;
use glam::Vec2;

use crate::{
    Button,
    assets::{self, Image},
    date_utils::DurationExt,
    death::DeathCause,
    display::{CENTER_X, ComplexRenderOption, GameDisplay},
    fonts::FONT_VARIABLE_SMALL,
    pet::{planet_location_from_upid, render::PetRender},
    scene::{RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, home_scene::HomeScene},
    sprite::Sprite,
};

enum State {
    Select,
}

pub struct PetRecordsScene {
    pet_render: PetRender,
    selected: usize,
    state: State,
}

impl Default for PetRecordsScene {
    fn default() -> Self {
        Self::new()
    }
}

impl PetRecordsScene {
    pub fn new() -> Self {
        Self {
            pet_render: Default::default(),
            selected: 0,
            state: State::Select,
        }
    }
}

impl Scene for PetRecordsScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        self.pet_render.tick(args.delta);

        if let Some(record) = &args.game_ctx.pet_records.get_by_index(self.selected) {
            self.pet_render.set_def_id(record.def_id);
        }

        match self.state {
            State::Select => {
                self.pet_render.pos = Vec2::new(CENTER_X, 30.);

                let mut change = 0;

                if args.input.pressed(Button::Left) {
                    change = -1;
                }

                if args.input.pressed(Button::Right) {
                    change += 1;
                }

                let mut updated = self.selected as isize + change;
                if updated < 0 {
                    return SceneOutput::new(SceneEnum::Home(HomeScene::new()));
                } else if updated >= args.game_ctx.pet_records.count() as isize {
                    updated = 0;
                }

                if updated as usize != self.selected {
                    self.selected = updated as usize;
                }
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        const TEXT_X_OFFSET: f32 = 2.;
        const Y_BUFFER: f32 = 7.;

        if let Some(record) = args.game_ctx.pet_records.get_by_index(self.selected) {
            match self.state {
                State::Select => {
                    let str = str_format!(fixedstr::str32, "PID:{:010X}", record.upid);

                    display.render_text_complex(
                        Vec2::new(CENTER_X, 10.),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );

                    display.render_sprite(&self.pet_render);

                    let mut current_y = self.pet_render.pos.y
                        + self.pet_render.image().size().y as f32 / 2.
                        + Y_BUFFER;

                    display.render_text_complex(
                        Vec2::new(CENTER_X, current_y),
                        &record.name,
                        ComplexRenderOption::new()
                            .with_center()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );

                    current_y += Y_BUFFER;

                    let str = fixedstr::str_format!(
                        fixedstr::str24,
                        "B{}/{:0>2}/{:0>2}",
                        record.born.inner().year(),
                        record.born.inner().month(),
                        record.born.inner().day()
                    );
                    display.render_text_complex(
                        Vec2::new(TEXT_X_OFFSET, current_y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    current_y += Y_BUFFER;

                    if !matches!(record.died_of, DeathCause::Leaving) {
                        let str = fixedstr::str_format!(
                            fixedstr::str24,
                            "D{}/{:0>2}/{:0>2}",
                            record.death.inner().year(),
                            record.death.inner().month(),
                            record.death.inner().day()
                        );
                        display.render_text_complex(
                            Vec2::new(TEXT_X_OFFSET, current_y),
                            &str,
                            ComplexRenderOption::new()
                                .with_white()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        current_y += Y_BUFFER;
                    }

                    let age = record.age();

                    display.render_image_top_left(
                        TEXT_X_OFFSET as i32,
                        current_y as i32,
                        &assets::IMAGE_AGE_SYMBOL,
                    );
                    let hours = age.as_hours() as i32;
                    let days = hours / 24;
                    let hours = hours % 24;
                    let str = str_format!(fixedstr::str32, "{}d{}h", days, hours);
                    display.render_text_complex(
                        Vec2::new(
                            TEXT_X_OFFSET + assets::IMAGE_AGE_SYMBOL.size.x as f32 + 2.,
                            current_y,
                        ),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );

                    current_y += Y_BUFFER + assets::IMAGE_AGE_SYMBOL.size.y as f32 / 2.;

                    let str = fixedstr::str_format!(fixedstr::str24, "WT:{:.0}g", record.weight());
                    display.render_text_complex(
                        Vec2::new(TEXT_X_OFFSET, current_y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    current_y += Y_BUFFER * 2.;

                    if matches!(record.died_of, DeathCause::Leaving) {
                        display.render_text_complex(
                            Vec2::new(CENTER_X, current_y),
                            "WHEREABOUTS",
                            ComplexRenderOption::new()
                                .with_white()
                                .with_center()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        current_y += Y_BUFFER;

                        let (planet, number) = planet_location_from_upid(record.upid);

                        let str =
                            fixedstr::str_format!(fixedstr::str24, "{}-{:03}", planet, number);

                        display.render_text_complex(
                            Vec2::new(CENTER_X, current_y),
                            &str,
                            ComplexRenderOption::new()
                                .with_white()
                                .with_center()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        current_y += Y_BUFFER;
                    } else {
                        display.render_text_complex(
                            Vec2::new(CENTER_X, current_y),
                            "DIED OF",
                            ComplexRenderOption::new()
                                .with_white()
                                .with_center()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        current_y += Y_BUFFER;

                        display.render_text_complex(
                            Vec2::new(CENTER_X, current_y),
                            record.died_of.name(),
                            ComplexRenderOption::new()
                                .with_white()
                                .with_center()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        current_y += Y_BUFFER;
                    }
                }
            }
        }
    }
}
