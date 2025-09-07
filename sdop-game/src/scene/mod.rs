pub mod breed_scene;
pub mod death_scene;
pub mod eat_scene;
pub mod egg_hatch;
pub mod enter_date_scene;
pub mod enter_text_scene;
pub mod evolve_scene;
pub mod fishing_scene;
pub mod food_select;
pub mod game_select;
pub mod home_scene;
pub mod inventory_scene;
pub mod mg_doge_em;
pub mod mg_fanfare;
pub mod mg_link_four;
pub mod mg_tic_tac_toe;
pub mod mg_weight_lift;
pub mod new_pet_scene;
pub mod pet_info_scene;
pub mod place_furniture_scene;
pub mod poop_clear_scene;
pub mod shop_scene;

use core::time::Duration;

use chrono::{NaiveDate, NaiveTime};

use crate::{
    display::GameDisplay, game_context::GameContext, input::Input,
    scene::enter_text_scene::EnterTextStr, Timestamp,
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
    EggHatch(egg_hatch::EggHatchScene),
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
    pub fn get_scene<'a>(&'a mut self) -> &'a mut dyn Scene {
        match self {
            SceneEnum::NewPet(new_pet_scene) => new_pet_scene,
            SceneEnum::Home(home_scene) => home_scene,
            SceneEnum::Eat(eat_scene) => eat_scene,
            SceneEnum::GameSelect(game_select_scene) => game_select_scene,
            SceneEnum::FoodSelect(food_select_scene) => food_select_scene,
            SceneEnum::PetInfo(pet_info) => pet_info,
            SceneEnum::PoopClear(poop_clear_scene) => poop_clear_scene,
            SceneEnum::Shop(shop_scene) => shop_scene,
            SceneEnum::Death(death) => death,
            SceneEnum::Fishing(fishing) => fishing,
            SceneEnum::Inventory(inventory) => inventory,
            SceneEnum::EnterText(enter_text) => enter_text,
            SceneEnum::EnterDate(enter_date) => enter_date,
            SceneEnum::Evovle(evovle_scene) => evovle_scene,
            SceneEnum::PlaceFurniture(place_furniture_scene) => place_furniture_scene,
            SceneEnum::Breed(breed_scene) => breed_scene,
            SceneEnum::EggHatch(egg_hatch) => egg_hatch,
            SceneEnum::MgDogeEm(mg_doge_em_scene) => mg_doge_em_scene,
            SceneEnum::MgFanFare(mg_fan_fare_scene) => mg_fan_fare_scene,
            SceneEnum::MgTicTacToe(mg_tic_tac_toe_scene) => mg_tic_tac_toe_scene,
            SceneEnum::MgTicLinkFour(mg_link_four_scene) => mg_link_four_scene,
            SceneEnum::MgWeightLift(mg_weight_lift_scene) => mg_weight_lift_scene,
        }
    }
}

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

impl Default for SceneOutput {
    fn default() -> Self {
        Self {
            next_scene: Default::default(),
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

    pub fn scene<'a>(&'a mut self) -> &'a mut dyn Scene {
        self.active_scene.get_scene()
    }

    pub fn scene_enum(&self) -> &SceneEnum {
        &self.active_scene
    }
}

pub struct SharedSceneOutput {
    enter_text_out: EnterTextStr,
    date_out: NaiveDate,
    time_out: NaiveTime,
}

impl Default for SharedSceneOutput {
    fn default() -> Self {
        Self {
            enter_text_out: Default::default(),
            date_out: Default::default(),
            time_out: Default::default(),
        }
    }
}
