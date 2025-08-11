use bincode::{
    error::{DecodeError, EncodeError},
    Decode, Encode,
};

use crate::{
    game_context::GameContext,
    items::Inventory,
    money::Money,
    pet::{record::PetHistory, PetInstance},
    poop::{Poop, MAX_POOPS},
    shop::Shop,
    Game, Timestamp,
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode)]
pub struct SaveFile {
    pub pet: PetInstance,
    pub poops: [Option<Poop>; MAX_POOPS],
    pub money: Money,
    pub inventory: Inventory,
    pub shop: Shop,
    pub pet_records: PetHistory,
    pub last_timestamp: Timestamp,
}

impl Default for SaveFile {
    fn default() -> Self {
        Self {
            pet: Default::default(),
            poops: Default::default(),
            money: Default::default(),
            inventory: Default::default(),
            shop: Default::default(),
            pet_records: Default::default(),
            last_timestamp: Default::default(),
        }
    }
}

const BINCODE_CONFIG: bincode::config::Configuration = bincode::config::standard();

const SAVE_SIZE: usize = size_of::<SaveFile>();

impl SaveFile {
    pub fn generate(timestamp: Timestamp, game_ctx: &GameContext) -> Self {
        Self {
            pet: game_ctx.pet,
            poops: game_ctx.poops,
            money: game_ctx.money,
            inventory: game_ctx.inventory,
            shop: game_ctx.shop,
            pet_records: game_ctx.pet_records,
            last_timestamp: timestamp,
        }
    }

    pub fn load(self, game_ctx: &mut GameContext) {
        game_ctx.pet = self.pet;
        game_ctx.money = self.money;
        game_ctx.poops = self.poops;
        game_ctx.inventory = self.inventory;
        game_ctx.shop = self.shop;
        game_ctx.pet_records = self.pet_records;
    }

    pub const fn size() -> usize {
        size_of::<Self>()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (save, _): (SaveFile, usize) = bincode::decode_from_slice(bytes, BINCODE_CONFIG)?;
        Ok(save)
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

    pub fn to_bytes(&self) -> Result<[u8; SAVE_SIZE], EncodeError> {
        let mut result = [0; SAVE_SIZE];
        bincode::encode_into_slice(self, &mut result, BINCODE_CONFIG)?;
        Ok(result)
    }

    fn save_to_bytes(
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
    ) -> Option<Result<[u8; Self::size()], EncodeError>> {
        if !game.game_ctx.should_save {
            return None;
        }

        let mut bytes = [0; Self::size()];
        match Self::save_to_bytes(&mut bytes, timestamp, game) {
            Ok(it) => it,
            Err(err) => return Some(Err(err)),
        };
        Some(Ok(bytes))
    }
}
