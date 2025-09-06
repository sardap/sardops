use core::time::Duration;

use glam::Vec2;

use crate::{
    assets,
    display::{GameDisplay, CENTER_VEC},
    pet::{definition::PetDefinitionId, render::PetRender},
    scene::{home_scene::HomeScene, RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs},
    sprite::Sprite,
    Timestamp,
};

struct ExpandingCircle {
    size: i32,
    speed: Duration,
    duration: Duration,
}

impl Default for ExpandingCircle {
    fn default() -> Self {
        Self {
            size: 0,
            speed: Duration::from_millis(25),
            duration: Duration::ZERO,
        }
    }
}

#[derive(Clone, Copy)]
struct Star {
    dir: Vec2,
    pos: Vec2,
}

impl Sprite for Star {
    fn pos<'a>(&'a self) -> &'a Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl crate::assets::Image {
        &assets::IMAGE_EVOLVE_STAR
    }
}

impl Default for Star {
    fn default() -> Self {
        Self {
            dir: Default::default(),
            pos: CENTER_VEC,
        }
    }
}

enum State {
    Flashing,
    Circles,
    White,
    Complete,
}

pub struct EvolveScene {
    from_pet_render: PetRender,
    to_pet_render: PetRender,
    circles: [Option<ExpandingCircle>; 5],
    circle_spawn_timer: Duration,
    show_from: bool,
    flash_timer: Duration,
    flash_increase: Duration,
    flash_speed: f32,
    start_time: Timestamp,
    invert: bool,
    invert_timer: Duration,
    state: State,
    stars: [Star; 30],
}

impl EvolveScene {
    pub fn new(from_id: PetDefinitionId, to_id: PetDefinitionId) -> Self {
        Self {
            from_pet_render: PetRender::new(from_id),
            to_pet_render: PetRender::new(to_id),
            circles: Default::default(),
            circle_spawn_timer: Duration::ZERO,
            show_from: false,
            flash_timer: Duration::ZERO,
            flash_increase: Duration::ZERO,
            flash_speed: 1.,
            invert: false,
            invert_timer: Duration::ZERO,
            start_time: Timestamp::default(),
            state: State::Flashing,
            stars: [Default::default(); 30],
        }
    }
}

impl Scene for EvolveScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.to_pet_render.pos = CENTER_VEC;
        self.from_pet_render
            .set_animation(crate::pet::definition::PetAnimationSet::Sad);
        self.from_pet_render.pos = CENTER_VEC;

        self.start_time = args.timestamp;

        let rng = &mut args.game_ctx.rng;

        for star in &mut self.stars {
            let x = (rng.i32(-1000..1000) as f32) / 1000.;
            let y = rng.i32(-1000..1000) as f32 / 1000.;
            star.dir = Vec2::new(x, y)
        }
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.pet.evolve(self.to_pet_render.def_id());
    }

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        self.to_pet_render.tick(args.delta.mul_f32(2.));
        self.from_pet_render.tick(args.delta.mul_f32(2.));

        self.flash_increase += args.delta;
        if self.flash_increase > Duration::from_secs(5) {
            self.flash_speed = (self.flash_speed - 0.25).max(0.1);
            self.flash_increase = Duration::ZERO;
        }

        self.flash_timer += args.delta;
        if self.flash_timer > Duration::from_millis(300).mul_f32(self.flash_speed) {
            self.show_from = !self.show_from;
            self.flash_timer = Duration::ZERO;
        }

        match self.state {
            State::Flashing => {
                if args.timestamp - self.start_time > Duration::from_secs(3) {
                    self.state = State::Circles;
                }
            }
            State::Circles => {
                if args.timestamp - self.start_time > Duration::from_secs(10) {
                    self.state = State::White;
                }

                self.invert_timer += args.delta;
                if !self.invert && self.invert_timer > Duration::from_secs_f32(1.5) {
                    self.invert = true;
                    self.invert_timer = Duration::ZERO;
                } else if self.invert && self.invert_timer > Duration::from_millis(500) {
                    self.invert = false;
                    self.invert_timer = Duration::ZERO;
                }

                self.circle_spawn_timer += args.delta;
                if self.circle_spawn_timer > Duration::from_millis(300) {
                    for circle in &mut self.circles {
                        if circle.is_none() {
                            *circle = Some(ExpandingCircle::default());
                            break;
                        }
                    }
                    self.circle_spawn_timer = Duration::ZERO;
                }

                for circle in &mut self.circles {
                    if let Some(circle) = circle {
                        circle.duration += args.delta;
                        if circle.duration > circle.speed {
                            circle.duration = Duration::ZERO;
                            circle.size += 1;
                            if circle.size > 75 {
                                circle.size = 0;
                            }
                            log::info!("{}", circle.size);
                        }
                    }
                }
            }
            State::White => {
                if args.timestamp - self.start_time > Duration::from_secs(12) {
                    self.state = State::Complete;
                }
            }
            State::Complete => {
                self.to_pet_render
                    .set_animation(crate::pet::definition::PetAnimationSet::Happy);
                if args.timestamp - self.start_time > Duration::from_secs(20) {
                    return SceneOutput::new(SceneEnum::Home(HomeScene::new()));
                }

                const SPEEDS: [f32; 5] = [20.3, 10.5, 60.7, 50.2, 30.];
                for (i, star) in self.stars.iter_mut().enumerate() {
                    let speed = if args.timestamp - self.start_time < Duration::from_secs(15) {
                        40.
                    } else {
                        SPEEDS[i % SPEEDS.len()]
                    };

                    star.pos += star.dir * speed * args.delta.as_secs_f32();
                }
            }
        }
        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        if matches!(self.state, State::Flashing) || matches!(self.state, State::Circles) {
            display.render_sprite(if self.show_from {
                &self.from_pet_render
            } else {
                &self.to_pet_render
            });
        }

        match self.state {
            State::Flashing => {}
            State::Circles => {
                // display.render_sprite(&self.to_pet_render);
                for circle in &self.circles {
                    if let Some(circle) = circle {
                        display.render_circle(CENTER_VEC, circle.size, true);
                    }
                }

                if self.invert {
                    display.invert();
                }
            }
            State::White => {
                display.invert();
            }
            State::Complete => {
                display.render_sprite(&self.to_pet_render);

                for star in self.stars {
                    display.render_sprite(&star);
                }
            }
        }
        // display.invert();
    }
}
