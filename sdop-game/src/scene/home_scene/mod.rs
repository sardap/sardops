mod activity;
mod menu_options;
mod weather;

use core::{time::Duration, u8};

use chrono::{Datelike, Timelike};
use fixedstr::{str_format, str32};
use glam::{IVec2, Vec2};

use crate::{
    Button, Timestamp, WIDTH,
    anime::{HasAnime, MaskedAnimeSprite, tick_all_anime},
    assets::{
        self, DynamicImage, FRAMES_GONE_OUT_SIGN, FRAMES_GONE_OUT_SIGN_MASK, FRAMES_SKULL,
        FRAMES_SKULL_MASK, FRAMES_TELESCOPE_HOME, FRAMES_TELESCOPE_HOME_MASK, IMAGE_STOMACH_MASK,
        Image,
    },
    book::on_book_completed,
    date_utils::DurationExt,
    display::{
        CENTER_VEC, CENTER_X, CENTER_X_I32, CENTER_Y, ComplexRenderOption, GameDisplay, HEIGHT_F32,
        WIDTH_F32, WIDTH_I32,
    },
    dream_bubble::DreamBubble,
    egg::EggRender,
    fonts::FONT_VARIABLE_SMALL,
    food::FOOD_COFFEE,
    furniture::{HomeFurnitureKind, HomeFurnitureLocation, HomeFurnitureRender},
    geo::{RectIVec2, RectVec2, vec2_direction, vec2_distance},
    items::ItemKind,
    night_sky::generate_night_sky_image,
    particle_system::{ParticleSystem, ParticleTemplate, ParticleTickArgs, SpawnTrigger, Spawner},
    pc::{PcKind, PcRender},
    pet::{Mood, definition::PetAnimationSet, render::PetRender},
    poop::{MAX_POOPS, PoopRender, poop_count, update_poop_renders},
    scene::{
        RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs,
        death_scene::DeathScene,
        egg_hatch_scene::EggHatchScene,
        evolve_scene::EvolveScene,
        explore_select_scene::ExploreSelectScene,
        exploring_post_scene::ExploringPostScene,
        food_select::FoodSelectScene,
        game_select::GameSelectScene,
        heal_scene::HealScene,
        home_scene::{
            activity::{ActivityHistory, reset_wonder_end, wonder_end},
            menu_options::{MenuOption, MenuOptions},
        },
        inventory_scene::InventoryScene,
        pet_info_scene::PetInfoScene,
        pet_records_scene::PetRecordsScene,
        place_furniture_scene::PlaceFurnitureScene,
        poop_clear_scene::PoopClearScene,
        settings_scene::SettingsScene,
        shop_scene::ShopScene,
        suiters_scene::SuitersScene,
    },
    sounds::{SONG_ALARM, SONG_HUNGRY, SONG_POOPED, SONG_SICK, SongPlayOptions},
    sprite::{BasicAnimeSprite, MusicNote, Sprite},
    stomach::StomachRender,
    temperature::TemperatureLevel,
    tv::{SHOW_RUN_TIME, TvKind, TvRender, get_show_for_time},
};

const WONDER_SPEED: f32 = 5.;
const DANCING_SPEED: f32 = 15.;
pub const WONDER_RECT: RectVec2 = RectVec2::new_center(CENTER_VEC, Vec2::new(WIDTH as f32, 90.0));
pub const DANCING_RECT: RectVec2 = RectVec2::new_center(CENTER_VEC, Vec2::new(10., 10.));

pub const GREATER_WONDER_RECT: RectVec2 = WONDER_RECT.grow(50.);

const BORDER_HEIGHT: i32 = 1;

pub const HOME_SCENE_TOP_BORDER_RECT: RectIVec2 = RectIVec2::new_center(
    IVec2::new(CENTER_X_I32, 24),
    IVec2::new(WIDTH_I32, BORDER_HEIGHT),
);

pub const HOME_SCENE_TOP_AREA_RECT: RectIVec2 = RectIVec2::new_top_left(
    IVec2::new(0, 0),
    IVec2::new(WIDTH_I32, HOME_SCENE_TOP_BORDER_RECT.y2()),
);

pub const PROGRAM_RUN_TIME_RANGE: core::ops::Range<Duration> =
    Duration::from_secs(30)..Duration::from_mins(3);

const BOOK_POS: Vec2 = Vec2::new(CENTER_X, 90.);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
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
    GoneOut {
        outing_end_time: Duration,
    },
    Telescope {
        end_time: Duration,
    },
    Exploring,
}

struct Word {
    pos: Vec2,
    dir: Vec2,
    text: &'static str,
}

pub struct HomeSceneData {
    pet_render: PetRender,
    poops: [Option<PoopRender>; MAX_POOPS],
    target: Vec2,
    options: MenuOptions,
    sleeping_z: BasicAnimeSprite,
    dream_bubble: DreamBubble,
    show_dream_bubble: bool,
    dream_bubble_timer: Duration,
    tv: TvRender,
    pc: PcRender,
    next_word_spawn: Duration,
    floating_words: [Option<Word>; 5],
    pub state: State,
    state_elapsed: Duration,
    wonder_end: Duration,
    weather: weather::Weather,
    shake_duration: Duration,
    shake_right: bool,
    music_notes: [MusicNote; 7],
    last_poop_count: usize,
    last_poop_sound: Timestamp,
    last_is_sick: bool,
    last_was_hungry: bool,
    gone_out_sign: MaskedAnimeSprite,
    telescope: MaskedAnimeSprite,
    activity_history: ActivityHistory,
}

impl Default for HomeSceneData {
    fn default() -> Self {
        Self {
            pet_render: PetRender::default(),
            poops: [None; MAX_POOPS],
            target: Vec2::default(),
            options: MenuOptions::default(),
            sleeping_z: BasicAnimeSprite::new(CENTER_VEC, &assets::FRAMES_SLEEPING_Z),
            dream_bubble: DreamBubble::new(Vec2::new(CENTER_X + 15., CENTER_Y)),
            show_dream_bubble: false,
            dream_bubble_timer: Duration::ZERO,
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
            weather: weather::Weather::default(),
            shake_duration: Duration::ZERO,
            shake_right: false,
            music_notes: Default::default(),
            last_poop_count: 0,
            last_poop_sound: Timestamp::default(),
            last_is_sick: false,
            last_was_hungry: false,
            gone_out_sign: MaskedAnimeSprite::new(
                CENTER_VEC,
                &FRAMES_GONE_OUT_SIGN,
                &FRAMES_GONE_OUT_SIGN_MASK,
            ),
            telescope: MaskedAnimeSprite::new(
                CENTER_VEC,
                &FRAMES_TELESCOPE_HOME,
                &FRAMES_TELESCOPE_HOME_MASK,
            ),
            activity_history: Default::default(),
        }
    }
}

impl HomeSceneData {
    fn wonder_rect(&self) -> RectVec2 {
        RectVec2::new_center(
            WONDER_RECT.pos,
            WONDER_RECT.size - self.pet_render.anime.current_frame().size.x as f32,
        )
    }

    pub fn change_state(&mut self, new_state: State) {
        if self.state == new_state {
            return;
        }

        self.state = new_state;
        self.state_elapsed = Duration::ZERO;
    }
}

const STAR_SPAWNER: Spawner = Spawner::new(
    "star",
    SpawnTrigger::timer_range(Duration::from_secs(1)..Duration::from_secs(10)),
    |args| {
        const LEFT_STAR: ParticleTemplate = ParticleTemplate::new(
            Duration::from_secs(10)..Duration::from_secs(20),
            RectVec2::new_top_left(
                Vec2::new(
                    HOME_SCENE_TOP_AREA_RECT.x2() as f32 + 20.,
                    HOME_SCENE_TOP_AREA_RECT.y2() as f32,
                ),
                Vec2::new(1., 20.),
            ),
            Vec2::new(-50.0, -2.0)..Vec2::new(-20.0, 2.0),
            &[&assets::IMAGE_SHOOTING_STAR],
        );
        const RIGHT_STAR: ParticleTemplate = ParticleTemplate::new(
            Duration::from_secs(1)..Duration::from_secs(10),
            RectVec2::new_top_left(
                Vec2::new(-20., HOME_SCENE_TOP_AREA_RECT.y2() as f32),
                Vec2::new(1., 20.),
            ),
            Vec2::new(20.0, -2.0)..Vec2::new(50.0, 2.0),
            &[&assets::IMAGE_SHOOTING_STAR],
        );

        return (
            if args.rng.bool() {
                &LEFT_STAR
            } else {
                &RIGHT_STAR
            },
            1,
        );
    },
);

const EGG_RIGHT: Vec2 = Vec2::new(
    WIDTH_F32 - assets::IMAGE_EGG.size.x as f32,
    WONDER_RECT.y2() - assets::IMAGE_EGG.size.y as f32,
);

const EGG_LEFT: Vec2 = Vec2::new(
    assets::IMAGE_EGG.size.x as f32,
    WONDER_RECT.y2() - assets::IMAGE_EGG.size.y as f32,
);

const NIGHT_SKY_HEIGHT: usize = 30;

type PartialNightSky = DynamicImage<{ WIDTH * NIGHT_SKY_HEIGHT / 8 }>;

pub struct HomeScene {
    left_render: HomeFurnitureRender,
    top_render: HomeFurnitureRender,
    right_render: HomeFurnitureRender,
    egg_render: EggRender,
    egg_bounce: f32,
    particle_system: ParticleSystem<20, 2>,
    night_sky: PartialNightSky,
    skull: MaskedAnimeSprite,
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
            particle_system: ParticleSystem::default(),
            night_sky: PartialNightSky::default(),
            skull: MaskedAnimeSprite::new(CENTER_VEC, &FRAMES_SKULL, &FRAMES_SKULL_MASK),
        }
    }
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

            args.game_ctx.home.weather.setup(&mut args.game_ctx.rng);

            for note in &mut args.game_ctx.home.music_notes {
                note.pos = Vec2::new(-100., -100.);
            }

            generate_night_sky_image::<NIGHT_SKY_HEIGHT>(
                &mut self.night_sky,
                args.timestamp.inner().num_days_from_ce(),
            );

            args.game_ctx.home.show_dream_bubble = true;
            args.game_ctx.home.dream_bubble_timer = Duration::ZERO;
        }

        self.egg_render.pos = EGG_RIGHT;
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

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        if args.game_ctx.home.state_elapsed == Duration::ZERO || args.frames % 10 == 0 {
            args.game_ctx.home.options.refresh(
                args.game_ctx.home.state,
                &args.game_ctx.suiter_system,
                &args.game_ctx.inventory,
                args.game_ctx.poop_count(),
                &args.game_ctx.pet_history,
                &args.game_ctx.pet,
            );
        }

        args.game_ctx.home.state_elapsed += args.delta;

        args.game_ctx
            .home
            .pet_render
            .set_def_id(args.game_ctx.pet.def_id);

        self.particle_system.tick(&mut ParticleTickArgs {
            delta: args.delta,
            rng: &mut args.game_ctx.rng,
        });

        update_poop_renders(&mut args.game_ctx.home.poops, &args.game_ctx.poops);

        args.game_ctx.home.options.tick(args.delta);
        args.game_ctx.home.pet_render.tick(args.delta);
        tick_all_anime(&mut args.game_ctx.home.poops, args.delta);

        if args.game_ctx.explore_system.currently_exploring() {
            args.game_ctx.home.change_state(State::Exploring);
        } else {
            if args.game_ctx.pet.is_sleeping()
                && !matches!(args.game_ctx.home.state, State::Sleeping)
            {
                args.game_ctx.home.change_state(State::Sleeping);
            } else if !args.game_ctx.pet.is_sleeping()
                && matches!(args.game_ctx.home.state, State::Sleeping)
            {
                args.game_ctx.home.change_state(State::Wondering);
            }
        }

        // Pending sounds
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
                if egg.should_hatch(args.timestamp) {
                    output.set(SceneEnum::EggHatch(EggHatchScene::new(
                        *egg,
                        args.game_ctx.pet.def_id,
                    )));
                    return;
                }
            }

            if let Some(cause_of_death) = args.game_ctx.pet.should_die() {
                output.set(SceneEnum::Death(DeathScene::new(
                    cause_of_death,
                    args.game_ctx.pet.def_id,
                )));
                return;
            }

            if let Some(next_pet_id) = args.game_ctx.pet.should_evolve() {
                output.set(SceneEnum::Evovle(EvolveScene::new(
                    args.game_ctx.pet.def_id,
                    next_pet_id,
                )));
                return;
            }
        }

        if args.game_ctx.pet.is_ill() {
            self.skull.pos = args.game_ctx.home.pet_render.pos
                - Vec2::new(
                    0.,
                    args.game_ctx.home.pet_render.anime.current_frame().size.y as f32 / 2.
                        + self.skull.anime.current_frame().size.y as f32 / 2.
                        + 2.,
                );
            self.skull.anime().tick(args.delta);
        }

        args.game_ctx.home.weather.tick(
            args.delta,
            &mut args.game_ctx.rng,
            TemperatureLevel::from(args.input.temperature()),
        );

        if matches!(
            args.game_ctx.home.state,
            State::Wondering | State::PlayingMp3 { jam_end_time: _ } | State::Alarm
        ) {
            self.top_render.tick(args);
            self.left_render.tick(args);
            self.right_render.tick(args);
        }

        match args.game_ctx.home.state {
            State::Wondering => {
                self.egg_render.pos = EGG_RIGHT;
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

                {
                    let pet = &mut args.game_ctx.pet;
                    let home = &mut args.game_ctx.home;

                    home.pet_render.pos += vec2_direction(home.pet_render.pos, home.target)
                        * pet.definition().wonder_speed()
                        * if pet.is_starving() { 0.5 } else { 1. }
                        * if pet.is_ill() { 0.5 } else { 1. }
                        * if home.weather.is_weather_none() {
                            1.
                        } else {
                            0.5
                        }
                        * if pet.food_history.sick_of(&FOOD_COFFEE)
                            && pet.food_history.ate_since_time(
                                &FOOD_COFFEE,
                                args.timestamp - Duration::from_hours(2),
                            )
                        {
                            3.
                        } else {
                            0.5
                        }
                        * args.delta.as_secs_f32();
                }
            }
            State::Sleeping => {
                self.egg_render.pos = EGG_RIGHT;

                self.top_render.tick(args);
                self.left_render.tick(args);
                self.right_render.tick(args);

                let home = &mut args.game_ctx.home;

                home.dream_bubble_timer = home
                    .dream_bubble_timer
                    .checked_sub(args.delta)
                    .unwrap_or_default();

                if home.dream_bubble_timer <= Duration::ZERO {
                    if home.show_dream_bubble {
                        home.show_dream_bubble = false;
                        home.dream_bubble_timer = Duration::from_mins(args.game_ctx.rng.u64(3..30));
                    } else {
                        home.show_dream_bubble = true;
                        home.dream_bubble_timer =
                            Duration::from_secs(args.game_ctx.rng.u64(60..600));
                    }
                }

                home.dream_bubble.tick(args.delta, &mut args.game_ctx.rng);

                home.pet_render.set_animation(PetAnimationSet::Sleeping);

                home.dream_bubble.pos.y =
                    CENTER_Y - home.pet_render.anime.current_frame().size.y as f32 / 2.;

                home.sleeping_z.anime().tick(args.delta);

                home.pet_render.pos = CENTER_VEC + Vec2::new(0., 10.);
                home.sleeping_z.pos = Vec2::new(
                    home.pet_render.pos.x + (home.pet_render.static_image().size.x as f32 * 0.5),
                    home.pet_render.pos.y - (home.pet_render.static_image().size.x as f32 * 0.7),
                );
            }
            State::WatchingTv {
                mut last_checked,
                watch_end,
            } => {
                self.egg_render.pos = EGG_LEFT;

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
                self.egg_render.pos = EGG_RIGHT;

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
                self.egg_render.pos = EGG_RIGHT;

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
                    let completed = {
                        let book = args.game_ctx.pet.book_history.get_mut_read(book);
                        book.complete_chapter();
                        book.completed()
                    };

                    if completed {
                        on_book_completed(args.game_ctx, book);
                    }
                    args.game_ctx.home.change_state(State::Wondering);
                }
            }
            State::PlayingMp3 { jam_end_time } => {
                self.egg_render.pos = EGG_RIGHT;

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
                    if note.pos.y < { -note.size().y }
                        || note.pos.y > HEIGHT_F32 + note.size().y
                        || note.pos.x > WIDTH_F32 + note.size().x
                        || note.pos.x < -{ note.size().x }
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
                self.egg_render.pos = EGG_RIGHT;

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
            State::GoneOut { outing_end_time } => {
                args.game_ctx.home.gone_out_sign.anime().tick(args.delta);

                if args.game_ctx.home.state_elapsed > outing_end_time {
                    args.game_ctx.home.change_state(State::Wondering);
                }
            }
            State::Telescope { end_time } => {
                self.egg_render.pos = EGG_RIGHT;

                args.game_ctx.home.telescope.pos = Vec2::new(15., 80.);
                args.game_ctx.home.pet_render.pos = Vec2::new(45., 85.);

                args.game_ctx.home.telescope.anime().tick(args.delta);

                if args.game_ctx.home.state_elapsed > end_time {
                    self.particle_system.remove_spawner(STAR_SPAWNER.name);
                    args.game_ctx.home.change_state(State::Wondering);
                }
            }
            State::Exploring => {
                self.egg_render.pos = EGG_RIGHT;

                if !args.game_ctx.explore_system.currently_exploring() {
                    args.game_ctx.home.change_state(State::Wondering);
                    output.set(SceneEnum::ExploringPost(ExploringPostScene::new()));
                    return;
                }

                let passed = args.game_ctx.explore_system.current_percent_passed();
                args.game_ctx
                    .home
                    .pet_render
                    .set_animation(if passed > 0.8 {
                        PetAnimationSet::Normal
                    } else if passed > 0.5 {
                        PetAnimationSet::Normal
                    } else {
                        PetAnimationSet::Sad
                    });
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

        if args.game_ctx.home.weather.should_shake() {
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
                args.game_ctx.home.options.change_option(-1);
            }
            if args.input.pressed(Button::Right) {
                args.game_ctx.home.options.change_option(1);
            }
            if args.input.pressed(Button::Middle) {
                args.game_ctx.sound_system.push_song(
                    *args.game_ctx.home.options.current().get_song(),
                    SongPlayOptions::new().with_effect(),
                );
                output.set(match *args.game_ctx.home.options.current() {
                    MenuOption::Breed => SceneEnum::Suiters(SuitersScene::new(
                        args.game_ctx.suiter_system.suiter.unwrap_or_default(),
                    )),
                    MenuOption::Poop => SceneEnum::PoopClear(PoopClearScene::new()),
                    MenuOption::PetInfo => SceneEnum::PetInfo(PetInfoScene::new()),
                    MenuOption::GameSelect => SceneEnum::GameSelect(GameSelectScene::new()),
                    MenuOption::FoodSelect => SceneEnum::FoodSelect(FoodSelectScene::new()),
                    MenuOption::Shop => SceneEnum::Shop(ShopScene::new()),
                    MenuOption::Inventory => SceneEnum::Inventory(InventoryScene::new()),
                    MenuOption::PlaceFurniture => {
                        SceneEnum::PlaceFurniture(PlaceFurnitureScene::new())
                    }
                    MenuOption::PetRecords => SceneEnum::PetRecords(PetRecordsScene::new()),
                    MenuOption::Heal => SceneEnum::Heal(HealScene::new()),
                    MenuOption::Settings => SceneEnum::Settings(SettingsScene::new()),
                    MenuOption::Explore => SceneEnum::ExploreSelect(ExploreSelectScene::new()),
                });
                return;
            }
        }
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        if matches!(
            args.game_ctx.home.state,
            State::Wondering
                | State::Sleeping
                | State::PlayingMp3 { jam_end_time: _ }
                | State::Alarm
        ) {
            display.render_complex(&self.top_render);
            display.render_complex(&self.left_render);
            display.render_complex(&self.right_render);
        }

        match args.game_ctx.home.state {
            State::Wondering => {
                display.render_sprite(&args.game_ctx.home.pet_render);
            }
            State::Sleeping => {
                if args.game_ctx.home.show_dream_bubble {
                    display.render_complex(&args.game_ctx.home.dream_bubble);
                } else {
                    display.render_sprite(&args.game_ctx.home.sleeping_z);
                }

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
                    &IVec2::new(CENTER_X_I32, 34),
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
                    &IVec2::new(CENTER_X_I32, 40),
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
                    &IVec2::new(CENTER_X_I32, 46),
                    &str,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL),
                );

                for word in args.game_ctx.home.floating_words.iter().flatten() {
                    display.render_text_complex(
                        &IVec2::new(word.pos.x as i32, word.pos.y as i32),
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
            State::GoneOut { outing_end_time } => {
                display.render_sprite(&args.game_ctx.home.gone_out_sign);
            }
            State::Telescope { end_time } => {
                display.render_image_complex(
                    0,
                    HOME_SCENE_TOP_AREA_RECT.y2() as i32,
                    &self.night_sky,
                    ComplexRenderOption::new().with_white(),
                );

                display.render_sprite(&args.game_ctx.home.telescope);
                display.render_sprite(&args.game_ctx.home.pet_render);
            }
            State::Exploring => {
                let explore = &args.game_ctx.explore_system;
                let pet = &args.game_ctx.pet;

                let y = 48;

                display.render_image_complex(
                    CENTER_X as i32,
                    y + PREVIEW_RECT_HEIGHT / 2,
                    args.game_ctx.home.pet_render.image(),
                    ComplexRenderOption::new().with_white().with_center(),
                );

                display.render_rect_solid(
                    &RectIVec2::new_bottom_left(IVec2::new(0, y), IVec2::new(WIDTH_I32, 20)),
                    false,
                );

                display.render_rect_solid(
                    &RectIVec2::new_top_left(
                        IVec2::new(0, y + PREVIEW_RECT_HEIGHT - 1),
                        IVec2::new(WIDTH_I32, 20),
                    ),
                    false,
                );

                display.render_image_complex(
                    CENTER_X_I32,
                    y + PREVIEW_RECT_HEIGHT / 2,
                    &assets::IMAGE_EXPLORE_HOME_WINDOW,
                    ComplexRenderOption::new().with_center().with_white(),
                );

                let mut y = 1;
                display.render_image_complex(
                    WIDTH_I32 / 2 - assets::IMAGE_CURRENTLY_EXPLORING.size.x as i32 / 2,
                    y,
                    &assets::IMAGE_CURRENTLY_EXPLORING,
                    ComplexRenderOption::new().with_white(),
                );
                y += assets::IMAGE_CURRENTLY_EXPLORING.size.y as i32;

                display.render_text_complex(
                    &IVec2::new(CENTER_X_I32, y + 4),
                    explore.current_location().name,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font_wrapping_x(WIDTH_I32)
                        .with_font(&FONT_VARIABLE_SMALL),
                );

                y += 17;

                {
                    let total_seconds =
                        (explore.current_location().length - explore.elapsed()).as_secs() as i32; // Get total time in seconds
                    let hours = total_seconds / 3600;
                    let remaining = total_seconds % 3600;
                    let mins = remaining / 60;
                    let seconds = remaining % 60;

                    let str = str_format!(fixedstr::str12, "{}h{:02}m{:02}s", hours, mins, seconds);
                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, y),
                        &str,
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                }
                y += 4;

                // Render percent
                const PROGRESS_RECT_HEIGHT: i32 = 5;
                display.render_rect_outline(
                    &RectIVec2::new_top_left(
                        IVec2::new(0, y),
                        IVec2::new(WIDTH_I32, PROGRESS_RECT_HEIGHT),
                    ),
                    true,
                );
                display.render_rect_solid(
                    &RectIVec2::new_top_left(
                        IVec2::new(0, y as i32),
                        IVec2::new(
                            (WIDTH_F32 * explore.percent_complete()) as i32,
                            PROGRESS_RECT_HEIGHT,
                        ),
                    ),
                    true,
                );

                y += PROGRESS_RECT_HEIGHT as i32 + 2;

                const PREVIEW_RECT_HEIGHT: i32 = 20;

                // already rendered the pet

                y += PREVIEW_RECT_HEIGHT + 3;

                let str = str_format!(fixedstr::str32, "{} now is", pet.name.trim());
                display.render_text_complex(
                    &IVec2::new(CENTER_X_I32 + 1, y),
                    &str,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL),
                );

                y += 7;

                display.render_text_complex(
                    &IVec2::new(CENTER_X_I32 + 1, y),
                    explore.current_activity(),
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_font(&FONT_VARIABLE_SMALL)
                        .with_font_wrapping_x(WIDTH_I32 - 2),
                );
            }
        }

        if !matches!(args.game_ctx.home.state, State::Exploring) {
            display.render_sprites(&args.game_ctx.home.poops);

            display.render_complex(&args.game_ctx.home.weather);
        }

        display.render_complex(&self.particle_system);

        if args.game_ctx.pet.is_ill()
            && !matches!(
                args.game_ctx.home.state,
                State::GoneOut { outing_end_time: _ } | State::Exploring
            )
        {
            display.render_sprite(&self.skull);
        }

        if !matches!(args.game_ctx.home.state, State::Exploring) {
            let pet = &args.game_ctx.pet;

            display.render_rect_solid(&HOME_SCENE_TOP_AREA_RECT, false);

            let total_filled = pet.stomach_filled / pet.definition().stomach_size;
            display.render_complex(&StomachRender {
                pos_center: Vec2::new(
                    9. + if total_filled < 0.05 && args.frames % 5 == 0 {
                        if args.game_ctx.rng.bool() { 2. } else { -2. }
                    } else {
                        0.
                    },
                    IMAGE_STOMACH_MASK.size.y as f32,
                ),
                filled: total_filled,
            });

            const STOMACH_END_X: i32 = IMAGE_STOMACH_MASK.isize.y + 1;
            display.render_image_top_left(STOMACH_END_X, 0, &assets::IMAGE_AGE_SYMBOL);
            let hours = pet.age.as_hours() as i32;
            let days = hours / 24;
            let hours = hours % 24;
            let str = str_format!(str32, "{}d{}h", days, hours);
            display.render_text_complex(
                &IVec2::new(STOMACH_END_X + assets::IMAGE_AGE_SYMBOL.isize.x + 2, 1),
                &str,
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );

            let money_str = fixedstr::str_format!(str32, "${}", args.game_ctx.money);
            display.render_text_complex(
                &IVec2::new(STOMACH_END_X, 10),
                &money_str,
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );

            display.render_rect_solid(&HOME_SCENE_TOP_BORDER_RECT, true);

            if args.game_ctx.egg.is_some()
                && matches!(
                    args.game_ctx.home.state,
                    State::GoneOut { outing_end_time: _ } | State::Exploring
                )
            {
                display.render_complex(&self.egg_render);
            }

            // No lights if sleeping
            if matches!(
                args.game_ctx.home.state,
                State::Wondering | State::PlayingMp3 { jam_end_time: _ } | State::Alarm
            ) {
                for i in [&self.top_render, &self.right_render, &self.left_render] {
                    if let HomeFurnitureRender::InvetroLight(light) = i {
                        display.render_complex(light);
                    }
                }
            }
        }

        display.render_complex(&args.game_ctx.home.options);
    }
}
