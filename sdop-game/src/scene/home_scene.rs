use fixedstr::str32;
use glam::Vec2;

use crate::{
    Button, WIDTH, WrappingEnum,
    anime::{Anime, tick_all_anime},
    assets::{self, IMAGE_STOMACH_MASK},
    date_utils::DurationExt,
    display::{CENTER_VEC, CENTER_X, CENTER_Y, GameDisplay, WIDTH_F32},
    geo::{Rect, vec2_direction, vec2_distance},
    pet::{definition::PetDefinition, render::PetRender},
    poop::{MAX_POOPS, PoopRender, update_poop_renders},
    scene::{
        Scene, SceneEnum, SceneOutput, SceneTickArgs, evolve_scene::EvolveScene,
        food_select::FoodSelectScene, game_select::GameSelectScene, pet_info::PetInfoScene,
        poop_clear_scene::PoopClearScene,
    },
    wrapping_enum,
};

const WONDER_SPEED: f32 = 5.;
pub const WONDER_RECT: Rect = Rect::new_center(CENTER_VEC, Vec2::new(WIDTH as f32, 90.0));

wrapping_enum! {
    enum MenuOption {
        Poop,
        PetInfo,
        GameSelect,
        FoodSelect,
    }
}

pub struct HomeScene {
    pet_render: PetRender,
    poops: [Option<PoopRender>; MAX_POOPS],
    target: Vec2,
    food_anime: Anime,
    selected_option: MenuOption,
}

impl HomeScene {
    pub fn new() -> Self {
        Self {
            pet_render: PetRender::default(),
            poops: [None; MAX_POOPS],
            target: Vec2::default(),
            food_anime: Anime::new(&assets::FRAMES_FOOD_SYMBOL),
            selected_option: MenuOption::Poop,
        }
    }
}

impl Scene for HomeScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {
        self.pet_render.pos = Vec2::new(CENTER_X, CENTER_Y);
        self.target = self.pet_render.pos;
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        let pet = &mut args.game_ctx.pet;
        let rng = &mut args.game_ctx.rng;
        self.pet_render.set_def_id(pet.def_id);

        update_poop_renders(&mut self.poops, &args.game_ctx.poops);

        self.food_anime.tick(args.delta);
        self.pet_render.tick(args.delta);
        tick_all_anime(&mut self.poops, args.delta);

        let dist = vec2_distance(self.pet_render.pos, self.target);
        if dist.abs() < 5. {
            let rect = Rect::new_center(
                WONDER_RECT.pos,
                WONDER_RECT.size - PetDefinition::get_by_id(pet.def_id).images.width as f32,
            );
            self.target = rect.random_point_inside(rng);
        }

        self.pet_render.pos += vec2_direction(self.pet_render.pos, self.target)
            * WONDER_SPEED
            * args.delta.as_secs_f32();

        if let Some(next_pet_id) = pet.should_evolve(rng) {
            return SceneOutput::new(SceneEnum::Evovle(EvolveScene::new(pet.def_id, next_pet_id)));
        }

        if args.input.pressed(Button::Left) {
            self.selected_option = self.selected_option.prev();
        }
        if args.input.pressed(Button::Right) {
            self.selected_option = self.selected_option.next();
        }

        if args.input.pressed(Button::Middle) {
            match self.selected_option {
                MenuOption::Poop => {
                    return SceneOutput::new(SceneEnum::PoopClear(PoopClearScene::new()));
                }
                MenuOption::PetInfo => {
                    return SceneOutput::new(SceneEnum::PetInfo(PetInfoScene::new()));
                }
                MenuOption::GameSelect => {
                    return SceneOutput::new(SceneEnum::GameSelect(GameSelectScene::new()));
                }
                MenuOption::FoodSelect => {
                    return SceneOutput::new(SceneEnum::FoodSelect(FoodSelectScene::new()));
                }
            };
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, args: &mut SceneTickArgs) {
        let pet = &args.game_ctx.pet;
        display.render_sprite(&self.pet_render);

        display.render_sprites(&self.poops);

        let total_filled = pet.stomach_filled / pet.definition().stomach_size;
        display.render_stomach(
            Vec2::new(9., IMAGE_STOMACH_MASK.size.y as f32),
            total_filled,
        );

        const BORDER_HEIGHT: f32 = 1.;

        const STOMACH_END_X: i32 = IMAGE_STOMACH_MASK.size.y as i32 + 1;
        display.render_image_top_left(STOMACH_END_X, 0, &assets::IMAGE_AGE_SYMBOL);
        let age_str = fixedstr::str_format!(str32, "{:.0}", pet.age.as_mins());
        display.render_text(
            Vec2::new(
                STOMACH_END_X as f32 + assets::IMAGE_AGE_SYMBOL.size.x as f32,
                -1.,
            ),
            &age_str,
        );

        let money_str = fixedstr::str_format!(str32, "${}", args.game_ctx.money);
        display.render_text(Vec2::new(STOMACH_END_X as f32, 10.), &money_str);

        const TOP_BORDER_RECT: Rect = Rect::new_center(
            Vec2::new(CENTER_X, 24.),
            Vec2::new(WIDTH_F32, BORDER_HEIGHT),
        );
        display.render_rect_solid(TOP_BORDER_RECT, true);

        const BOTTOM_BORDER_RECT: Rect = Rect::new_center(
            Vec2::new(CENTER_X, WONDER_RECT.pos_top_left().y + WONDER_RECT.size.y),
            Vec2::new(WIDTH_F32, BORDER_HEIGHT),
        );
        // display.render_rect_solid(BOTTOM_BORDER_RECT, true);

        const SYMBOL_BUFFER: f32 = 2.;
        const IMAGE_Y_START: f32 = BOTTOM_BORDER_RECT.pos.y + BORDER_HEIGHT + SYMBOL_BUFFER;

        const POOP_X_OFFSET: f32 = SYMBOL_BUFFER;
        display.render_image_top_left(
            POOP_X_OFFSET as i32,
            IMAGE_Y_START as i32,
            &assets::IMAGE_POOP_SYMBOL,
        );

        const PET_INFO_X_OFFSET: f32 =
            POOP_X_OFFSET + assets::IMAGE_GAME_SYMBOL.size.x as f32 + SYMBOL_BUFFER;
        display.render_image_top_left(
            PET_INFO_X_OFFSET as i32,
            IMAGE_Y_START as i32,
            &assets::IMAGE_INFO_SYMBOL,
        );

        const GAME_X_OFFSET: f32 =
            PET_INFO_X_OFFSET + assets::IMAGE_GAME_SYMBOL.size.x as f32 + SYMBOL_BUFFER;
        display.render_image_top_left(
            GAME_X_OFFSET as i32,
            IMAGE_Y_START as i32,
            &assets::IMAGE_GAME_SYMBOL,
        );

        const FOOD_X_OFFSET: f32 =
            GAME_X_OFFSET + assets::IMAGE_GAME_SYMBOL.size.x as f32 + SYMBOL_BUFFER;
        display.render_image_top_left(
            FOOD_X_OFFSET as i32,
            IMAGE_Y_START as i32,
            self.food_anime.current_frame(),
        );

        let select_rect = Rect::new_top_left(
            Vec2::new(
                match self.selected_option {
                    MenuOption::Poop => POOP_X_OFFSET,
                    MenuOption::PetInfo => PET_INFO_X_OFFSET,
                    MenuOption::GameSelect => GAME_X_OFFSET,
                    MenuOption::FoodSelect => FOOD_X_OFFSET,
                } - SYMBOL_BUFFER / 2.,
                IMAGE_Y_START - SYMBOL_BUFFER / 2.,
            ),
            Vec2::new(
                assets::IMAGE_GAME_SYMBOL.size.x as f32 + SYMBOL_BUFFER,
                assets::IMAGE_GAME_SYMBOL.size.y as f32 + SYMBOL_BUFFER,
            ),
        );
        display.render_rect_outline(select_rect, true);
    }
}
