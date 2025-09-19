use core::time::Duration;

use glam::Vec2;

use crate::{
    assets::{IMAGE_HEART, IMAGE_HEART_MASK},
    display::{CENTER_X, CENTER_Y, ComplexRenderOption, GameDisplay, HEIGHT_F32, WIDTH_F32},
    egg::{EggRender, SavedEgg},
    geo::Rect,
    pet::{ParentInfo, PetParents, combine_pid, definition::PetAnimationSet, render::PetRender},
    scene::{RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, home_scene::HomeScene},
};

#[derive(PartialEq, Eq)]
enum State {
    Entering,
    Happy,
    BlindsUp,
    Egg,
}

pub struct BreedScene {
    left_render: PetRender,
    left: ParentInfo,
    right_render: PetRender,
    right: ParentInfo,
    hearts: [f32; 10],
    blinds_y: f32,
    state: State,
    state_elapsed: Duration,
    egg: EggRender,
    egg_bounce: f32,
}

const EGG_Y: f32 = CENTER_Y + 20.;

impl BreedScene {
    pub fn new(left: ParentInfo, right: ParentInfo) -> Self {
        Self {
            egg: EggRender::new(
                Vec2::new(CENTER_X, EGG_Y),
                combine_pid(left.upid(), right.upid()),
            ),
            left_render: PetRender::new(left.def_id()),
            right_render: PetRender::new(right.def_id()),
            left: left,
            right: right,
            hearts: Default::default(),
            blinds_y: 0.,
            state: State::Entering,
            state_elapsed: Duration::ZERO,
            egg_bounce: 0.,
        }
    }
}

impl Scene for BreedScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {
        self.left_render.pos = Vec2::new(-20., 100.);
        self.right_render.pos = Vec2::new(WIDTH_F32 + 20., 100.);
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.egg = Some(SavedEgg::new(
            combine_pid(self.left.upid(), self.right.upid()),
            Some(PetParents::new([self.left, self.right])),
        ));
    }

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        self.state_elapsed += args.delta;
        self.left_render.tick(args.delta);
        self.right_render.tick(args.delta);

        const BLINDS_SPEED: f32 = 10.;
        const EGG_BOUNCE_SPEED: f32 = 3.;
        const BOUNCE_RANGE: i32 = 5;

        self.egg_bounce += EGG_BOUNCE_SPEED * args.delta.as_secs_f32();
        self.egg.pos = Vec2::new(
            self.egg.pos.x,
            EGG_Y
                + if (self.egg_bounce as i32) % BOUNCE_RANGE * 2 > BOUNCE_RANGE {
                    BOUNCE_RANGE * 2 + -(self.egg_bounce as i32 % BOUNCE_RANGE * 2)
                } else {
                    self.egg_bounce as i32 % BOUNCE_RANGE * 2
                } as f32,
        );

        match self.state {
            State::Entering => {
                self.left_render.set_animation(PetAnimationSet::Normal);
                self.right_render.set_animation(PetAnimationSet::Normal);

                const SPEED: f32 = 10.;
                const END: f32 = 15.;

                self.left_render.pos.x += args.delta.as_secs_f32() * SPEED;
                self.right_render.pos.x -= args.delta.as_secs_f32() * SPEED;

                if self.left_render.pos.x > WIDTH_F32 / 2. - END {
                    self.left_render.pos.x = WIDTH_F32 / 2. - END;
                    self.right_render.pos.x = WIDTH_F32 / 2. + END;
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::Happy;
                }
            }
            State::Happy => {
                self.left_render.set_animation(PetAnimationSet::Happy);
                self.right_render.set_animation(PetAnimationSet::Happy);

                const HEART_SPEED: f32 = 15.;
                const HEARTS_SPAWN_LENGTH: Duration = Duration::from_secs(10);

                for heart in &mut self.hearts {
                    if *heart == 0. {
                        continue;
                    }
                    *heart -= args.delta.as_secs_f32() * HEART_SPEED;
                }

                let percent = self.state_elapsed.as_secs_f32() / HEARTS_SPAWN_LENGTH.as_secs_f32();
                let count = ((self.hearts.len() as f32 * percent) as usize).min(self.hearts.len());

                for i in 0..count {
                    if self.hearts[i] == 0. {
                        self.hearts[i] = self.left_render.pos.y;
                    }
                }

                self.blinds_y += args.delta.as_secs_f32() * BLINDS_SPEED;

                if self.blinds_y > HEIGHT_F32 {
                    self.left_render.pos.y = 40.;
                    self.right_render.pos.y = 40.;
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::BlindsUp;
                }
            }
            State::BlindsUp => {
                self.blinds_y -= args.delta.as_secs_f32() * BLINDS_SPEED;

                if self.blinds_y < 0. {
                    self.state_elapsed = Duration::ZERO;
                    self.state = State::Egg;
                }
            }
            State::Egg => {
                if self.state_elapsed > Duration::from_secs(5) {
                    return SceneOutput::new(SceneEnum::Home(HomeScene::new()));
                }
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        if self.state == State::Happy {
            for y in self.hearts {
                if y == 0. {
                    continue;
                }
                display.render_image_complex(
                    self.left_render.pos.x as i32,
                    y as i32,
                    &IMAGE_HEART,
                    ComplexRenderOption::new().with_white().with_center(),
                );
                display.render_image_complex(
                    self.left_render.pos.x as i32,
                    y as i32,
                    &IMAGE_HEART_MASK,
                    ComplexRenderOption::new().with_black().with_center(),
                );
                display.render_image_complex(
                    self.right_render.pos.x as i32,
                    y as i32,
                    &IMAGE_HEART,
                    ComplexRenderOption::new().with_white().with_center(),
                );
                display.render_image_complex(
                    self.right_render.pos.x as i32,
                    y as i32,
                    &IMAGE_HEART_MASK,
                    ComplexRenderOption::new().with_black().with_center(),
                );
            }
        }

        display.render_sprite(&self.left_render);
        display.render_sprite(&self.right_render);

        if self.state == State::Egg || self.state == State::BlindsUp {
            display.render_complex(&self.egg);
        }

        display.render_rect_solid(
            Rect::new_top_left(Vec2::new(0., 0.), Vec2::new(WIDTH_F32, self.blinds_y)),
            true,
        );
    }
}
