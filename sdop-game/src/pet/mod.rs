use core::time::Duration;

use bincode::{de, Decode, Encode};
use fastrand::Rng;

use crate::{
    date_utils::duration_from_hours,
    food::Food,
    game_context::GameContext,
    pet::definition::{
        PetAnimationSet, PetDefinition, PetDefinitionId, PET_BLOB_ID, PET_CKCS_ID,
        PET_PAWN_WHITE_ID,
    },
    poop::{poop_count, Poop, POOP_INTERVNAL},
    Timestamp,
};

pub mod definition;
pub mod render;

#[derive(Encode, Decode, Copy, Clone)]
pub enum StomachMood {
    Full { elapsed: Duration },
    Starving { elapsed: Duration },
}

#[derive(Encode, Decode, Copy, Clone)]
pub struct PetInstance {
    pub def_id: PetDefinitionId,
    pub born: Timestamp,
    pub age: Duration,
    pub stomach_mood: StomachMood,
    pub stomach_filled: f32,
    pub extra_weight: f32,
    pub since_poop: Duration,
    pub since_game: Duration,
}

impl PetInstance {
    pub fn definition(&self) -> &'static PetDefinition {
        PetDefinition::get_by_id(self.def_id)
    }

    pub fn digest(&mut self, food: &Food) {
        self.stomach_filled =
            self.stomach_filled + food.fill_factor * self.definition().food_multiplier(food);
        let extra = self.stomach_filled - self.definition().stomach_size;
        if extra > 0. {
            self.stomach_filled = self.definition().stomach_size;
            self.extra_weight += extra;
        }
    }

    pub fn tick_age(&mut self, delta: Duration) {
        self.age += delta;
    }

    pub fn tick_hunger(&mut self, delta: Duration, sleep: bool) {
        const GRAMS_LOSS_PER_SECOND: f32 = 1.;
        let sleep_modifer = if sleep { 0.4 } else { 1. };
        self.extra_weight = (self.extra_weight
            - GRAMS_LOSS_PER_SECOND * delta.as_secs_f32() * sleep_modifer)
            .max(0.);
        const HUNGER_LOSS_PER_SECOND: f32 = 0.1;
        self.stomach_filled = (self.stomach_filled
            - HUNGER_LOSS_PER_SECOND * delta.as_secs_f32() * sleep_modifer)
            .max(0.);
        if !sleep && self.stomach_filled <= 0. {
            let elapsed = if let StomachMood::Starving { elapsed } = self.stomach_mood {
                elapsed
            } else {
                Duration::ZERO
            };

            self.stomach_mood = StomachMood::Starving {
                elapsed: elapsed + delta,
            }
        } else if self.stomach_filled > 10. {
            let elapsed = if let StomachMood::Full { elapsed } = self.stomach_mood {
                elapsed
            } else {
                Duration::ZERO
            };

            self.stomach_mood = StomachMood::Full {
                elapsed: elapsed + delta,
            }
        }
    }

    pub fn tick_poop(&mut self, delta: Duration) {
        if self.stomach_filled == 0. {
            return;
        }

        self.since_poop += delta;
    }

    pub fn tick_since_game(&mut self, delta: Duration, sleep: bool) {
        if !sleep {
            self.since_game += delta;
        }
    }

    pub fn played_game(&mut self) {
        self.since_game = Duration::ZERO;
    }

    pub fn should_poop(&mut self, sleeping: bool) -> bool {
        if !sleeping
            && self
                .since_poop
                .mul_f32(self.definition().poop_time_multiplier())
                > POOP_INTERVNAL
        {
            self.since_poop = Duration::ZERO;
            return true;
        }

        false
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

    pub fn stomach_mood(&self) -> StomachMood {
        return self.stomach_mood;
    }

    pub fn mood(&self, poops: &[Option<Poop>]) -> Mood {
        let is_starved = matches!(self.stomach_mood, StomachMood::Starving { elapsed: _ });

        if is_starved || poop_count(poops) > 0 || self.since_game > Duration::from_hours(6) {
            return Mood::Sad;
        }

        let tummy_full_time = if let StomachMood::Full { elapsed } = self.stomach_mood {
            elapsed
        } else {
            Duration::ZERO
        };

        if tummy_full_time > Duration::from_secs(60) {
            return Mood::Happy;
        }

        return Mood::Normal;
    }
}

impl Default for PetInstance {
    fn default() -> Self {
        Self {
            def_id: crate::pet::definition::PET_CKCS_ID,
            born: Timestamp::default(),
            age: Duration::ZERO,
            stomach_mood: StomachMood::Full {
                elapsed: Duration::ZERO,
            },
            stomach_filled: 0.,
            extra_weight: 0.,
            since_poop: Duration::ZERO,
            since_game: Duration::ZERO,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mood {
    Normal,
    Sad,
    Happy,
}

impl Default for Mood {
    fn default() -> Self {
        Mood::Normal
    }
}

impl Mood {
    pub fn anime_set(&self) -> PetAnimationSet {
        match self {
            Mood::Normal => PetAnimationSet::Normal,
            Mood::Sad => PetAnimationSet::Sad,
            Mood::Happy => PetAnimationSet::Happy,
        }
    }
}
