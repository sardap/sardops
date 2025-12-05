use core::time::Duration;

use glam::Vec2;
use sdop_common::{MelodyEntry, Note};

use crate::{
    anime::Anime,
    assets::{self},
    display::{ComplexRender, ComplexRenderOption, GameDisplay, CENTER_X, WIDTH_F32},
    game_context::GameContext,
    geo::Rect,
    Song,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MenuOption {
    PetInfo,
    Breed,
    Poop,
    Heal,
    GameSelect,
    FoodSelect,
    Shop,
    Inventory,
    PlaceFurniture,
    PetRecords,
    Settings,
}

impl MenuOption {
    pub fn get_song(&self) -> &'static Song {
        const MENU_DURATION: i16 = 8;
        match self {
            MenuOption::PetInfo => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::C2, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::Breed => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::C3, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::Poop => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::C4, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::Heal => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::C5, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::GameSelect => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::D2, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::FoodSelect => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::D3, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::Shop => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::D4, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::Inventory => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::D5, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::PlaceFurniture => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::E3, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::PetRecords => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::E4, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
            MenuOption::Settings => {
                const MELODY: &[MelodyEntry] = &[MelodyEntry::new(Note::D3, MENU_DURATION)];
                const SONG: Song = Song::new(MELODY, 85);
                &SONG
            }
        }
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
    pub fn new(state: super::State, game_ctx: &GameContext) -> Self {
        let mut options = heapless::Vec::new();
        let _ = options.push(MenuOption::PetInfo);
        let _ = options.push(MenuOption::Settings);
        if game_ctx.suiter_system.suiter_waiting() {
            let _ = options.push(MenuOption::Breed);
        }
        if game_ctx.inventory.has_any_item() {
            let _ = options.push(MenuOption::Inventory);
        }
        if game_ctx.inventory.has_any_furniture() {
            let _ = options.push(MenuOption::PlaceFurniture);
        }

        if game_ctx.poop_count() > 0 {
            let _ = options.push(MenuOption::Poop);
        }

        if game_ctx.pet_records.count() > 0 {
            let _ = options.push(MenuOption::PetRecords);
        }

        if game_ctx.pet.is_ill() {
            let _ = options.push(MenuOption::Heal);
        }

        if state != super::State::Sleeping {
            if !matches!(state, super::State::GoneOut { outing_end_time: _ }) {
                let _ = options.push(MenuOption::GameSelect);
                let _ = options.push(MenuOption::FoodSelect);
            }
            let _ = options.push(MenuOption::Shop);
        }

        options.sort_unstable();

        let mut result = MenuOptions::default();
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
        &self.inner[self.selected_index]
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
            assets::IMAGE_POOP_SYMBOL.size.x as f32,
            assets::IMAGE_POOP_SYMBOL.size.y as f32,
        );

        for (i, option) in self.inner.iter().enumerate() {
            let image = match option {
                MenuOption::Breed => &assets::IMAGE_SYMBOL_BREED,
                MenuOption::Poop => &assets::IMAGE_POOP_SYMBOL,
                MenuOption::PetInfo => &assets::IMAGE_INFO_SYMBOL,
                MenuOption::GameSelect => &assets::IMAGE_GAME_SYMBOL,
                MenuOption::FoodSelect => self.food_anime.current_frame(),
                MenuOption::Shop => &assets::IMAGE_SHOP_SYMBOL,
                MenuOption::Inventory => &assets::IMAGE_SYMBOL_INVENTORY,
                MenuOption::PlaceFurniture => &assets::IMAGE_SYMBOL_PLACE_FURNITURE,
                MenuOption::PetRecords => &assets::IMAGE_SYMBOL_RECORDS,
                MenuOption::Heal => &assets::IMAGE_SYMBOL_HEALTHCARE,
                MenuOption::Settings => &assets::IMAGE_SYMBOL_SETTINGS,
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
