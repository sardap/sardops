use core::time::Duration;

use fixedstr::str32;
use glam::Vec2;

use crate::{
    anime::{tick_all_anime, Anime, HasAnime},
    assets::{self, Image, IMAGE_STOMACH_MASK},
    clock::{ClockKind, RenderClock},
    date_utils::DurationExt,
    display::{ComplexRenderOption, GameDisplay, CENTER_VEC, CENTER_X, CENTER_Y, WIDTH_F32},
    fish_tank::FishtankRender,
    fonts::FONT_VARIABLE_SMALL,
    geo::{vec2_direction, vec2_distance, Rect},
    items::ItemKind,
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
    sprite::{BasicAnimeSprite, Sprite},
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
    fish_tank: FishtankRender,
    state: State,
    state_elapsed: Duration,
    wonder_end: Duration,
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
            fish_tank: FishtankRender::new(Vec2::new(15., 70.)),
            state: State::Wondering,
            state_elapsed: Duration::ZERO,
            wonder_end: Duration::ZERO,
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

        for fish in args.game_ctx.home_fish_tank.fish {
            if fish == 0. {
                break;
            }
            self.fish_tank.add_fish(&mut args.game_ctx.rng, fish);
        }
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        let pet = &mut args.game_ctx.pet;
        let rng = &mut args.game_ctx.rng;
        self.pet_render.set_def_id(pet.def_id);

        update_poop_renders(&mut self.poops, &args.game_ctx.poops);

        self.food_anime.tick(args.delta);
        self.pet_render.tick(args.delta);
        tick_all_anime(&mut self.poops, args.delta);

        let should_be_sleeping = pet.definition().should_be_sleeping(&args.timestamp);
        if should_be_sleeping && !matches!(self.state, State::Sleeping) {
            self.change_state(State::Sleeping);
        } else if !should_be_sleeping && matches!(self.state, State::Sleeping) {
            self.change_state(State::Wondering);
        }

        if !matches!(self.state, State::Sleeping) {
            if let Some(cause_of_death) = pet.should_die() {
                return SceneOutput::new(SceneEnum::Death(DeathScene::new(
                    cause_of_death,
                    args.game_ctx.pet.def_id,
                )));
            }

            if let Some(next_pet_id) = pet.should_evolve(rng) {
                return SceneOutput::new(SceneEnum::Evovle(EvolveScene::new(
                    pet.def_id,
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
                self.fish_tank.tick(args.delta, rng);

                if self.state_elapsed > self.wonder_end {
                    // if args.game_ctx.inventory.has_item(ItemKind::TvLcd) {
                    //     self.tv.kind = TvKind::LCD;
                    //     self.change_state(State::WatchingTv {
                    //         show_timer: Duration::ZERO,
                    //         show_end: Duration::from_secs(30),
                    //         watch_end: Duration::from_secs(3 * 60),
                    //     });
                    // }
                    // if args.game_ctx.inventory.has_item(ItemKind::TvCrt) {
                    //     self.tv.kind = TvKind::CRT;
                    //     self.change_state(State::WatchingTv {
                    //         show_timer: Duration::ZERO,
                    //         show_end: Duration::from_secs(30),
                    //         watch_end: Duration::from_secs(2 * 60),
                    //     });
                    // }

                    self.wonder_end = reset_wonder_end(rng);
                }

                self.pet_render
                    .set_animation(pet.mood(&args.game_ctx.poops).anime_set());

                let dist = vec2_distance(self.pet_render.pos, self.target);
                if dist.abs() < 5. {
                    self.target = self.wonder_rect().random_point_inside(rng);
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

        const BORDER_HEIGHT: f32 = 1.;

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

        const TOP_BORDER_RECT: Rect = Rect::new_center(
            Vec2::new(CENTER_X, 24.),
            Vec2::new(WIDTH_F32, BORDER_HEIGHT),
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
                if args.game_ctx.inventory.has_item(ItemKind::AnalogClock) {
                    display.render_complex(
                        &RenderClock::new(
                            ClockKind::Clock21,
                            Vec2::new(CENTER_X, TOP_BORDER_RECT.y2() + 21. / 2.),
                            args.timestamp.inner().time(),
                        )
                        .without_second_hand(),
                    );
                }

                display.render_complex(&self.fish_tank);
            }
            State::Sleeping => {
                if args.game_ctx.inventory.has_item(ItemKind::AnalogClock) {
                    display.render_complex(
                        &RenderClock::new(
                            ClockKind::Clock21,
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
