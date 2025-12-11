use core::{ops::Range, time::Duration};

use fixedstr::{str_format, str12, str16};
use glam::Vec2;

use crate::{
    Button, Timestamp,
    assets::{self, Image, StaticImage},
    display::{CENTER_X, ComplexRenderOption, GameDisplay, HEIGHT_F32, WIDTH_F32},
    geo::Rect,
    pet::{
        definition::{PetAnimationSet, PetDefinitionId},
        render::PetRender,
    },
    scene::{RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, mg_fanfare::MgFanFareScene},
    sprite::{BasicSprite, Sprite},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lane {
    Left,
    Center,
    Right,
}

const LANE_WIDTH: f32 = WIDTH_F32 / 3.0;

impl Lane {
    pub fn center_x(&self) -> f32 {
        match self {
            Lane::Left => LANE_WIDTH * 1.0 - LANE_WIDTH / 2.0,
            Lane::Center => LANE_WIDTH * 2.0 - LANE_WIDTH / 2.0,
            Lane::Right => LANE_WIDTH * 3.0 - LANE_WIDTH / 2.0,
        }
    }

    pub fn left(&self) -> Self {
        match self {
            Lane::Left | Lane::Center => Lane::Left,
            Lane::Right => Lane::Center,
        }
    }

    pub fn right(&self) -> Self {
        match self {
            Lane::Left => Lane::Center,
            Lane::Right | Lane::Center => Lane::Right,
        }
    }

    pub fn random(rng: &mut fastrand::Rng) -> Self {
        match rng.u8(0..=2) {
            0 => Lane::Left,
            1 => Lane::Center,
            2 => Lane::Right,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy)]
enum GarbageKinds {
    Handheld,
    Branch,
    Brick,
}

impl GarbageKinds {
    pub fn speed_modifier(&self) -> f32 {
        match self {
            Self::Handheld => 1.,
            Self::Branch => 0.8,
            Self::Brick => 1.2,
        }
    }

    pub fn image(&self) -> &'static StaticImage {
        match self {
            Self::Handheld => &assets::IMAGE_HANDHELD,
            Self::Branch => &assets::IMAGE_BRANCH,
            Self::Brick => &assets::IMAGE_BRICK,
        }
    }

    pub fn random(rng: &mut fastrand::Rng) -> Self {
        match rng.u8(0..=2) {
            0 => Self::Handheld,
            1 => Self::Branch,
            2 => Self::Brick,
            _ => unreachable!(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Garbage {
    sprite: BasicSprite,
    speed: f32,
}

impl Sprite for Garbage {
    fn pos(&self) -> &Vec2 {
        self.sprite.pos()
    }

    fn image(&self) -> &impl Image {
        self.sprite.image
    }
}

const MAX_GARBAGE_COUNT: usize = 20;
const STARTING_GARBAGE_COUNT: usize = 2;

enum State {
    Playing,
    GameOverFreeze { won: bool, elapsed: Duration },
}

pub struct MgDogeEmScene {
    pet_def_id: PetDefinitionId,
    pet_render: PetRender,
    current_lane: Lane,
    falling_garbage: [Option<Garbage>; MAX_GARBAGE_COUNT],
    speed_range: Range<i32>,
    start_time: Timestamp,
    state: State,
    last_lane: Lane,
}

impl MgDogeEmScene {
    pub fn new(pet_def_id: PetDefinitionId) -> Self {
        Self {
            pet_def_id,
            pet_render: PetRender::default(),
            current_lane: Lane::Center,
            falling_garbage: [None; MAX_GARBAGE_COUNT],
            speed_range: 1000..2000,
            start_time: Timestamp::default(),
            state: State::Playing,
            last_lane: Lane::Left,
        }
    }
}

impl Scene for MgDogeEmScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.pet_render.set_def_id(self.pet_def_id);
        self.pet_render.pos.y = HEIGHT_F32 - self.pet_render.image().size().y as f32;

        self.start_time = args.timestamp;
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        self.pet_render.pos.x = self.current_lane.center_x();

        self.pet_render.tick(args.delta);

        match self.state {
            State::Playing => {
                let elapsed = args.timestamp - self.start_time;
                if elapsed > Duration::from_secs(60) {
                    self.start_time = args.timestamp;
                    self.state = State::GameOverFreeze { won: true, elapsed };
                }

                let max_falling = (elapsed.as_secs_f32() / Duration::from_secs(6).as_secs_f32())
                    as usize
                    + STARTING_GARBAGE_COUNT;

                let mut current_active = {
                    let mut count: usize = 0;

                    for garbage in &self.falling_garbage {
                        if garbage.is_some() {
                            count += 1;
                        }
                    }

                    count
                };

                if current_active != max_falling {
                    log::info!("Active falling {} {}", current_active, max_falling);
                }

                // Make it so the hit box is only on top
                let player_rect = Rect::new_center(
                    self.pet_render.pos
                        - Vec2::new(0., self.pet_render.image().size().y as f32 / 2.),
                    Vec2::new(LANE_WIDTH / 2., 3.),
                );

                for garbage in &mut self.falling_garbage {
                    match garbage {
                        Some(instance) => {
                            let move_amount = instance.speed * args.delta.as_secs_f32();
                            instance.sprite.pos.y += move_amount;
                            if instance.sprite.pos.y
                                > HEIGHT_F32 + instance.sprite.image.size.y as f32
                            {
                                *garbage = None;
                                continue;
                            }

                            if player_rect.overlapping(&instance.rect()) {
                                self.state = State::GameOverFreeze {
                                    won: false,
                                    elapsed,
                                };
                                self.start_time = args.timestamp;
                                break;
                            }
                        }
                        None => {
                            if current_active >= max_falling {
                                continue;
                            }
                            current_active += 1;

                            // Think deeper need to check for overlap
                            let kind = GarbageKinds::random(&mut args.game_ctx.rng);
                            let mut lane = Lane::random(&mut args.game_ctx.rng);
                            if lane == self.last_lane {
                                lane = if args.game_ctx.rng.bool() {
                                    lane.left()
                                } else {
                                    lane.right()
                                };
                            }
                            self.last_lane = lane;
                            let y_min = -(kind.image().size.y as f32);
                            let y = args
                                .game_ctx
                                .rng
                                .i32(((y_min * 1.3 * 1000.) as i32)..((y_min * 1000.) as i32));
                            let y = y as f32 / 1000.;
                            let speed = (args.game_ctx.rng.i32(self.speed_range.clone()) as f32
                                / 100.0)
                                * kind.speed_modifier();
                            *garbage = Some(Garbage {
                                sprite: BasicSprite::new(
                                    Vec2::new(lane.center_x(), y),
                                    kind.image(),
                                ),
                                speed,
                            });
                        }
                    }
                }

                if args.input.pressed(Button::Left) {
                    self.current_lane = self.current_lane.left();
                }
                if args.input.pressed(Button::Right) {
                    self.current_lane = self.current_lane.right();
                }
            }
            State::GameOverFreeze { won, elapsed } => {
                self.pet_render.set_animation(if won {
                    PetAnimationSet::Happy
                } else {
                    PetAnimationSet::Sad
                });

                if args.timestamp - self.start_time > Duration::from_secs(3) {
                    let winnings = if won {
                        (60. * 100.) as i32
                    } else {
                        (elapsed.as_secs_f32() * 100. / 2.) as i32
                    };
                    return SceneOutput::new(SceneEnum::MgFanFare(MgFanFareScene::new(
                        won,
                        winnings,
                        self.pet_def_id,
                    )));
                }
            }
        };

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        display.render_sprite(&self.pet_render);
        display.render_sprites(&self.falling_garbage);

        match self.state {
            State::Playing => {
                const SCORE_RECT: Rect =
                    Rect::new_top_left(Vec2::new(0., 0.), Vec2::new(WIDTH_F32, 15.));
                display.render_rect_solid(SCORE_RECT, false);

                const SCORE_BOTTOM_RECT: Rect =
                    Rect::new_top_left(Vec2::new(0., SCORE_RECT.size.y), Vec2::new(WIDTH_F32, 2.));
                display.render_rect_solid(SCORE_BOTTOM_RECT, true);

                let elapsed = args.timestamp - self.start_time;
                let text = fixedstr::str_format!(str16, "{:.1}", elapsed.as_secs_f32());
                display.render_text(Vec2::new(4., 2.), &text);
            }
            State::GameOverFreeze { won, elapsed } => {
                let text = if won {
                    str_format!(str12, "WON")
                } else {
                    str_format!(str12, "FAILURE")
                };
                const FAILURE_RECT: Rect =
                    Rect::new_center(Vec2::new(CENTER_X, 20.), Vec2::new(WIDTH_F32, 20.));
                display.render_rect_solid(FAILURE_RECT, false);
                display.render_text_complex(
                    FAILURE_RECT.pos - Vec2::new(0., 8.),
                    &text,
                    ComplexRenderOption::new().with_white().with_center(),
                );

                let text = str_format!(str12, "ELAPSED");
                display.render_text_complex(
                    FAILURE_RECT.pos,
                    &text,
                    ComplexRenderOption::new().with_white().with_center(),
                );

                let text = str_format!(str12, "{:.2}", elapsed.as_secs_f32());
                display.render_text_complex(
                    FAILURE_RECT.pos + Vec2::new(0., 9.),
                    &text,
                    ComplexRenderOption::new().with_white().with_center(),
                );
            }
        }
    }
}
