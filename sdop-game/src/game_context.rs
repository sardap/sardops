use crate::{
    date_utils::SpecialDayUpdater,
    items::Inventory,
    money::Money,
    pet::PetInstance,
    poop::{Poop, MAX_POOPS},
    scene::SharedSceneOutput,
    shop::Shop,
    Timestamp,
};

pub struct GameContext {
    pub pet: PetInstance,
    pub poops: [Option<Poop>; MAX_POOPS],
    pub money: Money,
    pub inventory: Inventory,
    pub shop: Shop,
    pub rng: fastrand::Rng,
    pub speical_days: SpecialDayUpdater,
    pub should_save: bool,
    pub shared_out: SharedSceneOutput,
    pub set_timestamp: Option<Timestamp>,
}

impl GameContext {
    pub fn new(timestamp: Timestamp) -> Self {
        Self {
            pet: PetInstance::default(),
            poops: Default::default(),
            money: Money::default(),
            inventory: Inventory::default(),
            shop: Shop::default(),
            rng: fastrand::Rng::with_seed(timestamp.seed()),
            speical_days: SpecialDayUpdater::new(timestamp.inner().date()),
            should_save: false,
            shared_out: Default::default(),
            set_timestamp: None,
        }
    }
}
