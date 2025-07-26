use crate::{Timestamp, food::UnlockedFood, money::Money, pet::PetInstance};

pub struct GameContext {
    pub pet: PetInstance,
    pub money: Money,
    pub unlocked_food: UnlockedFood,
    pub rng: fastrand::Rng,
}

impl GameContext {
    pub fn new(timestamp: Timestamp) -> Self {
        Self {
            pet: PetInstance::default(),
            money: Money::default(),
            unlocked_food: UnlockedFood::default(),
            rng: fastrand::Rng::with_seed(timestamp.0.as_millis() as u64),
        }
    }
}
