pub mod breed_scene;
pub mod death_scene;
pub mod eat_scene;
pub mod egg_hatch_scene;
pub mod enter_date_scene;
pub mod enter_text_scene;
pub mod evolve_scene;
pub mod fishing_scene;
pub mod food_select;
pub mod game_select;
pub mod heal_scene;
pub mod home_scene;
pub mod inventory_scene;
pub mod mg_doge_em;
pub mod mg_fanfare;
pub mod mg_link_four;
pub mod mg_tic_tac_toe;
pub mod mg_weight_lift;
pub mod new_pet_scene;
pub mod pet_info_scene;
pub mod pet_records_scene;
pub mod place_furniture_scene;
pub mod poop_clear_scene;
pub mod shop_scene;
pub mod star_gazing_scene;
pub mod suiters_scene;

use core::time::Duration;

use chrono::{NaiveDate, NaiveTime};

use crate::{
    Timestamp, display::GameDisplay, game_context::GameContext, input::Input,
    scene::enter_text_scene::EnterTextStr,
};

pub enum SceneEnum {
    NewPet(new_pet_scene::NewPetScene),
    Home(home_scene::HomeScene),
    Eat(eat_scene::EatScene),
    GameSelect(game_select::GameSelectScene),
    FoodSelect(food_select::FoodSelectScene),
    Evovle(evolve_scene::EvolveScene),
    PetInfo(pet_info_scene::PetInfoScene),
    PoopClear(poop_clear_scene::PoopClearScene),
    Shop(shop_scene::ShopScene),
    Death(death_scene::DeathScene),
    Fishing(fishing_scene::FishingScene),
    Inventory(inventory_scene::InventoryScene),
    EnterText(enter_text_scene::EnterTextScene),
    EnterDate(enter_date_scene::EnterDateScene),
    PlaceFurniture(place_furniture_scene::PlaceFurnitureScene),
    Breed(breed_scene::BreedScene),
    Suiters(suiters_scene::SuitersScene),
    EggHatch(egg_hatch_scene::EggHatchScene),
    PetRecords(pet_records_scene::PetRecordsScene),
    Heal(heal_scene::HealScene),
    StarGazing(star_gazing_scene::StarGazingScene),
    MgFanFare(mg_fanfare::MgFanFareScene),
    MgDogeEm(mg_doge_em::MgDogeEmScene),
    MgTicTacToe(mg_tic_tac_toe::MgTicTacToeScene),
    MgTicLinkFour(mg_link_four::MgLinkFourScene),
    MgWeightLift(mg_weight_lift::MgWeightLift),
}

impl Default for SceneEnum {
    fn default() -> Self {
        Self::Home(home_scene::HomeScene::new())
    }
}

impl SceneEnum {
    pub fn get_scene(&mut self) -> &mut dyn Scene {
        match self {
            Self::NewPet(new_pet_scene) => new_pet_scene,
            Self::Home(home_scene) => home_scene,
            Self::Eat(eat_scene) => eat_scene,
            Self::GameSelect(game_select_scene) => game_select_scene,
            Self::FoodSelect(food_select_scene) => food_select_scene,
            Self::PetInfo(pet_info) => pet_info,
            Self::PoopClear(poop_clear_scene) => poop_clear_scene,
            Self::Shop(shop_scene) => shop_scene,
            Self::Death(death) => death,
            Self::Fishing(fishing) => fishing,
            Self::Inventory(inventory) => inventory,
            Self::EnterText(enter_text) => enter_text,
            Self::EnterDate(enter_date) => enter_date,
            Self::Evovle(evovle_scene) => evovle_scene,
            Self::PlaceFurniture(place_furniture_scene) => place_furniture_scene,
            Self::Breed(breed_scene) => breed_scene,
            Self::Suiters(suiters_scene) => suiters_scene,
            Self::PetRecords(pet_records_scene) => pet_records_scene,
            Self::Heal(heal_scene) => heal_scene,
            Self::EggHatch(egg_hatch) => egg_hatch,
            Self::StarGazing(star_gazing) => star_gazing,
            Self::MgDogeEm(mg_doge_em_scene) => mg_doge_em_scene,
            Self::MgFanFare(mg_fan_fare_scene) => mg_fan_fare_scene,
            Self::MgTicTacToe(mg_tic_tac_toe_scene) => mg_tic_tac_toe_scene,
            Self::MgTicLinkFour(mg_link_four_scene) => mg_link_four_scene,
            Self::MgWeightLift(mg_weight_lift_scene) => mg_weight_lift_scene,
        }
    }
}

#[derive(Default)]
pub struct SceneOutput {
    pub next_scene: Option<SceneEnum>,
}

impl SceneOutput {
    pub fn new(scene: SceneEnum) -> Self {
        Self {
            next_scene: Some(scene),
        }
    }
}

pub struct SceneTickArgs<'a> {
    pub timestamp: Timestamp,
    pub delta: Duration,
    pub input: &'a Input,
    pub game_ctx: &'a mut GameContext,
    pub last_scene: Option<SceneEnum>,
}

pub struct RenderArgs<'a> {
    pub timestamp: Timestamp,
    pub game_ctx: &'a mut GameContext,
}

pub trait Scene {
    fn setup(&mut self, args: &mut SceneTickArgs);

    fn teardown(&mut self, args: &mut SceneTickArgs);

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput;

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs);
}

pub struct SceneManger {
    first_loop: bool,
    next_scene: Option<SceneEnum>,
    active_scene: SceneEnum,
    last_scene: Option<SceneEnum>,
}

impl Default for SceneManger {
    fn default() -> Self {
        Self {
            first_loop: true,
            next_scene: None,
            active_scene: SceneEnum::default(),
            last_scene: None,
        }
    }
}

impl SceneManger {
    pub fn tick(&mut self, game_ctx: &mut SceneTickArgs) {
        if self.first_loop {
            self.active_scene.get_scene().setup(game_ctx);
            self.first_loop = false;
        }

        if let Some(next_scene) = self.next_scene.take() {
            let mut old_scene = core::mem::replace(&mut self.active_scene, next_scene);
            old_scene.get_scene().teardown(game_ctx);
            self.last_scene = Some(old_scene);
            self.active_scene.get_scene().setup(game_ctx);
        }
    }

    pub fn set_next(&mut self, next: SceneEnum) {
        self.next_scene = Some(next);
    }

    pub fn take_last_scene(&mut self) -> Option<SceneEnum> {
        self.last_scene.take()
    }

    pub fn restore_last_scene(&mut self, last_scene: Option<SceneEnum>) {
        self.last_scene = last_scene;
    }

    pub fn scene(&mut self) -> &mut dyn Scene {
        self.active_scene.get_scene()
    }

    pub fn scene_enum(&self) -> &SceneEnum {
        &self.active_scene
    }
}

#[derive(Default)]
pub struct SharedSceneOutput {
    enter_text_out: EnterTextStr,
    date_out: NaiveDate,
    time_out: NaiveTime,
}
