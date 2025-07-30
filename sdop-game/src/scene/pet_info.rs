use fixedstr::{str_format, str32};
use glam::Vec2;

use crate::{
    assets::{self, Image},
    date_utils::DurationExt,
    display::{CENTER_X, GameDisplay},
    pet::render::PetRender,
    scene::{Scene, SceneEnum, SceneOutput, SceneTickArgs, home_scene::HomeScene},
    sprite::Sprite,
};

pub struct PetInfoScene {
    pet_render: PetRender,
}

impl PetInfoScene {
    pub fn new() -> Self {
        Self {
            pet_render: PetRender::default(),
        }
    }
}

const PLAYER_Y: f32 = 20.;

impl Scene for PetInfoScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.pet_render.set_def_id(args.game_ctx.pet.def_id);
        self.pet_render.pos = Vec2::new(CENTER_X, PLAYER_Y)
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        self.pet_render.tick(args.delta);

        if args.input.any_pressed() {
            return SceneOutput::new(SceneEnum::Home(HomeScene::new()));
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, args: &mut SceneTickArgs) {
        let pet = &args.game_ctx.pet;

        display.render_sprite(&self.pet_render);

        const TEXT_X_OFFSET: f32 = 2.;
        let mut current_y = self.pet_render.pos.y + self.pet_render.image().size_vec2().y + 5.;

        {
            let str = str_format!(str32, "{}", pet.definition().name);
            display.render_text(Vec2::new(TEXT_X_OFFSET, current_y), &str);
            current_y += 10.;
        }

        {
            let str = str_format!(
                str32,
                "WT:{:.0}g",
                pet.definition().base_weight + pet.extra_weight
            );
            display.render_text(Vec2::new(TEXT_X_OFFSET, current_y), &str);
            current_y += 10.;
        }

        {
            display.render_image_top_left(
                TEXT_X_OFFSET as i32,
                current_y as i32,
                &assets::IMAGE_AGE_SYMBOL,
            );
            let hours = pet.age.as_hours() as i32;
            let days = hours / 24;
            let hours = hours % 24;
            let str = str_format!(str32, ":{}d{}h", days, hours);
            display.render_text(
                Vec2::new(
                    TEXT_X_OFFSET + assets::IMAGE_AGE_SYMBOL.size.x as f32,
                    current_y,
                ),
                &str,
            );
            current_y += 10.;
        }
    }
}
