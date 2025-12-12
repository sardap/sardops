use bincode::{
    Decode, Encode,
    error::{DecodeError, EncodeError},
};

use crate::{
    Game, Timestamp,
    alarm::{AlarmConfig, AlarmState},
    egg::SavedEgg,
    explore::ExploreSystemSave,
    fish_tank::HomeFishTank,
    furniture::HomeLayout,
    game_context::GameContext,
    items::Inventory,
    money::Money,
    pet::{PetInstance, record::PetHistory},
    poop::{MAX_POOPS, Poop},
    shop::Shop,
    sounds::SoundOptions,
    suiter::SuiterSystem,
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Default)]
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
    pub sim_rng_seed: u64,
    pub alarm: AlarmConfig,
    pub sound: SoundOptions,
    pub explore_save: ExploreSystemSave,
}

const BINCODE_CONFIG: bincode::config::Configuration = bincode::config::standard();

pub const SAVE_SIZE: usize = size_of::<SaveFile>();

impl SaveFile {
    pub fn generate(timestamp: Timestamp, game_ctx: &GameContext) -> Self {
        Self {
            pet: game_ctx.pet,
            poops: game_ctx.poops,
            money: game_ctx.money,
            inventory: game_ctx.inventory,
            shop: game_ctx.shop,
            pet_records: game_ctx.pet_history,
            fish_tank: game_ctx.home_fish_tank,
            home_layout: game_ctx.home_layout,
            egg: game_ctx.egg,
            suiter_system: game_ctx.suiter_system,
            last_timestamp: timestamp,
            sim_rng_seed: game_ctx.sim_rng.get_seed(),
            alarm: *game_ctx.alarm.config(),
            sound: *game_ctx.sound_system.sound_options(),
            explore_save: game_ctx.explore_system.save(),
        }
    }

    pub fn load(self, game_ctx: &mut GameContext) {
        game_ctx.pet = self.pet;
        game_ctx.money = self.money;
        game_ctx.poops = self.poops;
        game_ctx.inventory = self.inventory;
        game_ctx.shop = self.shop;
        game_ctx.pet_history = self.pet_records;
        game_ctx.home_fish_tank = self.fish_tank;
        game_ctx.home_layout = self.home_layout;
        game_ctx.suiter_system = self.suiter_system;
        game_ctx.egg = self.egg;
        game_ctx.sim_rng = fastrand::Rng::with_seed(self.sim_rng_seed);
        game_ctx.alarm = AlarmState::new(self.alarm);
        game_ctx.sound_system.set_sound_options(self.sound);
        game_ctx.explore_system = self.explore_save.into();
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
