use core::time::Duration;

use glam::Vec2;

use crate::{
    assets::{DynamicImage, IMAGE_STOMACH_MASK},
    display::{CENTER_VEC, CENTER_X, GameDisplay},
    food::Food,
    pet::{
        definition::{PetAnimationSet, PetDefinition, PetDefinitionId},
        render::PetRender,
    },
    scene::{RenderArgs, Scene, SceneOutput, SceneTickArgs},
    sounds::{SONG_EATING, SONG_FAN_FARE, SongPlayOptions},
    stomach::StomachRender,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum EatSceneState {
    Intro,
    Eating,
    Finished,
}

pub struct EatScene {
    food: &'static Food,
    pet_render: PetRender,
    pet_def_id: PetDefinitionId,
    food_texture: DynamicImage<500>,
    last_end: usize,
    state: EatSceneState,
    fill_factor: f32,
    state_elapsed: Duration,
}

impl EatScene {
    pub fn new(food: &'static Food, pet_def_id: PetDefinitionId) -> Self {
        Self {
            food,
            pet_def_id,
            pet_render: PetRender::default(),
            food_texture: DynamicImage::default(),
            state_elapsed: Duration::ZERO,
            last_end: 0,
            state: EatSceneState::Intro,
            fill_factor: 0.,
        }
    }
}

impl Scene for EatScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.pet_render.pos = CENTER_VEC;
        self.pet_render.pos.x += 15.0;
        self.pet_render.set_def_id(self.pet_def_id);
        self.pet_render.set_animation(PetAnimationSet::Eat);

        // Copy food texture into dynamic texture
        self.food_texture.duplcaite(self.food.image);
        self.fill_factor = args.game_ctx.pet.stomach_filled;
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.sound_system.clear_song();
        args.game_ctx.pet.eat(self.food, args.timestamp);
    }

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        let pet = &args.game_ctx.pet;

        self.pet_render.tick(args.delta);

        self.state_elapsed += args.delta;

        match self.state {
            EatSceneState::Intro => {
                if self.state_elapsed > Duration::from_secs_f32(0.5) {
                    args.game_ctx
                        .sound_system
                        .push_song(SONG_EATING, SongPlayOptions::new().with_effect());
                    self.state = EatSceneState::Eating;
                    self.state_elapsed = Duration::ZERO;
                }
            }
            EatSceneState::Eating => {
                let eat_duration =
                    Duration::from_secs_f32(self.food.fill_factor / 7.).max(Duration::from_secs(3));
                let complete_percent =
                    self.state_elapsed.as_millis_f32() / eat_duration.as_millis_f32();
                let new_end = (self.food_texture.used_length as f32 * complete_percent) as usize;

                if (self.last_end == 0
                    || new_end == self.food_texture.used_length
                    || (new_end as f32 - self.last_end as f32)
                        > self.food_texture.used_length as f32 * 0.05)
                    && self.pet_render.anime.current_frame_index() == 0
                {
                    for i in self.last_end..new_end {
                        self.food_texture.texture[i] = 0;
                    }
                    self.last_end = new_end;
                    self.fill_factor = (pet.stomach_filled
                        + ((self.food.fill_factor * pet.definition().food_multiplier(self.food))
                            * complete_percent))
                        .min(pet.definition().stomach_size)
                }

                if self.state_elapsed > eat_duration {
                    args.game_ctx
                        .sound_system
                        .push_song(SONG_FAN_FARE, SongPlayOptions::new().with_effect());
                    self.state = EatSceneState::Finished;
                    self.state_elapsed = Duration::ZERO;
                }
            }
            EatSceneState::Finished => {
                self.pet_render.set_animation(PetAnimationSet::Happy);
                if self.state_elapsed > Duration::from_secs_f32(2.5) {
                    output.set_home();
                    return;
                }
            }
        }
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        const EAT_Y: i32 = 50;
        let def = PetDefinition::get_by_id(self.pet_def_id);
        display.render_sprite(&self.pet_render);
        if self.state != EatSceneState::Finished {
            display.render_image_center(CENTER_X as i32 - 10, EAT_Y, &self.food_texture);
        }

        let total_filled = (self.fill_factor / def.stomach_size).min(1.);
        display.render_complex(&StomachRender {
            pos_center: Vec2::new(CENTER_X, IMAGE_STOMACH_MASK.size.y as f32 + 10.),
            filled: total_filled,
        });
    }
}
