use const_for::const_for;
use core::i32;

use crate::{
    assets::{self, StaticImage},
    items::ItemKind,
};

include!(concat!(env!("OUT_DIR"), "/dist_foods.rs"));

#[derive(Clone, Copy)]
pub struct Food {
    pub id: u32,
    pub name: &'static str,
    pub image: &'static StaticImage,
    pub fill_factor: f32,
    pub item: ItemKind,
}

impl Food {
    pub const fn new(
        id: u32,
        name: &'static str,
        image: &'static StaticImage,
        fill_factor: f32,
        item: ItemKind,
    ) -> Self {
        Self {
            id,
            name,
            image,
            fill_factor,
            item,
        }
    }

    pub fn get_by_id(id: usize) -> &'static Self {
        if id >= FOODS.len() {
            return FOODS[0];
        }
        FOODS[id]
    }
}

pub const STARTING_FOOD: &[&Food] = &[&FOOD_BISCUIT];

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
