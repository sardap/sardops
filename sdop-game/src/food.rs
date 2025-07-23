use bincode::{Decode, Encode};

use crate::{
    assets::{self, StaticImage},
    bit_array::{BitArray, bytes_for_bits},
};

include!(concat!(env!("OUT_DIR"), "/dist_foods.rs"));

#[derive(Clone, Copy)]
pub struct Food {
    pub id: u32,
    pub name: &'static str,
    pub image: &'static StaticImage,
    pub fill_factor: f32,
}

impl Food {
    pub const fn new(
        id: u32,
        name: &'static str,
        image: &'static StaticImage,
        fill_factor: f32,
    ) -> Self {
        Self {
            id,
            name,
            image,
            fill_factor,
        }
    }

    pub fn get_by_id(id: usize) -> &'static Self {
        if id >= FOODS.len() {
            return &FOODS[0];
        }
        &FOODS[id]
    }
}

const STARTING_FOOD: &[&'static Food] = &[&FOOD_BISCUIT, &FOOD_SANDWICH, &FOOD_SOUP];

const FOOD_SAVE_MAX: usize = bytes_for_bits(100);

#[derive(Clone, Copy, Encode, Decode)]
pub struct UnlockedFood {
    data: BitArray<FOOD_SAVE_MAX>,
}

impl Default for UnlockedFood {
    fn default() -> Self {
        let mut result = Self {
            data: Default::default(),
        };
        for food in STARTING_FOOD {
            result.unlock(food);
        }
        result
    }
}

impl UnlockedFood {
    pub fn unlock(&mut self, food: &Food) {
        self.data.set_bit(food.id as usize, true);
    }

    pub fn is_unlocked_id(&self, id: u32) -> bool {
        self.data.get_bit(id as usize)
    }

    pub fn is_unlocked(&self, food: &Food) -> bool {
        self.data.get_bit(food.id as usize)
    }
}
