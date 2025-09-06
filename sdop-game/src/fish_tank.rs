use core::time::Duration;

use bincode::{Decode, Encode};
use glam::Vec2;
use heapless::Vec;

use crate::{
    anime::{Anime, HasAnime},
    assets,
    display::{ComplexRender, ComplexRenderOption},
    geo::Rect,
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Encode, Decode)]
pub struct HomeFishTank {
    #[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
    pub fish: [u8; MAX_FISH],
}

impl HomeFishTank {
    pub fn add(&mut self, rng: &mut fastrand::Rng) {
        for i in 0..self.fish.len() {
            if self.fish[i] == 0 {
                self.fish[i] = rng.u8(1..=3);
                break;
            }
        }
    }

    pub fn count(&self) -> usize {
        for i in 0..self.fish.len() {
            if self.fish[i] == 0 {
                return i;
            }
        }

        return self.fish.len();
    }
}

impl Default for HomeFishTank {
    fn default() -> Self {
        Self {
            fish: [0; MAX_FISH],
        }
    }
}

pub const MAX_FISH: usize = 20;

struct Fish {
    pos: Vec2,
    direction: Vec2,
    speed: f32,
}

impl Fish {
    pub fn new(pos: Vec2, direction: Vec2, speed: f32) -> Self {
        Self {
            pos,
            direction,
            speed,
        }
    }
}

pub struct FishTankRender {
    pub pos: Vec2,
    fish: Vec<Fish, MAX_FISH>,
    anime: Anime,
}

const FISH_AREA: Rect = Rect::new_top_left(Vec2::new(1., 4.), Vec2::new(16., 10.));

impl FishTankRender {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            fish: Vec::new(),
            anime: Anime::new(&assets::FRAMES_FISH_TANK_EMPTY),
        }
    }

    pub const fn size() -> Vec2 {
        Vec2::new(
            assets::IMAGE_FISH_TANK_EMPTY_0.size.x as f32,
            assets::IMAGE_FISH_TANK_EMPTY_0.size.y as f32,
        )
    }

    pub fn add_fish(&mut self, rng: &mut fastrand::Rng, speed: f32) {
        let angle = rng.i32(0..360) as f32;
        let x_dir = libm::sinf(angle);
        let y_dir = libm::cosf(angle);

        let _ = self.fish.push(Fish::new(
            FISH_AREA.random_point_inside(rng),
            Vec2::new(x_dir, y_dir),
            speed,
        ));
    }

    pub fn fish_count(&self) -> usize {
        self.fish.len()
    }

    pub fn tick(&mut self, delta: Duration, rng: &mut fastrand::Rng) {
        for fish in &mut self.fish {
            let change = Vec2::new(
                fish.direction.x * fish.speed * delta.as_secs_f32(),
                fish.direction.y * fish.speed * delta.as_secs_f32(),
            );

            if !FISH_AREA.point_inside(&(fish.pos + change)) {
                let angle = rng.i32(0..360) as f32;
                let x_dir = libm::sinf(angle);
                let y_dir = libm::cosf(angle);
                fish.direction = Vec2::new(x_dir, y_dir);
            } else {
                fish.pos += change
            }
        }

        self.anime.tick(delta);
    }
}

impl HasAnime for FishTankRender {
    fn anime(&mut self) -> &mut Anime {
        &mut self.anime
    }
}

impl ComplexRender for FishTankRender {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            self.anime.current_frame(),
            ComplexRenderOption::new().with_white().with_center(),
        );

        let top_left = self.pos - Vec2::new(Self::size().x / 2., Self::size().y / 2.);
        for fish in &self.fish {
            let pos = top_left + fish.pos;
            display.render_point(pos.x as i32, pos.y as i32, true);
            display.render_point(pos.x as i32 + 1, pos.y as i32, true);
        }
    }
}
