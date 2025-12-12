use core::time::Duration;

use glam::Vec2;
use sdop_common::MelodyEntry;

use crate::{
    Song,
    anime::Anime,
    assets::{self},
    display::{CENTER_X, ComplexRender, ComplexRenderOption, GameDisplay, WIDTH_F32},
    game_context::GameContext,
    geo::Rect,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MenuOption {
    PetInfo,
    FoodSelect,
    Poop,
    Breed,
    Heal,
    Shop,
    Inventory,
    PlaceFurniture,
    Explore,
    PetRecords,
    GameSelect,
    Settings,
}

impl MenuOption {
    pub fn get_song(&self) -> &'static Song {
        const MELODY: &[MelodyEntry] = &[];
        const SONG: Song = Song::new(MELODY, 85);
        &SONG
    }
}

const MENU_OPTIONS_COUNT: usize = core::mem::variant_count::<MenuOption>();

type MenuOptionsInner = heapless::Vec<MenuOption, MENU_OPTIONS_COUNT>;

pub struct MenuOptions {
    inner: MenuOptionsInner,
    selected_index: usize,
    food_anime: Anime,
}

impl Default for MenuOptions {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            selected_index: Default::default(),
            food_anime: Anime::new(&assets::FRAMES_FOOD_SYMBOL),
        }
    }
}

impl MenuOptions {
    // SLOW POINT
    pub fn new(state: super::State, current: usize, game_ctx: &GameContext) -> Self {
        let mut options = heapless::Vec::new();
        let _ = options.push(MenuOption::PetInfo);
        let _ = options.push(MenuOption::Settings);
        if game_ctx.suiter_system.suiter_waiting()
            && !matches!(
                state,
                super::State::Exploring | super::State::GoneOut { outing_end_time: _ }
            )
        {
            let _ = options.push(MenuOption::Breed);
        }
        if game_ctx.inventory.has_any_item() {
            let _ = options.push(MenuOption::Inventory);
        }
        if game_ctx.inventory.has_any_furniture() && !matches!(state, super::State::Exploring) {
            let _ = options.push(MenuOption::PlaceFurniture);
        }

        if game_ctx.poop_count() > 0 && !matches!(state, super::State::Exploring) {
            let _ = options.push(MenuOption::Poop);
        }

        if game_ctx.pet_records.count() > 0 {
            let _ = options.push(MenuOption::PetRecords);
        }

        if game_ctx.pet.is_ill() && !matches!(state, super::State::Exploring) {
            let _ = options.push(MenuOption::Heal);
        }

        if state != super::State::Sleeping {
            if !matches!(
                state,
                super::State::GoneOut { outing_end_time: _ } | super::State::Exploring
            ) {
                let _ = options.push(MenuOption::GameSelect);
                let _ = options.push(MenuOption::FoodSelect);
            }
            let _ = options.push(MenuOption::Shop);
        }

        if !matches!(
            state,
            super::State::Exploring | super::State::GoneOut { outing_end_time: _ }
        ) {
            let _ = options.push(MenuOption::Explore);
        }

        options.sort_unstable();

        let mut result = MenuOptions::default();
        if current < options.len() {
            result.selected_index = current;
        }
        result.inner = options;

        result
    }

    pub fn tick(&mut self, delta: Duration) {
        self.food_anime.tick(delta);
    }

    pub fn change_option(&mut self, change: i32) {
        let index = self.selected_index as i32 + change;

        self.selected_index = if index >= self.inner.len() as i32 {
            0usize
        } else if index < 0 {
            self.inner.len() - 1
        } else {
            index as usize
        };
    }

    pub fn current(&self) -> &MenuOption {
        &self
            .inner
            .get(self.selected_index)
            .unwrap_or(&MenuOption::PetInfo)
    }

    pub fn current_index(&self) -> usize {
        self.selected_index
    }

    pub fn inner(&self) -> &MenuOptionsInner {
        &self.inner
    }
}

impl ComplexRender for MenuOptions {
    fn render(&self, display: &mut GameDisplay) {
        use super::{BORDER_HEIGHT, WONDER_RECT};

        const BOTTOM_BORDER_RECT: Rect = Rect::new_center(
            Vec2::new(CENTER_X, WONDER_RECT.pos_top_left().y + WONDER_RECT.size.y),
            Vec2::new(WIDTH_F32, BORDER_HEIGHT),
        );

        const SYMBOL_BUFFER: f32 = 2.;
        const IMAGE_Y_START: f32 = BOTTOM_BORDER_RECT.pos.y + BORDER_HEIGHT + SYMBOL_BUFFER;

        const SIZE: Vec2 = Vec2::new(
            assets::IMAGE_SYMBOL_POOP.size.x as f32,
            assets::IMAGE_SYMBOL_POOP.size.y as f32,
        );

        for (i, option) in self.inner.iter().enumerate() {
            let image = match option {
                MenuOption::Breed => &assets::IMAGE_SYMBOL_BREED,
                MenuOption::Poop => &assets::IMAGE_SYMBOL_POOP,
                MenuOption::PetInfo => &assets::IMAGE_SYMBOL_INFO,
                MenuOption::GameSelect => &assets::IMAGE_SYMBOL_GAME,
                MenuOption::FoodSelect => self.food_anime.current_frame(),
                MenuOption::Shop => &assets::IMAGE_SYMBOL_SHOP,
                MenuOption::Inventory => &assets::IMAGE_SYMBOL_INVENTORY,
                MenuOption::PlaceFurniture => &assets::IMAGE_SYMBOL_PLACE_FURNITURE,
                MenuOption::PetRecords => &assets::IMAGE_SYMBOL_RECORDS,
                MenuOption::Heal => &assets::IMAGE_SYMBOL_HEALTHCARE,
                MenuOption::Settings => &assets::IMAGE_SYMBOL_SETTINGS,
                MenuOption::Explore => &assets::IMAGE_SYMBOL_EXPLORE,
            };
            let x = if self.selected_index > 0 {
                let x_index = i as i32 - self.selected_index as i32 + 1;
                SYMBOL_BUFFER + (x_index as f32 * (SIZE.x + SYMBOL_BUFFER))
            } else {
                SYMBOL_BUFFER + ((i + 1) as f32 * (SIZE.x + SYMBOL_BUFFER))
            };
            display.render_image_complex(
                x as i32,
                IMAGE_Y_START as i32,
                image,
                ComplexRenderOption::new().with_white().with_black(),
            );
        }

        let select_rect = Rect::new_top_left(
            Vec2::new(
                SYMBOL_BUFFER + (1_f32 * (SIZE.x + SYMBOL_BUFFER)) - (SYMBOL_BUFFER),
                IMAGE_Y_START - (SYMBOL_BUFFER),
            ),
            Vec2::new(SIZE.x + SYMBOL_BUFFER * 2., SIZE.y + SYMBOL_BUFFER * 2.),
        );
        display.render_rect_outline(select_rect, true);
    }
}
