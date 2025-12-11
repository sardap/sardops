use core::{ops::Range, time::Duration};

use chrono::NaiveTime;

use crate::{death::Threshold, money::Money};

pub const UI_FLASH_TIMER: Duration = Duration::from_millis(250);
pub const UI_FLASHING_TIMER: Duration = Duration::from_millis(500);

pub const ROOM_TEMPTURE: f32 = 25.;

pub const LOW_POWER_THRESHOLD: Duration = Duration::from_mins(1);

pub const SIM_LENGTH_STEP: Duration = Duration::from_millis(100);

pub const DEATH_CHECK_INTERVERAL: Duration = Duration::from_mins(5);

pub const EVOLVE_CHECK_INTERVERAL: Duration = Duration::from_mins(1);

pub const ALIEN_ODDS: f32 = 0.05;

pub const STARTING_FILLED: f32 = 10.;

const HOUR: Duration = Duration::from_hours(1);
const DAY: Duration = Duration::from_days(1);

pub const fn death_odds_per_hour(chance_per_hour: f32) -> f32 {
    let multipler = DEATH_CHECK_INTERVERAL.as_millis_f32() / HOUR.as_millis_f32();

    chance_per_hour * multipler
}

pub const fn death_odds_per_day(chance_per_day: f32) -> f32 {
    let multipler = DEATH_CHECK_INTERVERAL.as_millis_f32() / DAY.as_millis_f32();

    chance_per_day * multipler
}

pub const fn sim_tick_odds_per_hour(chance_per_hour: f32) -> f32 {
    let multipler = SIM_LENGTH_STEP.as_millis_f32() / HOUR.as_millis_f32();

    chance_per_hour * multipler
}

pub const fn sim_tick_odds_per_day(chance_per_hour: f32) -> f32 {
    let multipler = SIM_LENGTH_STEP.as_millis_f32() / DAY.as_millis_f32();

    chance_per_hour * multipler
}

pub const fn death_odds_per_day_waking_hours(chance_per_day: f32) -> f32 {
    let multipler = DEATH_CHECK_INTERVERAL.as_millis_f32() / DAY.as_millis_f32();

    chance_per_day * multipler
}

pub const DEATH_BY_LIGHTING_STRIKE_ODDS: f32 = death_odds_per_day(0.01);

pub const DEATH_STARVE_THRESHOLDS: &[Threshold<Duration>] = &[
    Threshold::new(Duration::from_hours(7), death_odds_per_hour(0.0)),
    Threshold::new(Duration::from_hours(8), death_odds_per_hour(0.01)),
    Threshold::new(Duration::from_hours(16), death_odds_per_hour(0.025)),
    Threshold::new(Duration::from_days(1), death_odds_per_hour(0.05)),
    Threshold::new(Duration::MAX, death_odds_per_hour(0.1)),
];

pub const OLD_AGE_THRESHOLD: &[Threshold<Duration>] = &[
    Threshold::new(Duration::from_days(4), death_odds_per_day(0.0)),
    Threshold::new(Duration::from_days(5), death_odds_per_day(0.05)),
    Threshold::new(Duration::from_days(6), death_odds_per_day(0.1)),
    Threshold::new(Duration::from_days(7), death_odds_per_day(0.25)),
    Threshold::new(Duration::MAX, death_odds_per_hour(0.3)),
];

pub const DEATH_TOXIC_SHOCK_THRESHOLD: &[Threshold<u8>] = &[
    Threshold::new(3, death_odds_per_day_waking_hours(0.0)),
    Threshold::new(4, death_odds_per_day_waking_hours(0.05)),
    Threshold::new(5, death_odds_per_day_waking_hours(0.1)),
];

pub const DEATH_BY_ILLNESS_THRESHOLD: &[Threshold<Duration>] = &[
    Threshold::new(Duration::from_hours(2), death_odds_per_hour(0.001)),
    Threshold::new(Duration::from_hours(4), death_odds_per_hour(0.05)),
    Threshold::new(Duration::from_days(1), death_odds_per_hour(0.1)),
    Threshold::new(Duration::MAX, death_odds_per_hour(0.5)),
];

pub const DEATH_BY_HYPOTHERMIA_THRESHOLD: &[Threshold<Duration>] = &[
    Threshold::new(Duration::from_hours(4), death_odds_per_hour(0.0)),
    Threshold::new(Duration::from_hours(8), death_odds_per_hour(0.05)),
    Threshold::new(Duration::from_days(1), death_odds_per_hour(0.1)),
    Threshold::new(Duration::MAX, death_odds_per_hour(0.2)),
];

// Base stomach size is 30 Drain 7 poiints per hour so 4 hours empty stomach
pub const HUNGER_LOSS_PER_SECOND: f32 = 7. / Duration::from_hours(1).as_secs_f32();

pub const POOP_INTERVNAL: Duration = Duration::from_hours(1);

pub const BREED_ODDS_THRESHOLD: &[Threshold<Duration>] = &[
    Threshold::new(Duration::from_days(1), sim_tick_odds_per_hour(0.0)),
    Threshold::new(Duration::from_days(2), sim_tick_odds_per_hour(0.25)),
    Threshold::new(Duration::from_days(3), sim_tick_odds_per_hour(0.5)),
    Threshold::new(Duration::from_days(4), sim_tick_odds_per_hour(0.75)),
    Threshold::new(Duration::MAX, sim_tick_odds_per_hour(0.9)),
];

pub const SUITER_SHOW_UP_ODDS_THRESHOLD: &[Threshold<Duration>] = &[
    Threshold::new(Duration::from_hours(2), sim_tick_odds_per_hour(0.1)),
    Threshold::new(Duration::from_hours(5), sim_tick_odds_per_hour(0.2)),
    Threshold::new(Duration::MAX, sim_tick_odds_per_hour(0.3)),
];

pub const SUITER_LEAVE_ODDS: f32 = sim_tick_odds_per_hour(0.25);

pub const EGG_HATCH_ODDS_THRESHOLD: &[Threshold<Duration>] = &[
    Threshold::new(Duration::from_days(1), sim_tick_odds_per_hour(0.0)),
    Threshold::new(Duration::from_days(2), sim_tick_odds_per_hour(0.1)),
    Threshold::new(Duration::MAX, sim_tick_odds_per_hour(0.99)),
];

pub const ILLNESS_SINCE_GAME_DURATION: Duration = Duration::from_hours(6);

pub const ILLNESS_BASE_ODDS: f32 = sim_tick_odds_per_day(0.05);
pub const ILLNESS_STARVING_ODDS: f32 = sim_tick_odds_per_day(0.3);
pub const ILLNESS_SINCE_GAME_ODDS: f32 = sim_tick_odds_per_day(0.05);
pub const ILLNESS_BABY_ODDS: f32 = sim_tick_odds_per_day(0.2);
pub const ILLNESS_CHILD_ODDS: f32 = sim_tick_odds_per_day(0.1);

pub const ILLNESS_SINCE_ODDS: &[Threshold<Duration>] = &[
    Threshold::new(Duration::from_hours(4), sim_tick_odds_per_day(0.0)),
    Threshold::new(Duration::from_hours(8), sim_tick_odds_per_hour(0.025)),
    Threshold::new(Duration::from_days(2), sim_tick_odds_per_hour(0.1)),
    Threshold::new(Duration::MAX, sim_tick_odds_per_hour(0.15)),
];

pub const HEALING_COST_RANGE: Range<Money> = 1000..10000;

pub const RANDOM_NAMES: &[&str] = &[
    "Abel", "Adam", "Amos", "Cain", "Caleb", "Dan", "David", "Eli", "Esau", "Gad", "Hagar",
    "Isaac", "Jacob", "Japhet", "Jonah", "Job", "Joel", "Judah", "Levi", "Lot", "Micah", "Moab",
    "Nahum", "Noah", "Obed", "Omar", "Perez", "Ruth", "Seth", "Shem", "Uriah", "Zerah", "Zimri",
    "Andrew", "Demas", "Enoch", "James", "Jason", "John", "Judas", "Luke", "Mark", "Mary", "Paul",
    "Peter", "Silas", "Simon", "Titus",
];

pub const SPLACE_LOCATIONS: &[&str] = &[
    "TRAPPIST",
    "KEPLER",
    "WASP",
    "GLIESE",
    "HD",
    "HIP",
    "HR",
    "K2",
    "COROT",
    "OGLE",
    "PSR",
    "TOI",
    "HAT-P",
    "LHS",
    "ROSS",
    "WOLF",
    "KAPTEYN",
    "BARNARD",
    "TAU CETI",
    "BETA PICTORIS",
    "FOMALHAUT",
    "PI MENSAE",
    "UPSILON ANDROMEDAE",
    "MU ARAE",
    "NU2 LUPI",
    "YZ CETI",
    "KELT",
    "KIC",
    "LUYTEN",
    "HATS",
    "NGTS",
    "TRES",
    "XO",
    "BD",
    "EPIC",
    "2MASS",
    "TESS",
    "GAIA",
    "SDSS",
    "SOPHIE",
    "ASAS-SN",
    "MOA",
    "KMTNET",
    "LTT",
    "GSC",
    "TYC",
    "CD",
    "SAO",
    "PLATO",
    "RAVE",
    "APOGEE",
    "MACHO",
    "WISE",
];

pub const SHOP_OPEN_TIMES: [[NaiveTime; 2]; 7] = [
    // Monday
    [
        NaiveTime::from_hms_opt(8, 00, 00).unwrap(),
        NaiveTime::from_hms_opt(19, 00, 00).unwrap(),
    ],
    [
        NaiveTime::from_hms_opt(8, 00, 00).unwrap(),
        NaiveTime::from_hms_opt(19, 00, 00).unwrap(),
    ],
    [
        NaiveTime::from_hms_opt(8, 00, 00).unwrap(),
        NaiveTime::from_hms_opt(19, 00, 00).unwrap(),
    ],
    [
        NaiveTime::from_hms_opt(8, 00, 00).unwrap(),
        NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
    ],
    [
        NaiveTime::from_hms_opt(8, 00, 00).unwrap(),
        NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
    ],
    // Sat
    [
        NaiveTime::from_hms_opt(10, 00, 00).unwrap(),
        NaiveTime::from_hms_opt(17, 00, 00).unwrap(),
    ],
    [
        NaiveTime::from_hms_opt(10, 00, 00).unwrap(),
        NaiveTime::from_hms_opt(15, 00, 00).unwrap(),
    ],
];

pub const TELESCOPE_USE_RANGE: Range<NaiveTime> =
    NaiveTime::from_hms_opt(5, 30, 0).unwrap()..NaiveTime::from_hms_opt(20, 30, 0).unwrap();
