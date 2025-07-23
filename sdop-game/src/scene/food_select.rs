use glam::Vec2;

use crate::{
    display::{CENTER_X, ComplexRenderOption, GameDisplay, WIDTH_F32},
    food::{self, FOOD_COUNT, Food},
    geo::Rect,
    scene::{Scene, SceneEnum, SceneOutput, SceneTickArgs, eat_scene::EatScene},
    sprite::Sprite,
};

struct FoodOption {
    food: &'static Food,
    pos: Vec2,
}

impl Sprite for FoodOption {
    fn pos<'a>(&'a self) -> &'a Vec2 {
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
        if self.selected >= self.food_count as i32 {
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
            if args.game_ctx.unlocked_food.is_unlocked(food) {
                let x = if food_count % 2 == 0 {
                    WIDTH_F32 / 4.
                } else {
                    (WIDTH_F32 / 4.) * 3.
                };
                let y = (libm::floorf(food_count as f32 / 2.) * COL_HEIGHT) + 30.;
                self.foods[food_count] = Some(FoodOption {
                    pos: Vec2::new(x, y),
                    food: food,
                });
                food_count += 1;
            }
        }

        self.food_count = food_count;
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        if args.input.pressed(crate::Button::Right) {
            self.move_cursor(1);
        }

        if args.input.pressed(crate::Button::Left) {
            self.move_cursor(-1);
        }

        if args.input.pressed(crate::Button::Middle) {
            return SceneOutput::new(SceneEnum::Eat(EatScene::new(
                self.foods[self.selected as usize].as_ref().unwrap().food,
                args.game_ctx.pet.def_id,
            )));
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut SceneTickArgs) {
        display.render_text_complex(
            Vec2::new(CENTER_X, 8.),
            "FOOD",
            ComplexRenderOption::default().with_white().with_center(),
        );

        display.render_sprites(&self.foods);

        if let Some(food_option) = self.foods[self.selected as usize].as_ref() {
            let selected_rect =
                Rect::new_center(food_option.pos.clone(), Vec2::new(COL_WIDTH, COL_HEIGHT));
            display.render_rect_outline(selected_rect, true);
        }
    }
}
