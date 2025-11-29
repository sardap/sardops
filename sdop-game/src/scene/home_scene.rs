use core::{time::Duration, u8};

use chrono::Timelike;
use fixedstr::{str32, str_format};
use glam::Vec2;
use sdop_common::{MelodyEntry, Note};

use crate::{
    anime::{tick_all_anime, Anime, HasAnime, MaskedAnimeRender},
    assets::{self, Image, FRAMES_SKULL, FRAMES_SKULL_MASK, IMAGE_STOMACH_MASK},
    date_utils::DurationExt,
    display::{
        ComplexRenderOption, GameDisplay, CENTER_VEC, CENTER_X, CENTER_Y, HEIGHT_F32, WIDTH_F32,
    },
    egg::EggRender,
    fonts::FONT_VARIABLE_SMALL,
    furniture::{HomeFurnitureKind, HomeFurnitureLocation, HomeFurnitureRender},
    game_context::GameContext,
    geo::{vec2_direction, vec2_distance, Rect},
    items::ItemKind,
    pc::{PcKind, PcRender},
    pet::{
        definition::{PetAnimationSet, PET_BRAINO_ID},
        render::PetRender,
        LifeStage, Mood,
    },
    poop::{poop_count, update_poop_renders, PoopRender, MAX_POOPS},
    scene::{
        death_scene::DeathScene, egg_hatch_scene::EggHatchScene, evolve_scene::EvolveScene,
        food_select::FoodSelectScene, game_select::GameSelectScene, heal_scene::HealScene,
        inventory_scene::InventoryScene, pet_info_scene::PetInfoScene,
        pet_records_scene::PetRecordsScene, place_furniture_scene::PlaceFurnitureScene,
        poop_clear_scene::PoopClearScene, settings_scene::SettingsScene, shop_scene::ShopScene,
        suiters_scene::SuitersScene, weekday_select_scene::WeekdaySelectScene, RenderArgs, Scene,
        SceneEnum, SceneOutput, SceneTickArgs,
    },
    sounds::{SongPlayOptions, SONG_ALARM, SONG_HUNGRY, SONG_POOPED, SONG_SICK},
    sprite::{BasicAnimeSprite, MusicNote, Snowflake, Sprite},
    temperature::TemperatureLevel,
    tv::{get_show_for_time, TvKind, TvRender, SHOW_RUN_TIME},
    Button, Song, Timestamp, WIDTH,
};

const WONDER_SPEED: f32 = 5.;
const DANCING_SPEED: f32 = 15.;
pub const WONDER_RECT: Rect = Rect::new_center(CENTER_VEC, Vec2::new(WIDTH as f32, 90.0));
pub const DANCING_RECT: Rect = Rect::new_center(CENTER_VEC, Vec2::new(10., 10.));

pub const GREATER_WONDER_RECT: Rect = WONDER_RECT.grow(50.);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MenuOption {
    PetInfo,
    Breed,
    Poop,
    Heal,
    GameSelect,
    FoodSelect,
    Shop,
    Inventory,
    PlaceFurniture,
    PetRecords,
    Settings,
}

impl MenuOption {
    pub fn get_song(&self) -> &'static Song {
        const MENU_DURATION: i16 = 8;
        match self {
            MenuOption::PetInfo => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::C2, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::Breed => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::C3, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::Poop => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::C4, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::Heal => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::C5, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::GameSelect => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::D2, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::FoodSelect => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::D3, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::Shop => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::D4, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::Inventory => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::D5, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::PlaceFurniture => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::E3, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::PetRecords => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::E4, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::Settings => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::D3, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
        }
    }
}

fn change_option(options: &[MenuOption], current: usize, change: i32) -> usize {
    let index = current as i32 + change;

    if index >= options.len() as i32 {
        0usize
    } else if index < 0 {
        options.len() - 1
    } else {
        index as usize
    }
}

const MENU_OPTIONS_COUNT: usize = core::mem::variant_count::<MenuOption>();
type MenuOptions = heapless::Vec<MenuOption, MENU_OPTIONS_COUNT>;

// SLOW POINT
fn get_options(state: State, game_ctx: &GameContext) -> MenuOptions {
    let mut result = heapless::Vec::new();
    let _ = result.push(MenuOption::PetInfo);
    let _ = result.push(MenuOption::Settings);
    if game_ctx.suiter_system.suiter_waiting() {
        let _ = result.push(MenuOption::Breed);
    }
    if game_ctx.inventory.has_any_item() {
        let _ = result.push(MenuOption::Inventory);
    }
    if game_ctx.inventory.has_any_furniture() {
        let _ = result.push(MenuOption::PlaceFurniture);
    }

    if game_ctx.poop_count() > 0 {
        let _ = result.push(MenuOption::Poop);
    }

    if game_ctx.pet_records.count() > 0 {
        let _ = result.push(MenuOption::PetRecords);
    }

    if game_ctx.pet.is_ill() {
        let _ = result.push(MenuOption::Heal);
    }

    if state != State::Sleeping {
        let _ = result.push(MenuOption::GameSelect);
        let _ = result.push(MenuOption::FoodSelect);
        let _ = result.push(MenuOption::Shop);
    }

    result.sort_unstable();

    result
}

const BORDER_HEIGHT: f32 = 1.;

pub const HOME_SCENE_TOP_BORDER_RECT: Rect = Rect::new_center(
    Vec2::new(CENTER_X, 24.),
    Vec2::new(WIDTH_F32, BORDER_HEIGHT),
);

pub const HOME_SCENE_TOP_AREA_RECT: Rect = Rect::new_top_left(
    Vec2::new(0., 0.),
    Vec2::new(WIDTH_F32, HOME_SCENE_TOP_BORDER_RECT.y2()),
);

const PROGRAM_RUN_TIME_RANGE: core::ops::Range<Duration> =
    Duration::from_secs(30)..Duration::from_mins(3);

const BOOK_POS: Vec2 = Vec2::new(CENTER_X, 90.);

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Wondering,
    Sleeping,
    WatchingTv {
        last_checked: u8,
        watch_end: Duration,
    },
    PlayingComputer {
        program_run_time: Duration,
        program_end_time: Duration,
        watch_end: Duration,
    },
    ReadingBook {
        book: ItemKind,
    },
    PlayingMp3 {
        jam_end_time: Duration,
    },
    Alarm,
}

struct Word {
    pos: Vec2,
    dir: Vec2,
    text: &'static str,
}

enum Weather {
    None,
    Cold,
    Snow,
    Hot,
}

impl From<TemperatureLevel> for Weather {
    fn from(value: TemperatureLevel) -> Self {
        match value {
            TemperatureLevel::VeryHot | TemperatureLevel::Hot => Self::Hot,
            TemperatureLevel::Pleasant => Self::None,
            TemperatureLevel::Cold => Self::Cold,
            TemperatureLevel::VeryCold => Self::Snow,
        }
    }
}

pub struct HomeSceneData {
    pet_render: PetRender,
    poops: [Option<PoopRender>; MAX_POOPS],
    target: Vec2,
    food_anime: Anime,
    options: MenuOptions,
    selected_index: usize,
    sleeping_z: BasicAnimeSprite,
    tv: TvRender,
    pc: PcRender,
    next_word_spawn: Duration,
    floating_words: [Option<Word>; 5],
    state: State,
    state_elapsed: Duration,
    wonder_end: Duration,
    skull: MaskedAnimeRender,
    weather: Weather,
    shake_duration: Duration,
    shake_right: bool,
    snow_flakes: [Snowflake; 30],
    music_notes: [MusicNote; 7],
    last_poop_count: usize,
    last_poop_sound: Timestamp,
    last_is_sick: bool,
    last_was_hungry: bool,
}

impl Default for HomeSceneData {
    fn default() -> Self {
        Self {
            pet_render: PetRender::default(),
            poops: [None; MAX_POOPS],
            target: Vec2::default(),
            food_anime: Anime::new(&assets::FRAMES_FOOD_SYMBOL),
            options: heapless::Vec::new(),
            selected_index: 0,
            sleeping_z: BasicAnimeSprite::new(CENTER_VEC, &assets::FRAMES_SLEEPING_Z),
            tv: TvRender::new(
                TvKind::Lcd,
                Vec2::new(20., 40.),
                &assets::FRAMES_TV_SHOW_SPORT,
            ),
            pc: PcRender::new(
                PcKind::Desktop,
                Vec2::new(20., 50.),
                &assets::FRAMES_PC_PROGRAM_RTS,
            ),
            floating_words: Default::default(),
            next_word_spawn: Duration::ZERO,
            state: State::Wondering,
            state_elapsed: Duration::ZERO,
            wonder_end: Duration::ZERO,
            skull: MaskedAnimeRender::new(CENTER_VEC, &FRAMES_SKULL, &FRAMES_SKULL_MASK),
            weather: Weather::None,
            shake_duration: Duration::ZERO,
            shake_right: false,
            snow_flakes: Default::default(),
            music_notes: Default::default(),
            last_poop_count: 0,
            last_poop_sound: Timestamp::default(),
            last_is_sick: false,
            last_was_hungry: false,
        }
    }
}

impl HomeSceneData {
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
        self.state_elapsed = Duration::ZERO;
    }
}

fn wonder_end(args: &mut SceneTickArgs) {
    #[derive(Debug, Copy, Clone)]
    enum Activity {
        PlayingComputer,
        WatchTv,
        ReadBook,
        ListenMusic,
    }

    // Just set to 10 who cares you will forget to update the number at some point and cause some issue
    let mut options: heapless::Vec<Activity, 10> = heapless::Vec::new();
    if args.game_ctx.pet.definition().life_stage == LifeStage::Adult
        && args.game_ctx.inventory.has_item(ItemKind::PersonalComputer)
        && args.game_ctx.inventory.has_item(ItemKind::Screen)
        && args.game_ctx.inventory.has_item(ItemKind::Keyboard)
        && args.game_ctx.pet.def_id != PET_BRAINO_ID
    {
        let _ = options.push(Activity::PlayingComputer);
    }
    if args.game_ctx.pet.definition().life_stage != LifeStage::Baby
        && (args.game_ctx.inventory.has_item(ItemKind::TvLcd)
            || args.game_ctx.inventory.has_item(ItemKind::TvCrt))
        && args.game_ctx.pet.def_id != PET_BRAINO_ID
    {
        let _ = options.push(Activity::WatchTv);
    }
    if args
        .game_ctx
        .pet
        .book_history
        .has_book_to_read(&args.game_ctx.inventory)
    {
        let _ = options.push(Activity::ReadBook);
    }
    if args.game_ctx.inventory.has_item(ItemKind::Mp3Player)
        && args.game_ctx.pet.mood() == Mood::Happy
    {
        let _ = options.push(Activity::ListenMusic);
    }

    if !options.is_empty() {
        let option = args.game_ctx.rng.choice(options.iter()).cloned().unwrap();

        match option {
            Activity::PlayingComputer => {
                args.game_ctx.home.change_state(State::PlayingComputer {
                    watch_end: reset_wonder_end(&mut args.game_ctx.rng),
                    program_end_time: Duration::from_millis(args.game_ctx.rng.u64(
                        PROGRAM_RUN_TIME_RANGE.start.as_millis() as u64
                            ..PROGRAM_RUN_TIME_RANGE.end.as_millis() as u64,
                    )),
                    program_run_time: Duration::ZERO,
                });
            }
            Activity::WatchTv => {
                let mut kinds: heapless::Vec<TvKind, 2> = Default::default();
                if args.game_ctx.inventory.has_item(ItemKind::TvLcd) {
                    let _ = kinds.push(TvKind::Lcd);
                }

                if args.game_ctx.inventory.has_item(ItemKind::TvCrt) {
                    let _ = kinds.push(TvKind::Crt);
                }
                args.game_ctx.home.tv.kind =
                    args.game_ctx.rng.choice(kinds.iter()).cloned().unwrap();

                args.game_ctx.home.change_state(State::WatchingTv {
                    last_checked: u8::MAX,
                    watch_end: reset_wonder_end(&mut args.game_ctx.rng),
                });
            }
            Activity::ReadBook => {
                let book_history = &args.game_ctx.pet.book_history;
                let inventory = &args.game_ctx.inventory;
                args.game_ctx.home.change_state(State::ReadingBook {
                    book: book_history.get_reading_book(inventory).unwrap_or(
                        book_history
                            .pick_random_unread_book(&mut args.game_ctx.rng, inventory)
                            .unwrap_or_default(),
                    ),
                });
            }
            Activity::ListenMusic => {
                args.game_ctx.home.pet_render.pos = CENTER_VEC;
                args.game_ctx.home.target = CENTER_VEC;
                args.game_ctx.home.change_state(State::PlayingMp3 {
                    jam_end_time: Duration::from_secs(args.game_ctx.rng.u64(60..300)),
                });
            }
        }
    }

    args.game_ctx.home.wonder_end = reset_wonder_end(&mut args.game_ctx.rng);
}

pub struct HomeScene {
    left_render: HomeFurnitureRender,
    top_render: HomeFurnitureRender,
    right_render: HomeFurnitureRender,
    egg_render: EggRender,
    egg_bounce: f32,
}

impl Default for HomeScene {
    fn default() -> Self {
        Self::new()
    }
}

impl HomeScene {
    pub fn new() -> Self {
        Self {
            left_render: HomeFurnitureRender::None,
            top_render: HomeFurnitureRender::None,
            right_render: HomeFurnitureRender::None,
            egg_render: Default::default(),
            egg_bounce: 0.,
        }
    }
}

fn reset_wonder_end(rng: &mut fastrand::Rng) -> Duration {
    Duration::from_secs(rng.u64(0..(5 * 60)))
}

impl Scene for HomeScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        if args.game_ctx.home.wonder_end == Duration::ZERO {
            args.game_ctx.home.last_poop_count = poop_count(&args.game_ctx.poops);
            args.game_ctx.home.last_is_sick = args.game_ctx.pet.is_ill();
            args.game_ctx.home.last_was_hungry = args.game_ctx.pet.is_starving();

            args.game_ctx.home.wonder_end = reset_wonder_end(&mut args.game_ctx.rng);
            args.game_ctx.home.pet_render.pos = args
                .game_ctx
                .home
                .wonder_rect()
                .random_point_inside(&mut args.game_ctx.rng);
            args.game_ctx.home.target = args.game_ctx.home.pet_render.pos;

            for flake in &mut args.game_ctx.home.snow_flakes {
                flake.reset(true, &mut args.game_ctx.rng);
            }

            for note in &mut args.game_ctx.home.music_notes {
                note.pos = Vec2::new(-100., -100.);
            }
        }

        self.egg_render.pos = Vec2::new(
            WIDTH_F32 - assets::IMAGE_EGG.size.x as f32,
            WONDER_RECT.y2() - assets::IMAGE_EGG.size.y as f32,
        );
        if let Some(egg) = &args.game_ctx.egg {
            self.egg_render.set_pid(egg.upid);
        }

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
        args.game_ctx
            .home
            .pet_render
            .set_def_id(args.game_ctx.pet.def_id);

        update_poop_renders(&mut args.game_ctx.home.poops, &args.game_ctx.poops);

        args.game_ctx.home.food_anime.tick(args.delta);
        args.game_ctx.home.pet_render.tick(args.delta);
        tick_all_anime(&mut args.game_ctx.home.poops, args.delta);

        let should_be_sleeping = args
            .game_ctx
            .pet
            .definition()
            .should_be_sleeping(&args.timestamp);
        if should_be_sleeping && !matches!(args.game_ctx.home.state, State::Sleeping) {
            args.game_ctx.home.change_state(State::Sleeping);
        } else if !should_be_sleeping && matches!(args.game_ctx.home.state, State::Sleeping) {
            args.game_ctx.home.change_state(State::Wondering);
        }

        // Penidng sounds
        {
            let poop_count = poop_count(&args.game_ctx.poops);
            if args.game_ctx.home.last_poop_count != poop_count
                || (args.game_ctx.home.last_poop_count > 0
                    && args.timestamp - args.game_ctx.home.last_poop_sound
                        > Duration::from_hours(1))
            {
                args.game_ctx.home.last_poop_sound = args.timestamp;
                args.game_ctx.home.last_poop_count = poop_count;
                if poop_count > 0 {
                    args.game_ctx
                        .sound_system
                        .push_song(SONG_POOPED, SongPlayOptions::new().with_essential());
                }
            }
        }

        {
            if args.game_ctx.pet.is_ill() != args.game_ctx.home.last_is_sick {
                if args.game_ctx.pet.is_ill() {
                    args.game_ctx
                        .sound_system
                        .push_song(SONG_SICK, SongPlayOptions::new().with_essential());
                }
                args.game_ctx.home.last_is_sick = args.game_ctx.pet.is_ill();
            }
        }

        {
            if args.game_ctx.pet.is_starving() != args.game_ctx.home.last_was_hungry {
                if args.game_ctx.pet.is_starving() {
                    args.game_ctx
                        .sound_system
                        .push_song(SONG_HUNGRY, SongPlayOptions::new().with_essential());
                }
                args.game_ctx.home.last_was_hungry = args.game_ctx.pet.is_starving();
            }
        }

        const EGG_BOUNCE_SPEED: f32 = 3.;
        const BOUNCE_RANGE: i32 = 5;

        self.egg_bounce += EGG_BOUNCE_SPEED * args.delta.as_secs_f32();
        self.egg_render.pos = Vec2::new(
            self.egg_render.pos.x,
            WONDER_RECT.y2() - assets::IMAGE_EGG.size.y as f32
                + if (self.egg_bounce as i32) % BOUNCE_RANGE * 2 > BOUNCE_RANGE {
                    BOUNCE_RANGE * 2 + -(self.egg_bounce as i32 % BOUNCE_RANGE * 2)
                } else {
                    self.egg_bounce as i32 % BOUNCE_RANGE * 2
                } as f32,
        );

        if !matches!(args.game_ctx.home.state, State::Sleeping) {
            if let Some(egg) = &args.game_ctx.egg {
                if egg.hatch {
                    return SceneOutput::new(SceneEnum::EggHatch(EggHatchScene::new(
                        *egg,
                        args.game_ctx.pet.def_id,
                    )));
                }
            }

            if let Some(cause_of_death) = args.game_ctx.pet.should_die() {
                return SceneOutput::new(SceneEnum::Death(DeathScene::new(
                    cause_of_death,
                    args.game_ctx.pet.def_id,
                )));
            }

            if let Some(next_pet_id) = args.game_ctx.pet.should_evolve() {
                return SceneOutput::new(SceneEnum::Evovle(EvolveScene::new(
                    args.game_ctx.pet.def_id,
                    next_pet_id,
                )));
            }
        }

        args.game_ctx.home.options = get_options(args.game_ctx.home.state, args.game_ctx);

        if args.game_ctx.pet.is_ill() {
            args.game_ctx.home.skull.pos = args.game_ctx.home.pet_render.pos
                - Vec2::new(
                    0.,
                    args.game_ctx.home.pet_render.anime.current_frame().size.y as f32 / 2.
                        + args.game_ctx.home.skull.anime.current_frame().size.y as f32 / 2.
                        + 2.,
                );
            args.game_ctx.home.skull.anime().tick(args.delta);
        }

        args.game_ctx.home.state_elapsed += args.delta;

        args.game_ctx.home.weather = TemperatureLevel::from(args.input.temperature()).into();

        match args.game_ctx.home.weather {
            Weather::None => {}
            Weather::Cold => {}
            Weather::Snow => {
                for flake in &mut args.game_ctx.home.snow_flakes {
                    flake.pos += flake.dir * args.delta.as_secs_f32();
                    if flake.pos.y > HEIGHT_F32 + assets::IMAGE_SNOWFLAKE.size.y as f32
                        || flake.pos.x > WIDTH_F32 + assets::IMAGE_SNOWFLAKE.size.x as f32
                        || flake.pos.x < -(assets::IMAGE_SNOWFLAKE.size.x as f32)
                    {
                        flake.reset(false, &mut args.game_ctx.rng);
                    }
                }
            }
            Weather::Hot => {}
        }

        if matches!(args.game_ctx.home.state, State::Wondering)
            || matches!(
                args.game_ctx.home.state,
                State::PlayingMp3 { jam_end_time: _ }
            )
            || matches!(args.game_ctx.home.state, State::Alarm)
        {
            self.top_render.tick(args);
            self.left_render.tick(args);
            self.right_render.tick(args);
        }

        match args.game_ctx.home.state {
            State::Wondering => {
                if args.game_ctx.home.state_elapsed > args.game_ctx.home.wonder_end {
                    wonder_end(args);
                }

                args.game_ctx
                    .home
                    .pet_render
                    .set_animation(args.game_ctx.pet.mood().anime_set());

                let dist =
                    vec2_distance(args.game_ctx.home.pet_render.pos, args.game_ctx.home.target);
                if dist.abs() < 5. {
                    args.game_ctx.home.target = args
                        .game_ctx
                        .home
                        .wonder_rect()
                        .random_point_inside(&mut args.game_ctx.rng);
                }

                args.game_ctx.home.pet_render.pos +=
                    vec2_direction(args.game_ctx.home.pet_render.pos, args.game_ctx.home.target)
                        * WONDER_SPEED
                        * if matches!(args.game_ctx.home.weather, Weather::None) {
                            1.
                        } else {
                            0.5
                        }
                        * args.delta.as_secs_f32();
            }
            State::Sleeping => {
                self.top_render.tick(args);
                self.left_render.tick(args);
                self.right_render.tick(args);

                args.game_ctx
                    .home
                    .pet_render
                    .set_animation(PetAnimationSet::Sleeping);

                args.game_ctx.home.sleeping_z.anime().tick(args.delta);

                args.game_ctx.home.pet_render.pos = CENTER_VEC + Vec2::new(0., 10.);
                args.game_ctx.home.sleeping_z.pos = Vec2::new(
                    args.game_ctx.home.pet_render.pos.x
                        + (args.game_ctx.home.pet_render.image().size_vec2().x * 0.5),
                    args.game_ctx.home.pet_render.pos.y
                        - (args.game_ctx.home.pet_render.image().size_vec2().y * 0.7),
                );
            }
            State::WatchingTv {
                mut last_checked,
                watch_end,
            } => {
                if args.game_ctx.home.state_elapsed > watch_end {
                    args.game_ctx.home.change_state(State::Wondering);
                } else {
                    if args.timestamp.inner().minute() as u8 / SHOW_RUN_TIME != last_checked {
                        last_checked = args.timestamp.inner().minute() as u8 / SHOW_RUN_TIME;
                        args.game_ctx.home.tv.change_show(
                            get_show_for_time(args.timestamp.inner()),
                            &mut args.game_ctx.rng,
                        );
                    }

                    args.game_ctx.home.tv.anime().tick(args.delta);

                    args.game_ctx.home.tv.pos = Vec2::new(
                        args.game_ctx.home.tv.size().x * 0.5 + 1.,
                        CENTER_Y - args.game_ctx.home.tv.size().y * 0.5,
                    );
                    args.game_ctx
                        .home
                        .pet_render
                        .set_animation(PetAnimationSet::Normal);
                    args.game_ctx.home.pet_render.pos = Vec2::new(
                        WIDTH_F32 - args.game_ctx.home.pet_render.image().size().x as f32 / 2. - 5.,
                        args.game_ctx.home.tv.pos.y
                            + args.game_ctx.home.tv.size().y / 2.
                            + args.game_ctx.home.pet_render.image().size().y as f32,
                    );

                    args.game_ctx.home.state = State::WatchingTv {
                        last_checked,
                        watch_end,
                    }
                }
            }
            State::PlayingComputer {
                watch_end,
                mut program_end_time,
                mut program_run_time,
            } => {
                args.game_ctx
                    .home
                    .pc
                    .tick(args.delta, &mut args.game_ctx.rng);
                args.game_ctx.home.pet_render.pos = Vec2::new(CENTER_X, CENTER_Y + 20.);
                program_run_time += args.delta;

                if program_run_time > program_end_time {
                    program_run_time = Duration::ZERO;
                    program_end_time = Duration::from_millis(args.game_ctx.rng.u64(
                        PROGRAM_RUN_TIME_RANGE.start.as_millis() as u64
                            ..PROGRAM_RUN_TIME_RANGE.end.as_millis() as u64,
                    ));
                    // Should probably make it always switch to the OS between programs
                    args.game_ctx.home.pc.change_program(
                        &mut args.game_ctx.rng,
                        &args.game_ctx.inventory,
                        &args.game_ctx.pet,
                    );
                }

                if args.game_ctx.home.state_elapsed > watch_end {
                    args.game_ctx.home.change_state(State::Wondering);
                } else {
                    args.game_ctx.home.state = State::PlayingComputer {
                        program_run_time,
                        program_end_time,
                        watch_end,
                    }
                }
            }
            State::ReadingBook { book } => {
                args.game_ctx.home.next_word_spawn = args
                    .game_ctx
                    .home
                    .next_word_spawn
                    .checked_sub(args.delta)
                    .unwrap_or_default();

                if args.game_ctx.home.next_word_spawn <= Duration::ZERO {
                    args.game_ctx.home.next_word_spawn =
                        Duration::from_millis(args.game_ctx.rng.u64(2500..10000));

                    // get first free index
                    let mut index = 0;
                    for i in 0..args.game_ctx.home.floating_words.len() {
                        if args.game_ctx.home.floating_words[i].is_none() {
                            index = i;
                            break;
                        }
                    }

                    const SPEED_RANGE: core::ops::Range<i32> = 5..15;

                    args.game_ctx.home.floating_words[index] = Some(Word {
                        pos: BOOK_POS,
                        dir: Vec2::new(
                            args.game_ctx.rng.i32(-10..10) as f32,
                            -(args.game_ctx.rng.i32(SPEED_RANGE) as f32),
                        ),
                        text: args
                            .game_ctx
                            .rng
                            .choice(book.book_info().word_bank.iter())
                            .cloned()
                            .unwrap_or(""),
                    })
                }

                for i in 0..args.game_ctx.home.floating_words.len() {
                    if args.game_ctx.home.floating_words[i].is_some() {
                        let outside = {
                            let word = &mut args.game_ctx.home.floating_words[i].as_mut().unwrap();
                            word.pos += Vec2::new(
                                word.dir.x * args.delta.as_secs_f32(),
                                word.dir.y * args.delta.as_secs_f32(),
                            );

                            GREATER_WONDER_RECT.point_inside(&word.pos)
                        };
                        if !outside {
                            args.game_ctx.home.floating_words[i] = None;
                        }
                    }
                }

                args.game_ctx.home.pet_render.pos = Vec2::new(
                    CENTER_X,
                    BOOK_POS.y - book.book_info().open_book.size.y as f32 / 2.,
                );

                if args.game_ctx.home.state_elapsed
                    > book.book_info().chapter_length(args.game_ctx.pet.def_id)
                {
                    args.game_ctx
                        .pet
                        .book_history
                        .get_mut_read(book)
                        .compelte_chapter();
                    args.game_ctx.home.change_state(State::Wondering);
                }
            }
            State::PlayingMp3 { jam_end_time } => {
                // Shake a bit
                let dist =
                    vec2_distance(args.game_ctx.home.pet_render.pos, args.game_ctx.home.target);
                if dist.abs() < 1. {
                    args.game_ctx.home.target =
                        DANCING_RECT.random_point_inside(&mut args.game_ctx.rng);
                }
                args.game_ctx.home.pet_render.pos +=
                    vec2_direction(args.game_ctx.home.pet_render.pos, args.game_ctx.home.target)
                        * DANCING_SPEED
                        * args.delta.as_secs_f32();

                for note in &mut args.game_ctx.home.music_notes {
                    note.pos += note.dir * args.delta.as_secs_f32();
                    if note.pos.y < -note.size().y as f32
                        || note.pos.y > HEIGHT_F32 + note.size().y as f32
                        || note.pos.x > WIDTH_F32 + note.size().x as f32
                        || note.pos.x < -(note.size().x as f32)
                    {
                        note.reset(args.game_ctx.home.pet_render.pos, &mut args.game_ctx.rng);
                    }
                }

                if args.game_ctx.home.state_elapsed > jam_end_time
                    || args.game_ctx.pet.mood() != Mood::Happy
                {
                    args.game_ctx.home.change_state(State::Wondering);
                }
            }
            State::Alarm => {
                args.game_ctx
                    .home
                    .pet_render
                    .set_animation(PetAnimationSet::Sad);
                args.game_ctx.home.pet_render.pos = CENTER_VEC;

                if !args.game_ctx.sound_system.get_playing() {
                    args.game_ctx
                        .sound_system
                        .push_song(SONG_ALARM, SongPlayOptions::new().with_essential());
                }

                if !args.game_ctx.alarm.should_be_rining() {
                    args.game_ctx.sound_system.clear_song();
                    args.game_ctx.home.change_state(State::Wondering);
                }
            }
        }

        if !matches!(args.game_ctx.home.state, State::Alarm)
            && args.game_ctx.alarm.should_be_rining()
            && args
                .game_ctx
                .home_layout
                .furniture_present(HomeFurnitureKind::Alarm)
        {
            args.game_ctx.home.change_state(State::Alarm);
        }

        if matches!(args.game_ctx.home.weather, Weather::Cold)
            || matches!(args.game_ctx.home.weather, Weather::Snow)
        {
            const SHAKE_X: f32 = 10.;
            args.game_ctx.home.shake_duration += args.delta;
            if args.game_ctx.home.shake_duration > Duration::from_millis(100) {
                args.game_ctx.home.shake_right = !args.game_ctx.home.shake_right;
                args.game_ctx.home.shake_duration = Duration::ZERO;
            }

            args.game_ctx.home.pet_render.pos.x += if args.game_ctx.home.shake_right {
                SHAKE_X
            } else {
                -SHAKE_X
            } * args.delta.as_secs_f32();
        }

        if matches!(args.game_ctx.home.state, State::Alarm) {
            if args.input.any_pressed() {
                args.game_ctx.alarm.ack();
            }
        } else {
            if args.input.pressed(Button::Left) {
                args.game_ctx.home.selected_index = change_option(
                    &args.game_ctx.home.options,
                    args.game_ctx.home.selected_index,
                    -1,
                );
            }
            if args.input.pressed(Button::Right) {
                args.game_ctx.home.selected_index = change_option(
                    &args.game_ctx.home.options,
                    args.game_ctx.home.selected_index,
                    1,
                );
            }

            if args.input.pressed(Button::Middle) {
                args.game_ctx.sound_system.push_song(
                    *args.game_ctx.home.options[args.game_ctx.home.selected_index].get_song(),
                    SongPlayOptions::new().with_effect(),
                );
                match args.game_ctx.home.options[args.game_ctx.home.selected_index] {
                    MenuOption::Breed => {
                        return SceneOutput::new(SceneEnum::Suiters(SuitersScene::new(
                            args.game_ctx.suiter_system.suiter.unwrap_or_default(),
                        )));
                    }
                    MenuOption::Poop => {
                        return SceneOutput::new(SceneEnum::PoopClear(PoopClearScene::new()));
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
                    MenuOption::PlaceFurniture => {
                        return SceneOutput::new(SceneEnum::PlaceFurniture(
                            PlaceFurnitureScene::new(),
                        ));
                    }
                    MenuOption::PetRecords => {
                        return SceneOutput::new(SceneEnum::PetRecords(PetRecordsScene::new()));
                    }
                    MenuOption::Heal => {
                        return SceneOutput::new(SceneEnum::Heal(HealScene::new()));
                    }
                    MenuOption::Settings => {
                        return SceneOutput::new(SceneEnum::Settings(SettingsScene::new()));
                    }
                };
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        const BOTTOM_BORDER_RECT: Rect = Rect::new_center(
            Vec2::new(CENTER_X, WONDER_RECT.pos_top_left().y + WONDER_RECT.size.y),
            Vec2::new(WIDTH_F32, BORDER_HEIGHT),
        );

        const SYMBOL_BUFFER: f32 = 2.;
        const IMAGE_Y_START: f32 = BOTTOM_BORDER_RECT.pos.y + BORDER_HEIGHT + SYMBOL_BUFFER;

        if matches!(args.game_ctx.home.state, State::Wondering)
            || matches!(args.game_ctx.home.state, State::Sleeping)
            || matches!(
                args.game_ctx.home.state,
                State::PlayingMp3 { jam_end_time: _ }
            )
            || matches!(args.game_ctx.home.state, State::Alarm)
        {
            display.render_complex(&self.top_render);
            display.render_complex(&self.left_render);
            display.render_complex(&self.right_render);
        }

        if args.game_ctx.pet.is_ill() {
            display.render_complex(&args.game_ctx.home.skull);
        }

        match args.game_ctx.home.state {
            State::Wondering => {
                display.render_sprite(&args.game_ctx.home.pet_render);
            }
            State::Sleeping => {
                display.render_sprite(&args.game_ctx.home.sleeping_z);
                display.render_sprite(&args.game_ctx.home.pet_render);
            }
            State::WatchingTv {
                last_checked: _,
                watch_end: _,
            } => {
                display.render_complex(&args.game_ctx.home.tv);
                display.render_sprite(&args.game_ctx.home.pet_render);
            }
            State::PlayingComputer {
                watch_end: _,
                program_end_time: _,
                program_run_time: _,
            } => {
                display.render_complex(&args.game_ctx.home.pc);
                display.render_sprite(&args.game_ctx.home.pet_render);
            }
            State::ReadingBook { book } => {
                display.render_text_complex(
                    Vec2::new(CENTER_X, 34.),
                    "CHAPTER",
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                let current_chapter = args.game_ctx.pet.book_history.get_read(book).chapters();
                let str = str_format!(
                    fixedstr::str24,
                    "{} of {}",
                    current_chapter + 1,
                    book.book_info().chapters
                );
                display.render_text_complex(
                    Vec2::new(CENTER_X, 40.),
                    &str,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                let percent_complete = args.game_ctx.home.state_elapsed.as_millis_f32()
                    / book
                        .book_info()
                        .chapter_length(args.game_ctx.pet.def_id)
                        .as_millis_f32();
                let str = str_format!(fixedstr::str24, "{:.0}%", percent_complete * 100.,);
                display.render_text_complex(
                    Vec2::new(CENTER_X, 46.),
                    &str,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL),
                );

                for word in args.game_ctx.home.floating_words.iter().flatten() {
                    display.render_text_complex(
                        word.pos,
                        word.text,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                }

                display.render_sprite(&args.game_ctx.home.pet_render);

                display.render_image_complex(
                    BOOK_POS.x as i32,
                    BOOK_POS.y as i32,
                    book.book_info().open_book,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_black()
                        .with_center(),
                );
            }
            State::PlayingMp3 { jam_end_time: _ } => {
                display.render_sprite(&args.game_ctx.home.pet_render);
                display.render_image_complex(
                    args.game_ctx.home.pet_render.x2() as i32,
                    args.game_ctx.home.pet_render.pos.y as i32,
                    &assets::IMAGE_PORTABLE_AUDIO_PLAYER_0,
                    ComplexRenderOption::new().with_white().with_black(),
                );

                for note in &args.game_ctx.home.music_notes {
                    display.render_sprite(note);
                }
            }
            State::Alarm => {
                display.render_sprite(&args.game_ctx.home.pet_render);
            }
        }

        display.render_sprites(&args.game_ctx.home.poops);

        if matches!(args.game_ctx.home.weather, Weather::Snow) {
            for flake in &args.game_ctx.home.snow_flakes {
                if flake.pos.y > -((assets::IMAGE_SNOWFLAKE.size.y / 2) as f32) {
                    display.render_sprite(flake);
                }
            }
        }

        let pet = &args.game_ctx.pet;

        display.render_rect_solid(HOME_SCENE_TOP_AREA_RECT, false);

        let total_filled = pet.stomach_filled / pet.definition().stomach_size;
        display.render_stomach(
            Vec2::new(9., IMAGE_STOMACH_MASK.size.y as f32),
            total_filled,
        );

        const STOMACH_END_X: i32 = IMAGE_STOMACH_MASK.size.y as i32 + 1;
        display.render_image_top_left(STOMACH_END_X, 0, &assets::IMAGE_AGE_SYMBOL);
        let hours = pet.age.as_hours() as i32;
        let days = hours / 24;
        let hours = hours % 24;
        let str = str_format!(str32, "{}d{}h", days, hours);
        display.render_text_complex(
            Vec2::new(
                STOMACH_END_X as f32 + assets::IMAGE_AGE_SYMBOL.size.x as f32 + 2.,
                1.,
            ),
            &str,
            ComplexRenderOption::new()
                .with_white()
                .with_font(&FONT_VARIABLE_SMALL),
        );

        let money_str = fixedstr::str_format!(str32, "${}", args.game_ctx.money);
        display.render_text_complex(
            Vec2::new(STOMACH_END_X as f32, 10.),
            &money_str,
            ComplexRenderOption::new()
                .with_white()
                .with_font(&FONT_VARIABLE_SMALL),
        );

        display.render_rect_solid(HOME_SCENE_TOP_BORDER_RECT, true);

        let options = &args.game_ctx.home.options;

        const SIZE: Vec2 = Vec2::new(
            assets::IMAGE_POOP_SYMBOL.size.x as f32,
            assets::IMAGE_POOP_SYMBOL.size.y as f32,
        );

        for i in 0..options.len() {
            let image = match options[i] {
                MenuOption::Breed => &assets::IMAGE_SYMBOL_BREED,
                MenuOption::Poop => &assets::IMAGE_POOP_SYMBOL,
                MenuOption::PetInfo => &assets::IMAGE_INFO_SYMBOL,
                MenuOption::GameSelect => &assets::IMAGE_GAME_SYMBOL,
                MenuOption::FoodSelect => args.game_ctx.home.food_anime.current_frame(),
                MenuOption::Shop => &assets::IMAGE_SHOP_SYMBOL,
                MenuOption::Inventory => &assets::IMAGE_SYMBOL_INVENTORY,
                MenuOption::PlaceFurniture => &assets::IMAGE_SYMBOL_PLACE_FURNITURE,
                MenuOption::PetRecords => &assets::IMAGE_SYMBOL_RECORDS,
                MenuOption::Heal => &assets::IMAGE_SYMBOL_HEALTHCARE,
                MenuOption::Settings => &assets::IMAGE_SYMBOL_SETTINGS,
            };
            let x = if args.game_ctx.home.selected_index > 0 {
                let x_index = i as i32 - args.game_ctx.home.selected_index as i32 + 1;
                SYMBOL_BUFFER + (x_index as f32 * (SIZE.x + SYMBOL_BUFFER))
            } else {
                SYMBOL_BUFFER + ((i + 1) as f32 * (SIZE.x + SYMBOL_BUFFER))
            };
            display.render_image_complex(
                x as i32,
                IMAGE_Y_START as i32,
                image,
                ComplexRenderOption::new().with_white().with_black(),
            );
        }

        let select_rect = Rect::new_top_left(
            Vec2::new(
                SYMBOL_BUFFER + (1_f32 * (SIZE.x + SYMBOL_BUFFER)) - (SYMBOL_BUFFER),
                IMAGE_Y_START - (SYMBOL_BUFFER),
            ),
            Vec2::new(SIZE.x + SYMBOL_BUFFER * 2., SIZE.y + SYMBOL_BUFFER * 2.),
        );
        display.render_rect_outline(select_rect, true);

        if args.game_ctx.egg.is_some() {
            display.render_complex(&self.egg_render);
        }

        // No lights if sleeping
        if matches!(args.game_ctx.home.state, State::Wondering)
            || matches!(
                args.game_ctx.home.state,
                State::PlayingMp3 { jam_end_time: _ }
            )
            || matches!(args.game_ctx.home.state, State::Alarm)
        {
            for i in [&self.top_render, &self.right_render, &self.left_render] {
                if let HomeFurnitureRender::InvetroLight(light) = i {
                    display.render_complex(light);
                }
            }
        }
    }
}
