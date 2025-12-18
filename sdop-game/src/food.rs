use bincode::{Decode, Encode};
use const_for::const_for;
use core::{i32, time::Duration};

use crate::{
    Timestamp,
    assets::{self, StaticImage},
    items::ItemKind,
};

include!(concat!(env!("OUT_DIR"), "/dist_foods.rs"));

#[derive(Clone, Copy)]
pub struct Food {
    pub id: usize,
    pub name: &'static str,
    pub image: &'static StaticImage,
    pub fill_factor: f32,
    pub item: ItemKind,
    pub expire: Duration,
    pub max_eat: usize,
}

impl Food {
    pub const fn new(
        id: usize,
        name: &'static str,
        image: &'static StaticImage,
        fill_factor: f32,
        item: ItemKind,
        max_eat: usize,
        expire: Duration,
    ) -> Self {
        Self {
            id,
            name,
            image,
            fill_factor,
            item,
            expire,
            max_eat,
        }
    }

    pub fn get_by_id(id: usize) -> &'static Self {
        if id >= FOODS.len() {
            return FOODS[0];
        }
        FOODS[id]
    }

    pub fn get_eat_expire(&self) -> Duration {
        self.expire
    }
}

impl Eq for Food {}

impl PartialEq for Food {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

const fn max_food_width() -> i32 {
    let mut max = 0;
    const_for!(i in 0..FOODS.len() => {
        if FOODS[i].image.size.x as i32 > max {
            max = FOODS[i].image.size.x as i32;
        }
    });

    max
}

pub const MAX_FOOD_X: i32 = max_food_width();

pub const FOOD_HISTORY_SIZE: usize = 5;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone, Default)]
pub struct FoodHistoryEntry {
    history: [Option<Timestamp>; FOOD_HISTORY_SIZE],
    total: u32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone, Default)]
pub struct FoodHistory {
    entries: [FoodHistoryEntry; FOOD_COUNT],
    next_refresh: Timestamp,
}

impl FoodHistory {
    pub fn add(&mut self, food: &Food, now: Timestamp) {
        let entry = &mut self.entries[food.id];

        for i in 0..entry.history.len() {
            if entry.history[i].is_none() {
                entry.history[i] = Some(now + food.get_eat_expire());
                return;
            }
        }

        entry.total = entry.total.checked_add(1).unwrap_or(u32::MAX)
    }

    pub fn sick_of(&self, food: &Food) -> bool {
        self.entries[food.id].history[food.max_eat - 1].is_some()
    }

    pub fn consumed_count(&self, food: &Food) -> usize {
        for i in (0..food.max_eat).rev() {
            if self.entries[food.id].history[i].is_some() {
                return i + 1;
            }
        }

        0
    }

    pub fn sim_tick(&mut self, now: Timestamp) {
        if now < self.next_refresh {
            return;
        }

        for (i, entry) in self.entries.iter_mut().enumerate() {
            let food = FOODS[i];
            for i in 0..food.max_eat {
                if let Some(expire) = entry.history[i]
                    && expire < now
                {
                    entry.history[i] = None;
                }
            }

            let mut write_index = 0;
            for i in 0..food.max_eat {
                if entry.history[i].is_some() {
                    entry.history.swap(write_index, i);
                    write_index += 1;
                }
            }
        }

        self.next_refresh = now + Duration::from_secs(1);
    }

    pub fn get_entry(&self, food: &Food) -> &FoodHistoryEntry {
        &self.entries[food.id]
    }
}
