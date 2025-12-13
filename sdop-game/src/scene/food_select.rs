use glam::Vec2;

use crate::{
    HEIGHT, assets,
    display::{CENTER_X, ComplexRenderOption, GameDisplay, WIDTH_F32},
    food::{self, FOOD_COUNT, Food},
    geo::Rect,
    items::ItemKind,
    scene::{RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, eat_scene::EatScene},
    sprite::Sprite,
};

struct FoodOption {
    food: &'static Food,
    pos: Vec2,
}

impl Sprite for FoodOption {
    fn pos(&self) -> &Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl crate::assets::Image {
        self.food.image
    }
}

pub struct FoodSelectScene {
    foods: [Option<FoodOption>; FOOD_COUNT],
    food_count: usize,
    selected: i32,
}

impl Default for FoodSelectScene {
    fn default() -> Self {
        Self::new()
    }
}

impl FoodSelectScene {
    pub fn new() -> Self {
        Self {
            foods: Default::default(),
            food_count: 0,
            selected: 0,
        }
    }

    pub fn move_cursor(&mut self, change: i32) {
        self.selected += change;
        if self.selected < 0 {
            self.selected = self.food_count as i32;
        }
        if self.selected > self.food_count as i32 {
            self.selected = 0;
        }
    }
}

const COL_HEIGHT: f32 = 30.;
const COL_WIDTH: f32 = WIDTH_F32 / 2.;

impl Scene for FoodSelectScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        let mut food_count = 0;
        for food in food::FOODS {
            if args
                .game_ctx
                .inventory
                .has_item(ItemKind::from_food(food.id))
            {
                let x = if food_count % 2 == 0 {
                    WIDTH_F32 / 4.
                } else {
                    (WIDTH_F32 / 4.) * 3.
                };
                let y = (libm::floorf(food_count as f32 / 2.) * COL_HEIGHT) + 30.;
                self.foods[food_count] = Some(FoodOption {
                    pos: Vec2::new(x, y),
                    food,
                });
                food_count += 1;
            }
        }

        self.food_count = food_count;
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        if args.input.pressed(crate::Button::Right) {
            self.move_cursor(1);
        }

        if args.input.pressed(crate::Button::Left) {
            self.move_cursor(-1);
        }

        if args.input.pressed(crate::Button::Middle) {
            if self.food_count == self.selected as usize {
                output.set_home();
                return;
            }

            output.set(SceneEnum::Eat(EatScene::new(
                self.foods[self.selected as usize].as_ref().unwrap().food,
                args.game_ctx.pet.def_id,
            )));
            return;
        }
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        display.render_text_complex(
            Vec2::new(CENTER_X, 8.),
            "FOOD",
            ComplexRenderOption::new().with_white().with_center(),
        );

        display.render_sprites(&self.foods);

        const BACK_Y: i32 = HEIGHT as i32 - assets::IMAGE_BACK_SYMBOL.size.y as i32 / 2 - 15;
        display.render_image_center(CENTER_X as i32, BACK_Y, &assets::IMAGE_BACK_SYMBOL);

        if self.selected as usize == self.food_count {
            const RECT: Rect = Rect::new_center(
                Vec2::new(CENTER_X, BACK_Y as f32),
                Vec2::new(
                    assets::IMAGE_BACK_SYMBOL.size.x as f32,
                    assets::IMAGE_BACK_SYMBOL.size.y as f32 + 1.,
                ),
            );

            display.render_rect_outline(RECT, true);
        } else if let Some(food_option) = self.foods[self.selected as usize].as_ref() {
            let selected_rect = Rect::new_center(food_option.pos, Vec2::new(COL_WIDTH, COL_HEIGHT));
            display.render_rect_outline(selected_rect, true);
        }
    }
}
