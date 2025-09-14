use bincode::Encode;
use fixedstr::str_format;
use glam::Vec2;
use strum::IntoEnumIterator;

use crate::{
    Button,
    date_utils::DurationExt,
    display::{CENTER_X, ComplexRenderOption, GameDisplay, HEIGHT_F32, WIDTH_F32},
    fonts::FONT_VARIABLE_SMALL,
    geo::Rect,
    items::{ITEM_COUNT, Inventory, ItemKind},
    scene::{
        RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs,
        home_scene::{self},
    },
};

fn change_item(inventory: &Inventory, current: usize, change: i32) -> usize {
    let start = ((current as isize + change as isize) % ITEM_COUNT as isize).max(0) as usize;

    // Bloody no box and type issues
    if change >= 0 {
        for (i, item) in ItemKind::iter().skip(start).enumerate() {
            if inventory.has_item(item) {
                return start + i;
            }
        }

        change_item(inventory, 0, 0)
    } else {
        for i in (0..=start).rev() {
            if inventory.has_item(ItemKind::iter().nth(i).unwrap()) {
                return i;
            }
        }

        change_item(inventory, ITEM_COUNT, -1)
    }
}

enum InputState {
    View,
}

pub struct InventoryScene {
    selected_index: usize,
    state: InputState,
}

impl InventoryScene {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            state: InputState::View,
        }
    }
}

impl Scene for InventoryScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        for (i, item) in ItemKind::iter().enumerate() {
            if args.game_ctx.inventory.has_item(item) {
                self.selected_index = i;
                break;
            }
        }
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        let item = ItemKind::iter()
            .nth(self.selected_index)
            .unwrap_or_default();

        match self.state {
            InputState::View => {
                if args.input.pressed(Button::Right) {
                    self.selected_index =
                        change_item(&args.game_ctx.inventory, self.selected_index, 1);
                }

                if args.input.pressed(Button::Left) {
                    return SceneOutput::new(SceneEnum::Home(home_scene::HomeScene::new()));
                }

                if args.input.pressed(Button::Middle) {
                    if let Some(output) = item.use_item(&mut args.game_ctx) {
                        if let Some(scene) = output.new_scene {
                            return SceneOutput::new(scene);
                        }
                    } else if item.toggleable() {
                        let entry = args.game_ctx.inventory.get_entry_mut(item);
                        entry.item_extra.enabled = !entry.item_extra.enabled;
                    }

                    if !args.game_ctx.inventory.has_item(item) {
                        for (i, item) in ItemKind::iter().enumerate() {
                            if args.game_ctx.inventory.has_item(item) {
                                self.selected_index = i;
                                break;
                            }
                        }
                    }
                }
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        let item = ItemKind::iter()
            .nth(self.selected_index)
            .unwrap_or_default();

        match self.state {
            InputState::View => {
                let mut y = 10.;
                {
                    let str =
                        str_format!(fixedstr::str32, "#{} {} ", self.selected_index, item.name());
                    display.render_text_complex(
                        Vec2::new(5., y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL)
                            .with_font_wrapping_x((WIDTH_F32 - 10.) as i32),
                    );
                    y += 14.
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
                        str_format!(
                            fixedstr::str32,
                            "OWN: {}",
                            args.game_ctx.inventory.item_count(item)
                        )
                    };

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

                if item.is_book() {
                    let read = args.game_ctx.pet.book_history.get_read(item);
                    let completed = read.chapters();

                    let str = fixedstr::str_format!(
                        fixedstr::str12,
                        "{} / {}",
                        completed,
                        item.book_info().chapters
                    );

                    display.render_text_complex(
                        Vec2::new(CENTER_X, y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );

                    y += 7.;

                    let str = fixedstr::str_format!(
                        fixedstr::str12,
                        "Length {} mins",
                        item.book_info().length.as_mins()
                    );

                    display.render_text_complex(
                        Vec2::new(CENTER_X, y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );

                    y += 7.;
                }

                display.render_text_complex(
                    Vec2::new(5., y),
                    item.desc(),
                    ComplexRenderOption::new()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL)
                        .with_font_wrapping_x((WIDTH_F32 - 10.) as i32),
                );

                y = HEIGHT_F32 - 10.;

                if item.is_usable(args.game_ctx) {
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
                } else if item.toggleable() {
                    display.render_text_complex(
                        Vec2::new(CENTER_X, y),
                        if args.game_ctx.inventory.get_entry(item).item_extra.enabled {
                            "ENABLED"
                        } else {
                            "DISABLED"
                        },
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    let rect = Rect::new_center(Vec2::new(CENTER_X, y + 6.), Vec2::new(20., 1.));
                    display.render_rect_outline(rect, true);
                }
            }
        }
    }
}
