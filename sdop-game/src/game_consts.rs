use core::time::Duration;

use crate::death::Threshold;

pub const DEATH_CHECK_INTERVAL: Duration = Duration::from_mins(5);

pub const fn odds_per_hour(chance_per_hour: f32) -> f32 {
    const HOUR: Duration = Duration::from_hours(1);
    let multipler = DEATH_CHECK_INTERVAL.as_millis_f32() / HOUR.as_millis_f32();

    chance_per_hour * multipler
}

pub const fn odds_per_day(chance_per_day: f32) -> f32 {
    const DAY: Duration = Duration::from_days(1);
    let multipler = DEATH_CHECK_INTERVAL.as_millis_f32() / DAY.as_millis_f32();

    chance_per_day * multipler
}

pub const DEATH_BY_LIGHTING_STRIKE_ODDS: f32 = odds_per_hour(0.001);

pub const DEATH_STARVE_THRESHOLDS: &[Threshold] = &[
    Threshold::new(Duration::from_hours(16), odds_per_hour(0.05)),
    Threshold::new(Duration::from_hours(24), odds_per_hour(0.1)),
    Threshold::new(Duration::MAX, odds_per_hour(0.2)),
];

pub const OLD_AGE_THRESHOLD: &[Threshold] = &[
    Threshold::new(Duration::from_days(4), odds_per_day(0.0)),
    Threshold::new(Duration::from_days(5), odds_per_day(0.05)),
    Threshold::new(Duration::from_days(6), odds_per_day(0.1)),
    Threshold::new(Duration::from_days(7), odds_per_day(0.25)),
];

// Base stomach size is 30 Drain 7 poiints per hour so 4 hours empty stomach
pub const HUNGER_LOSS_PER_SECOND: f32 = 7. / Duration::from_hours(1).as_secs_f32();

pub const POOP_INTERVNAL: Duration = Duration::from_mins(10);
