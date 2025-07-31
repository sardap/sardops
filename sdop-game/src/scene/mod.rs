pub mod eat_scene;
pub mod evolve_scene;
pub mod food_select;
pub mod game_select;
pub mod home_scene;
pub mod mg_doge_em;
pub mod mg_fanfare;
pub mod mg_link_four;
pub mod mg_tic_tac_toe;
pub mod pet_info;
pub mod poop_clear_scene;

use core::time::Duration;

use crate::{Timestamp, display::GameDisplay, game_context::GameContext, input::Input};

pub enum SceneEnum {
    Home(home_scene::HomeScene),
    Eat(eat_scene::EatScene),
    GameSelect(game_select::GameSelectScene),
    FoodSelect(food_select::FoodSelectScene),
    Evovle(evolve_scene::EvolveScene),
    PetInfo(pet_info::PetInfoScene),
    PoopClear(poop_clear_scene::PoopClearScene),
    MgFanFare(mg_fanfare::MgFanFareScene),
    MgDogeEm(mg_doge_em::MgDogeEmScene),
    MgTicTacToe(mg_tic_tac_toe::MgTicTacToeScene),
    MgTicLinkFour(mg_link_four::MgLinkFourScene),
}

impl Default for SceneEnum {
    fn default() -> Self {
        Self::Home(home_scene::HomeScene::new())
    }
}

impl SceneEnum {
    pub fn get_scene<'a>(&'a mut self) -> &'a mut dyn Scene {
        match self {
            SceneEnum::Home(home_scene) => home_scene,
            SceneEnum::Eat(eat_scene) => eat_scene,
            SceneEnum::GameSelect(game_select_scene) => game_select_scene,
            SceneEnum::FoodSelect(food_select_scene) => food_select_scene,
            SceneEnum::PetInfo(pet_info) => pet_info,
            SceneEnum::PoopClear(poop_clear_scene) => poop_clear_scene,
            SceneEnum::MgDogeEm(mg_doge_em_scene) => mg_doge_em_scene,
            SceneEnum::MgFanFare(mg_fan_fare_scene) => mg_fan_fare_scene,
            SceneEnum::MgTicTacToe(mg_tic_tac_toe_scene) => mg_tic_tac_toe_scene,
            SceneEnum::MgTicLinkFour(mg_link_four_scene) => mg_link_four_scene,
            SceneEnum::Evovle(evovle_scene) => evovle_scene,
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
}

pub trait Scene {
    fn setup(&mut self, args: &mut SceneTickArgs);

    fn teardown(&mut self, args: &mut SceneTickArgs);

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput;

    fn render(&self, display: &mut GameDisplay, args: &mut SceneTickArgs);
}

pub struct SceneManger {
    first_loop: bool,
    next_scene: Option<SceneEnum>,
    active_scene: SceneEnum,
}

impl Default for SceneManger {
    fn default() -> Self {
        Self {
            first_loop: true,
            next_scene: None,
            active_scene: SceneEnum::default(),
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
            self.active_scene.get_scene().teardown(game_ctx);
            self.active_scene = next_scene;
            self.active_scene.get_scene().setup(game_ctx);
        }
    }

    pub fn set_next(&mut self, next: SceneEnum) {
        self.next_scene = Some(next);
    }

    pub fn scene<'a>(&'a mut self) -> &'a mut dyn Scene {
        self.active_scene.get_scene()
    }
}
