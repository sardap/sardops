use core::time::Duration;

use bincode::{Decode, Encode};
use fastrand::Rng;

use crate::{
    death::{passed_threshold_chance, DeathCause},
    food::Food,
    game_consts::{
        BREED_ODDS_THRESHOLD, DEATH_BY_LIGHTING_STRIKE_ODDS, DEATH_BY_TOXIC_SHOCK_LARGE,
        DEATH_BY_TOXIC_SHOCK_SMALL, DEATH_CHECK_INTERVAL, DEATH_STARVE_THRESHOLDS,
        HUNGER_LOSS_PER_SECOND, OLD_AGE_THRESHOLD, POOP_INTERVNAL,
    },
    pet::definition::{
        PetAnimationSet, PetDefinition, PetDefinitionId, PET_CKCS_ID, PET_PAWN_WHITE_ID,
    },
    poop::{poop_count, Poop, MAX_POOPS},
    Timestamp,
};

pub mod definition;
pub mod record;
pub mod render;

pub type PetName = fixedstr::str7;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone)]
pub enum StomachMood {
    Full { elapsed: Duration },
    Starving { elapsed: Duration },
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone)]
pub struct ParentInfo {
    upid: UniquePetId,
    def_id: PetDefinitionId,
}

impl ParentInfo {
    pub const fn new(upid: UniquePetId, def_id: PetDefinitionId) -> Self {
        Self { upid, def_id }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone)]
pub struct PetParents {
    values: [ParentInfo; 2],
}

impl PetParents {
    pub const fn new(values: [ParentInfo; 2]) -> Self {
        Self { values }
    }
}

pub type UniquePetId = u64;

pub fn combine_pid(a: UniquePetId, b: UniquePetId) -> UniquePetId {
    const EVEN: UniquePetId = 0b1010101010101010101010101010101010101010101010101010101010101010;
    let combined = (a & EVEN) | (b & !EVEN);
    gen_pid(&mut fastrand::Rng::with_seed(combined))
}

pub fn gen_pid(rng: &mut fastrand::Rng) -> UniquePetId {
    rng.u64(u64::MIN..0xFFFFFFFFFF)
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone)]
pub struct PetInstance {
    pub upid: UniquePetId,
    pub def_id: PetDefinitionId,
    pub name: PetName,
    pub born: Timestamp,
    pub age: Duration,
    pub life_stage_age: Duration,
    pub stomach_mood: StomachMood,
    pub total_starve_time: Duration,
    pub stomach_filled: f32,
    pub extra_weight: f32,
    pub since_poop: Duration,
    pub since_game: Duration,
    pub since_death_check: Duration,
    pub should_die: Option<DeathCause>,
    pub parents: Option<PetParents>,
    mood: Mood,
    should_breed: bool,
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
        self.life_stage_age += delta;
    }

    pub fn tick_hunger(&mut self, delta: Duration, sleep: bool) {
        const GRAMS_LOSS_PER_SECOND: f32 = 1.;
        let sleep_modifer = if sleep { 0.4 } else { 1. };
        self.extra_weight = (self.extra_weight
            - GRAMS_LOSS_PER_SECOND * delta.as_secs_f32() * sleep_modifer)
            .max(0.);
        self.stomach_filled = (self.stomach_filled
            - HUNGER_LOSS_PER_SECOND * delta.as_secs_f32() * sleep_modifer)
            .max(0.);
        if !sleep && self.stomach_filled <= 0. {
            let elapsed = if let StomachMood::Starving { elapsed } = self.stomach_mood {
                self.total_starve_time += delta;
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

    pub fn tick_death(
        &mut self,
        delta: Duration,
        rng: &mut fastrand::Rng,
        sleep: bool,
        poop_count: u8,
    ) {
        if sleep || self.should_die.is_some() {
            return;
        }

        self.since_death_check += delta;

        if self.since_death_check > DEATH_CHECK_INTERVAL {
            // Random death
            if rng.f32() < DEATH_BY_LIGHTING_STRIKE_ODDS {
                self.should_die = Some(DeathCause::LightingStrike);
                return;
            }

            if let StomachMood::Starving { elapsed } = self.stomach_mood {
                if passed_threshold_chance(rng, DEATH_STARVE_THRESHOLDS, elapsed) {
                    self.should_die = Some(DeathCause::Starvation);
                }
            }

            if passed_threshold_chance(rng, OLD_AGE_THRESHOLD, self.age) {
                self.should_die = Some(DeathCause::OldAge);
            }

            if !sleep {
                if poop_count >= MAX_POOPS as u8 {
                    if rng.f32() < DEATH_BY_TOXIC_SHOCK_LARGE {
                        self.should_die = Some(DeathCause::ToxicShock);
                        return;
                    }
                } else if poop_count >= (MAX_POOPS / 2) as u8 {
                    if rng.f32() < DEATH_BY_TOXIC_SHOCK_SMALL {
                        self.should_die = Some(DeathCause::ToxicShock);
                        return;
                    }
                }
            }

            self.since_death_check = Duration::ZERO;
        }
    }

    pub fn played_game(&mut self) {
        self.since_game = Duration::ZERO;
    }

    pub fn should_die(&self) -> Option<DeathCause> {
        self.should_die
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
            crate::pet::definition::PET_BLOB_ID => {
                if self.age > Duration::from_hours(4) {
                    return Some(PET_PAWN_WHITE_ID);
                }
            }
            crate::pet::definition::PET_PAWN_WHITE_ID => {
                if self.age > Duration::from_hours(24) {
                    return Some(PET_CKCS_ID);
                }
            }
            _ => {}
        }

        return None;
    }

    pub fn evolve(&mut self, next: PetDefinitionId) {
        self.life_stage_age = Duration::ZERO;
        self.def_id = next;
    }

    pub fn stomach_mood(&self) -> StomachMood {
        return self.stomach_mood;
    }

    pub fn mood(&self) -> Mood {
        self.mood
    }

    pub fn tick_mood(&mut self, poops: &[Option<Poop>]) {
        self.mood = self.calc_mood(poops);
    }

    fn calc_mood(&self, poops: &[Option<Poop>]) -> Mood {
        let is_starved = matches!(self.stomach_mood, StomachMood::Starving { elapsed: _ });

        if is_starved || poop_count(poops) > 0 {
            return Mood::Sad;
        }

        let tummy_full = matches!(self.stomach_mood, StomachMood::Full { elapsed: _ });

        if tummy_full {
            return Mood::Happy;
        }

        return Mood::Normal;
    }

    pub fn weight(&self) -> f32 {
        self.extra_weight + self.definition().base_weight
    }

    fn can_breed(&self) -> bool {
        if self.definition().life_stage != LifeStage::Adult {
            return false;
        }

        self.mood != Mood::Sad
    }

    pub fn should_breed(&self) -> bool {
        self.should_breed
    }

    pub fn tick_breed(&mut self, rng: &mut fastrand::Rng, egg_exists: bool) {
        if egg_exists {
            self.should_breed = false;
            return;
        }

        if !self.can_breed() || self.should_breed {
            return;
        }

        if passed_threshold_chance(rng, BREED_ODDS_THRESHOLD, self.life_stage_age) {
            self.should_breed = true;
        }
    }
}

impl Default for PetInstance {
    fn default() -> Self {
        Self {
            upid: 0,
            def_id: crate::pet::definition::PET_BLOB_ID,
            name: fixedstr::str_format!(PetName, "AAAAAAAA"),
            born: Timestamp::default(),
            age: Duration::ZERO,
            life_stage_age: Duration::ZERO,
            stomach_mood: StomachMood::Full {
                elapsed: Duration::ZERO,
            },
            total_starve_time: Duration::ZERO,
            stomach_filled: 0.,
            extra_weight: 0.,
            since_poop: Duration::ZERO,
            since_game: Duration::ZERO,
            since_death_check: Duration::ZERO,
            should_die: None,
            parents: None,
            mood: Mood::Normal,
            should_breed: false,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LifeStage {
    Baby,
    Child,
    Adult,
    Elder,
}
