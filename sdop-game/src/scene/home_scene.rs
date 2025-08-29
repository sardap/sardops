use core::time::Duration;

use fixedstr::str32;
use glam::Vec2;

use crate::{
    anime::{tick_all_anime, Anime, HasAnime},
    assets::{self, Image, IMAGE_STOMACH_MASK},
    clock::{AnalogueClockKind, AnalogueRenderClock, DigitalClockRender},
    date_utils::DurationExt,
    death::DeathCause,
    display::{
        ComplexRender, ComplexRenderOption, GameDisplay, CENTER_VEC, CENTER_X, CENTER_Y,
        HEIGHT_F32, WIDTH_F32,
    },
    fish_tank::FishTankRender,
    fonts::FONT_VARIABLE_SMALL,
    geo::{vec2_direction, vec2_distance, Rect},
    items::{HomeFurnitureKind, ItemKind},
    pet::{
        definition::{PetAnimationSet, PET_BLOB_ID},
        render::PetRender,
    },
    poop::{update_poop_renders, PoopRender, MAX_POOPS},
    scene::{
        death_scene::DeathScene, evolve_scene::EvolveScene, food_select::FoodSelectScene,
        game_select::GameSelectScene, inventory_scene::InventoryScene,
        pet_info_scene::PetInfoScene, poop_clear_scene::PoopClearScene, shop_scene::ShopScene,
        RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs,
    },
    sprite::{BasicAnimeSprite, BasicSprite, Sprite},
    tv::{TvKind, TvRender},
    Button, WIDTH,
};

const WONDER_SPEED: f32 = 5.;
pub const WONDER_RECT: Rect = Rect::new_center(CENTER_VEC, Vec2::new(WIDTH as f32, 90.0));

#[derive(Clone, Copy, PartialEq, Eq)]
enum MenuOption {
    Poop,
    PetInfo,
    GameSelect,
    FoodSelect,
    Shop,
    Inventory,
}

const AWAKE_OPTIONS: &[MenuOption] = &[
    MenuOption::PetInfo,
    MenuOption::Poop,
    MenuOption::GameSelect,
    MenuOption::FoodSelect,
    MenuOption::Shop,
    MenuOption::Inventory,
];

const SLEEP_OPTIONS: &[MenuOption] = &[MenuOption::PetInfo, MenuOption::Inventory];

fn change_option(options: &[MenuOption], current: usize, change: i32) -> usize {
    let index = current as i32 + change;
    let index = if index >= options.len() as i32 {
        0usize
    } else if index < 0 {
        options.len() - 1
    } else {
        index as usize
    };

    index as usize
}

fn get_options(state: State) -> &'static [MenuOption] {
    match state {
        State::Wondering
        | State::WatchingTv {
            show_timer: _,
            show_end: _,
            watch_end: _,
        } => AWAKE_OPTIONS,
        State::Sleeping => SLEEP_OPTIONS,
    }
}

const BORDER_HEIGHT: f32 = 1.;

const TOP_BORDER_RECT: Rect = Rect::new_center(
    Vec2::new(CENTER_X, 24.),
    Vec2::new(WIDTH_F32, BORDER_HEIGHT),
);

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Wondering,
    Sleeping,
    WatchingTv {
        show_timer: Duration,
        show_end: Duration,
        watch_end: Duration,
    },
}

pub struct HomeScene {
    pet_render: PetRender,
    poops: [Option<PoopRender>; MAX_POOPS],
    target: Vec2,
    food_anime: Anime,
    selected_index: usize,
    sleeping_z: BasicAnimeSprite,
    tv: TvRender,
    state: State,
    state_elapsed: Duration,
    wonder_end: Duration,
    left_render: HomeFurnitureRender,
    top_render: HomeFurnitureRender,
    right_render: HomeFurnitureRender,
}

impl HomeScene {
    pub fn new() -> Self {
        Self {
            pet_render: PetRender::default(),
            poops: [None; MAX_POOPS],
            target: Vec2::default(),
            food_anime: Anime::new(&assets::FRAMES_FOOD_SYMBOL),
            selected_index: 0,
            sleeping_z: BasicAnimeSprite::new(CENTER_VEC, &assets::FRAMES_SLEEPING_Z),
            tv: TvRender::new(
                TvKind::LCD,
                Vec2::new(20., 40.),
                &assets::FRAMES_TV_SHOW_SPORT,
            ),
            state: State::Wondering,
            state_elapsed: Duration::ZERO,
            wonder_end: Duration::ZERO,
            left_render: HomeFurnitureRender::None,
            top_render: HomeFurnitureRender::None,
            right_render: HomeFurnitureRender::None,
        }
    }

    fn wonder_rect(&self) -> Rect {
        Rect::new_center(
            WONDER_RECT.pos,
            WONDER_RECT.size - self.pet_render.anime.current_frame().size.x as f32,
        )
    }

    fn change_state(&mut self, new_state: State) {
        if self.state == new_state {
            return;
        }

        self.state = new_state;

        let options = get_options(self.state);
        if self.selected_index >= options.len() {
            self.selected_index = 0;
        }
    }
}

fn reset_wonder_end(rng: &mut fastrand::Rng) -> Duration {
    Duration::from_millis(rng.u64(0..(1 * 60000)))
}

impl Scene for HomeScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.pet_render.pos = self
            .wonder_rect()
            .random_point_inside(&mut args.game_ctx.rng);
        self.target = self.pet_render.pos;
        self.selected_index = 0;
        self.tv.random_show(&mut args.game_ctx.rng);
        self.wonder_end = reset_wonder_end(&mut args.game_ctx.rng);

        self.top_render =
            HomeFurnitureRender::new(HomeFurnitureLocation::Top, args.game_ctx.home_layout.top);
        self.left_render =
            HomeFurnitureRender::new(HomeFurnitureLocation::Left, args.game_ctx.home_layout.left);
        self.right_render = HomeFurnitureRender::new(
            HomeFurnitureLocation::Right,
            args.game_ctx.home_layout.right,
        );
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        self.pet_render.set_def_id(args.game_ctx.pet.def_id);

        update_poop_renders(&mut self.poops, &args.game_ctx.poops);

        self.food_anime.tick(args.delta);
        self.pet_render.tick(args.delta);
        tick_all_anime(&mut self.poops, args.delta);

        let should_be_sleeping = args
            .game_ctx
            .pet
            .definition()
            .should_be_sleeping(&args.timestamp);
        if should_be_sleeping && !matches!(self.state, State::Sleeping) {
            self.change_state(State::Sleeping);
        } else if !should_be_sleeping && matches!(self.state, State::Sleeping) {
            self.change_state(State::Wondering);
        }

        if !matches!(self.state, State::Sleeping) {
            if let Some(cause_of_death) = args.game_ctx.pet.should_die() {
                return SceneOutput::new(SceneEnum::Death(DeathScene::new(
                    cause_of_death,
                    args.game_ctx.pet.def_id,
                )));
            }

            if let Some(next_pet_id) = args.game_ctx.pet.should_evolve(&mut args.game_ctx.rng) {
                return SceneOutput::new(SceneEnum::Evovle(EvolveScene::new(
                    args.game_ctx.pet.def_id,
                    next_pet_id,
                )));
            }
        }

        let options = get_options(self.state);

        if args.input.pressed(Button::Left) {
            self.selected_index = change_option(options, self.selected_index, -1);
        }
        if args.input.pressed(Button::Right) {
            self.selected_index = change_option(options, self.selected_index, 1);
        }

        self.state_elapsed += args.delta;

        match self.state {
            State::Wondering => {
                self.top_render.tick(args);
                self.left_render.tick(args);
                self.right_render.tick(args);

                if self.state_elapsed > self.wonder_end {
                    self.wonder_end = reset_wonder_end(&mut args.game_ctx.rng);
                }

                self.pet_render
                    .set_animation(args.game_ctx.pet.mood(&args.game_ctx.poops).anime_set());

                let dist = vec2_distance(self.pet_render.pos, self.target);
                if dist.abs() < 5. {
                    self.target = self
                        .wonder_rect()
                        .random_point_inside(&mut args.game_ctx.rng);
                }

                self.pet_render.pos += vec2_direction(self.pet_render.pos, self.target)
                    * WONDER_SPEED
                    * args.delta.as_secs_f32();
            }
            State::Sleeping => {
                self.pet_render.set_animation(PetAnimationSet::Sleeping);

                self.sleeping_z.anime().tick(args.delta);

                self.pet_render.pos = CENTER_VEC + Vec2::new(0., 10.);
                self.sleeping_z.pos = Vec2::new(
                    self.pet_render.pos.x + (self.pet_render.image().size_vec2().x * 0.5),
                    self.pet_render.pos.y - (self.pet_render.image().size_vec2().y * 0.7),
                );
            }
            State::WatchingTv {
                mut show_timer,
                show_end,
                watch_end,
            } => {
                if self.state_elapsed > watch_end {
                    self.change_state(State::Wondering);
                }

                self.tv.anime().tick(args.delta);
                show_timer += args.delta;

                if show_timer > show_end {
                    self.tv.random_show(&mut args.game_ctx.rng);
                    show_timer = Duration::ZERO;
                }

                self.tv.pos = Vec2::new(
                    self.tv.size().x * 0.5 + 1.,
                    CENTER_Y - self.tv.size().y * 0.5,
                );
                self.pet_render.set_animation(PetAnimationSet::Normal);
                self.pet_render.pos = Vec2::new(
                    WIDTH_F32 - self.pet_render.image().size().x as f32 / 2. - 5.,
                    self.tv.pos.y
                        + self.tv.size().y as f32 / 2.
                        + self.pet_render.image().size().y as f32,
                );

                self.state = State::WatchingTv {
                    show_timer,
                    show_end,
                    watch_end,
                }
            }
        }

        if args.input.pressed(Button::Middle) {
            match get_options(self.state)[self.selected_index] {
                MenuOption::Poop => {
                    if args.game_ctx.poops.iter().any(|i| i.is_some()) {
                        return SceneOutput::new(SceneEnum::PoopClear(PoopClearScene::new()));
                    }
                }
                MenuOption::PetInfo => {
                    return SceneOutput::new(SceneEnum::PetInfo(PetInfoScene::new()));
                }
                MenuOption::GameSelect => {
                    return SceneOutput::new(SceneEnum::GameSelect(GameSelectScene::new()));
                }
                MenuOption::FoodSelect => {
                    return SceneOutput::new(SceneEnum::FoodSelect(FoodSelectScene::new()));
                }
                MenuOption::Shop => {
                    return SceneOutput::new(SceneEnum::Shop(ShopScene::new()));
                }
                MenuOption::Inventory => {
                    return SceneOutput::new(SceneEnum::Inventory(InventoryScene::new()));
                }
            };
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        let pet = &args.game_ctx.pet;

        let total_filled = pet.stomach_filled / pet.definition().stomach_size;
        display.render_stomach(
            Vec2::new(9., IMAGE_STOMACH_MASK.size.y as f32),
            total_filled,
        );

        const STOMACH_END_X: i32 = IMAGE_STOMACH_MASK.size.y as i32 + 1;
        display.render_image_top_left(STOMACH_END_X, 0, &assets::IMAGE_AGE_SYMBOL);
        let age_str = fixedstr::str_format!(str32, "{:.0}", pet.age.as_mins());
        display.render_text(
            Vec2::new(
                STOMACH_END_X as f32 + assets::IMAGE_AGE_SYMBOL.size.x as f32,
                -1.,
            ),
            &age_str,
        );

        let money_str = fixedstr::str_format!(str32, "${}", args.game_ctx.money);
        display.render_text_complex(
            Vec2::new(STOMACH_END_X as f32, 10.),
            &money_str,
            ComplexRenderOption::new()
                .with_white()
                .with_font(&FONT_VARIABLE_SMALL),
        );

        display.render_rect_solid(TOP_BORDER_RECT, true);

        const BOTTOM_BORDER_RECT: Rect = Rect::new_center(
            Vec2::new(CENTER_X, WONDER_RECT.pos_top_left().y + WONDER_RECT.size.y),
            Vec2::new(WIDTH_F32, BORDER_HEIGHT),
        );

        const SYMBOL_BUFFER: f32 = 2.;
        const IMAGE_Y_START: f32 = BOTTOM_BORDER_RECT.pos.y + BORDER_HEIGHT + SYMBOL_BUFFER;

        match self.state {
            State::Wondering => {
                display.render_complex(&self.top_render);
                display.render_complex(&self.left_render);
                display.render_complex(&self.right_render);
            }
            State::Sleeping => {
                if args.game_ctx.inventory.has_item(ItemKind::AnalogClock) {
                    display.render_complex(
                        &AnalogueRenderClock::new(
                            AnalogueClockKind::Clock21,
                            Vec2::new(CENTER_X, TOP_BORDER_RECT.y2() + 21. / 2.),
                            args.timestamp.inner().time(),
                        )
                        .without_second_hand(),
                    );
                }

                display.render_sprite(&self.sleeping_z);
            }
            State::WatchingTv {
                show_timer: _,
                show_end: _,
                watch_end: _,
            } => {
                display.render_complex(&self.tv);
            }
        }

        display.render_sprite(&self.pet_render);

        display.render_sprites(&self.poops);

        let options = get_options(self.state);

        const SIZE: Vec2 = Vec2::new(
            assets::IMAGE_POOP_SYMBOL.size.x as f32,
            assets::IMAGE_POOP_SYMBOL.size.y as f32,
        );

        for i in 0..options.len() {
            let image = match options[i] {
                MenuOption::Poop => &assets::IMAGE_POOP_SYMBOL,
                MenuOption::PetInfo => &assets::IMAGE_INFO_SYMBOL,
                MenuOption::GameSelect => &assets::IMAGE_GAME_SYMBOL,
                MenuOption::FoodSelect => self.food_anime.current_frame(),
                MenuOption::Shop => &assets::IMAGE_SHOP_SYMBOL,
                MenuOption::Inventory => &assets::IMAGE_SYMBOL_INVENTORY,
            };
            let x = if self.selected_index > 0 {
                let x_index = i as i32 - self.selected_index as i32 + 1;
                SYMBOL_BUFFER + (x_index as f32 * (SIZE.x + SYMBOL_BUFFER))
            } else {
                SYMBOL_BUFFER + ((i + 1) as f32 * (SIZE.x + SYMBOL_BUFFER))
            };
            display.render_image_top_left(x as i32, IMAGE_Y_START as i32, image);
        }

        let select_rect = Rect::new_top_left(
            Vec2::new(
                SYMBOL_BUFFER + (1 as f32 * (SIZE.x + SYMBOL_BUFFER)) - (SYMBOL_BUFFER),
                IMAGE_Y_START - (SYMBOL_BUFFER),
            ),
            Vec2::new(SIZE.x + SYMBOL_BUFFER * 2., SIZE.y + SYMBOL_BUFFER * 2.),
        );
        display.render_rect_outline(select_rect, true);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HomeFurnitureLocation {
    Top,
    Left,
    Right,
}

impl HomeFurnitureLocation {
    pub const fn pos(&self) -> Vec2 {
        match self {
            HomeFurnitureLocation::Top => Vec2::new(CENTER_X, TOP_BORDER_RECT.y2()),
            HomeFurnitureLocation::Left => Vec2::new(0., HEIGHT_F32 / 2.),
            HomeFurnitureLocation::Right => Vec2::new(WIDTH_F32, HEIGHT_F32 / 2.),
        }
    }

    pub const fn index(&self) -> usize {
        match self {
            HomeFurnitureLocation::Top => 0,
            HomeFurnitureLocation::Left => 1,
            HomeFurnitureLocation::Right => 2,
        }
    }

    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(HomeFurnitureLocation::Top),
            1 => Some(HomeFurnitureLocation::Left),
            2 => Some(HomeFurnitureLocation::Right),
            _ => None,
        }
    }
}

pub enum HomeFurnitureRender {
    None,
    DigitalClock(DigitalClockRender),
    AnalogueClock(AnalogueRenderClock),
    FishTank(FishTankRender),
    Sprite(BasicSprite),
}

impl HomeFurnitureRender {
    pub fn new(location: HomeFurnitureLocation, kind: HomeFurnitureKind) -> Self {
        let pos = location.pos()
            + match location {
                HomeFurnitureLocation::Top => Vec2::new(0., kind.size().y / 2.),
                HomeFurnitureLocation::Left => Vec2::new(kind.size().x / 2. + 1., 0.),
                HomeFurnitureLocation::Right => Vec2::new(-(kind.size().x / 2. + 1.), 0.),
            };

        match kind {
            HomeFurnitureKind::None => HomeFurnitureRender::None,
            HomeFurnitureKind::DigitalClock => {
                HomeFurnitureRender::DigitalClock(DigitalClockRender::new(pos, Default::default()))
            }
            HomeFurnitureKind::AnalogueClock => HomeFurnitureRender::AnalogueClock(
                AnalogueRenderClock::new(AnalogueClockKind::Clock21, pos, Default::default()),
            ),
            HomeFurnitureKind::FishTank => HomeFurnitureRender::FishTank(FishTankRender::new(pos)),
            HomeFurnitureKind::PaintingBranch => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_PAINTING_BRANCH))
            }
            HomeFurnitureKind::PaintingDude => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_PAINTING_DUDE))
            }
            HomeFurnitureKind::PaintingMan => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_PAINTING_MAN))
            }
            HomeFurnitureKind::PaintingPc => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_PAINTING_PC))
            }
            HomeFurnitureKind::PaintingSun => {
                HomeFurnitureRender::Sprite(BasicSprite::new(pos, &assets::IMAGE_PAINTING_SUN))
            }
        }
    }

    pub fn tick(&mut self, args: &mut SceneTickArgs) {
        match self {
            HomeFurnitureRender::None => {}
            HomeFurnitureRender::DigitalClock(digital_clock_render) => {
                digital_clock_render.update_time(&args.timestamp.inner().time());
            }
            HomeFurnitureRender::AnalogueClock(analogue_render_clock) => {
                analogue_render_clock.update_time(&args.timestamp.inner().time());
            }
            HomeFurnitureRender::FishTank(fishtank_render) => {
                fishtank_render.tick(args.delta, &mut args.game_ctx.rng);
            }
            HomeFurnitureRender::Sprite(_) => {}
        }
    }
}

impl ComplexRender for HomeFurnitureRender {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        match self {
            HomeFurnitureRender::None => {}
            HomeFurnitureRender::DigitalClock(digital_clock_render) => {
                display.render_complex(digital_clock_render)
            }
            HomeFurnitureRender::AnalogueClock(analogue_render_clock) => {
                display.render_complex(analogue_render_clock)
            }
            HomeFurnitureRender::FishTank(fishtank_render) => {
                display.render_complex(fishtank_render)
            }
            HomeFurnitureRender::Sprite(basic_sprite) => display.render_sprite(basic_sprite),
        }
    }
}
