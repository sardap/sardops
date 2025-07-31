use core::{
    ops::{Add, Sub},
    time::Duration,
};

use bincode::{Decode, Encode};

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode)]
pub struct Timestamp(pub Duration);

impl Timestamp {
    pub fn from_nano(seconds: u64, nano: u32) -> Self {
        Self(Duration::new(seconds, nano))
    }

    pub fn from_millis(mils: u64) -> Self {
        Self(Duration::from_millis(mils))
    }

    pub fn from_duration(duration: Duration) -> Self {
        Self(duration)
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl Sub for Timestamp {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0.checked_sub(rhs.0).unwrap_or(Duration::ZERO)
    }
}

impl Add<Duration> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: Duration) -> Self::Output {
        Timestamp::from_duration(self.0 + rhs)
    }
}

impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

pub trait DurationExt {
    fn as_mins(&self) -> f32;

    #[allow(dead_code)]
    fn as_hours(&self) -> f32;

    #[allow(dead_code)]
    fn as_days(&self) -> f32;

    #[allow(dead_code)]
    fn as_weeks(&self) -> f32;
}

// Why can't traits support const fn
pub const fn duration_from_mins(mins: f32) -> Duration {
    Duration::from_millis((mins * 60000.) as u64)
}

pub const fn duration_from_hours(hours: f32) -> Duration {
    duration_from_mins(hours * 60.)
}

#[allow(dead_code)]
pub const fn duration_from_days(days: f32) -> Duration {
    duration_from_hours(days * 24.)
}

#[allow(dead_code)]
pub const fn duration_from_weeks(weeks: f32) -> Duration {
    duration_from_hours(weeks * 7.)
}

impl DurationExt for Duration {
    fn as_mins(&self) -> f32 {
        self.as_secs_f32() / 60.
    }

    fn as_hours(&self) -> f32 {
        self.as_mins() / 60.
    }

    fn as_days(&self) -> f32 {
        self.as_hours() / 24.
    }

    fn as_weeks(&self) -> f32 {
        self.as_days() / 7.
    }
}
