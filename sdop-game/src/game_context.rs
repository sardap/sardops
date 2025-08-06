use crate::{
    date_utils::SpecialDayUpdater,
    food::UnlockedFood,
    items::Inventory,
    money::Money,
    pet::PetInstance,
    poop::{Poop, MAX_POOPS},
    shop::Shop,
    Timestamp,
};

pub struct GameContext {
    pub pet: PetInstance,
    pub poops: [Option<Poop>; MAX_POOPS],
    pub money: Money,
    pub inventory: Inventory,
    pub unlocked_food: UnlockedFood,
    pub shop: Shop,
    pub rng: fastrand::Rng,
    pub speical_days: SpecialDayUpdater,
}

impl GameContext {
    pub fn new(timestamp: Timestamp) -> Self {
        Self {
            pet: PetInstance::default(),
            poops: Default::default(),
            money: Money::default(),
            inventory: Inventory::default(),
            unlocked_food: UnlockedFood::default(),
            shop: Shop::default(),
            rng: fastrand::Rng::with_seed(timestamp.seed()),
            speical_days: SpecialDayUpdater::new(timestamp.inner().date()),
        }
    }
}
