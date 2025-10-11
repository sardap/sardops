use core::time::Duration;

use bincode::{Decode, Encode};
use glam::{I8Vec2, Vec2};

use crate::{
    anime::Anime,
    assets,
    death::passed_threshold_chance,
    display::{ComplexRender, ComplexRenderOption, GameDisplay},
    game_consts::EGG_HATCH_ODDS_THRESHOLD,
    pet::{PetParents, UniquePetId},
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone)]
pub struct SavedEgg {
    pub age: Duration,
    pub upid: UniquePetId,
    pub parents: Option<PetParents>,
    pub hatch: bool,
}

impl SavedEgg {
    pub const fn new(pid: UniquePetId, parents: Option<PetParents>) -> Self {
        Self {
            age: Duration::ZERO,
            upid: pid,
            parents,
            hatch: false,
        }
    }

    pub fn sim_tick(&mut self, delta: Duration, rng: &mut fastrand::Rng) {
        self.age += delta;

        if passed_threshold_chance(rng, EGG_HATCH_ODDS_THRESHOLD, self.age) {
            self.hatch = true
        }
    }
}

const EGG_DOT_POSTIONS: &[I8Vec2] = &[
    I8Vec2::new(5, 1),
    I8Vec2::new(6, 1),
    I8Vec2::new(4, 2),
    I8Vec2::new(5, 2),
    I8Vec2::new(6, 2),
    I8Vec2::new(7, 2),
    I8Vec2::new(3, 3),
    I8Vec2::new(4, 3),
    I8Vec2::new(5, 3),
    I8Vec2::new(6, 3),
    I8Vec2::new(7, 3),
    I8Vec2::new(8, 3),
    I8Vec2::new(9, 3),
    I8Vec2::new(2, 4),
    I8Vec2::new(3, 4),
    I8Vec2::new(4, 4),
    I8Vec2::new(5, 4),
    I8Vec2::new(6, 4),
    I8Vec2::new(7, 4),
    I8Vec2::new(8, 4),
    I8Vec2::new(9, 4),
    I8Vec2::new(2, 5),
    I8Vec2::new(3, 5),
    I8Vec2::new(4, 5),
    I8Vec2::new(5, 5),
    I8Vec2::new(6, 5),
    I8Vec2::new(7, 5),
    I8Vec2::new(8, 5),
    I8Vec2::new(9, 5),
    I8Vec2::new(1, 6),
    I8Vec2::new(2, 6),
    I8Vec2::new(3, 6),
    I8Vec2::new(4, 6),
    I8Vec2::new(5, 6),
    I8Vec2::new(6, 6),
    I8Vec2::new(7, 6),
    I8Vec2::new(8, 6),
    I8Vec2::new(9, 6),
    I8Vec2::new(10, 6),
    I8Vec2::new(1, 7),
    I8Vec2::new(2, 7),
    I8Vec2::new(3, 7),
    I8Vec2::new(4, 7),
    I8Vec2::new(5, 7),
    I8Vec2::new(6, 7),
    I8Vec2::new(7, 7),
    I8Vec2::new(8, 7),
    I8Vec2::new(9, 7),
    I8Vec2::new(10, 7),
    I8Vec2::new(1, 8),
    I8Vec2::new(2, 8),
    I8Vec2::new(3, 8),
    I8Vec2::new(4, 8),
    I8Vec2::new(5, 8),
    I8Vec2::new(6, 8),
    I8Vec2::new(7, 8),
    I8Vec2::new(8, 8),
    I8Vec2::new(9, 8),
    I8Vec2::new(10, 8),
    I8Vec2::new(2, 9),
    I8Vec2::new(3, 9),
    I8Vec2::new(4, 9),
    I8Vec2::new(5, 9),
    I8Vec2::new(6, 9),
    I8Vec2::new(7, 9),
    I8Vec2::new(8, 9),
    I8Vec2::new(9, 9),
    I8Vec2::new(2, 10),
    I8Vec2::new(3, 10),
    I8Vec2::new(4, 10),
    I8Vec2::new(5, 10),
    I8Vec2::new(6, 10),
    I8Vec2::new(7, 10),
    I8Vec2::new(8, 10),
    I8Vec2::new(9, 10),
    I8Vec2::new(3, 11),
    I8Vec2::new(4, 11),
    I8Vec2::new(5, 11),
    I8Vec2::new(6, 11),
    I8Vec2::new(7, 11),
    I8Vec2::new(8, 11),
    I8Vec2::new(4, 12),
    I8Vec2::new(5, 12),
    I8Vec2::new(6, 12),
    I8Vec2::new(7, 12),
    I8Vec2::new(5, 13),
    I8Vec2::new(6, 13),
];

pub struct EggRender {
    pub pos: Vec2,
    crack_anime: Anime,
    dots: [isize; 15],
}

impl Default for EggRender {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            crack_anime: Anime::new(&assets::FRAMES_EGG_CRACK_MASK),
            dots: Default::default(),
        }
    }
}

impl EggRender {
    pub fn new(pos: Vec2, pid: UniquePetId) -> Self {
        let mut result = EggRender::default();
        result.pos = pos;

        result.set_pid(pid);

        result
    }

    pub fn cracked(&self) -> bool {
        self.crack_anime.current_frame() == self.crack_anime.last_frame()
    }

    pub fn tick(&mut self, delta: Duration) {
        if !self.cracked() {
            self.crack_anime.tick(delta);
        }
    }

    pub fn set_pid(&mut self, pid: UniquePetId) {
        let mut dots = [
            -1_isize, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ];

        let mut rng = fastrand::Rng::with_seed(pid);

        for dot in &mut dots {
            if rng.f32() > 0.8 {
                continue;
            }

            *dot = rng.isize(0..EGG_DOT_POSTIONS.len() as isize);
        }

        self.dots = dots;
    }
}

impl ComplexRender for EggRender {
    fn render(&self, display: &mut GameDisplay) {
        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            &assets::IMAGE_EGG,
            ComplexRenderOption::new().with_white().with_center(),
        );

        let top_left = self.pos
            - Vec2::new(
                assets::IMAGE_EGG.size.x as f32 / 2.,
                assets::IMAGE_EGG.size.y as f32 / 2.,
            );

        for index in &self.dots {
            if *index < 0 {
                continue;
            }

            let point = &EGG_DOT_POSTIONS[*index as usize];

            display.render_point(
                top_left.x as i32 + point.x as i32,
                top_left.y as i32 + point.y as i32,
                false,
            );
        }

        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            self.crack_anime.current_frame(),
            ComplexRenderOption::new().with_black().with_center(),
        );
    }
}
