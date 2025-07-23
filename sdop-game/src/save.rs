use bincode::{Decode, Encode};

use crate::{Timestamp, game_context::GameContext, money::Money, pet::PetInstance};

// TODO here save timestamp then run sim when reboot
#[derive(Encode, Decode)]
pub struct SaveFile {
    pet: PetInstance,
    money: Money,
    pub last_timestamp: Timestamp,
}

impl SaveFile {
    pub fn generate(timestamp: Timestamp, game_ctx: &GameContext) -> Self {
        Self {
            pet: game_ctx.pet,
            money: game_ctx.money,
            last_timestamp: timestamp,
        }
    }

    pub fn load(self, game_ctx: &mut GameContext) {
        game_ctx.pet = self.pet;
        game_ctx.money = self.money;
    }
}
