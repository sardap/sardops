use core::time::Duration;

use glam::Vec2;

use crate::{
    Timestamp,
    assets::{DynamicImage, IMAGE_STOMACH_MASK},
    display::{CENTER_VEC, CENTER_X, GameDisplay},
    food::Food,
    pet::{
        definition::{PetAnimationSet, PetDefinition, PetDefinitionId},
        render::PetRender,
    },
    scene::{Scene, SceneEnum, SceneOutput, SceneTickArgs, home_scene::HomeScene},
};

#[derive(Clone, Copy)]
enum EatSceneState {
    Intro,
    Eating,
    Finished,
}

impl EatSceneState {
    fn duration(self) -> Duration {
        match self {
            EatSceneState::Intro => Duration::from_secs_f32(0.5),
            EatSceneState::Eating => Duration::from_secs_f32(3.5),
            EatSceneState::Finished => Duration::from_secs_f32(2.5),
        }
    }
}

pub struct EatScene {
    food: &'static Food,
    pet_render: PetRender,
    pet_def_id: PetDefinitionId,
    food_texture: DynamicImage<500>,
    start_time: Timestamp,
    last_end: usize,
    state: EatSceneState,
    fill_factor: f32,
}

impl EatScene {
    pub fn new(food: &'static Food, pet_def_id: PetDefinitionId) -> Self {
        Self {
            food,
            pet_def_id,
            pet_render: PetRender::default(),
            food_texture: DynamicImage::default(),
            start_time: Timestamp::default(),
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

        self.start_time = args.timestamp;

        // Copy food texture into dynamic texture
        self.food_texture.duplcaite(self.food.image);
        self.fill_factor = args.game_ctx.pet.stomach_filled;
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.pet.digest(&self.food);
    }

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        let pet = &args.game_ctx.pet;

        self.pet_render.tick(args.delta);

        match self.state {
            EatSceneState::Intro => {
                let end_time = self.start_time + EatSceneState::Intro.duration();
                if args.timestamp > end_time {
                    self.state = EatSceneState::Eating;
                    self.start_time = args.timestamp;
                }
            }
            EatSceneState::Eating => {
                let end_time = self.start_time + EatSceneState::Eating.duration();
                let complete_percent = 1.
                    - ((end_time - args.timestamp).as_secs_f32()
                        / EatSceneState::Eating.duration().as_secs_f32());
                let new_end = (self.food_texture.used_length as f32 * complete_percent) as usize;

                if self.last_end == 0
                    || new_end == self.food_texture.used_length
                    || (new_end as f32 - self.last_end as f32)
                        > self.food_texture.used_length as f32 * 0.05
                {
                    for i in self.last_end..new_end {
                        self.food_texture.texture[i] = 0;
                    }
                    self.last_end = new_end;
                    self.fill_factor = (pet.stomach_filled
                        + ((self.food.fill_factor * pet.definition().food_multiplier(&self.food))
                            * complete_percent))
                        .min(pet.definition().stomach_size)
                }

                if args.timestamp > end_time {
                    self.state = EatSceneState::Finished;
                    self.start_time = args.timestamp;
                }
            }
            EatSceneState::Finished => {
                self.pet_render.set_animation(PetAnimationSet::Happy);
                let end_time = self.start_time + EatSceneState::Finished.duration();
                if args.timestamp > end_time {
                    return SceneOutput::new(SceneEnum::Home(HomeScene::new()));
                }
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut SceneTickArgs) {
        const EAT_Y: i32 = 50;
        let def = PetDefinition::get_by_id(self.pet_def_id);
        display.render_sprite(&self.pet_render);
        display.render_image_center(CENTER_X as i32 - 10, EAT_Y, &self.food_texture);

        let total_filled = (self.fill_factor / def.stomach_size).min(1.);
        display.render_stomach(
            Vec2::new(CENTER_X, IMAGE_STOMACH_MASK.size.y as f32 + 10.),
            total_filled,
        );
        // const RECT_HEIGHT: f32 = 20.;
        // const RECT_Y: f32 = HEIGHT as f32 - RECT_HEIGHT;
        // let filled_rect = Rect::new_top_left(
        //     Vec2::new(
        //         CENTER_X - (IMAGE_STOMACH_MASK.size.x / 2) as f32,
        //         RECT_Y - IMAGE_STOMACH_MASK.size.y as f32,
        //     ),
        //     Vec2::new(IMAGE_STOMACH_MASK.size.x as f32, RECT_HEIGHT * total_filled),
        // );
        // display.render_rect_solid(filled_rect, true);
        // display.render_image_complex(
        //     CENTER_X as i32 - (IMAGE_STOMACH_MASK.size.x / 2) as i32,
        //     RECT_Y as i32 - IMAGE_STOMACH_MASK.size.y as i32,
        //     &IMAGE_STOMACH,
        //     ComplexRenderOption::default().with_white(),
        // );
        // display.render_image_complex(
        //     CENTER_X as i32 - (IMAGE_STOMACH_MASK.size.x / 2) as i32,
        //     RECT_Y as i32 - IMAGE_STOMACH_MASK.size.y as i32,
        //     &IMAGE_STOMACH_MASK,
        //     ComplexRenderOption::default().with_flip().with_black(),
        // );

        // let outline_rect =
        //     Rect::new_top_left(Vec2::new(0., RECT_Y), Vec2::new(WIDTH as f32, RECT_HEIGHT));
        // display.render_rect_outline(outline_rect, true);

        // let fill_rect = Rect::new_top_left(
        //     Vec2::new(0., RECT_Y),
        //     Vec2::new(WIDTH as f32 * total_filled, RECT_HEIGHT),
        // );
        // display.render_rect_solid(fill_rect, true);
    }
}
