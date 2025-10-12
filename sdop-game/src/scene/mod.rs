pub mod alarm_set_scene;
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
pub mod settings_scene;
pub mod shop_scene;
pub mod star_gazing_scene;
pub mod suiters_scene;
pub mod weekday_select_scene;

use core::time::Duration;

use chrono::{NaiveDate, NaiveTime, WeekdaySet};

use crate::{
    Timestamp, display::GameDisplay, game_context::GameContext, input::Input,
    scene::enter_text_scene::EnterTextStr,
};

#[macro_export]
macro_rules! define_scence_enum {
    (
        $enum_name:ident {
            $($variant:ident($scene_ty:ty)),+ $(,)?
        }
    ) => {
        pub enum $enum_name {
            $(
                $variant($scene_ty),
            )+
        }

        impl $enum_name {
            #[inline(always)]
            pub fn setup(&mut self, args: &mut SceneTickArgs) {
                match self {
                    $(
                        Self::$variant(inner) => inner.setup(args),
                    )+
                }
            }

            #[inline(always)]
            pub fn teardown(&mut self, args: &mut SceneTickArgs) {
                match self {
                    $(
                        Self::$variant(inner) => inner.teardown(args),
                    )+
                }
            }

            #[inline(always)]
            pub fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
                match self {
                    $(
                        Self::$variant(inner) => inner.tick(args),
                    )+
                }
            }

            #[inline(always)]
            pub fn render(&mut self, display: &mut GameDisplay, args: &mut RenderArgs) {
                match self {
                    $(
                        Self::$variant(inner) => inner.render(display, args),
                    )+
                }
            }
        }
    };
}

define_scence_enum!(SceneEnum {
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
    WeekDaySelect(weekday_select_scene::WeekdaySelectScene),
    PlaceFurniture(place_furniture_scene::PlaceFurnitureScene),
    Breed(breed_scene::BreedScene),
    Suiters(suiters_scene::SuitersScene),
    EggHatch(egg_hatch_scene::EggHatchScene),
    PetRecords(pet_records_scene::PetRecordsScene),
    Heal(heal_scene::HealScene),
    StarGazing(star_gazing_scene::StarGazingScene),
    AlarmSet(alarm_set_scene::AlarmSetScene),
    Settings(settings_scene::SettingsScene),
    MgFanFare(mg_fanfare::MgFanFareScene),
    MgDogeEm(mg_doge_em::MgDogeEmScene),
    MgTicTacToe(mg_tic_tac_toe::MgTicTacToeScene),
    MgTicLinkFour(mg_link_four::MgLinkFourScene),
    MgWeightLift(mg_weight_lift::MgWeightLift),
});

impl Default for SceneEnum {
    fn default() -> Self {
        Self::Home(home_scene::HomeScene::new())
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
            self.active_scene.setup(game_ctx);
            self.first_loop = false;
        }

        // This is crashing on GBA maybe stack overflow?
        if let Some(next_scene) = self.next_scene.take() {
            let mut old_scene = core::mem::replace(&mut self.active_scene, next_scene);
            old_scene.teardown(game_ctx);
            self.last_scene = Some(old_scene);
            self.active_scene.setup(game_ctx);
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

    pub fn scene_enum(&self) -> &SceneEnum {
        &self.active_scene
    }

    pub fn scene_enum_mut(&mut self) -> &mut SceneEnum {
        &mut self.active_scene
    }
}

#[derive(Default)]
pub struct SharedSceneOutput {
    enter_text_out: EnterTextStr,
    date_out: NaiveDate,
    time_out: NaiveTime,
    weekday_out: WeekdaySet,
}
