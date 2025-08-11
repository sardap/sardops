use fixedstr::str_format;
use glam::Vec2;
use strum::IntoEnumIterator;

use crate::{
    display::{ComplexRenderOption, GameDisplay, CENTER_X, HEIGHT_F32, WIDTH_F32},
    fonts::FONT_VARIABLE_SMALL,
    geo::Rect,
    items::{Inventory, Item, ITEM_COUNT},
    scene::{home_scene, RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs},
    Button,
};

fn change_item(inventory: &Inventory, current: usize, change: i32) -> usize {
    let start = ((current as isize + change as isize) % ITEM_COUNT as isize).max(0) as usize;

    // Bloody no box and type issues
    if change >= 0 {
        for (i, item) in Item::iter().skip(start).enumerate() {
            if inventory.has_item(item) {
                return start + i;
            }
        }

        change_item(inventory, 0, 0)
    } else {
        for i in (0..=start).rev() {
            if inventory.has_item(Item::iter().nth(i).unwrap()) {
                return i;
            }
        }

        change_item(inventory, ITEM_COUNT, -1)
    }
}

pub struct InventoryScene {
    selected_index: usize,
}

impl InventoryScene {
    pub fn new() -> Self {
        Self { selected_index: 0 }
    }
}

impl Scene for InventoryScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        for (i, item) in Item::iter().enumerate() {
            if args.game_ctx.inventory.has_item(item) {
                self.selected_index = i;
                break;
            }
        }
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        // Here gotta get next and wrap
        if args.input.pressed(Button::Right) {
            self.selected_index = change_item(&args.game_ctx.inventory, self.selected_index, 1);
        }

        if args.input.pressed(Button::Left) {
            if change_item(&args.game_ctx.inventory, 0, 0) == self.selected_index {
                return SceneOutput::new(SceneEnum::Home(home_scene::HomeScene::new()));
            } else {
                self.selected_index =
                    change_item(&args.game_ctx.inventory, self.selected_index, -1);
            }
        }

        if args.input.pressed(Button::Middle) {
            let item = Item::iter().nth(self.selected_index).unwrap_or_default();
            item.use_item(&mut args.game_ctx);
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        let inventory = &args.game_ctx.inventory;
        let item = Item::iter().nth(self.selected_index).unwrap_or_default();

        let mut y = 10.;
        {
            let str = str_format!(fixedstr::str32, "#{} {} ", self.selected_index, item.name());
            display.render_text_complex(
                Vec2::new(CENTER_X, y),
                &str,
                ComplexRenderOption::new()
                    .with_white()
                    .with_center()
                    .with_font(&FONT_VARIABLE_SMALL),
            );
            y += 7.
        }

        display.render_image_complex(
            ((WIDTH_F32 / 2.) - (item.image().size.x as f32 / 2.)) as i32,
            y as i32,
            item.image(),
            ComplexRenderOption::new().with_white(),
        );
        y += item.image().size.y as f32 + 5.;

        {
            let str = if item.unique() {
                str_format!(fixedstr::str32, "OWN")
            } else {
                str_format!(fixedstr::str32, "OWN: {}", inventory.item_count(item))
            };

            display.render_text_complex(
                Vec2::new(CENTER_X, y),
                &str,
                ComplexRenderOption::new()
                    .with_white()
                    .with_center()
                    .with_font(&FONT_VARIABLE_SMALL),
            );
            y += 5.
        }

        display.render_text_complex(
            Vec2::new(5., y),
            item.desc(),
            ComplexRenderOption::new()
                .with_white()
                .with_font(&FONT_VARIABLE_SMALL)
                .with_font_wrapping_x((WIDTH_F32 - 5.) as i32),
        );
        y = HEIGHT_F32 - 20.;

        if item.is_usable() && args.game_ctx.inventory.has_item(item) {
            display.render_text_complex(
                Vec2::new(CENTER_X, y),
                "USE",
                ComplexRenderOption::new()
                    .with_white()
                    .with_center()
                    .with_font(&FONT_VARIABLE_SMALL),
            );
            let rect = Rect::new_center(Vec2::new(CENTER_X, y), Vec2::new(20., 10.));
            display.render_rect_outline(rect, true);
        }
    }
}
