use core::time::Duration;

use glam::Vec2;

use crate::{
    anime::tick_all_anime,
    display::{GameDisplay, CENTER_VEC, HEIGHT_F32, WIDTH_F32},
    geo::Rect,
    pet::{definition::PetAnimationSet, render::PetRender},
    poop::{update_poop_renders, PoopRender, MAX_POOPS},
    scene::{home_scene, RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs},
    sounds::{SongPlayOptions, SONG_FAN_FARE},
};

const WIPE_SPEED: f32 = 20.;

enum State {
    PreWipe { elapsed: Duration },
    Wiping { x: f32 },
    Cheering { elapsed: Duration },
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
    pet_render: PetRender,
}

impl Default for PoopClearScene {
    fn default() -> Self {
        Self::new()
    }
}

impl PoopClearScene {
    pub fn new() -> Self {
        Self {
            poops: Default::default(),
            state: State::default(),
            pet_render: PetRender::default(),
        }
    }
}

impl Scene for PoopClearScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.pet_render.set_def_id(args.game_ctx.pet.def_id);
        self.pet_render.set_animation(PetAnimationSet::Happy);
        self.pet_render.pos = CENTER_VEC;
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.sound_system.clear_song();
        args.game_ctx.poops = Default::default();
    }

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        update_poop_renders(&mut self.poops, &args.game_ctx.poops);

        tick_all_anime(&mut self.poops, args.delta);

        self.pet_render.tick(args.delta);

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
                    args.game_ctx
                        .sound_system
                        .push_song(SONG_FAN_FARE, SongPlayOptions::new().with_effect());
                    self.state = State::Cheering {
                        elapsed: Duration::ZERO,
                    };
                }
            }
            State::Cheering { elapsed } => {
                *elapsed += args.delta;
                if *elapsed > Duration::from_secs(3) {
                    return SceneOutput::new(SceneEnum::Home(home_scene::HomeScene::new()));
                }
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        match &self.state {
            State::PreWipe { elapsed: _elapsed } => {
                display.render_sprites(&self.poops);
            }
            State::Wiping { x } => {
                display.render_sprites(&self.poops);
                display.invert();
                let rect = Rect::new_top_left(Vec2::ZERO, Vec2::new(*x, HEIGHT_F32));
                display.render_rect_solid(rect, false);
            }
            State::Cheering { elapsed: _elapsed } => {
                display.render_sprite(&self.pet_render);
            }
        }
    }
}
