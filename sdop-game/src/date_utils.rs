use core::{
    ops::{Add, Sub},
    time::Duration,
};

use bincode::{Decode, Encode};
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

impl Default for Timestamp {
    fn default() -> Self {
        Self(Default::default())
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
