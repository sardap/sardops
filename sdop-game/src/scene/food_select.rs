use core::time::Duration;

use glam::IVec2;

use crate::{
    assets,
    display::{
        CENTER_X, CENTER_X_I32, ComplexRenderOption, GameDisplay, HEIGHT_I32, WIDTH_F32, WIDTH_I32,
    },
    fonts::FONT_VARIABLE_SMALL,
    food::{FOODS, Food, MAX_FOOD_X},
    geo::RectIVec2,
    scene::{RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, eat_scene::EatScene},
    sounds::{SONG_ERROR, SongPlayOptions},
};

pub struct FoodSelectScene {
    current: &'static Food,
    sick_of_shake_remaining: Duration,
}

impl Default for FoodSelectScene {
    fn default() -> Self {
        Self::new()
    }
}

impl FoodSelectScene {
    pub fn new() -> Self {
        Self {
            current: FOODS[0],
            sick_of_shake_remaining: Duration::ZERO,
        }
    }
}

const COL_HEIGHT: f32 = 30.;
const COL_WIDTH: f32 = WIDTH_F32 / 2.;

impl Scene for FoodSelectScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {}

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        self.sick_of_shake_remaining = self
            .sick_of_shake_remaining
            .checked_sub(args.delta)
            .unwrap_or_default();

        if args.input.pressed(crate::Button::Right) {
            self.current = FOODS
                .iter()
                .skip(self.current.id as usize + 1)
                .filter(|f| args.game_ctx.inventory.has_item(f.item))
                .next()
                .unwrap_or(&FOODS[0]);
        }

        if args.input.pressed(crate::Button::Left) {
            if self.current.id == 0 {
                output.set_home();
                return;
            }

            loop {
                let next = match self.current.id.checked_sub(1) {
                    Some(index) => index as usize,
                    None => break,
                };

                if args.game_ctx.inventory.has_item(self.current.item) {
                    self.current = FOODS[next];
                    break;
                }

                self.current = FOODS[next];
            }
        }

        if args.input.pressed(crate::Button::Middle) {
            if !args.game_ctx.pet.food_history.sick_of(self.current) {
                output.set(SceneEnum::Eat(EatScene::new(
                    self.current,
                    args.game_ctx.pet.def_id,
                )));
                return;
            } else {
                args.game_ctx
                    .sound_system
                    .push_song(SONG_ERROR, SongPlayOptions::new().with_effect());
                self.sick_of_shake_remaining = Duration::from_millis(200);
            }
        }
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        const Y_START: i32 = 20;
        const FOOD_SPACE: i32 = (HEIGHT_I32 - Y_START) as i32 / 3;
        const INFO_COL_X: i32 = MAX_FOOD_X + 5;

        let current_filled = args.game_ctx.pet.food_fill_percent();

        let str =
            fixedstr::str_format!(fixedstr::str12, "{}%", libm::roundf(current_filled * 100.),);
        display.render_text_complex(
            &IVec2::new(CENTER_X_I32, 5),
            &str,
            ComplexRenderOption::new()
                .with_white()
                .with_center()
                .with_font(&FONT_VARIABLE_SMALL),
        );

        let selected_index = FOODS
            .iter()
            .filter(|f| args.game_ctx.inventory.has_item(f.item))
            .position(|f| f.id == self.current.id)
            .unwrap_or_default();

        let iter = FOODS
            .iter()
            .filter(|f| args.game_ctx.inventory.has_item(f.item))
            .enumerate();
        let mut y = Y_START;
        for (i, food) in iter {
            if selected_index != 0 && i < selected_index - 1 {
                continue;
            }

            if y > HEIGHT_I32 {
                break;
            }

            let select_rect_y = y;

            let text_y_height = display
                .render_text_complex(
                    &IVec2::new(CENTER_X_I32, y),
                    food.name,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL),
                )
                .y;

            y += (text_y_height - y) + 1;

            display.render_image_complex(
                WIDTH_I32 / 4 - food.image.size.x as i32 / 2,
                y,
                food.image,
                ComplexRenderOption::new().with_white(),
            );

            let fill = args.game_ctx.pet.food_fill(food);
            let fill_percent = (args.game_ctx.pet.stomach_filled + fill)
                / args.game_ctx.pet.definition().stomach_size;

            let str =
                fixedstr::str_format!(fixedstr::str12, "{}%", libm::roundf(fill_percent * 100.),);
            display
                .render_text_complex(
                    &IVec2::new(INFO_COL_X, y),
                    &str,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                )
                .x;

            y += 7;

            let str = fixedstr::str_format!(
                fixedstr::str12,
                "+{}%",
                libm::roundf((fill_percent - current_filled) * 100.),
            );
            display.render_text_complex(
                &IVec2::new(INFO_COL_X, y),
                &str,
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );

            y += 7;

            {
                let pet = &args.game_ctx.pet;
                if pet.stomach_filled + food.fill_factor > pet.definition().stomach_size {
                    let extra =
                        (pet.stomach_filled + food.fill_factor) - pet.definition().stomach_size;
                    let str = fixedstr::str_format!(fixedstr::str12, "+{}g", extra as i32);
                    display.render_text_complex(
                        &IVec2::new(INFO_COL_X, y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                }
            }

            y += 7;

            let ate_count = args.game_ctx.pet.food_history.consumed_count(food);
            if ate_count > 0 && ate_count < food.max_eat {
                let str =
                    fixedstr::str_format!(fixedstr::str12, "ate {}/{}", ate_count, food.max_eat);
                display.render_text_complex(
                    &IVec2::new(INFO_COL_X, y),
                    &str,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );

                y += 7;
            }

            let is_sick_of = args.game_ctx.pet.food_history.sick_of(food);

            let y_end = y.max(
                select_rect_y + food.image.size.y as i32 + 7 + if is_sick_of { 15 } else { 0 },
            );

            if (self.current.id == 0 && i == 0) || (self.current.id > 0 && i == selected_index) {
                display.render_rect_outline(
                    &RectIVec2::new_top_left(
                        IVec2::new(1, select_rect_y - 4),
                        IVec2::new(WIDTH_I32 - 3, (y_end - select_rect_y) + 5),
                    ),
                    true,
                );
            }

            if is_sick_of {
                let x_offset =
                    if self.sick_of_shake_remaining > Duration::ZERO && food == &self.current {
                        args.game_ctx.rng.i32(-2..2)
                    } else {
                        0
                    };
                display.render_image_complex(
                    x_offset,
                    select_rect_y + 4,
                    &assets::IMAGE_SICK_OF_LABEL,
                    ComplexRenderOption::new().with_white(),
                );
                display.render_image_complex(
                    x_offset,
                    select_rect_y + 4,
                    &assets::IMAGE_SICK_OF_LABEL_MASK,
                    ComplexRenderOption::new().with_black(),
                );

                if let Some(last) = args.game_ctx.pet.food_history.get_entry(food).first_ate() {
                    let expires = last + food.expire;

                    y += 5;

                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, y),
                        "EAT IN",
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );

                    y += 7;

                    let total_secs = (expires - args.timestamp).as_secs() as i32;
                    let hours = total_secs / 3600;
                    let mins = (total_secs % 3600) / 60;
                    let seconds = total_secs % 60;
                    let str =
                        fixedstr::str_format!(fixedstr::str32, "{}h{}m{}s", hours, mins, seconds);
                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );

                    y += 5;
                }
            }

            y += 10;
        }
    }
}
