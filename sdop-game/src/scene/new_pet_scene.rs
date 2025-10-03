use chrono::NaiveDateTime;
use fixedstr::str_format;

use crate::{
    Timestamp,
    display::GameDisplay,
    pet::{PetInstance, PetName, PetParents, UniquePetId, definition::PetDefinitionId, gen_pid},
    scene::{
        RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs,
        enter_date_scene::{self, EnterDateScene},
        enter_text_scene::EnterTextScene,
        home_scene::HomeScene,
    },
    sounds::{SONG_NEW_PET, SongPlayOptions},
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
            need_timestamp,
            upid: upid.unwrap_or_default(),
            parents,
        }
    }
}

impl Scene for NewPetScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx
            .sound_system
            .push_song(SONG_NEW_PET, SongPlayOptions::new().with_essential());
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.sound_system.clear_song();
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
        args.game_ctx.sim_rng = fastrand::Rng::with_seed(args.game_ctx.pet.upid);
    }

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        match self.state {
            State::EnterDate => {
                self.state = State::EnterName;
                SceneOutput::new(SceneEnum::EnterDate(EnterDateScene::new(
                    enter_date_scene::Required::DateTime,
                    str_format!(fixedstr::str12, "WHEN IS IT?"),
                )))
            }
            State::EnterName => {
                if self.need_timestamp {
                    args.game_ctx.set_timestamp = Some(Timestamp::new(NaiveDateTime::new(
                        args.game_ctx.shared_out.date_out,
                        args.game_ctx.shared_out.time_out,
                    )));
                }

                self.state = State::NameEntered;
                SceneOutput::new(SceneEnum::EnterText(EnterTextScene::new(
                    6,
                    str_format!(fixedstr::str12, "ENTER NAME"),
                    Some(|text| !text.is_empty() && text.chars().any(|c| !c.is_whitespace())),
                )))
            }
            State::NameEntered => SceneOutput::new(SceneEnum::Home(HomeScene::new())),
        }
    }

    fn render(&self, _display: &mut GameDisplay, _args: &mut RenderArgs) {}
}
