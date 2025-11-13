use core::time::Duration;

use crate::{
    alarm::AlarmState,
    date_utils::SpecialDayUpdater,
    egg::SavedEgg,
    fish_tank::HomeFishTank,
    furniture::HomeLayout,
    items::Inventory,
    money::Money,
    pet::{record::PetHistory, PetInstance},
    poop::{Poop, MAX_POOPS},
    scene::{home_scene::HomeSceneData, SharedSceneOutput},
    shop::Shop,
    sounds::SoundSystem,
    suiter::SuiterSystem,
    Timestamp,
};

pub struct GameContext {
    pub pet: PetInstance,
    pub poops: [Option<Poop>; MAX_POOPS],
    pub money: Money,
    pub inventory: Inventory,
    pub home_fish_tank: HomeFishTank,
    pub home_layout: HomeLayout,
    pub shop: Shop,
    pub pet_records: PetHistory,
    pub rng: fastrand::Rng,
    pub speical_days: SpecialDayUpdater,
    pub should_save: bool,
    pub shared_out: SharedSceneOutput,
    pub set_timestamp: Option<Timestamp>,
    pub home: HomeSceneData,
    pub egg: Option<SavedEgg>,
    pub suiter_system: SuiterSystem,
    pub sound_system: SoundSystem,
    pub sim_extra: Duration,
    pub sim_rng: fastrand::Rng,
    pub alarm: AlarmState,
}

impl GameContext {
    pub fn new(timestamp: Timestamp) -> Self {
        Self {
            pet: PetInstance::default(),
            poops: Default::default(),
            money: Money::default(),
            inventory: Inventory::default(),
            home_fish_tank: Default::default(),
            home_layout: Default::default(),
            shop: Shop::default(),
            pet_records: Default::default(),
            rng: fastrand::Rng::with_seed(timestamp.seed()),
            speical_days: SpecialDayUpdater::new(timestamp.inner().date()),
            should_save: false,
            shared_out: Default::default(),
            set_timestamp: None,
            home: Default::default(),
            egg: None,
            suiter_system: SuiterSystem::default(),
            sound_system: Default::default(),
            sim_extra: Duration::ZERO,
            sim_rng: fastrand::Rng::with_seed(0),
            alarm: AlarmState::default(),
        }
    }

    pub fn poop_count(&self) -> usize {
        self.poops.iter().filter(|i| i.is_some()).count()
    }
}
