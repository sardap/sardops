use chrono::Datelike;
use fixedstr::str_format;
use glam::{IVec2, Vec2};

use crate::{
    Button,
    assets::{self, Image},
    date_utils::DurationExt,
    death::DeathCause,
    display::{CENTER_X, CENTER_X_I32, ComplexRenderOption, GameDisplay},
    fonts::FONT_VARIABLE_SMALL,
    pet::{planet_location_from_upid, render::PetRender},
    scene::{RenderArgs, Scene, SceneOutput, SceneTickArgs},
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

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        self.pet_render.tick(args.delta);

        if let Some(record) = &args.game_ctx.pet_history.get_by_index(self.selected) {
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
                    output.set_home();
                    return;
                } else if updated >= args.game_ctx.pet_history.count() as isize {
                    updated = 0;
                }

                if updated as usize != self.selected {
                    self.selected = updated as usize;
                }
            }
        }
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        const TEXT_X_OFFSET: i32 = 2;
        const Y_BUFFER: i32 = 7;

        if let Some(record) = args.game_ctx.pet_history.get_by_index(self.selected) {
            match self.state {
                State::Select => {
                    let str = str_format!(fixedstr::str32, "PID:{:010X}", record.upid);

                    let mut render_pos = IVec2::new(CENTER_X_I32, 10);
                    display.render_text_complex(
                        &render_pos,
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );

                    display.render_sprite(&self.pet_render);

                    render_pos.y = self.pet_render.pos.y as i32
                        + self.pet_render.static_image().isize.y / 2
                        + Y_BUFFER;

                    display.render_text_complex(
                        &render_pos,
                        &record.name,
                        ComplexRenderOption::new()
                            .with_center()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );

                    render_pos.y += Y_BUFFER;
                    render_pos.x = TEXT_X_OFFSET;

                    let str = fixedstr::str_format!(
                        fixedstr::str24,
                        "B{}/{:0>2}/{:0>2}",
                        record.born.inner().year(),
                        record.born.inner().month(),
                        record.born.inner().day()
                    );
                    display.render_text_complex(
                        &render_pos,
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    render_pos.y += Y_BUFFER;

                    if !matches!(record.died_of, DeathCause::Leaving) {
                        let str = fixedstr::str_format!(
                            fixedstr::str24,
                            "D{}/{:0>2}/{:0>2}",
                            record.death.inner().year(),
                            record.death.inner().month(),
                            record.death.inner().day()
                        );
                        display.render_text_complex(
                            &render_pos,
                            &str,
                            ComplexRenderOption::new()
                                .with_white()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        render_pos.y += Y_BUFFER;
                    }

                    let age = record.age();

                    display.render_image_top_left(
                        TEXT_X_OFFSET,
                        render_pos.y,
                        &assets::IMAGE_AGE_SYMBOL,
                    );
                    let hours = age.as_hours() as i32;
                    let days = hours / 24;
                    let hours = hours % 24;
                    let str = str_format!(fixedstr::str32, "{}d{}h", days, hours);
                    render_pos.x = TEXT_X_OFFSET + assets::IMAGE_AGE_SYMBOL.isize.x + 2;
                    display.render_text_complex(
                        &render_pos,
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );

                    render_pos.y += Y_BUFFER + assets::IMAGE_AGE_SYMBOL.isize.y / 2;
                    render_pos.x = TEXT_X_OFFSET;
                    let str = fixedstr::str_format!(fixedstr::str24, "WT:{:.0}g", record.weight());
                    display.render_text_complex(
                        &render_pos,
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    render_pos.y += Y_BUFFER * 2;
                    render_pos.x = CENTER_X_I32;
                    if matches!(record.died_of, DeathCause::Leaving) {
                        display.render_text_complex(
                            &render_pos,
                            "WHEREABOUTS",
                            ComplexRenderOption::new()
                                .with_white()
                                .with_center()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        render_pos.y += Y_BUFFER;

                        let (planet, number) = planet_location_from_upid(record.upid);

                        let str =
                            fixedstr::str_format!(fixedstr::str24, "{}-{:03}", planet, number);

                        render_pos.x = CENTER_X_I32;
                        display.render_text_complex(
                            &render_pos,
                            &str,
                            ComplexRenderOption::new()
                                .with_white()
                                .with_center()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );

                        // current_y += Y_BUFFER;
                    } else {
                        render_pos.x = CENTER_X_I32;
                        display.render_text_complex(
                            &render_pos,
                            "DIED OF",
                            ComplexRenderOption::new()
                                .with_white()
                                .with_center()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        render_pos.y += Y_BUFFER;

                        display.render_text_complex(
                            &render_pos,
                            record.died_of.name(),
                            ComplexRenderOption::new()
                                .with_white()
                                .with_center()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );

                        // current_y += Y_BUFFER;
                    }
                }
            }
        }
    }
}
