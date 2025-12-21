use core::time::Duration;

use glam::Vec2;

use crate::{
    anime::Anime,
    assets::{self, Frame},
    display::{ComplexRender, ComplexRenderOption},
};

const DREAMS: &[&[Frame]] = &[
    &assets::FRAMES_DREAM_CONSUMED_MASK,
    &assets::FRAMES_DREAM_SLEEPING_MASK,
    &assets::FRAMES_DREAM_EATING_MASK,
    &assets::FRAMES_DREAM_WORDS_MASK,
    &assets::FRAMES_DREAM_WAVES_MASK,
    &assets::FRAMES_DREAM_BIRD_MASK,
    &assets::FRAMES_DREAM_CUBE_MASK,
    &assets::FRAMES_DREAM_CONNECTIONS_MASK,
    &assets::FRAMES_DREAM_SMILE_MASK,
    &assets::FRAMES_DREAM_WORM_MASK,
    &assets::FRAMES_DREAM_BUBBLE_MASK,
    &assets::FRAMES_DREAM_EXAM_MASK,
];

pub struct DreamBubble {
    pub pos: Vec2,
    dream_bubble: Anime,
    current_dream: Anime,
    dream_left: Duration,
}

impl DreamBubble {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            dream_bubble: Anime::new(&assets::FRAMES_DREAM_BUBBLE),
            current_dream: Anime::new(DREAMS[0]),
            dream_left: Duration::ZERO,
        }
    }

    pub fn tick(&mut self, delta: Duration, rng: &mut fastrand::Rng) {
        if self.dream_left <= Duration::ZERO {
            self.dream_left = Duration::from_secs(rng.u64(60..180));
            self.current_dream = Anime::new(rng.choice(DREAMS).unwrap());
        }
        self.dream_left = self.dream_left.checked_sub(delta).unwrap_or(Duration::ZERO);

        self.dream_bubble.tick(delta);
        self.current_dream.tick(delta);
    }
}

impl ComplexRender for DreamBubble {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            self.dream_bubble.current_frame(),
            ComplexRenderOption::new().with_white().with_center(),
        );

        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32 - 2,
            self.current_dream.current_frame(),
            ComplexRenderOption::new().with_black().with_center(),
        );
    }
}
