use core::time::Duration;

use fixedstr::str_format;
use glam::Vec2;

use crate::{
    anime::HasAnime,
    assets::{self, Image},
    death::{DeathCause, GraveStone},
    display::{ComplexRenderOption, GameDisplay, CENTER_VEC, CENTER_X, CENTER_Y},
    pet::{
        definition::{PetAnimationSet, PetDefinitionId},
        record::PetRecord,
        render::PetRender,
    },
    scene::{new_pet_scene::NewPetScene, RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs},
    sprite::{BasicAnimeSprite, Sprite},
    stomach::StomachRender,
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

struct Starvation {
    stomach_x_offset: f32,
    moving_left: bool,
}

impl Default for Starvation {
    fn default() -> Self {
        Self {
            stomach_x_offset: 0.,
            moving_left: false,
        }
    }
}

pub struct DeathScene {
    cause: DeathCause,
    state: State,
    state_elapsed: Duration,
    lighting: Lighting,
    starving: Starvation,
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
            starving: Starvation::default(),
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
                DeathCause::Starvation => {
                    self.pet_render.set_animation(PetAnimationSet::Sad);

                    const SPEED: f32 = 10.;

                    let change = (SPEED * args.delta.as_secs_f32())
                        * (1. + self.state_elapsed.as_secs_f32());

                    self.starving.moving_left = if self.starving.stomach_x_offset + change > 2. {
                        true
                    } else if self.starving.stomach_x_offset - change < -2. {
                        false
                    } else {
                        self.starving.moving_left
                    };

                    self.starving.stomach_x_offset += if self.starving.moving_left {
                        -change
                    } else {
                        change
                    };

                    if self.state_elapsed > Duration::from_millis(5000) {
                        self.state = State::Tombstone;
                        self.state_elapsed = Duration::ZERO;
                    }
                }
                DeathCause::OldAge => todo!(),
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
                DeathCause::Starvation => {
                    display.render_sprite(&self.pet_render);

                    display.render_image_complex(
                        (CENTER_X + self.starving.stomach_x_offset) as i32,
                        (CENTER_Y
                            - self.pet_render.image().size_vec2().y / 2.
                            - StomachRender::size().y / 2.) as i32,
                        &assets::IMAGE_STOMACH,
                        ComplexRenderOption::new().with_white().with_center(),
                    );

                    if self.state_elapsed > Duration::from_millis(4700) {
                        display.invert();
                    }
                }
                DeathCause::OldAge => todo!(),
            },
            State::Tombstone => {
                display.render_complex(&self.grave_stone);
            }
        }
    }
}
