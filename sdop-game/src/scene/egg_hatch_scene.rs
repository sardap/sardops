use core::time::Duration;

use glam::Vec2;

use crate::{
    assets,
    death::DeathCause,
    display::{CENTER_X, CENTER_Y, ComplexRenderOption, GameDisplay, WIDTH_F32},
    egg::{EggRender, SavedEgg},
    geo::Rect,
    pet::{
        definition::{PET_BABIES, PET_BLOB_ID, PetAnimationSet, PetDefinitionId},
        record::PetRecord,
        render::PetRender,
    },
    scene::{RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, new_pet_scene::NewPetScene},
    sounds::{SONG_EGG_HATCH, SONG_FAN_FARE, SongPlayOptions},
    sprite::{BasicMaskedSprite, Sprite},
};

const EGG_CENTER_X: f32 = CENTER_X + 12.;

enum State {
    Shaking,
    Explode,
    Dancing,
    UfoEnter,
    UfoTake,
    UfoLeave,
    NewGuy,
}

pub struct EggHatchScene {
    state: State,
    state_elapsed: Duration,
    egg_left: bool,
    egg: SavedEgg,
    egg_render: EggRender,
    egg_shells: [Vec2; 2],
    ufo_render: BasicMaskedSprite,
    baby_pet_render: PetRender,
    parent_pet_render: PetRender,
    def_id: PetDefinitionId,
}

impl EggHatchScene {
    pub fn new(egg: SavedEgg, parent_def_id: PetDefinitionId) -> Self {
        Self {
            state: State::Shaking,
            state_elapsed: Duration::ZERO,
            egg_left: false,
            egg,
            egg_render: EggRender::new(Vec2::new(EGG_CENTER_X, CENTER_Y), egg.upid),
            baby_pet_render: PetRender::new(PET_BLOB_ID),
            egg_shells: Default::default(),
            parent_pet_render: PetRender::new(parent_def_id),
            ufo_render: BasicMaskedSprite::new(
                Vec2::new(-50., 30.),
                &assets::IMAGE_UFO,
                &assets::IMAGE_UFO_MASK,
            ),
            def_id: Default::default(),
        }
    }
}

impl Scene for EggHatchScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.def_id = args.game_ctx.rng.choice(PET_BABIES).unwrap();
        self.baby_pet_render.set_def_id(self.def_id);
        self.parent_pet_render.pos = Vec2::new(20., CENTER_Y + 30.);
        self.parent_pet_render.set_animation(PetAnimationSet::Happy);
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.sound_system.clear_song();
        args.game_ctx.egg = None;
        args.game_ctx.pet_records.add(PetRecord::from_pet_instance(
            &args.game_ctx.pet,
            args.timestamp,
            DeathCause::Leaving,
        ));
    }

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        self.state_elapsed += args.delta;
        self.baby_pet_render.tick(args.delta);
        self.parent_pet_render.tick(args.delta);

        match self.state {
            State::Shaking => {
                if !args.game_ctx.sound_system.get_playing() {
                    args.game_ctx
                        .sound_system
                        .push_song(SONG_EGG_HATCH, SongPlayOptions::new().with_music());
                }

                let (range, speed) = if self.state_elapsed < Duration::from_secs(2) {
                    (1., 5.)
                } else if self.state_elapsed < Duration::from_secs(5) {
                    (2., 7.)
                } else {
                    (2., 10.)
                };

                if self.state_elapsed > Duration::from_secs(4) {
                    self.egg_render.tick(args.delta);
                }

                let move_ammount = speed * args.delta.as_secs_f32();
                log::info!("Amount {}", move_ammount);
                if self.egg_left {
                    self.egg_render.pos.x -= move_ammount;
                    if self.egg_render.pos.x < EGG_CENTER_X - range {
                        self.egg_render.pos.x = EGG_CENTER_X - range;
                        self.egg_left = false;
                    }
                } else {
                    self.egg_render.pos.x += move_ammount;
                    if self.egg_render.pos.x > EGG_CENTER_X + range {
                        self.egg_render.pos.x = EGG_CENTER_X + range;
                        self.egg_left = true;
                    }
                }

                // Gotta do some circles and some inverto lights
                if self.egg_render.cracked() {
                    self.baby_pet_render.set_animation(PetAnimationSet::Happy);
                    self.baby_pet_render.pos = self.egg_render.pos;
                    self.egg_shells = [self.egg_render.pos, self.egg_render.pos];
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::Explode;
                    args.game_ctx.sound_system.clear_song();
                }
            }
            State::Explode => {
                self.baby_pet_render.set_animation(PetAnimationSet::Sad);
                const SPEED: f32 = 50.;

                self.egg_shells[0] += Vec2::new(
                    -SPEED * args.delta.as_secs_f32(),
                    1. * SPEED * args.delta.as_secs_f32(),
                );

                self.egg_shells[1] += Vec2::new(
                    1. * SPEED * args.delta.as_secs_f32(),
                    -SPEED * args.delta.as_secs_f32(),
                );

                if self.egg_shells[0].x < -5. && self.egg_shells[1].x > WIDTH_F32 + 5. {
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::Dancing;
                    args.game_ctx
                        .sound_system
                        .push_song(SONG_FAN_FARE, SongPlayOptions::new().with_effect());
                }
            }
            State::Dancing => {
                // TODO conffeit falling or something
                self.baby_pet_render.set_animation(PetAnimationSet::Happy);

                if self.state_elapsed > Duration::from_secs(2) {
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::UfoEnter;
                }
            }
            State::UfoEnter => {
                const SPEED: f32 = 10.;
                self.ufo_render.pos.x += SPEED * args.delta.as_secs_f32();
                if self.ufo_render.pos.x >= self.parent_pet_render.pos.x {
                    self.ufo_render.pos.x = self.parent_pet_render.pos.x;
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::UfoTake;
                }
            }
            State::UfoTake => {
                self.parent_pet_render.set_animation(PetAnimationSet::Happy);
                const SPEED: f32 = 15.;
                self.parent_pet_render.pos.y -= SPEED * args.delta.as_secs_f32();
                if self.parent_pet_render.pos.y <= self.ufo_render.rect().y() {
                    self.parent_pet_render.pos.y = -100.;
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::UfoLeave;
                }
            }
            State::UfoLeave => {
                const SPEED: f32 = 20.;
                self.ufo_render.pos.x += SPEED * args.delta.as_secs_f32();
                if self.ufo_render.rect().x() > WIDTH_F32 {
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::NewGuy;
                }
            }
            State::NewGuy => {
                if self.state_elapsed > Duration::from_secs(1) {
                    output.set(SceneEnum::NewPet(NewPetScene::new(
                        self.def_id,
                        false,
                        Some(self.egg.upid),
                        self.egg.parents,
                    )));
                    return;
                }
            }
        }
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        match self.state {
            State::Shaking => {
                display.render_sprite(&self.parent_pet_render);
                display.render_complex(&self.egg_render);

                if self.state_elapsed > Duration::from_secs(9) {
                    display.invert();
                }
            }
            State::Explode => {
                display.render_sprite(&self.parent_pet_render);
                display.render_sprite(&self.baby_pet_render);
                display.render_image_complex(
                    self.egg_shells[0].x as i32,
                    self.egg_shells[0].y as i32,
                    &assets::IMAGE_EGG_BITS_1,
                    ComplexRenderOption::new().with_white(),
                );
                display.render_image_complex(
                    self.egg_shells[1].x as i32,
                    self.egg_shells[1].y as i32,
                    &assets::IMAGE_EGG_BITS_2,
                    ComplexRenderOption::new().with_white(),
                );
            }
            State::Dancing => {
                display.render_sprite(&self.parent_pet_render);
                display.render_sprite(&self.baby_pet_render)
            }
            State::UfoEnter => {
                display.render_sprite(&self.parent_pet_render);
                display.render_sprite(&self.ufo_render);
                display.render_sprite(&self.baby_pet_render);
            }
            State::UfoTake => {
                display.render_sprite(&self.parent_pet_render);
                const BLOCK_OUT_RECT: Rect = Rect::new_top_left(
                    Vec2::ZERO,
                    Vec2::new(WIDTH_F32, 30. + (assets::IMAGE_UFO.size.y / 2) as f32),
                );
                display.render_rect_solid(BLOCK_OUT_RECT, false);
                display.render_image_complex(
                    self.ufo_render.pos.x as i32,
                    self.ufo_render.pos.y as i32
                        + assets::IMAGE_UFO.size.y as i32 / 2
                        + assets::IMAGE_UFO_BEAM.size.y as i32 / 2
                        - 10,
                    &assets::IMAGE_UFO_BEAM_INSIDE,
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_invert(),
                );

                display.render_sprite(&self.ufo_render);
                display.render_sprite(&self.baby_pet_render);
            }
            State::UfoLeave => {
                display.render_sprite(&self.ufo_render);
                display.render_sprite(&self.baby_pet_render);
            }
            State::NewGuy => {
                display.render_sprite(&self.baby_pet_render);
            }
        }
    }
}
