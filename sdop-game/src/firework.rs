use core::time::Duration;

use glam::Vec2;

use crate::{
    anime::Anime,
    assets::{FRAMES_FIREWORK_EXPLODE, IMAGE_FIREWORK_FLYING},
    sprite::Sprite,
};

pub struct Firework {
    pos: Vec2,
    dir: Vec2,
    explode_y: f32,
    anime: Anime,
}

impl Firework {
    pub fn new(start: Vec2, dir: Vec2, explode_y: f32) -> Self {
        Self {
            pos: start,
            dir,
            explode_y,
            anime: Anime::new(&FRAMES_FIREWORK_EXPLODE),
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        if self.explode_y >= self.pos.y {
            if !self.done() {
                self.anime.tick(delta);
            }
        } else {
            self.pos += self.dir * delta.as_secs_f32();
        }
    }

    pub fn done(&self) -> bool {
        self.anime.current_frame() == self.anime.last_frame()
    }
}

impl Sprite for Firework {
    fn pos(&self) -> &Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl crate::assets::Image {
        if self.explode_y >= self.pos.y {
            self.anime.current_frame()
        } else {
            &IMAGE_FIREWORK_FLYING
        }
    }

    fn size_x(&self) -> i32 {
        if self.explode_y >= self.pos.y {
            self.anime.current_frame().isize.x
        } else {
            IMAGE_FIREWORK_FLYING.isize.x
        }
    }

    fn size_y(&self) -> i32 {
        if self.explode_y >= self.pos.y {
            self.anime.current_frame().isize.y
        } else {
            IMAGE_FIREWORK_FLYING.isize.y
        }
    }
}
