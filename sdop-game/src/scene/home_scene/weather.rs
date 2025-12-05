use core::{time::Duration, u8};

use chrono::Timelike;
use fixedstr::{str32, str_format};
use glam::Vec2;
use log::info;
use sdop_common::{MelodyEntry, Note};
use strum::EnumCount;
use strum_macros::EnumCount;

use crate::{
    anime::{tick_all_anime, Anime, HasAnime, MaskedAnimeRender},
    assets::{
        self, Image, FRAMES_GONE_OUT_SIGN, FRAMES_GONE_OUT_SIGN_MASK, FRAMES_SKULL,
        FRAMES_SKULL_MASK, IMAGE_STOMACH_MASK,
    },
    date_utils::DurationExt,
    display::{
        ComplexRender, ComplexRenderOption, GameDisplay, CENTER_VEC, CENTER_X, CENTER_Y,
        HEIGHT_F32, WIDTH_F32,
    },
    egg::EggRender,
    fonts::FONT_VARIABLE_SMALL,
    furniture::{HomeFurnitureKind, HomeFurnitureLocation, HomeFurnitureRender},
    game_context::GameContext,
    geo::{vec2_direction, vec2_distance, Rect},
    items::ItemKind,
    pc::{PcKind, PcRender},
    pet::{
        definition::{PetAnimationSet, PET_BRAINO_ID},
        render::PetRender,
        LifeStage, Mood,
    },
    poop::{poop_count, update_poop_renders, PoopRender, MAX_POOPS},
    scene::{
        death_scene::DeathScene,
        egg_hatch_scene::EggHatchScene,
        evolve_scene::EvolveScene,
        food_select::FoodSelectScene,
        game_select::GameSelectScene,
        heal_scene::HealScene,
        home_scene::menu_options::{MenuOption, MenuOptions},
        inventory_scene::InventoryScene,
        pet_info_scene::PetInfoScene,
        pet_records_scene::PetRecordsScene,
        place_furniture_scene::PlaceFurnitureScene,
        poop_clear_scene::PoopClearScene,
        settings_scene::SettingsScene,
        shop_scene::ShopScene,
        suiters_scene::SuitersScene,
        weekday_select_scene::WeekdaySelectScene,
        RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs,
    },
    sounds::{SongPlayOptions, SONG_ALARM, SONG_HUNGRY, SONG_POOPED, SONG_SICK},
    sprite::{BasicAnimeSprite, MusicNote, Snowflake, Sprite},
    temperature::{self, TemperatureLevel},
    tv::{get_show_for_time, TvKind, TvRender, SHOW_RUN_TIME},
    Button, Song, Timestamp, WIDTH,
};

#[derive(Clone, Copy)]
enum WeatherKind {
    None,
    Cold,
    Snow,
    Hot,
}

pub struct Weather {
    kind: WeatherKind,
    snow_flakes: [Snowflake; 30],
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            kind: WeatherKind::None,
            snow_flakes: Default::default(),
        }
    }
}

impl Weather {
    pub fn setup(&mut self, rng: &mut fastrand::Rng) {
        for flake in &mut self.snow_flakes {
            flake.reset(true, rng);
        }
    }

    pub fn tick<'a>(
        &mut self,
        delta: Duration,
        rng: &mut fastrand::Rng,
        temperature_level: TemperatureLevel,
    ) {
        self.kind = TemperatureLevel::from(temperature_level).into();

        match self.kind {
            WeatherKind::None => {}
            WeatherKind::Cold => {}
            WeatherKind::Snow => {
                for flake in &mut self.snow_flakes {
                    flake.pos += flake.dir * delta.as_secs_f32();
                    if flake.pos.y > HEIGHT_F32 + assets::IMAGE_SNOWFLAKE.size.y as f32
                        || flake.pos.x > WIDTH_F32 + assets::IMAGE_SNOWFLAKE.size.x as f32
                        || flake.pos.x < -(assets::IMAGE_SNOWFLAKE.size.x as f32)
                    {
                        flake.reset(false, rng);
                    }
                }
            }
            WeatherKind::Hot => {}
        }
    }

    pub fn should_shake(&self) -> bool {
        match self.kind {
            WeatherKind::Cold | WeatherKind::Snow => true,
            _ => false,
        }
    }

    pub fn is_weather_none(&self) -> bool {
        matches!(self.kind, WeatherKind::None)
    }
}

impl From<TemperatureLevel> for WeatherKind {
    fn from(value: TemperatureLevel) -> Self {
        match value {
            TemperatureLevel::VeryHot | TemperatureLevel::Hot => Self::Hot,
            TemperatureLevel::Pleasant => Self::None,
            TemperatureLevel::Cold => Self::Cold,
            TemperatureLevel::VeryCold => Self::Snow,
        }
    }
}

impl ComplexRender for Weather {
    fn render(&self, display: &mut GameDisplay) {
        if matches!(self.kind, WeatherKind::Snow) {
            for flake in &self.snow_flakes {
                if flake.pos.y > -((assets::IMAGE_SNOWFLAKE.size.y / 2) as f32) {
                    display.render_sprite(flake);
                }
            }
        }
    }
}
