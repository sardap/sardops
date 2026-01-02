use std::{
    fmt::{self},
    time::Duration,
};

use regex::Regex;
use sdop_common::ItemCategory;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, Visitor},
};
use strum_macros::{Display, EnumString};

pub const ASSETS_PATH: &str = "../assets";
pub const IMAGES_MISC_PATH: &str = "../assets/images/misc";
pub const IMAGES_TILESETS_PATH: &str = "../assets/images/misc/tilesets";
pub const PETS_RON_PATH: &str = "../assets/pets.ron";
pub const FOODS_RON_PATH: &str = "../assets/foods.ron";
pub const ITEMS_RON_PATH: &str = "../assets/items.ron";
pub const LOCATIONS_RON_PATH: &str = "../assets/locations.ron";
pub const SOUNDS_PATH: &str = "../assets/sounds";

#[derive(Debug, Clone, Copy)]
pub struct SdopDuration {
    pub duration: Duration,
}

fn parse_duration_string(s: &str) -> Duration {
    let re = Regex::new(r"(?P<hours>\d+h)?\s*(?P<minutes>\d+m)?\s*(?P<seconds>\d+s)?").unwrap();

    let captures = re.captures(s).unwrap();

    let mut total_duration = Duration::new(0, 0);

    if let Some(m_match) = captures.name("hours") {
        let val_str = m_match.as_str().trim_end_matches('h');
        if let Ok(hours) = val_str.parse::<u64>() {
            total_duration += Duration::from_hours(hours);
        }
    }

    if let Some(m_match) = captures.name("minutes") {
        let val_str = m_match.as_str().trim_end_matches('m');
        if let Ok(minutes) = val_str.parse::<u64>() {
            total_duration += Duration::from_mins(minutes);
        }
    }

    if let Some(s_match) = captures.name("seconds") {
        let val_str = s_match.as_str().trim_end_matches('s');
        if let Ok(seconds) = val_str.parse::<u64>() {
            total_duration += Duration::from_secs(seconds);
        }
    }

    total_duration
}

struct SdopDurationVisitor;

impl<'de> Visitor<'de> for SdopDurationVisitor {
    type Value = SdopDuration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a duration string like '5m 30s', '10m', or '45s'")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(SdopDuration {
            duration: parse_duration_string(value),
        })
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let s = std::str::from_utf8(v).map_err(E::custom)?;
        self.visit_str(s)
    }
}

impl<'de> Deserialize<'de> for SdopDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SdopDurationVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemReward {
    pub item: String,
    pub odds: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationRewards {
    pub money_start: i32,
    pub money_end: i32,
    pub items: Vec<ItemReward>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct LocationTemplate {
    pub name: String,
    pub length: SdopDuration,
    pub cooldown: SdopDuration,
    pub difficulty: i32,
    pub activities: Vec<String>,
    pub rewards: LocationRewards,
    #[serde(default = "default_true")]
    pub in_shop: bool,
    pub life_stages: Vec<sdop_common::LifeStage>,
}

#[derive(Serialize, Deserialize, EnumString, Display)]
pub enum RarityEnum {
    Common,
    Rare,
}

#[derive(Serialize, Deserialize)]
pub struct ItemTemplate {
    pub name: String,
    pub category: ItemCategory,
    pub cost: i32,
    pub rarity: RarityEnum,
    pub image: String,
    pub unique: bool,
    pub desc: String,
    #[serde(default)]
    pub fishing_odds: f32,
    #[serde(default = "default_true")]
    pub in_shop: bool,
    #[serde(default)]
    pub skill: i32,
}
