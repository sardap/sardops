use core::{time::Duration, u8};

use strum::EnumCount;
use strum_macros::EnumCount;

use crate::{
    date_utils::time_in_range,
    display::CENTER_VEC,
    game_consts::TELESCOPE_USE_RANGE,
    items::ItemKind,
    pet::{definition::PET_BRAINO_ID, LifeStage, Mood},
    scene::{
        home_scene::{State, PROGRAM_RUN_TIME_RANGE},
        SceneTickArgs,
    },
    tv::TvKind,
};

pub fn reset_wonder_end(rng: &mut fastrand::Rng) -> Duration {
    Duration::from_secs(rng.u64(0..(5 * 60)))
}

pub fn wonder_end(args: &mut SceneTickArgs) {
    #[derive(Debug, Copy, Clone, EnumCount)]
    enum Activity {
        PlayingComputer,
        WatchTv,
        ReadBook,
        ListenMusic,
        GoOut,
        Telescope,
    }
    const ACTIVITY_COUNT: usize = Activity::COUNT;

    let mut options: heapless::Vec<Activity, ACTIVITY_COUNT> = heapless::Vec::new();

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
    if !matches!(args.game_ctx.pet.definition().life_stage, LifeStage::Baby)
        && args
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

    if args.game_ctx.pet.definition().life_stage != LifeStage::Child
        && args.game_ctx.pet.mood() == Mood::Happy
    {
        let _ = options.push(Activity::GoOut);
    }
    if args.game_ctx.pet.definition().life_stage != LifeStage::Child
        && args.game_ctx.inventory.has_item(ItemKind::Telescope)
        && time_in_range(&args.timestamp.inner().time(), &TELESCOPE_USE_RANGE)
    {
        let _ = options.push(Activity::Telescope);
    }

    options.clear();
    let _ = options.push(Activity::GoOut);

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
            Activity::GoOut => {
                args.game_ctx.home.change_state(State::GoneOut {
                    outing_end_time: Duration::from_mins(args.game_ctx.rng.u64(5..10))
                        + Duration::from_millis(args.game_ctx.rng.u64(0..60000)),
                });
            }
            Activity::Telescope => {
                args.game_ctx.home.change_state(State::Telescope {
                    end_time: Duration::from_mins(args.game_ctx.rng.u64(1..10))
                        + Duration::from_secs(args.game_ctx.rng.u64(1..60)),
                });
            }
        }
    }

    args.game_ctx.home.wonder_end = reset_wonder_end(&mut args.game_ctx.rng);
}
