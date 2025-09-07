use core::time::Duration;

use crate::{
    display::{GameDisplay, CENTER_VEC, CENTER_X},
    egg::{EggRender, SavedEgg},
    pet::{
        definition::{PetAnimationSet, PetDefinitionId, PET_BABIES, PET_BLOB_ID},
        render::PetRender,
    },
    scene::{new_pet_scene::NewPetScene, RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs},
};

enum State {
    Shaking,
    NewGuy,
}

pub struct EggHatchScene {
    state: State,
    state_elapsed: Duration,
    egg_left: bool,
    egg: SavedEgg,
    egg_render: EggRender,
    pet_render: PetRender,
    def_id: PetDefinitionId,
}

impl EggHatchScene {
    pub fn new(egg: SavedEgg) -> Self {
        Self {
            state: State::Shaking,
            state_elapsed: Duration::ZERO,
            egg_left: false,
            egg,
            egg_render: EggRender::new(CENTER_VEC, egg.upid),
            pet_render: PetRender::new(PET_BLOB_ID),
            def_id: Default::default(),
        }
    }
}

impl Scene for EggHatchScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.def_id = args.game_ctx.rng.choice(PET_BABIES).unwrap();
        self.pet_render.pos = CENTER_VEC;
        self.pet_render.set_def_id(self.def_id);
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.egg = None;
    }

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        self.state_elapsed += args.delta;
        self.pet_render.tick(args.delta);

        match self.state {
            State::Shaking => {
                let (range, speed) = if self.state_elapsed < Duration::from_secs(2) {
                    (2., 1.)
                } else if self.state_elapsed < Duration::from_secs(5) {
                    (4., 3.)
                } else {
                    (4., 10.)
                };
                let move_ammount = speed * args.delta.as_secs_f32();
                if self.egg_left {
                    self.egg_render.pos.x -= move_ammount;
                    if self.egg_render.pos.x < CENTER_X - range {
                        self.egg_left = false;
                    }
                } else {
                    self.egg_render.pos.x += move_ammount;
                    if self.egg_render.pos.x > CENTER_X + range {
                        self.egg_left = true;
                    }
                }

                // Gotta do some circles and some inverto lights
                if self.state_elapsed > Duration::from_secs(10) {
                    self.pet_render.set_animation(PetAnimationSet::Happy);
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::NewGuy;
                }
            }
            State::NewGuy => {
                if self.state_elapsed > Duration::from_secs(5) {
                    return SceneOutput::new(SceneEnum::NewPet(NewPetScene::new(
                        self.def_id,
                        false,
                        Some(self.egg.upid),
                        self.egg.parents,
                    )));
                }
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        match self.state {
            State::Shaking => {
                display.render_complex(&self.egg_render);

                if self.state_elapsed > Duration::from_secs(9) {
                    display.invert();
                }
            }
            State::NewGuy => {
                display.render_sprite(&self.pet_render);

                if self.state_elapsed < Duration::from_secs(1) {
                    display.invert();
                }
            }
        }
    }
}
