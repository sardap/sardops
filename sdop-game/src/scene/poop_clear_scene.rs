use core::time::Duration;

use glam::Vec2;

use crate::{
    anime::tick_all_anime,
    display::{GameDisplay, HEIGHT_F32, WIDTH_F32},
    geo::Rect,
    poop::{MAX_POOPS, PoopRender, update_poop_renders},
    scene::{Scene, SceneEnum, SceneOutput, SceneTickArgs, home_scene},
};

const WIPE_SPEED: f32 = 20.;

enum State {
    PreWipe { elapsed: Duration },
    Wiping { x: f32 },
}

impl Default for State {
    fn default() -> Self {
        Self::PreWipe {
            elapsed: Duration::ZERO,
        }
    }
}

pub struct PoopClearScene {
    poops: [Option<PoopRender>; MAX_POOPS],
    state: State,
}

impl PoopClearScene {
    pub fn new() -> Self {
        Self {
            poops: Default::default(),
            state: State::default(),
        }
    }
}

impl Scene for PoopClearScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.poops = Default::default();
    }

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        update_poop_renders(&mut self.poops, &args.game_ctx.poops);

        tick_all_anime(&mut self.poops, args.delta);

        match &mut self.state {
            State::PreWipe { elapsed } => {
                *elapsed += args.delta;
                if *elapsed > Duration::from_secs(1) {
                    self.state = State::Wiping { x: 0. }
                }
            }
            State::Wiping { x } => {
                *x += WIPE_SPEED * args.delta.as_secs_f32();
                if *x > WIDTH_F32 {
                    return SceneOutput::new(SceneEnum::Home(home_scene::HomeScene::new()));
                }
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut SceneTickArgs) {
        display.render_sprites(&self.poops);

        match &self.state {
            State::PreWipe { elapsed: _elapsed } => {}
            State::Wiping { x } => {
                display.invert();
                let rect = Rect::new_top_left(Vec2::ZERO, Vec2::new(*x, HEIGHT_F32));
                display.render_rect_solid(rect, false);
            }
        }
    }
}
