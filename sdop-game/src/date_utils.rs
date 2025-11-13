use core::{
    ops::{Add, Sub},
    time::Duration,
};

use bincode::Encode;
use const_for::const_for;

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use glam::Vec2;
use strum_macros::EnumIter;

use crate::{
    assets::{self},
    sprite::{Sprite, SpriteMask},
};

include!(concat!(env!("OUT_DIR"), "/dist_dates.rs"));

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Timestamp(pub NaiveDateTime);

impl Timestamp {
    pub fn new(date_time: NaiveDateTime) -> Self {
        Self(date_time)
    }

    pub fn from_parts(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        miniute: u32,
        second: u32,
        nanos: u32,
    ) -> Option<Self> {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            if let Some(time) = NaiveTime::from_hms_nano_opt(hour, miniute, second, nanos) {
                return Some(Timestamp(NaiveDateTime::new(date, time)));
            }
        }

        None
    }

    pub fn epoch_seconds(&self) -> i64 {
        #[allow(deprecated)]
        self.0.timestamp_nanos()
    }

    pub fn seed(&self) -> u64 {
        #[allow(deprecated)]
        u64::from_ne_bytes(self.0.timestamp_nanos().to_ne_bytes())
    }

    pub fn date_seed(&self) -> u64 {
        let day = self.inner().day() as u8;
        let month = self.inner().month() as u8;
        let year = self.inner().year() as u16;
        let year_bytes = year.to_be_bytes();
        let year_left = year_bytes[0];
        let year_right = year_bytes[1];

        u32::from_be_bytes([day, month, year_left, year_right]) as u64
    }

    pub fn inner(&self) -> &NaiveDateTime {
        &self.0
    }
}

impl Encode for Timestamp {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        let year: i32 = self.0.year();
        let month: u32 = self.0.month();
        let day: u32 = self.0.day();
        let hour: u32 = self.0.hour();
        let miniute: u32 = self.0.minute();
        let second: u32 = self.0.second();
        let nano: u32 = self.0.nanosecond();
        bincode::Encode::encode(&year, encoder)?;
        bincode::Encode::encode(&month, encoder)?;
        bincode::Encode::encode(&day, encoder)?;
        bincode::Encode::encode(&hour, encoder)?;
        bincode::Encode::encode(&miniute, encoder)?;
        bincode::Encode::encode(&second, encoder)?;
        bincode::Encode::encode(&nano, encoder)?;
        Ok(())
    }
}

impl<Context> bincode::Decode<Context> for Timestamp {
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> core::result::Result<Self, bincode::error::DecodeError> {
        let year: i32 = bincode::Decode::decode(decoder)?;
        let month: u32 = bincode::Decode::decode(decoder)?;
        let day: u32 = bincode::Decode::decode(decoder)?;
        let hour: u32 = bincode::Decode::decode(decoder)?;
        let miniute: u32 = bincode::Decode::decode(decoder)?;
        let seconds: u32 = bincode::Decode::decode(decoder)?;
        let nano: u32 = bincode::Decode::decode(decoder)?;

        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            if let Some(time) = NaiveTime::from_hms_nano_opt(hour, miniute, seconds, nano) {
                return Ok(Self(NaiveDateTime::new(date, time)));
            }
        }

        Err(bincode::error::DecodeError::Other("bad date"))
    }
}
impl<'de, Context> bincode::BorrowDecode<'de, Context> for Timestamp {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> core::result::Result<Self, bincode::error::DecodeError> {
        let year: i32 = bincode::Decode::decode(decoder)?;
        let month: u32 = bincode::Decode::decode(decoder)?;
        let day: u32 = bincode::Decode::decode(decoder)?;
        let hour: u32 = bincode::Decode::decode(decoder)?;
        let miniute: u32 = bincode::Decode::decode(decoder)?;
        let seconds: u32 = bincode::Decode::decode(decoder)?;
        let nano: u32 = bincode::Decode::decode(decoder)?;

        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            if let Some(time) = NaiveTime::from_hms_nano_opt(hour, miniute, seconds, nano) {
                return Ok(Self(NaiveDateTime::new(date, time)));
            }
        }

        Err(bincode::error::DecodeError::Other("bad date"))
    }
}

impl Sub for Timestamp {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        let delta = self.0.sub(rhs.0);
        delta.to_std().unwrap_or_default()
    }
}

impl Add<Duration> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: Duration) -> Self::Output {
        let duration = chrono::Duration::from_std(rhs).unwrap_or_default();
        Timestamp::new(self.0.checked_add_signed(duration).unwrap_or_default())
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SpecialDayKind {
    NewYearsDay,
    EightHourDay,
    AnzacDay,
    MayDay,
    KingsBirthDay,
    Xmas,
    BoxingDay,
    EasterSunday,
    EasterMonday,
    GrandFinalEve,
    MarchEquinox,
    GoodFriday,
    MothersDay,
    WinterSolstice,
    FathersDay,
    SeptemberEquinox,
    SummerSolstice,
    MelbourneCup,
    SuperannuationDay,
}

impl SpecialDayKind {
    pub const fn is_trading_day(&self) -> bool {
        match self {
            Self::NewYearsDay
            | Self::EightHourDay
            | Self::AnzacDay
            | Self::KingsBirthDay
            | Self::Xmas
            | Self::BoxingDay
            | Self::EasterSunday
            | Self::EasterMonday
            | Self::GrandFinalEve
            | Self::GoodFriday
            | Self::MelbourneCup => false,
            _ => true,
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            SpecialDayKind::NewYearsDay => "NYD",
            SpecialDayKind::EightHourDay => "8 Hour Day",
            SpecialDayKind::AnzacDay => "Anzac Day",
            SpecialDayKind::MayDay => "May Day",
            SpecialDayKind::KingsBirthDay => "King's Birthday",
            SpecialDayKind::Xmas => "Christmas Day",
            SpecialDayKind::BoxingDay => "Boxing Day",
            SpecialDayKind::EasterSunday => "Easter Sunday",
            SpecialDayKind::EasterMonday => "Easter Monday",
            SpecialDayKind::GrandFinalEve => "Grand Final Eve",
            SpecialDayKind::MarchEquinox => "March Equinox",
            SpecialDayKind::GoodFriday => "Good Friday",
            SpecialDayKind::MothersDay => "Mother's Day",
            SpecialDayKind::WinterSolstice => "Winter Solstice",
            SpecialDayKind::FathersDay => "Father's Day",
            SpecialDayKind::SeptemberEquinox => "September Equinox",
            SpecialDayKind::SummerSolstice => "Summer Solstice",
            SpecialDayKind::MelbourneCup => "Melbourne Cup",
            SpecialDayKind::SuperannuationDay => "Superannuation Day",
        }
    }
}

pub struct SpecialDay {
    kind: SpecialDayKind,
    month: u32,
    day: u32,
}

pub const STATIC_SPECIAL_DAYS: &[SpecialDay] = &[
    SpecialDay::new(SpecialDayKind::NewYearsDay, 1, 1),
    SpecialDay::new(SpecialDayKind::EightHourDay, 3, 10),
    SpecialDay::new(SpecialDayKind::KingsBirthDay, 1, 18),
    SpecialDay::new(SpecialDayKind::AnzacDay, 4, 25),
    SpecialDay::new(SpecialDayKind::MayDay, 5, 1),
    SpecialDay::new(SpecialDayKind::SuperannuationDay, 7, 1),
    SpecialDay::new(SpecialDayKind::Xmas, 12, 25),
    SpecialDay::new(SpecialDayKind::BoxingDay, 12, 26),
];

// pub fn special_days_for_date(date: &NaiveDate) ->

impl SpecialDay {
    pub const fn new(kind: SpecialDayKind, month: u32, day: u32) -> Self {
        Self { kind, month, day }
    }

    #[allow(dead_code)]
    pub const fn is_trading_day(&self) -> bool {
        self.kind.is_trading_day()
    }

    pub const fn on(&self, date: NaiveDate) -> bool {
        self.month == date.const_month() && self.day == date.const_day()
    }
}

pub const START_YEAR: i32 = 2025;
pub const END_YEAR: i32 = 2100;

const fn max_speical_day_count() -> usize {
    let mut max = 0;
    let mut year = START_YEAR;

    loop {
        if year >= END_YEAR {
            break;
        }

        let year_dynamic_days = DYNAMIC_SPECIAL_DAYS[(year - START_YEAR) as usize];
        const_for!(i in 0..STATIC_SPECIAL_DAYS.len() => {
            let mut count = 0;
            let day = NaiveDate::from_ymd_opt(year, STATIC_SPECIAL_DAYS[i].month, STATIC_SPECIAL_DAYS[i].day).unwrap();
            const_for!(j in 0..STATIC_SPECIAL_DAYS.len() => {
                if j == i {
                    continue;
                }

                if STATIC_SPECIAL_DAYS[j].on(day) {
                    count += 1;
                }
            });

            const_for!(j in 0..year_dynamic_days.len() => {
                if year_dynamic_days[j].on(day) {
                    count += 1;
                }
            });

            if count > max {
                max = count;
            }
        });

        const_for!(i in 0..year_dynamic_days.len() => {
            let mut count = 0;
            let day = NaiveDate::from_ymd_opt(year, year_dynamic_days[i].month, year_dynamic_days[i].day).unwrap();
            const_for!(j in 0..STATIC_SPECIAL_DAYS.len() => {
                if STATIC_SPECIAL_DAYS[j].on(day) {
                    count += 1;
                }
            });

            const_for!(j in 0..year_dynamic_days.len() => {
                if j == i {
                    continue;
                }

                if year_dynamic_days[j].on(day) {
                    count += 1;
                }
            });

            if count > max {
                max = count;
            }
        });

        year += 1
    }

    if max >= 2 {
        panic!("overlap found");
    }

    max
}

const MAX_SPECIAL_DAYS: usize = max_speical_day_count();

pub type SpecialDays = [Option<SpecialDayKind>; MAX_SPECIAL_DAYS];

fn speical_days_for_date(day: NaiveDate) -> SpecialDays {
    let mut result: SpecialDays = Default::default();
    let mut top = 0;

    let year_index = (day.const_year() - START_YEAR) as usize % DYNAMIC_SPECIAL_DAYS.len();

    for speical_day in STATIC_SPECIAL_DAYS
        .iter()
        .chain(DYNAMIC_SPECIAL_DAYS[year_index].iter())
    {
        if speical_day.on(day) {
            result[top] = Some(speical_day.kind);
            top += 1;
        }
    }

    result
}

pub struct SpecialDayUpdater {
    last_date: NaiveDate,
    special_days: SpecialDays,
}

impl SpecialDayUpdater {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            last_date: date,
            special_days: speical_days_for_date(date),
        }
    }

    pub fn update(&mut self, date: NaiveDate) {
        if self.last_date == date {
            return;
        }

        self.special_days = speical_days_for_date(date);
    }

    pub fn special_days(&self) -> &SpecialDays {
        &self.special_days
    }

    pub fn is_non_trading_day(&self) -> bool {
        self.non_trading_day().is_some()
    }

    pub fn non_trading_day(&self) -> Option<SpecialDayKind> {
        for day in self.special_days {
            if let Some(day) = day {
                if !day.is_trading_day() {
                    return Some(day);
                }
            }
        }

        None
    }
}

#[derive(Debug, Clone, Copy, EnumIter, Default)]
pub enum MoonPhase {
    #[default]
    NewMoon,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    FullMoon,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

impl From<f64> for MoonPhase {
    fn from(v: f64) -> Self {
        // Multiply into 0..8 and round to nearest phase
        let idx = libm::floor((v * 8.0) + 0.5) as u8 % 8;

        match idx {
            0 => MoonPhase::NewMoon,
            1 => MoonPhase::WaxingCrescent,
            2 => MoonPhase::FirstQuarter,
            3 => MoonPhase::WaxingGibbous,
            4 => MoonPhase::FullMoon,
            5 => MoonPhase::WaningGibbous,
            6 => MoonPhase::LastQuarter,
            7 => MoonPhase::WaningCrescent,
            _ => MoonPhase::NewMoon,
        }
    }
}

#[derive(Default)]
pub struct MoonRender {
    pub pos: Vec2,
    pub since_ce: i32,
}

impl MoonRender {
    pub fn frame_index(&self) -> usize {
        self.since_ce.unsigned_abs() as usize % assets::FRAMES_MOON_ANIME.len()
    }
}

impl Sprite for MoonRender {
    fn pos(&self) -> &Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl assets::Image {
        assets::FRAMES_MOON_ANIME[self.frame_index()].frame
    }
}

impl SpriteMask for MoonRender {
    fn image_mask(&self) -> &impl assets::Image {
        assets::FRAMES_MOON_ANIME_MASK[self.frame_index()].frame
    }
}
