use crate::assets::{self, StaticImage};

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

pub const STARTING_FOOD: &[&'static Food] = &[&FOOD_BISCUIT];
