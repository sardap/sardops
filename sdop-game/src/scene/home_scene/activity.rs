use core::{time::Duration, u8};

use strum::EnumCount;
use strum_macros::EnumCount;

use crate::{
    Timestamp,
    date_utils::time_in_range,
    display::CENTER_VEC,
    game_consts::TELESCOPE_USE_RANGE,
    items::ItemKind,
    pet::{LifeStage, Mood, definition::PET_BRAINO_ID},
    scene::{
        SceneTickArgs,
        home_scene::{MUSIC_NOTE_SPAWNER, PROGRAM_RUN_TIME_RANGE, State},
    },
    tv::TvKind,
};

pub fn reset_wonder_end(rng: &mut fastrand::Rng) -> Duration {
    Duration::from_secs(rng.u64(0..(5 * 60)))
}

#[derive(Debug, Copy, Clone, EnumCount)]
enum Activity {
    Wonder = 0,
    PlayingComputer = 1,
    WatchTv = 2,
    ReadBook = 3,
    ListenMusic = 4,
    GoOut = 5,
    Telescope = 6,
}

impl Activity {
    pub fn cooldown(&self) -> Duration {
        match self {
            Activity::Wonder => Duration::ZERO,
            Activity::PlayingComputer => Duration::from_mins(15),
            Activity::WatchTv => Duration::from_mins(15),
            Activity::ReadBook => Duration::from_hours(2),
            Activity::ListenMusic => Duration::from_mins(10),
            Activity::GoOut => Duration::from_hours(1),
            Activity::Telescope => Duration::from_mins(45),
        }
    }
}

const ACTIVITY_COUNT: usize = Activity::COUNT;

pub type ActivityHistory = [Timestamp; ACTIVITY_COUNT];

fn add_option(
    options: &mut heapless::Vec<Activity, ACTIVITY_COUNT>,
    history: &ActivityHistory,
    now: &Timestamp,
    to_add: Activity,
) {
    if now > &(history[to_add as usize] + to_add.cooldown()) {
        let _ = options.push(to_add);
    }
}

pub fn wonder_end(args: &mut SceneTickArgs) {
    let mut options: heapless::Vec<Activity, ACTIVITY_COUNT> = heapless::Vec::new();

    if args.game_ctx.pet.definition().life_stage == LifeStage::Adult
        && args.game_ctx.inventory.has_item(ItemKind::PersonalComputer)
        && args.game_ctx.inventory.has_item(ItemKind::Screen)
        && args.game_ctx.inventory.has_item(ItemKind::Keyboard)
        && args.game_ctx.pet.def_id != PET_BRAINO_ID
    {
        add_option(
            &mut options,
            &args.game_ctx.home.activity_history,
            &args.timestamp,
            Activity::PlayingComputer,
        );
    }
    if args.game_ctx.pet.definition().life_stage != LifeStage::Baby
        && (args.game_ctx.inventory.has_item(ItemKind::TvLcd)
            || args.game_ctx.inventory.has_item(ItemKind::TvCrt))
        && args.game_ctx.pet.def_id != PET_BRAINO_ID
    {
        add_option(
            &mut options,
            &args.game_ctx.home.activity_history,
            &args.timestamp,
            Activity::WatchTv,
        );
    }

    if args.game_ctx.pet.book_history.has_book_to_read(
        args.game_ctx.pet.definition().life_stage,
        &args.game_ctx.inventory,
    ) {
        add_option(
            &mut options,
            &args.game_ctx.home.activity_history,
            &args.timestamp,
            Activity::ReadBook,
        );
    }

    if args.game_ctx.inventory.has_item(ItemKind::Mp3Player)
        && args.game_ctx.pet.mood() == Mood::Happy
    {
        add_option(
            &mut options,
            &args.game_ctx.home.activity_history,
            &args.timestamp,
            Activity::ListenMusic,
        );
    }

    if args.game_ctx.pet.definition().life_stage != LifeStage::Baby
        && args.game_ctx.pet.mood() == Mood::Happy
        && !crate::egg::will_hatch_soon(&args.game_ctx.egg, args.timestamp)
    {
        add_option(
            &mut options,
            &args.game_ctx.home.activity_history,
            &args.timestamp,
            Activity::GoOut,
        );
    }

    // This is broken
    if args.game_ctx.pet.definition().life_stage != LifeStage::Baby
        && args.game_ctx.inventory.has_item(ItemKind::Telescope)
        && time_in_range(&args.timestamp.inner().time(), &TELESCOPE_USE_RANGE)
    {
        add_option(
            &mut options,
            &args.game_ctx.home.activity_history,
            &args.timestamp,
            Activity::Telescope,
        );
    }

    add_option(
        &mut options,
        &args.game_ctx.home.activity_history,
        &args.timestamp,
        Activity::Wonder,
    );

    if !options.is_empty() {
        let option = args.game_ctx.rng.choice(options.iter()).cloned().unwrap();
        args.game_ctx.home.activity_history[option as usize] = args.timestamp;

        match option {
            Activity::Wonder => {
                args.game_ctx.home.change_state(State::Wondering);
            }
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
                            .pick_random_unread_book(
                                &mut args.game_ctx.rng,
                                args.game_ctx.pet.definition().life_stage,
                                inventory,
                            )
                            .unwrap_or_default(),
                    ),
                    read_time: Duration::from_secs(args.game_ctx.rng.u64(1200..3600)),
                });
            }
            Activity::ListenMusic => {
                args.game_ctx.home.pet_render.pos = CENTER_VEC;
                args.game_ctx.home.target = CENTER_VEC;
                args.game_ctx
                    .home
                    .particle_system
                    .add_spawner(MUSIC_NOTE_SPAWNER);
                args.game_ctx.home.change_state(State::PlayingMp3 {
                    jam_end_time: Duration::from_secs(args.game_ctx.rng.u64(60..300)),
                });
            }
            Activity::GoOut => {
                args.game_ctx.home.change_state(State::GoneOut {
                    outing_end_time: Duration::from_mins(args.game_ctx.rng.u64(5..10))
                        + Duration::from_millis(args.game_ctx.rng.u64(0..60000)),
                });
            }
            Activity::Telescope => {
                args.game_ctx.home.change_state(State::Telescope {
                    end_time: Duration::from_mins(args.game_ctx.rng.u64(5..20))
                        + Duration::from_secs(args.game_ctx.rng.u64(1..60)),
                });
            }
        }
    }

    args.game_ctx.home.wonder_end = reset_wonder_end(&mut args.game_ctx.rng);
}
