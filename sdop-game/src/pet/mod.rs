use core::time::Duration;

use bincode::{Decode, Encode};
use heapless::Vec;

use crate::{
    Timestamp,
    book::BookHistory,
    death::{DeathCause, get_threshold_odds, passed_threshold_chance},
    food::Food,
    furniture::{HomeFurnitureKind, HomeLayout},
    game_consts::{
        BREED_ODDS_THRESHOLD, DEATH_BY_HYPOTHERMIA_THRESHOLD, DEATH_BY_ILLNESS_THRESHOLD,
        DEATH_BY_LIGHTING_STRIKE_ODDS, DEATH_CHECK_INTERVERAL, DEATH_STARVE_THRESHOLDS,
        DEATH_TOXIC_SHOCK_THRESHOLD, EVOLVE_CHECK_INTERVERAL, HEALING_COST_RANGE,
        HUNGER_LOSS_PER_SECOND, ILLNESS_BABY_ODDS, ILLNESS_BASE_ODDS, ILLNESS_CHILD_ODDS,
        ILLNESS_SINCE_GAME_DURATION, ILLNESS_SINCE_GAME_ODDS, ILLNESS_SINCE_ODDS,
        ILLNESS_STARVING_ODDS, OLD_AGE_THRESHOLD, POOP_INTERVNAL, RANDOM_NAMES, SPLACE_LOCATIONS,
    },
    items::{Inventory, ItemKind},
    money::Money,
    pet::definition::{
        PET_BALLOTEE_ID, PET_BEERIE_ID, PET_BRAINO_ID, PET_CKCS_ID, PET_COMPUTIE_ID, PET_COUNT,
        PET_DEVIL_ID, PET_HUMBIE_ID, PET_PAWN_WHITE_ID, PET_SICKO_ID, PET_SNOWMAN_ID,
        PET_WAS_GAURD_ID, PetAnimationSet, PetDefinition, PetDefinitionId,
    },
    poop::{Poop, poop_count},
    temperature::TemperatureLevel,
};

pub mod definition;
pub mod record;
pub mod render;

pub type PetName = fixedstr::str7;

pub fn random_name(rng: &mut fastrand::Rng) -> PetName {
    let mut result = PetName::new();

    let name = rng.choice(RANDOM_NAMES.iter()).cloned().unwrap_or("");
    result.push(name);

    result
}

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
    name: PetName,
}

impl ParentInfo {
    pub fn upid(&self) -> UniquePetId {
        self.upid
    }

    pub fn def_id(&self) -> PetDefinitionId {
        self.def_id
    }

    pub fn name(&self) -> &PetName {
        &self.name
    }
}

impl ParentInfo {
    pub const fn new(upid: UniquePetId, def_id: PetDefinitionId, name: PetName) -> Self {
        Self { upid, def_id, name }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone)]
pub struct PetParents {
    pub values: [ParentInfo; 2],
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

pub fn planet_location_from_upid(upid: UniquePetId) -> (&'static str, u8) {
    let mut rng = fastrand::Rng::with_seed(upid);
    let planet = rng.choice(SPLACE_LOCATIONS.iter()).unwrap();
    let number = rng.u8(u8::MIN..u8::MAX);
    (planet, number)
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone, Default)]
struct PetIllness {
    since_ilness: Duration,
    with_ilness: Duration,
    #[cfg_attr(feature = "serde", serde(default))]
    cost: Money,
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
    pub since_evolve_check: Duration,
    pub should_die: Option<DeathCause>,
    should_evolve: Option<PetDefinitionId>,
    pub parents: Option<PetParents>,
    mood: Mood,
    should_breed: bool,
    pub book_history: BookHistory,
    illness: PetIllness,
    total_cold_for: Duration,
    cold_for: Duration,
    total_hot_for: Duration,
    pub seen_alien: bool,
}

impl PetInstance {
    pub fn definition(&self) -> &'static PetDefinition {
        PetDefinition::get_by_id(self.def_id)
    }

    pub fn digest(&mut self, food: &Food) {
        self.stomach_filled += food.fill_factor * self.definition().food_multiplier(food);
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
        const GRAMS_LOSS_PER_SECOND: f32 = 0.005;
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

        if self.since_death_check > DEATH_CHECK_INTERVERAL {
            // Random death
            if rng.f32() < DEATH_BY_LIGHTING_STRIKE_ODDS {
                self.should_die = Some(DeathCause::LightingStrike);
                return;
            }

            if let StomachMood::Starving { elapsed } = self.stomach_mood
                && passed_threshold_chance(rng, DEATH_STARVE_THRESHOLDS, elapsed)
            {
                self.should_die = Some(DeathCause::Starvation);
                return;
            }

            if passed_threshold_chance(rng, OLD_AGE_THRESHOLD, self.age) {
                self.should_die = Some(DeathCause::OldAge);
                return;
            }

            if !sleep && passed_threshold_chance(rng, DEATH_TOXIC_SHOCK_THRESHOLD, poop_count) {
                self.should_die = Some(DeathCause::ToxicShock);
                return;
            }

            if passed_threshold_chance(rng, DEATH_BY_ILLNESS_THRESHOLD, self.illness.with_ilness) {
                self.should_die = Some(DeathCause::Illness);
                return;
            }

            if self.def_id != PET_SNOWMAN_ID
                && passed_threshold_chance(rng, DEATH_BY_HYPOTHERMIA_THRESHOLD, self.cold_for)
            {
                self.should_die = Some(DeathCause::Hypothermia);
                return;
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

    pub fn should_die_of_leaving(&mut self) {
        self.should_die = Some(DeathCause::Leaving);
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

    pub fn tick_evolve(&mut self, delta: Duration, inv: &Inventory) {
        self.since_evolve_check += delta;

        if self.since_evolve_check < EVOLVE_CHECK_INTERVERAL {
            return;
        }

        self.since_evolve_check = Duration::ZERO;

        match self.definition().life_stage {
            LifeStage::Baby => {
                if self.life_stage_age < Duration::from_hours(4) {
                    return;
                }
            }
            LifeStage::Child => {
                if self.life_stage_age < Duration::from_days(1) {
                    return;
                }
            }
            LifeStage::Adult => {
                return;
            }
        }

        let mut rng = fastrand::Rng::with_seed(self.upid);

        let mut possible = Vec::<PetDefinitionId, PET_COUNT>::new();
        match self.definition().life_stage {
            LifeStage::Baby => {
                let _ = possible.push(PET_HUMBIE_ID);
                let _ = possible.push(PET_PAWN_WHITE_ID);
            }
            LifeStage::Child => {
                let _ = possible.push(PET_BEERIE_ID);
                let _ = possible.push(PET_WAS_GAURD_ID);
                let _ = possible.push(PET_BALLOTEE_ID);
                let _ = possible.push(PET_DEVIL_ID);
                if inv.has_item(ItemKind::PersonalComputer)
                    && inv.has_item(ItemKind::Screen)
                    && inv.has_item(ItemKind::Keyboard)
                {
                    let _ = possible.push(PET_COMPUTIE_ID);
                }
                if self.extra_weight > 50. {
                    let _ = possible.push(PET_CKCS_ID);
                }
                if self.is_ill() {
                    let _ = possible.push(PET_SICKO_ID);
                }
                if self.book_history.compelted_count() >= 3 {
                    let _ = possible.push(PET_BRAINO_ID);
                }
                if self.total_cold_for > Duration::from_hours(1) {
                    let _ = possible.push(PET_SNOWMAN_ID);
                }
            }
            LifeStage::Adult => {}
        };

        self.should_evolve = rng.choice(possible.iter()).cloned();
    }

    pub fn should_evolve(&self) -> Option<PetDefinitionId> {
        self.should_evolve
    }

    pub fn evolve(&mut self, next: PetDefinitionId) {
        self.life_stage_age = Duration::ZERO;
        self.def_id = next;
        self.should_evolve = None;
    }

    pub fn stomach_mood(&self) -> StomachMood {
        self.stomach_mood
    }

    pub fn mood(&self) -> Mood {
        self.mood
    }

    pub fn tick_mood(
        &mut self,
        poops: &[Option<Poop>],
        temperature: TemperatureLevel,
        layout: &HomeLayout,
    ) {
        self.mood = self.calc_mood(poops, temperature, layout);
    }

    fn calc_mood(
        &self,
        poops: &[Option<Poop>],
        temperature: TemperatureLevel,
        layout: &HomeLayout,
    ) -> Mood {
        let is_starved = matches!(self.stomach_mood, StomachMood::Starving { elapsed: _ });

        if is_starved
            || poop_count(poops) > 0
            || self.is_ill()
            || (temperature.is_hot() && !layout.furniture_present(HomeFurnitureKind::AirCon))
            || (temperature.is_cold() && !layout.furniture_present(HomeFurnitureKind::SpaceHeater))
        {
            return Mood::Sad;
        }

        let tummy_full = matches!(self.stomach_mood, StomachMood::Full { elapsed: _ });

        if tummy_full {
            return Mood::Happy;
        }

        Mood::Normal
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

    pub fn tick_tempeture(
        &mut self,
        delta: Duration,
        temperature: TemperatureLevel,
        home_layout: HomeLayout,
    ) {
        if temperature.is_cold() && !home_layout.furniture_present(HomeFurnitureKind::SpaceHeater) {
            let to_add = delta
                * if matches!(temperature, TemperatureLevel::VeryCold) {
                    2
                } else {
                    1
                };
            self.cold_for += to_add;
            self.total_cold_for += to_add;
        } else {
            self.cold_for = Duration::ZERO;
        }

        if temperature.is_hot() && !home_layout.furniture_present(HomeFurnitureKind::AirCon) {
            self.total_hot_for += delta;
        }
    }

    pub fn is_ill(&self) -> bool {
        self.illness.with_ilness > Duration::ZERO
    }

    pub fn is_starving(&self) -> bool {
        matches!(self.stomach_mood(), StomachMood::Starving { elapsed: _ })
    }

    pub fn heal_cost(&self) -> Money {
        self.illness.cost
    }

    pub fn cure(&mut self) {
        self.illness.with_ilness = Duration::ZERO;
    }

    pub fn tick_illness(&mut self, rng: &mut fastrand::Rng, delta: Duration) {
        if self.illness.with_ilness > Duration::ZERO {
            self.illness.since_ilness = Duration::ZERO;
            self.illness.with_ilness += delta;
        } else {
            self.illness.since_ilness += delta;
            let mut odds = ILLNESS_BASE_ODDS;
            let is_starved = matches!(self.stomach_mood, StomachMood::Starving { elapsed: _ });
            if is_starved {
                odds += ILLNESS_STARVING_ODDS;
            }
            if self.since_game > ILLNESS_SINCE_GAME_DURATION {
                odds += ILLNESS_SINCE_GAME_ODDS;
            }

            odds += match self.definition().life_stage {
                LifeStage::Baby => ILLNESS_BABY_ODDS,
                LifeStage::Child => ILLNESS_CHILD_ODDS,
                LifeStage::Adult => 0.,
            };

            odds += get_threshold_odds(ILLNESS_SINCE_ODDS, self.illness.since_ilness);

            if self.def_id == PET_SICKO_ID {
                odds *= 0.5;
            }

            if rng.f32() < odds {
                self.illness.cost = (rng.i32(HEALING_COST_RANGE) as f32
                    * match self.definition().life_stage {
                        LifeStage::Baby => 0.7,
                        LifeStage::Child => 1.,
                        LifeStage::Adult => 1.5,
                    }) as Money;
                self.illness.with_ilness = delta;
            }
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
            since_evolve_check: Duration::ZERO,
            should_evolve: None,
            should_die: None,
            parents: None,
            mood: Mood::Normal,
            should_breed: false,
            book_history: Default::default(),
            illness: Default::default(),
            cold_for: Duration::ZERO,
            total_cold_for: Duration::ZERO,
            total_hot_for: Duration::ZERO,
            seen_alien: false,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Default)]
pub enum Mood {
    #[default]
    Normal,
    Sad,
    Happy,
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
}
