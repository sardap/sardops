#![feature(generic_const_exprs)]
#![no_std]

use core::time::Duration;

use crate::{
    display::{ConvertFn, DrawDisplay},
    fps::FPSCounter,
    game_context::GameContext,
    input::Input,
    scene::{SceneManger, SceneTickArgs},
    sim::tick_sim,
};
mod anime;
mod assets;
mod bit_array;
mod date_utils;
mod display;
mod fonts;
mod food;
mod fps;
mod game_context;
mod geo;
mod input;
mod money;
mod pet;
mod poop;
mod save;
mod scene;
mod sim;
mod sprite;
mod tic_tac_toe;

pub use crate::date_utils::Timestamp;
pub use crate::display::{HEIGHT, WIDTH};
pub use crate::input::{Button, ButtonState, ButtonStates};
pub use crate::save::SaveFile;

pub struct Game {
    display: display::GameDisplay,
    input: input::Input,
    last_time: Timestamp,
    scene_manger: SceneManger,
    game_ctx: GameContext,
    time_scale: f32,
    fps: FPSCounter,
}

impl Game {
    pub fn new(timestamp: Timestamp) -> Self {
        Self {
            display: display::GameDisplay::default(),
            input: input::Input::default(),
            last_time: timestamp,
            scene_manger: SceneManger::default(),
            game_ctx: GameContext::new(timestamp),
            time_scale: 1.,
            fps: FPSCounter::new(timestamp),
        }
    }

    pub fn update_input_states(&mut self, input_states: ButtonStates) {
        self.input.update_state(input_states);
    }

    pub fn input(&self) -> Input {
        self.input
    }

    pub fn set_sim_time_scale(&mut self, time_scale: f32) {
        self.time_scale = time_scale;
    }

    pub fn tick(&mut self, timestamp: Timestamp) {
        let delta = timestamp - self.last_time;

        let mut scene_args = SceneTickArgs {
            timestamp,
            delta,
            input: &self.input,
            game_ctx: &mut self.game_ctx,
        };

        tick_sim(self.time_scale, &mut scene_args);

        self.scene_manger.tick(&mut scene_args);

        let scene = self.scene_manger.scene();

        let output = scene.tick(&mut scene_args);

        if let Some(next) = output.next_scene {
            self.scene_manger.set_next(next);
        }

        self.last_time = timestamp
    }

    pub fn refresh_display(&mut self, timestamp: Timestamp) {
        let delta = timestamp - self.last_time;

        let mut scene_args = SceneTickArgs {
            timestamp,
            delta,
            input: &self.input,
            game_ctx: &mut self.game_ctx,
        };

        self.display.clear();
        let scene = self.scene_manger.scene();
        scene.render(&mut self.display, &mut scene_args);
        self.display.render_fps(&self.fps);
        self.fps.update(timestamp);
    }

    pub fn get_display_image_data(&self) -> &[u8] {
        self.display.image_data()
    }

    pub fn get_display_bmp(&self) -> &[u8] {
        self.display.bmp()
    }

    pub fn drawable<'a, C>(&'a self, convert: ConvertFn<C>) -> DrawDisplay<'a, C> {
        DrawDisplay::new(self.get_display_image_data(), convert)
    }

    pub fn get_save(&self, timestamp: Timestamp) -> SaveFile {
        SaveFile::generate(timestamp, &self.game_ctx)
    }

    pub fn load_save(&mut self, timestamp: Timestamp, save: SaveFile) {
        let last_timestamp = save.last_timestamp;
        let delta = timestamp - last_timestamp;
        save.load(&mut self.game_ctx);
        const STEP_SIZE: Duration = Duration::from_millis(16);
        let steps = (delta.as_millis() / STEP_SIZE.as_millis()) as u64;
        for i in 0..steps {
            let mut scene_args = SceneTickArgs {
                timestamp: last_timestamp + Duration::from_millis(i * 16),
                delta: STEP_SIZE,
                input: &self.input,
                game_ctx: &mut self.game_ctx,
            };
            tick_sim(1., &mut scene_args);
        }
    }
}

pub trait WrappingEnum: Copy + Sized {
    const COUNT: usize;
    fn to_index(self) -> usize;
    fn from_index(index: usize) -> Self;

    fn next(self) -> Self {
        self.offset(1)
    }

    fn prev(self) -> Self {
        self.offset(-1)
    }

    fn offset(self, delta: isize) -> Self {
        let count = Self::COUNT as isize;
        let idx = self.to_index() as isize;
        let new_idx = (idx + delta).rem_euclid(count);
        Self::from_index(new_idx as usize)
    }
}

#[macro_export]
macro_rules! wrapping_enum {
    (
        $(#[$meta:meta])*
        enum $name:ident {
            $($variant:ident),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        pub enum $name {
            $($variant),*
        }

        impl $name {
            pub const VARIANTS: &'static [$name] = &[$($name::$variant),*];
        }

        impl crate::WrappingEnum for $name {
            const COUNT: usize = $name::VARIANTS.len();

            fn to_index(self) -> usize {
                self as usize
            }

            fn from_index(index: usize) -> Self {
                $name::VARIANTS[index % Self::COUNT]
            }
        }
    };
}
