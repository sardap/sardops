use core::time::Duration;

use bincode::{Decode, Encode};
use chrono::{Datelike, NaiveTime, TimeDelta, WeekdaySet};
use glam::Vec2;

use crate::{
    Timestamp,
    anime::{Anime, HasAnime},
    assets::{FRAMES_ALARM, FRAMES_ALARM_MASK, IMAGE_ALARM_0},
    bit_array::bytes_for_bits,
    sprite::{Sprite, SpriteMask},
};

pub struct AlarmRender {
    pos: Vec2,
    anime: Anime,
    rining: bool,
}

impl AlarmRender {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            anime: Anime::new(&FRAMES_ALARM).with_mask(&FRAMES_ALARM_MASK),
            rining: false,
        }
    }

    pub fn with_rining(mut self) -> Self {
        self.rining = true;
        self
    }

    pub fn set_rining(&mut self, rining: bool) {
        self.rining = rining;
    }

    pub const fn size() -> Vec2 {
        IMAGE_ALARM_0.const_size_vec2()
    }
}

impl HasAnime for AlarmRender {
    fn anime(&mut self) -> &mut Anime {
        &mut self.anime
    }
}

impl Sprite for AlarmRender {
    fn pos(&self) -> &Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl crate::assets::Image {
        if self.rining {
            self.anime.current_frame()
        } else {
            FRAMES_ALARM[0].frame
        }
    }
}

impl SpriteMask for AlarmRender {
    fn image_mask(&self) -> &impl crate::assets::Image {
        if self.rining {
            self.anime.current_frame_mask().unwrap()
        } else {
            FRAMES_ALARM_MASK[0].frame
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Encode, Decode)]
pub enum AlarmConfig {
    None,
    Time { days: WeekdaySet, time: NaiveTime },
}

impl Default for AlarmConfig {
    fn default() -> Self {
        Self::None
    }
}

impl AlarmConfig {
    pub fn should_trigger(&self, current: &Timestamp) -> bool {
        match self {
            AlarmConfig::None => {}
            AlarmConfig::Time {
                days,
                time: target_time,
            } => {
                let inner = current.inner();

                if days.contains(inner.date().weekday()) {
                    let current_time = inner.time();
                    let delta = current_time - *target_time;

                    if delta > TimeDelta::zero() && delta < TimeDelta::minutes(3) {
                        return true;
                    }
                }
            }
        }

        false
    }
}

pub struct AlarmState {
    config: AlarmConfig,
    pub triggered_time: Option<Timestamp>,
    pub acked: bool,
    pub ringing: bool,
}

impl Default for AlarmState {
    fn default() -> Self {
        Self::new(AlarmConfig::default())
    }
}

impl AlarmState {
    pub const fn new(config: AlarmConfig) -> Self {
        Self {
            config,
            triggered_time: None,
            acked: false,
            ringing: false,
        }
    }

    pub fn tick(&mut self, time: &Timestamp) {
        if let Some(triggered_time) = self.triggered_time
            && *time - triggered_time > Duration::from_mins(3)
        {
            self.triggered_time = None;
            self.acked = false;
        }

        if self.config.should_trigger(time) && !self.acked {
            if self.triggered_time.is_none() {
                self.triggered_time = Some(*time);
            }
            self.ringing = true;
            return;
        }

        self.ringing = false;
    }

    pub fn should_be_rining(&self) -> bool {
        self.ringing
    }

    pub fn ack(&mut self) {
        self.acked = true;
    }

    pub fn config(&self) -> &AlarmConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: AlarmConfig) {
        self.config = config;
        self.acked = false;
        self.ringing = false;
        self.triggered_time = None;
    }
}
