use glam::Vec2;

use crate::{
    display::{CENTER_X, ComplexRenderOption, GameDisplay, HEIGHT_I32, WIDTH_F32, WIDTH_I32},
    fonts::FONT_VARIABLE_SMALL,
    food::{FOODS, Food, MAX_FOOD_X},
    geo::Rect,
    items::ItemKind,
    scene::{RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, eat_scene::EatScene},
};

pub struct FoodSelectScene {
    current: &'static Food,
}

impl Default for FoodSelectScene {
    fn default() -> Self {
        Self::new()
    }
}

impl FoodSelectScene {
    pub fn new() -> Self {
        Self { current: FOODS[0] }
    }
}

const COL_HEIGHT: f32 = 30.;
const COL_WIDTH: f32 = WIDTH_F32 / 2.;

impl Scene for FoodSelectScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {}

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
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
            output.set(SceneEnum::Eat(EatScene::new(
                self.current,
                args.game_ctx.pet.def_id,
            )));
            return;
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
            Vec2::new(CENTER_X, 5.),
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
                    Vec2::new(CENTER_X, y as f32),
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
                    Vec2::new(INFO_COL_X as f32, y as f32),
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
                Vec2::new(INFO_COL_X as f32, y as f32),
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
                        Vec2::new(INFO_COL_X as f32, y as f32),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                }
            }

            y += 7;

            let y_end = y.max(select_rect_y + food.image.size.y as i32 + 7);

            if (self.current.id == 0 && i == 0) || (self.current.id > 0 && i == selected_index) {
                display.render_rect_outline(
                    Rect::new_top_left(
                        Vec2::new(1., select_rect_y as f32 - 4.),
                        Vec2::new(WIDTH_F32 - 3., (y_end - select_rect_y) as f32 + 5.),
                    ),
                    true,
                );
            }

            y += 10;
        }
    }
}
