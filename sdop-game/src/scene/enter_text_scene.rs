use fixedstr::str_format;
use glam::IVec2;

use crate::{
    Button, assets,
    display::{CENTER_X, CENTER_X_I32, CENTER_Y, CENTER_Y_I32, ComplexRenderOption, GameDisplay},
    fonts,
    geo::RectIVec2,
    pet::{definition::PetDefinitionId, render::PetRender},
    scene::{RenderArgs, Scene, SceneOutput, SceneTickArgs},
    sprite::Sprite,
};

enum State {
    SelectIndex,
    SelectChar,
}

const ENTERABLE_CHARS: &str = "abcdefghijklmnopqrstuvwxyz0123456789$ ";

pub type EnterTextStr = fixedstr::str32;

pub type ValidFn = fn(text: EnterTextStr) -> bool;

fn default_valid_fn(_: EnterTextStr) -> bool {
    true
}

pub struct EnterTextScene {
    state: State,
    display_text: fixedstr::str12,
    text: EnterTextStr,
    max_len: usize,
    selected_index: usize,
    char_index: usize,
    valid: ValidFn,
    pet_render: PetRender,
}

impl EnterTextScene {
    pub fn new(max_len: usize, display_text: fixedstr::str12, valid: Option<ValidFn>) -> Self {
        let mut text = fixedstr::str32::new().to_ascii_lower();

        for _ in 0..max_len {
            text.push_char(' ');
        }

        Self {
            state: State::SelectIndex,
            display_text,
            text,
            max_len: max_len.min(31),
            selected_index: 0,
            char_index: 0,
            valid: valid.unwrap_or(default_valid_fn),
            pet_render: PetRender::default(),
        }
    }

    pub fn with_show_pet(mut self, pet_def_id: PetDefinitionId) -> Self {
        self.pet_render.set_def_id(pet_def_id);
        self
    }
}

impl Scene for EnterTextScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        self.pet_render.tick(args.delta);

        match self.state {
            State::SelectIndex => {
                let mut updated = self.selected_index;
                if args.input.pressed(Button::Left) {
                    updated = updated.checked_sub(1).unwrap_or(self.max_len);
                } else if args.input.pressed(Button::Right) {
                    updated = (updated + 1) % (self.max_len + 1);
                }

                if updated == self.max_len && !(self.valid)(self.text) {
                    updated = 0;
                }

                self.selected_index = updated;

                if args.input.pressed(Button::Middle) {
                    if self.selected_index == self.max_len {
                        args.game_ctx.shared_out.enter_text_out = self.text;
                        output.set(args.last_scene.take().unwrap());
                        return;
                    } else {
                        let mut char = self.text.chars().nth(self.selected_index).unwrap_or('a');
                        if char.is_whitespace() {
                            char = 'a';
                        }
                        self.char_index = ENTERABLE_CHARS.find(char).unwrap();
                        self.state = State::SelectChar;
                    }
                }
            }
            State::SelectChar => {
                let mut updated = self.char_index;
                if args.input.pressed(Button::Left) {
                    updated = updated.checked_sub(1).unwrap_or(ENTERABLE_CHARS.len() - 1);
                }
                if args.input.pressed(Button::Right) {
                    updated = (updated + 1) % ENTERABLE_CHARS.len();
                }

                self.char_index = updated;

                if args.input.pressed(Button::Middle) {
                    self.text.set(
                        self.selected_index,
                        ENTERABLE_CHARS.chars().nth(self.char_index).unwrap(),
                    );
                    self.state = State::SelectIndex;
                }
            }
        }
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        display.render_image_complex(
            CENTER_X_I32,
            25,
            self.pet_render.image(),
            ComplexRenderOption::new().with_white().with_center(),
        );

        display.render_text_complex(
            &IVec2::new(CENTER_X_I32, 50),
            &self.display_text,
            ComplexRenderOption::new()
                .with_white()
                .with_center()
                .with_font(&fonts::FONT_VARIABLE_SMALL),
        );

        const LETTER_BUFFER_X: i32 = 2;
        const LETTER_START_X: i32 = 7;
        let mut rect = RectIVec2::new_top_left(IVec2::new(0, CENTER_Y_I32 + 5), IVec2::new(8, 2));

        for i in 0..self.max_len {
            rect.pos.x = LETTER_START_X + (i as i32 * (rect.size.x + LETTER_BUFFER_X));
            display.render_rect_solid(&rect, true);
        }

        for (i, c) in self.text.chars().enumerate() {
            let x = LETTER_START_X + (i as i32 * (rect.size.x as i32 + LETTER_BUFFER_X));
            display.render_text_complex(
                &IVec2::new(x, CENTER_Y_I32),
                &str_format!(fixedstr::str4, "{}", c),
                ComplexRenderOption::new()
                    .with_white()
                    .with_black()
                    .with_center(),
            );
        }

        match self.state {
            State::SelectIndex => {
                if (self.valid)(self.text) {
                    display.render_image_complex(
                        CENTER_X as i32,
                        (CENTER_Y + 30.) as i32,
                        &assets::IMAGE_SUBMIT_BUTTON,
                        ComplexRenderOption::new().with_white().with_center(),
                    );
                }

                if self.selected_index < self.max_len {
                    display.render_image_center(
                        (LETTER_START_X
                            + (self.selected_index as i32
                                * (rect.size.x as i32 + LETTER_BUFFER_X)))
                            - 1,
                        CENTER_Y_I32 + 15,
                        &assets::IMAGE_NAME_ARROW,
                    );
                } else {
                    let rect = RectIVec2::new_center(
                        IVec2::new(CENTER_X_I32, CENTER_Y_I32 + 30),
                        assets::IMAGE_SUBMIT_BUTTON.isize,
                    )
                    .grow(4);
                    display.render_rect_outline(&rect, true);
                }
            }
            State::SelectChar => {
                let x = LETTER_START_X
                    + (self.selected_index as i32 * (rect.size.x as i32 + LETTER_BUFFER_X));
                display.render_text_complex(
                    &IVec2::new(x, CENTER_Y_I32),
                    &str_format!(
                        fixedstr::str4,
                        "{}",
                        ENTERABLE_CHARS.chars().nth(self.char_index).unwrap()
                    ),
                    ComplexRenderOption::new()
                        .with_white()
                        .with_black()
                        .with_center(),
                );
            }
        }
    }
}
