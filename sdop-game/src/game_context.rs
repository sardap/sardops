use crate::{
    food::UnlockedFood,
    money::Money,
    pet::PetInstance,
    poop::{Poop, MAX_POOPS},
    Timestamp,
};

pub struct GameContext {
    pub pet: PetInstance,
    pub poops: [Option<Poop>; MAX_POOPS],
    pub money: Money,
    pub unlocked_food: UnlockedFood,
    pub rng: fastrand::Rng,
}

impl GameContext {
    pub fn new(timestamp: Timestamp) -> Self {
        Self {
            pet: PetInstance::default(),
            poops: Default::default(),
            money: Money::default(),
            unlocked_food: UnlockedFood::default(),
            rng: fastrand::Rng::with_seed(timestamp.seed()),
        }
    }
}
