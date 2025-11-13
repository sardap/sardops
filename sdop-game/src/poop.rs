use bincode::{Decode, Encode};
use glam::Vec2;

use crate::{
    anime::{Anime, HasAnime},
    assets,
    scene::home_scene::WONDER_RECT,
    sprite::Sprite,
    Timestamp,
};

pub const MAX_POOPS: usize = 5;

pub fn poop_count(poops: &[Option<Poop>]) -> usize {
    poops.iter().filter(|i| i.is_some()).count()
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Encode, Decode)]
pub struct Poop {
    spawned: Timestamp,
}

impl Poop {
    pub fn new(timestamp: Timestamp) -> Self {
        Self { spawned: timestamp }
    }
}

#[derive(Copy, Clone)]
pub struct PoopRender {
    pos: Vec2,
    anime: Anime,
}

impl PoopRender {
    pub fn from_poop(poop: &Poop) -> Self {
        let mut rng = fastrand::Rng::with_seed(poop.spawned.seed());

        Self {
            pos: WONDER_RECT
                .shrink(assets::IMAGE_POOP_0.size.x.max(assets::IMAGE_POOP_0.size.y) as f32)
                .random_point_inside(&mut rng),
            anime: Anime::new(&assets::FRAMES_POOP),
        }
    }
}

impl Sprite for PoopRender {
    fn pos(&self) -> &Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl crate::assets::Image {
        self.anime.current_frame()
    }
}

impl HasAnime for PoopRender {
    fn anime(&mut self) -> &mut Anime {
        &mut self.anime
    }
}

pub fn add_poop(poops: &mut [Option<Poop>], timestamp: Timestamp) {
    for poop in poops {
        if poop.is_none() {
            *poop = Some(Poop::new(timestamp));
            break;
        }
    }
}

pub fn update_poop_renders(renders: &mut [Option<PoopRender>], poops: &[Option<Poop>]) {
    for i in 0..MAX_POOPS {
        match &poops[i] {
            Some(poop) => {
                if renders[i].is_none() {
                    renders[i] = Some(PoopRender::from_poop(poop));
                }
            }
            None => renders[i] = None,
        }
    }
}
