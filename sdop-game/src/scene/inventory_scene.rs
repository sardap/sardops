use core::time::Duration;

use fixedstr::str_format;
use glam::Vec2;
use heapless::Vec;
use sdop_common::ItemCategory;
use strum::IntoEnumIterator;

use crate::{
    ALL_ITEMS, Button,
    assets::{self, Image},
    date_utils::DurationExt,
    display::{CENTER_X, ComplexRenderOption, GameDisplay, HEIGHT_F32, WIDTH_F32},
    fonts::FONT_VARIABLE_SMALL,
    game_consts::{UI_FLASH_TIMER, UI_FLASHING_TIMER},
    geo::Rect,
    items::{ITEM_COUNT, Inventory, ItemKind, icon_for_cata, items_for_cata},
    scene::{RenderArgs, Scene, SceneOutput, SceneTickArgs},
};

const SELECTABLE_CATEGORIRES: &[ItemCategory] = &[
    ItemCategory::Usable,
    ItemCategory::PlayThing,
    ItemCategory::Furniture,
    ItemCategory::Book,
    ItemCategory::Software,
    ItemCategory::Food,
];

enum State {
    SelectCategory,
    View,
}

pub struct InventoryScene {
    available_cata: Vec<ItemCategory, 10>,
    selected_index: usize,
    selected_cata: isize,
    flash_timer: Duration,
    flash: bool,
    state: State,
}

impl Default for InventoryScene {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryScene {
    pub fn new() -> Self {
        Self {
            available_cata: Vec::new(),
            selected_index: 0,
            selected_cata: 0,
            flash_timer: Duration::ZERO,
            flash: false,
            state: State::SelectCategory,
        }
    }

    fn resolved_cata(&self) -> ItemCategory {
        *self
            .available_cata
            .get(usize::try_from(self.selected_cata).unwrap_or_default())
            .unwrap_or(&ItemCategory::Book)
    }

    fn change_item(&self, inventory: &Inventory, current: usize, change: i32) -> usize {
        let mut all_items = ALL_ITEMS.iter();
        let items = items_for_cata(&self.resolved_cata());

        let start = ((current as isize + change as isize) % ITEM_COUNT as isize).max(0) as usize;

        // Bloody no box and type issues
        if change >= 0 {
            for (i, item) in all_items.skip(start).enumerate() {
                if items.contains(item) && inventory.has_item(*item) {
                    return start + i;
                }
            }

            self.change_item(inventory, 0, 0)
        } else {
            for i in (0..=start).rev() {
                let item = all_items.nth(i).unwrap();
                if items.contains(item) && inventory.has_item(*item) {
                    return i;
                }
            }

            self.change_item(inventory, ITEM_COUNT, -1)
        }
    }
}

impl Scene for InventoryScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        for cata in SELECTABLE_CATEGORIRES {
            if items_for_cata(cata)
                .iter()
                .any(|item| args.game_ctx.inventory.has_item(*item))
            {
                let _ = self.available_cata.push(*cata);
            }
        }
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        self.flash_timer += args.delta;
        if (!self.flash && self.flash_timer > UI_FLASH_TIMER)
            || (self.flash && self.flash_timer > UI_FLASHING_TIMER)
        {
            self.flash_timer = Duration::ZERO;
            self.flash = !self.flash;
        }

        let item = ItemKind::iter()
            .nth(self.selected_index)
            .unwrap_or_default();

        match self.state {
            State::SelectCategory => {
                let change = if args.input.pressed(Button::Right) {
                    1
                } else if args.input.pressed(Button::Left) {
                    -1
                } else {
                    0
                };

                if change != 0 {
                    self.flash = true;
                    self.flash_timer = Duration::ZERO;
                }

                self.selected_cata += change;
                if self.selected_cata < -1 {
                    self.selected_cata = 0;
                }
                if self.selected_cata >= self.available_cata.len() as isize {
                    self.selected_cata = -1;
                }

                if args.input.pressed(Button::Middle) {
                    if self.selected_cata <= -1 {
                        output.set_home();
                        return;
                    } else {
                        self.selected_index = self.change_item(&args.game_ctx.inventory, 0, 0);
                        self.state = State::View;
                    }
                }
            }
            State::View => {
                if args.input.pressed(Button::Right) {
                    self.selected_index =
                        self.change_item(&args.game_ctx.inventory, self.selected_index, 1);
                }

                if args.input.pressed(Button::Left) {
                    self.state = State::SelectCategory;
                }

                if args.input.pressed(Button::Middle) {
                    if let Some(item_output) = item.use_item(args.game_ctx)
                        && item.is_usable(args.game_ctx)
                    {
                        if let Some(scene) = item_output.new_scene {
                            output.set(scene);
                            return;
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
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        let item = ItemKind::iter()
            .nth(self.selected_index)
            .unwrap_or_default();

        match self.state {
            State::SelectCategory => {
                let mut y = 10.;
                for (i, cata) in self.available_cata.iter().enumerate() {
                    let x = if i % 2 == 0 { 5. } else { 37. };

                    display.render_image_complex(
                        x as i32,
                        y as i32,
                        icon_for_cata(cata),
                        ComplexRenderOption::new().with_white(),
                    );

                    if self.flash && i as isize == self.selected_cata {
                        let rect =
                            Rect::new_top_left(Vec2::new(x, y), icon_for_cata(cata).size_vec2())
                                .grow(4.);
                        display.render_rect_outline(rect, true);
                    }

                    if i % 2 != 0 {
                        y += icon_for_cata(cata).size.y as f32 + 5.;
                    }
                }

                display.render_image_complex(
                    CENTER_X as i32,
                    105,
                    &assets::IMAGE_BACK_SYMBOL,
                    ComplexRenderOption::new().with_white().with_center(),
                );
                if self.flash && self.selected_cata == -1 {
                    let rect = Rect::new_center(
                        Vec2::new(CENTER_X, 105.),
                        assets::IMAGE_BACK_SYMBOL.size_vec2(),
                    )
                    .grow(4.);
                    display.render_rect_outline(rect, true);
                }
            }
            State::View => {
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
