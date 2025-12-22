use glam::Vec2;

use crate::{
    Button,
    assets::{self, Image},
    display::{CENTER_X, ComplexRenderOption, GameDisplay, HEIGHT_F32},
    furniture::{HomeFurnitureKind, HomeFurnitureLocation, HomeFurnitureRender},
    geo::RectVec2,
    items::{FURNITURE_ITEMS, ItemKind},
    scene::{RenderArgs, Scene, SceneOutput, SceneTickArgs},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum PlaceSelection {
    Left,
    Top,
    Right,
    Exit,
}

impl PlaceSelection {
    pub const fn to_location(&self) -> Option<HomeFurnitureLocation> {
        Some(match self {
            PlaceSelection::Left => HomeFurnitureLocation::Left,
            PlaceSelection::Top => HomeFurnitureLocation::Top,
            PlaceSelection::Right => HomeFurnitureLocation::Right,
            PlaceSelection::Exit => return None,
        })
    }

    pub fn change(&self, change: isize) -> Self {
        let current = SELECT_ARRAY.iter().position(|p| p == self).unwrap() as isize + change;
        if current < 0 {
            return SELECT_ARRAY[SELECT_ARRAY.len() - 1];
        }
        if current as usize >= SELECT_ARRAY.len() {
            return SELECT_ARRAY[0];
        }

        SELECT_ARRAY[current as usize]
    }
}

const SELECT_ARRAY: &[PlaceSelection] = &[
    PlaceSelection::Left,
    PlaceSelection::Top,
    PlaceSelection::Right,
    PlaceSelection::Exit,
];

#[derive(Default)]
struct FurnitureSet {
    kind: HomeFurnitureKind,
    render: HomeFurnitureRender,
}

enum State {
    SelectingPlace,
    SelectingFurniture,
}

pub struct PlaceFurnitureScene {
    state: State,
    selected: PlaceSelection,
    left: FurnitureSet,
    top: FurnitureSet,
    right: FurnitureSet,
}

impl Default for PlaceFurnitureScene {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaceFurnitureScene {
    pub fn new() -> Self {
        Self {
            state: State::SelectingPlace,
            selected: PlaceSelection::Left,
            left: FurnitureSet::default(),
            top: FurnitureSet::default(),
            right: FurnitureSet::default(),
        }
    }
}

impl Scene for PlaceFurnitureScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.left.kind = args.game_ctx.home_layout.left;
        self.top.kind = args.game_ctx.home_layout.top;
        self.right.kind = args.game_ctx.home_layout.right;

        self.left.render = HomeFurnitureRender::new(HomeFurnitureLocation::Left, self.left.kind);
        self.top.render = HomeFurnitureRender::new(HomeFurnitureLocation::Top, self.top.kind);
        self.right.render = HomeFurnitureRender::new(HomeFurnitureLocation::Right, self.right.kind);
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        // args.game_ctx.home_layout.left = self.left.kind;
        // args.game_ctx.home_layout.top = self.top.kind;
        // args.game_ctx.home_layout.right = self.right.kind;
    }

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        self.left.render.tick(args);
        self.top.render.tick(args);
        self.right.render.tick(args);

        match self.state {
            State::SelectingPlace => {
                if args.input.pressed(Button::Left) {
                    self.selected = self.selected.change(-1);
                }
                if args.input.pressed(Button::Right) {
                    self.selected = self.selected.change(1);
                }

                if args.input.pressed(Button::Middle) {
                    if matches!(self.selected, PlaceSelection::Exit) {
                        output.set_home();
                        return;
                    }

                    self.state = State::SelectingFurniture;
                }
            }
            State::SelectingFurniture => {
                let selected = match self.selected {
                    PlaceSelection::Left => &mut self.left,
                    PlaceSelection::Top => &mut self.top,
                    PlaceSelection::Right => &mut self.right,
                    PlaceSelection::Exit => unreachable!(),
                };

                let mut dirty = false;
                if args.input.pressed(Button::Left) || args.input.pressed(Button::Right) {
                    let dir = if args.input.pressed(Button::Left) {
                        -1
                    } else {
                        1
                    };

                    for _ in 0..FURNITURE_ITEMS.len() {
                        selected.kind = selected.kind.change(dir);
                        let item = ItemKind::from(selected.kind);
                        if item == ItemKind::None
                            || (args.game_ctx.inventory.get_entry(item).owned
                                - if args.game_ctx.home_layout.furniture_present(selected.kind) {
                                    1
                                } else {
                                    0
                                })
                                > 0
                        {
                            break;
                        }
                    }

                    dirty = true;
                }

                if dirty {
                    match self.selected {
                        PlaceSelection::Left => args.game_ctx.home_layout.left = selected.kind,
                        PlaceSelection::Top => args.game_ctx.home_layout.top = selected.kind,
                        PlaceSelection::Right => args.game_ctx.home_layout.right = selected.kind,
                        PlaceSelection::Exit => unreachable!(),
                    };

                    selected.render = HomeFurnitureRender::new(
                        self.selected.to_location().unwrap_or_default(),
                        selected.kind,
                    )
                }

                if args.input.pressed(Button::Middle) {
                    self.state = State::SelectingPlace;
                }
            }
        }
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        display.render_complex(&self.left.render);
        display.render_complex(&self.top.render);
        display.render_complex(&self.right.render);

        display.render_image_complex(
            CENTER_X as i32,
            (HEIGHT_F32 - 20.) as i32,
            &assets::IMAGE_BACK_SYMBOL,
            ComplexRenderOption::new().with_white().with_center(),
        );

        match self.state {
            State::SelectingPlace => {
                if let Some(location) = self.selected.to_location() {
                    let rect: RectVec2 = RectVec2::new_center(
                        location.pos()
                            + match location {
                                HomeFurnitureLocation::Top => {
                                    Vec2::new(0., self.top.render.size().y / 2.)
                                }
                                HomeFurnitureLocation::Left => {
                                    Vec2::new(self.left.render.size().x / 2., 0.)
                                }
                                HomeFurnitureLocation::Right => {
                                    Vec2::new(-(self.right.render.size().x / 2.), 0.)
                                }
                            },
                        match location {
                            HomeFurnitureLocation::Top => self.top.render.size(),
                            HomeFurnitureLocation::Left => self.left.render.size(),
                            HomeFurnitureLocation::Right => self.right.render.size(),
                        },
                    )
                    .grow(11.);
                    display.render_rect_outline(rect, true);
                } else {
                    let rect: RectVec2 = RectVec2::new_center(
                        Vec2::new(CENTER_X, HEIGHT_F32 - 20.),
                        assets::IMAGE_BACK_SYMBOL.size_vec2(),
                    )
                    .grow(2.);
                    display.render_rect_outline(rect, true);
                }
            }
            State::SelectingFurniture => {}
        }

        for i in [&self.left, &self.top, &self.right] {
            if let HomeFurnitureRender::InvetroLight(light) = &i.render {
                display.render_complex(light);
            }
        }
    }
}
