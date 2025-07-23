use core::time::Duration;

use bincode::{Decode, Encode};
use fastrand::Rng;

use crate::{
    Timestamp,
    date_utils::duration_from_hours,
    food::Food,
    pet::definition::{
        PET_BLOB_ID, PET_CKCS_ID, PET_PAWN_WHITE_ID, PetDefinition, PetDefinitionId,
    },
};

pub mod definition;
pub mod render;

#[derive(Encode, Decode, Copy, Clone)]
pub struct PetInstance {
    pub def_id: PetDefinitionId,
    pub born: Timestamp,
    pub age: Duration,
    pub stomach_filled: f32,
    pub extra_weight: f32,
}

impl PetInstance {
    pub fn definition(&self) -> &'static PetDefinition {
        PetDefinition::get_by_id(self.def_id)
    }

    pub fn digest(&mut self, food: &Food) {
        self.stomach_filled = (self.stomach_filled
            + food.fill_factor * self.definition().food_multiplier(food))
        .min(self.definition().stomach_size);
    }

    pub fn tick_age(&mut self, delta: Duration) {
        self.age += delta;
    }

    pub fn tick_hunger(&mut self, delta: Duration) {
        const GRAMS_LOSS_PER_SECOND: f32 = 1.;
        self.extra_weight =
            (self.extra_weight - GRAMS_LOSS_PER_SECOND * delta.as_secs_f32()).max(0.);
        const HUNGER_LOSS_PER_SECOND: f32 = 0.1;
        self.stomach_filled =
            (self.stomach_filled - HUNGER_LOSS_PER_SECOND * delta.as_secs_f32()).max(0.);
    }

    pub fn should_evolve(&mut self, _rng: &mut Rng) -> Option<PetDefinitionId> {
        match self.def_id {
            PET_BLOB_ID => {
                if self.age > duration_from_hours(1.).div_f32(10.) {
                    return Some(PET_PAWN_WHITE_ID);
                }
            }
            PET_PAWN_WHITE_ID => {
                if self.age > duration_from_hours(2.).div_f32(10.) {
                    return Some(PET_CKCS_ID);
                }
            }
            _ => {}
        }

        return None;
    }
}

impl Default for PetInstance {
    fn default() -> Self {
        Self {
            def_id: 0,
            born: Timestamp::default(),
            age: Duration::default(),
            stomach_filled: 0.,
            extra_weight: 0.,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mood {
    Normal,
}

impl Default for Mood {
    fn default() -> Self {
        Mood::Normal
    }
}
