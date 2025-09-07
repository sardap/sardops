use chrono::NaiveDateTime;
use fixedstr::str_format;

use crate::{
    display::GameDisplay,
    pet::{definition::PetDefinitionId, gen_pid, PetInstance, PetName, PetParents, UniquePetId},
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
    def_id: PetDefinitionId,
    need_timestamp: bool,
    state: State,
    upid: UniquePetId,
    parents: Option<PetParents>,
}

impl NewPetScene {
    pub fn new(
        def_id: PetDefinitionId,
        need_timestamp: bool,
        upid: Option<UniquePetId>,
        parents: Option<PetParents>,
    ) -> Self {
        Self {
            def_id,
            state: if need_timestamp {
                State::EnterDate
            } else {
                State::EnterName
            },
            need_timestamp: need_timestamp,
            upid: upid.unwrap_or_default(),
            parents,
        }
    }
}

impl Scene for NewPetScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.pet = PetInstance::default();
        args.game_ctx.pet.parents = self.parents;
        args.game_ctx.pet.upid = if self.upid == 0 {
            gen_pid(&mut args.game_ctx.rng)
        } else {
            self.upid
        };
        args.game_ctx.pet.def_id = self.def_id;
        args.game_ctx.pet.born = args.timestamp;
        args.game_ctx.pet.name =
            str_format!(PetName, "{}", args.game_ctx.shared_out.enter_text_out);
    }

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
                return SceneOutput::new(SceneEnum::Home(HomeScene::new()));
            }
        }
    }

    fn render(&self, _display: &mut GameDisplay, _args: &mut RenderArgs) {}
}
