use core::time::Duration;

use fixedstr::str_format;
use glam::Vec2;

use crate::{
    anime::HasAnime,
    assets,
    death::{DeathCause, GraveStone},
    display::{ComplexRenderOption, GameDisplay, CENTER_VEC, CENTER_X},
    pet::{
        definition::{PetAnimationSet, PetDefinitionId},
        record::PetRecord,
        render::PetRender,
        PetInstance,
    },
    scene::{new_pet_scene::NewPetScene, RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs},
    sprite::BasicAnimeSprite,
};

enum State {
    Intro,
    Speific,
    Tombstone,
}

struct Lighting {
    clouds: BasicAnimeSprite,
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            clouds: BasicAnimeSprite::new(
                Vec2::new(CENTER_X, assets::IMAGE_CLOUDS_0.size.y as f32 / 2.),
                &assets::FRAMES_CLOUDS,
            ),
        }
    }
}

pub struct DeathScene {
    cause: DeathCause,
    state: State,
    state_elapsed: Duration,
    lighting: Lighting,
    pet_render: PetRender,
    grave_stone: GraveStone,
}

impl DeathScene {
    pub fn new(cause: DeathCause, pet_id: PetDefinitionId) -> Self {
        Self {
            cause,
            state: State::Intro,
            state_elapsed: Duration::ZERO,
            lighting: Lighting::default(),
            pet_render: PetRender::new(pet_id),
            grave_stone: GraveStone::default(),
        }
    }
}

impl Scene for DeathScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.pet_render.pos = CENTER_VEC;
        self.grave_stone = GraveStone::new(
            self.pet_render.pos,
            str_format!(fixedstr::str12, "{}", args.game_ctx.pet.name),
            args.game_ctx.pet.born.inner().date(),
            args.timestamp.inner().date(),
        );
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        self.state_elapsed += args.delta;

        self.pet_render.tick(args.delta);

        match self.state {
            State::Intro => {
                if self.state_elapsed > Duration::from_secs(3) {
                    self.state = State::Speific;
                    self.state_elapsed = Duration::ZERO;
                }
            }
            State::Speific => match self.cause {
                DeathCause::LightingStrike => {
                    self.pet_render.set_animation(PetAnimationSet::Sad);
                    self.lighting.clouds.anime().tick(args.delta);
                    if self.lighting.clouds.anime.frames() == &assets::FRAMES_CLOUDS
                        && self.lighting.clouds.anime.current_frame_index()
                            == self.lighting.clouds.anime.frames().len() - 1
                    {
                        self.lighting.clouds = BasicAnimeSprite::new(
                            self.lighting.clouds.pos,
                            &assets::FRAMES_CLOUDS_RUBBING,
                        );
                        log::info!("{:?}", self.state_elapsed.as_millis());
                    }

                    if self.state_elapsed > Duration::from_millis(5500) {
                        self.state = State::Tombstone;
                        self.state_elapsed = Duration::ZERO;
                    }
                }
            },
            State::Tombstone => {
                if args.input.any_pressed() {
                    args.game_ctx.pet_records.add(PetRecord::from_pet_instance(
                        &args.game_ctx.pet,
                        args.timestamp,
                        self.cause,
                    ));
                    return SceneOutput::new(SceneEnum::NewPet(NewPetScene::new(false)));
                }
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        match self.state {
            State::Intro => {
                display.render_sprite(&self.pet_render);
            }
            State::Speific => match self.cause {
                DeathCause::LightingStrike => {
                    display.render_sprite(&self.lighting.clouds);
                    display.render_sprite(&self.pet_render);

                    if self.state_elapsed > Duration::from_millis(5000) {
                        display.render_image_complex(
                            CENTER_X as i32 - 7,
                            self.pet_render.pos.y as i32,
                            &assets::IMAGE_LIGHTING_ONE,
                            ComplexRenderOption::new().with_white().with_bottom_left(),
                        );
                        display.invert();
                    }
                }
            },
            State::Tombstone => {
                display.render_complex(&self.grave_stone);
            }
        }
    }
}
