use core::{error::Error, time};

use bincode::{
    Decode, Encode,
    error::{DecodeError, EncodeError},
};

use crate::{
    Game, Timestamp,
    game_context::GameContext,
    money::Money,
    pet::PetInstance,
    poop::{MAX_POOPS, Poop},
};

#[derive(Encode, Decode)]
pub struct SaveFile {
    pet: PetInstance,
    poops: [Option<Poop>; MAX_POOPS],
    money: Money,
    pub last_timestamp: Timestamp,
}

const BINCODE_CONFIG: bincode::config::Configuration = bincode::config::standard();

impl SaveFile {
    pub fn generate(timestamp: Timestamp, game_ctx: &GameContext) -> Self {
        Self {
            pet: game_ctx.pet,
            poops: game_ctx.poops,
            money: game_ctx.money,
            last_timestamp: timestamp,
        }
    }

    pub fn load(self, game_ctx: &mut GameContext) {
        game_ctx.pet = self.pet;
        game_ctx.money = self.money;
        game_ctx.poops = self.poops;
    }

    pub const fn size() -> usize {
        size_of::<Self>()
    }

    pub fn load_from_bytes(
        bytes: &[u8],
        timestamp: Timestamp,
        game: &mut Game,
    ) -> Result<(), DecodeError> {
        let (save, _): (SaveFile, usize) = bincode::decode_from_slice(bytes, BINCODE_CONFIG)?;
        game.load_save(timestamp, save);
        Ok(())
    }

    pub fn save_to_bytes(
        bytes: &mut [u8],
        timestamp: Timestamp,
        game: &Game,
    ) -> Result<(), EncodeError> {
        let save_file = Self::generate(timestamp, &game.game_ctx);
        bincode::encode_into_slice(save_file, bytes, BINCODE_CONFIG)?;
        Ok(())
    }

    pub fn gen_save_bytes(
        timestamp: Timestamp,
        game: &Game,
    ) -> Result<[u8; Self::size()], EncodeError> {
        let mut bytes = [0; Self::size()];
        Self::save_to_bytes(&mut bytes, timestamp, game)?;
        Ok(bytes)
    }
}
