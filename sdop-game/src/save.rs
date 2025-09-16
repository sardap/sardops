use bincode::{
    Decode, Encode,
    error::{DecodeError, EncodeError},
};

use crate::{
    Game, Timestamp,
    egg::SavedEgg,
    fish_tank::HomeFishTank,
    furniture::HomeLayout,
    game_context::GameContext,
    items::Inventory,
    money::Money,
    pet::{PetInstance, record::PetHistory},
    poop::{MAX_POOPS, Poop},
    shop::Shop,
    suiter::SuiterSystem,
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
    pub fish_tank: HomeFishTank,
    pub home_layout: HomeLayout,
    pub last_timestamp: Timestamp,
    pub egg: Option<SavedEgg>,
    pub suiter_system: SuiterSystem,
    #[cfg_attr(feature = "serde", serde(default))]
    pub sim_rng_seed: u64,
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
            fish_tank: Default::default(),
            home_layout: Default::default(),
            last_timestamp: Default::default(),
            egg: Default::default(),
            suiter_system: Default::default(),
            sim_rng_seed: Default::default(),
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
            fish_tank: game_ctx.home_fish_tank,
            home_layout: game_ctx.home_layout,
            egg: game_ctx.egg,
            suiter_system: game_ctx.suiter_system,
            last_timestamp: timestamp,
            sim_rng_seed: game_ctx.sim_rng.get_seed(),
        }
    }

    pub fn load(self, game_ctx: &mut GameContext) {
        game_ctx.pet = self.pet;
        game_ctx.money = self.money;
        game_ctx.poops = self.poops;
        game_ctx.inventory = self.inventory;
        game_ctx.shop = self.shop;
        game_ctx.pet_records = self.pet_records;
        game_ctx.home_fish_tank = self.fish_tank;
        game_ctx.home_layout = self.home_layout;
        game_ctx.suiter_system = self.suiter_system;
        game_ctx.egg = self.egg;
        game_ctx.sim_rng = fastrand::Rng::with_seed(self.sim_rng_seed);
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
