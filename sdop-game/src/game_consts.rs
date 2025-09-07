use core::time::Duration;

use crate::{death::Threshold, sim::SIM_LENGTH_STEP};

pub const DEATH_CHECK_INTERVAL: Duration = Duration::from_mins(5);

pub const EVOLVE_CHECK_INTERVERAL: Duration = Duration::from_mins(1);

pub const fn death_odds_per_hour(chance_per_hour: f32) -> f32 {
    const HOUR: Duration = Duration::from_hours(1);
    let multipler = DEATH_CHECK_INTERVAL.as_millis_f32() / HOUR.as_millis_f32();

    chance_per_hour * multipler
}

pub const fn death_odds_per_day(chance_per_day: f32) -> f32 {
    const DAY: Duration = Duration::from_days(1);
    let multipler = DEATH_CHECK_INTERVAL.as_millis_f32() / DAY.as_millis_f32();

    chance_per_day * multipler
}

pub const fn sim_tick_odds_per_hour(chance_per_hour: f32) -> f32 {
    const HOUR: Duration = Duration::from_hours(1);
    let multipler = SIM_LENGTH_STEP.as_millis_f32() / HOUR.as_millis_f32();

    chance_per_hour * multipler
}

pub const fn odds_per_day_waking_hours(chance_per_day: f32) -> f32 {
    const DAY: Duration = Duration::from_hours(18);
    let multipler = DEATH_CHECK_INTERVAL.as_millis_f32() / DAY.as_millis_f32();

    chance_per_day * multipler
}

pub const DEATH_BY_LIGHTING_STRIKE_ODDS: f32 = death_odds_per_day(0.01);

pub const DEATH_STARVE_THRESHOLDS: &[Threshold<Duration>] = &[
    Threshold::new(Duration::from_hours(7), death_odds_per_hour(0.0)),
    Threshold::new(Duration::from_hours(8), death_odds_per_hour(0.025)),
    Threshold::new(Duration::from_hours(16), death_odds_per_hour(0.05)),
    Threshold::new(Duration::from_days(1), death_odds_per_hour(0.1)),
    Threshold::new(Duration::MAX, death_odds_per_hour(0.2)),
];

pub const OLD_AGE_THRESHOLD: &[Threshold<Duration>] = &[
    Threshold::new(Duration::from_days(4), death_odds_per_day(0.0)),
    Threshold::new(Duration::from_days(5), death_odds_per_day(0.05)),
    Threshold::new(Duration::from_days(6), death_odds_per_day(0.1)),
    Threshold::new(Duration::from_days(7), death_odds_per_day(0.25)),
    Threshold::new(Duration::MAX, death_odds_per_hour(0.3)),
];

pub const DEATH_BY_TOXIC_SHOCK_SMALL: f32 = death_odds_per_hour(0.05);
pub const DEATH_BY_TOXIC_SHOCK_LARGE: f32 = death_odds_per_hour(0.1);

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

pub const EGG_HATCH_ODDS_THRESHOLD: &[Threshold<Duration>] = &[
    Threshold::new(Duration::from_days(1), sim_tick_odds_per_hour(0.0)),
    Threshold::new(Duration::from_days(2), sim_tick_odds_per_hour(0.1)),
    Threshold::new(Duration::MAX, sim_tick_odds_per_hour(0.99)),
];
