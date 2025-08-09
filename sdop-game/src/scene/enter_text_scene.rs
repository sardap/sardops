use fixedstr::str_format;
use glam::Vec2;

use crate::{
    assets,
    display::{ComplexRenderOption, GameDisplay, CENTER_X, CENTER_Y},
    fonts,
    geo::Rect,
    scene::{RenderArgs, Scene, SceneOutput, SceneTickArgs},
    Button,
};

enum State {
    SelectIndex,
    SelectChar,
}

const ENTERABLE_CHARS: &'static str = "abcdefghijklmnopqrstuvwxyz0123456789$ ";

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
}

impl EnterTextScene {
    pub fn new(max_len: usize, display_text: fixedstr::str12, valid: Option<ValidFn>) -> Self {
        let mut text = fixedstr::str32::new().to_ascii_lower();

        for _ in 0..max_len {
            text.push_char(' ');
        }

        Self {
            state: State::SelectIndex,
            display_text: display_text,
            text: text,
            max_len: max_len.min(31),
            selected_index: 0,
            char_index: 0,
            valid: valid.unwrap_or(default_valid_fn),
        }
    }
}

impl Scene for EnterTextScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
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
                        return SceneOutput::new(args.last_scene.take().unwrap());
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

                self.char_index = updated as usize;

                if args.input.pressed(Button::Middle) {
                    self.text.set(
                        self.selected_index,
                        ENTERABLE_CHARS.chars().nth(self.char_index).unwrap(),
                    );
                    self.state = State::SelectIndex;
                }
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        display.render_text_complex(
            Vec2::new(CENTER_X, 30.),
            &self.display_text,
            ComplexRenderOption::new()
                .with_white()
                .with_center()
                .with_font(&fonts::FONT_VARIABLE_SMALL),
        );

        const LETTER_BUFFER_X: f32 = 2.;
        const LETTER_START_X: f32 = 7.;
        let mut rect = Rect::new_top_left(Vec2::new(0., CENTER_Y + 5.), Vec2::new(8., 2.));

        for i in 0..self.max_len {
            rect.pos.x = LETTER_START_X + (i as f32 * (rect.size.x + LETTER_BUFFER_X));
            display.render_rect_solid(rect, true);
        }

        for (i, c) in self.text.chars().enumerate() {
            let x = LETTER_START_X + (i as f32 * (rect.size.x + LETTER_BUFFER_X));
            display.render_text_complex(
                Vec2::new(x, CENTER_Y),
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
                            + (self.selected_index as f32 * (rect.size.x + LETTER_BUFFER_X)))
                            as i32
                            - 1,
                        CENTER_Y as i32 + 15,
                        &assets::IMAGE_NAME_ARROW,
                    );
                } else {
                    let rect = Rect::new_center(
                        Vec2::new(CENTER_X, CENTER_Y + 30.),
                        assets::IMAGE_SUBMIT_BUTTON.size.as_vec2(),
                    )
                    .grow(4.);
                    display.render_rect_outline(rect, true);
                }
            }
            State::SelectChar => {
                const SYMBOL_BUFFER: f32 = 2.;
                let x =
                    LETTER_START_X + (self.selected_index as f32 * (rect.size.x + LETTER_BUFFER_X));
                display.render_text_complex(
                    Vec2::new(x, CENTER_Y),
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
