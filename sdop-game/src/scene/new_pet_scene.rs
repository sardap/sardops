use chrono::NaiveDateTime;
use fixedstr::str_format;

use crate::{
    display::GameDisplay,
    pet::{definition::PET_BLOB_ID, gen_pid, PetInstance, PetName},
    scene::{
        enter_date_scene::{self, EnterDateScene},
        enter_text_scene::EnterTextScene,
        home_scene::HomeScene,
        RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs,
    },
    Timestamp,
};

#[derive(Clone)]
enum State {
    EnterDate,
    EnterName,
    NameEntered,
}

#[derive(Clone)]
pub struct NewPetScene {
    need_timestamp: bool,
    state: State,
}

impl NewPetScene {
    pub fn new(need_timestamp: bool) -> Self {
        Self {
            state: if need_timestamp {
                State::EnterDate
            } else {
                State::EnterName
            },
            need_timestamp: need_timestamp,
        }
    }
}

impl Scene for NewPetScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        match self.state {
            State::EnterDate => {
                self.state = State::EnterName;
                return SceneOutput::new(SceneEnum::EnterDate(EnterDateScene::new(
                    enter_date_scene::Required::DateTime,
                    str_format!(fixedstr::str12, "WHEN IS IT?"),
                )));
            }
            State::EnterName => {
                if self.need_timestamp {
                    args.game_ctx.set_timestamp = Some(Timestamp::new(NaiveDateTime::new(
                        args.game_ctx.shared_out.date_out,
                        args.game_ctx.shared_out.time_out,
                    )));
                }

                self.state = State::NameEntered;
                return SceneOutput::new(SceneEnum::EnterText(EnterTextScene::new(
                    6,
                    str_format!(fixedstr::str12, "ENTER NAME"),
                    Some(|text| text.len() > 0 && text.chars().any(|c| !c.is_whitespace())),
                )));
            }
            State::NameEntered => {
                args.game_ctx.pet = PetInstance::default();

                // Largest number that can fit on the info screen
                args.game_ctx.pet.upid = gen_pid(&mut args.game_ctx.rng);
                args.game_ctx.pet.def_id = PET_BLOB_ID;
                args.game_ctx.pet.born = args.timestamp;
                args.game_ctx.pet.name =
                    str_format!(PetName, "{}", args.game_ctx.shared_out.enter_text_out);
                return SceneOutput::new(SceneEnum::Home(HomeScene::new()));
            }
        }
    }

    fn render(&self, _display: &mut GameDisplay, _args: &mut RenderArgs) {}
}
