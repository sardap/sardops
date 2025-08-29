use fixedstr::str_format;
use glam::Vec2;
use strum::IntoEnumIterator;

use crate::{
    assets::{self, Image},
    display::{ComplexRenderOption, GameDisplay, CENTER_X, HEIGHT_F32, WIDTH_F32},
    fonts::FONT_VARIABLE_SMALL,
    geo::Rect,
    items::{HomeFurnitureKind, Inventory, ItemKind, ITEM_COUNT},
    scene::{
        home_scene::{self, HomeFurnitureLocation},
        RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs,
    },
    Button,
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
    PlacingFurniture { selected_index: usize },
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
                    if change_item(&args.game_ctx.inventory, 0, 0) == self.selected_index {
                        return SceneOutput::new(SceneEnum::Home(home_scene::HomeScene::new()));
                    } else {
                        self.selected_index =
                            change_item(&args.game_ctx.inventory, self.selected_index, -1);
                    }
                }

                if args.input.pressed(Button::Middle) {
                    if item.furniture().is_some() {
                        self.state = InputState::PlacingFurniture { selected_index: 0 };
                    } else if let Some(output) = item.use_item(&mut args.game_ctx) {
                        if let Some(scene) = output.new_scene {
                            return SceneOutput::new(scene);
                        }
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
            InputState::PlacingFurniture { selected_index } => {
                if args.input.pressed(Button::Right) {
                    self.state = InputState::PlacingFurniture {
                        selected_index: match selected_index {
                            0 => HomeFurnitureLocation::Right.index(),
                            1 => HomeFurnitureLocation::Top.index(),
                            2 => 3,
                            3 => HomeFurnitureLocation::Left.index(),
                            _ => 0,
                        },
                    };
                }

                if args.input.pressed(Button::Left) {
                    self.state = InputState::PlacingFurniture {
                        selected_index: match selected_index {
                            0 => HomeFurnitureLocation::Left.index(),
                            1 => 3,
                            2 => HomeFurnitureLocation::Top.index(),
                            3 => HomeFurnitureLocation::Right.index(),
                            _ => 0,
                        },
                    };
                }

                if args.input.pressed(Button::Middle) {
                    if let Some(location) = HomeFurnitureLocation::from_index(selected_index) {
                        args.game_ctx
                            .home_layout
                            .place(location, item.furniture().unwrap_or_default());
                    }

                    self.state = InputState::View;
                }
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        let inventory = &args.game_ctx.inventory;
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
                } else if item.furniture().is_some() {
                    display.render_text_complex(
                        Vec2::new(CENTER_X, y),
                        "PLACE",
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    let rect = Rect::new_center(Vec2::new(CENTER_X, y), Vec2::new(35., 10.));
                    display.render_rect_outline(rect, true);
                }
            }
            InputState::PlacingFurniture { selected_index } => {
                let mut y = 30.;

                display.render_image_complex(
                    ((WIDTH_F32 / 2.) - (item.image().size.x as f32 / 2.)) as i32,
                    y as i32,
                    item.image(),
                    ComplexRenderOption::new().with_white(),
                );
                y += item.image().size.y as f32 + 5.;

                const SIZE: Vec2 = Vec2::new(20., 15.);
                let locations = [
                    (HomeFurnitureLocation::Left, Vec2::new(1., y)),
                    (HomeFurnitureLocation::Top, Vec2::new(22., y)),
                    (HomeFurnitureLocation::Right, Vec2::new(43., y)),
                ];
                for (location, pos) in locations {
                    let rect: Rect = Rect::new_top_left(pos, SIZE);
                    if selected_index == location.index() {
                        display.render_rect_outline(rect, true);
                    } else {
                        display.render_rect_outline_dashed(rect, true, 2);
                    }
                    display.render_text_complex(
                        pos + Vec2::new(4., 2.),
                        match location {
                            HomeFurnitureLocation::Top => "TOP",
                            HomeFurnitureLocation::Left => "LFT",
                            HomeFurnitureLocation::Right => "RHT",
                        },
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    display.render_text_complex(
                        pos + Vec2::new(1., 7.),
                        if match location {
                            HomeFurnitureLocation::Top => args.game_ctx.home_layout.top,
                            HomeFurnitureLocation::Left => args.game_ctx.home_layout.left,
                            HomeFurnitureLocation::Right => args.game_ctx.home_layout.right,
                        } != HomeFurnitureKind::None
                        {
                            "FILL"
                        } else {
                            "EMPY"
                        },
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                }

                y += SIZE.y + 15.;

                display.render_image_complex(
                    CENTER_X as i32,
                    y as i32,
                    &assets::IMAGE_BACK_SYMBOL,
                    ComplexRenderOption::new().with_center().with_white(),
                );

                if selected_index == 3 {
                    let rect: Rect = Rect::new_center(
                        Vec2::new(CENTER_X, y),
                        assets::IMAGE_BACK_SYMBOL.size_vec2(),
                    )
                    .grow(5.);
                    display.render_rect_outline(rect, true);
                }
            }
        }
    }
}
