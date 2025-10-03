use core::time::Duration;

use chrono::{NaiveTime, TimeDelta};
use fixedstr::str_format;
use glam::Vec2;

use crate::{
    anime::{Anime, HasAnime, MaskedAnimeRender},
    assets::{self, Image},
    clock::AnalogueRenderClock,
    death::{DeathCause, GraveStone},
    display::{
        CENTER_VEC, CENTER_X, CENTER_Y, ComplexRenderOption, GameDisplay, HEIGHT_F32, WIDTH_F32,
    },
    geo::Rect,
    pet::{
        definition::{PET_BABIES, PetAnimationSet, PetDefinitionId},
        record::PetRecord,
        render::PetRender,
    },
    scene::{RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs, new_pet_scene::NewPetScene},
    sounds::{SONG_DEATH, SongPlayOptions},
    sprite::{BasicAnimeSprite, Snowflake, Sprite},
    stomach::StomachRender,
};

enum State {
    Intro,
    Speific,
    Tombstone,
}

struct Lighting {
    clouds: BasicAnimeSprite,
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            clouds: BasicAnimeSprite::new(
                Vec2::new(CENTER_X, assets::IMAGE_CLOUDS_0.size.y as f32 / 2.),
                &assets::FRAMES_CLOUDS,
            ),
        }
    }
}

struct Starvation {
    stomach_x_offset: f32,
    moving_left: bool,
}

impl Default for Starvation {
    fn default() -> Self {
        Self {
            stomach_x_offset: 0.,
            moving_left: false,
        }
    }
}

struct OldAge {
    clock: AnalogueRenderClock,
    time: NaiveTime,
    speed_mul: i64,
}

impl Default for OldAge {
    fn default() -> Self {
        Self {
            clock: AnalogueRenderClock::new(
                crate::clock::AnalogueClockKind::Clock41,
                CENTER_VEC,
                NaiveTime::default(),
            )
            .without_second_hand(),
            time: NaiveTime::default(),
            speed_mul: 1200,
        }
    }
}

#[derive(Default)]
struct ToxicShock {
    around_poops: [BasicAnimeSprite; 30],
    pet_poops: [BasicAnimeSprite; 10],
}

const POOP_SPAWN_DURATION: Duration = Duration::from_secs(7);
const PET_POOP_SPAWN_DURATION: Duration = Duration::from_secs(7);

struct Illness {
    skull: MaskedAnimeRender,
}

impl Default for Illness {
    fn default() -> Self {
        Self {
            skull: MaskedAnimeRender::new(
                CENTER_VEC,
                &assets::FRAMES_SKULL,
                &assets::FRAMES_SKULL_MASK,
            ),
        }
    }
}

const ILLNESS_RUN_TIME: Duration = Duration::from_secs(7);

struct Hypothermia {
    snow_flakes: [Snowflake; 50],
    shaking_duration: Duration,
    shaking_left: bool,
}

impl Default for Hypothermia {
    fn default() -> Self {
        Self {
            snow_flakes: [Snowflake::default(); 50],
            shaking_duration: Default::default(),
            shaking_left: Default::default(),
        }
    }
}

pub struct DeathScene {
    cause: DeathCause,
    state: State,
    state_elapsed: Duration,
    lighting: Lighting,
    starving: Starvation,
    old_age: OldAge,
    pet_render: PetRender,
    toxic_shock: ToxicShock,
    grave_stone: GraveStone,
    ilness: Illness,
    hypothermia: Hypothermia,
}

impl DeathScene {
    pub fn new(cause: DeathCause, pet_id: PetDefinitionId) -> Self {
        Self {
            cause,
            state: State::Intro,
            state_elapsed: Duration::ZERO,
            lighting: Lighting::default(),
            starving: Starvation::default(),
            old_age: Default::default(),
            pet_render: PetRender::new(pet_id),
            toxic_shock: Default::default(),
            grave_stone: GraveStone::default(),
            ilness: Illness::default(),
            hypothermia: Default::default(),
        }
    }
}

const AREA: Rect = Rect::new_top_left(Vec2::ZERO, Vec2::new(WIDTH_F32, HEIGHT_F32));

impl Scene for DeathScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.pet_render.pos = CENTER_VEC;
        self.grave_stone = GraveStone::new(
            Vec2::new(
                CENTER_X,
                HEIGHT_F32 - assets::IMAGE_GRAVESTONE.size.y as f32 / 2. - 5.,
            ),
            str_format!(fixedstr::str12, "{}", args.game_ctx.pet.name),
            self.pet_render.def_id(),
            args.game_ctx.pet.born.inner().date(),
            args.timestamp.inner().date(),
            self.cause,
        );

        if self.cause == DeathCause::ToxicShock {
            let pet_rect = Rect::new_center(
                self.pet_render.pos,
                self.pet_render.anime.current_frame().size.as_vec2(),
            )
            .grow(5.);
            for poop in &mut self.toxic_shock.around_poops {
                poop.anime = Anime::new(&assets::FRAMES_POOP);
                poop.anime.set_random_frame(&mut args.game_ctx.rng);
                loop {
                    poop.pos = AREA.random_point_inside(&mut args.game_ctx.rng);
                    if !pet_rect.point_inside(&poop.pos) {
                        break;
                    }
                }
            }

            for poop in &mut self.toxic_shock.pet_poops {
                poop.anime = Anime::new(&assets::FRAMES_POOP);
                poop.anime.set_random_frame(&mut args.game_ctx.rng);
                poop.pos = pet_rect.random_point_inside(&mut args.game_ctx.rng);
            }
        } else if self.cause == DeathCause::Hypothermia {
            for flake in &mut self.hypothermia.snow_flakes {
                flake.reset(true, &mut args.game_ctx.rng);
                flake.dir.y = args.game_ctx.rng.i32(5..15) as f32;
            }
        }

        args.game_ctx
            .sound_system
            .push_song(SONG_DEATH, SongPlayOptions::new().with_essential());
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.sound_system.clear_song();
        args.game_ctx.poops = Default::default();
        args.game_ctx.pet_records.add(PetRecord::from_pet_instance(
            &args.game_ctx.pet,
            args.timestamp,
            self.cause,
        ));
    }

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        self.state_elapsed += args.delta;

        self.pet_render.tick(args.delta);

        match self.state {
            State::Intro => {
                if self.state_elapsed > Duration::from_secs(3) {
                    self.state = State::Speific;
                    self.state_elapsed = Duration::ZERO;
                }
            }
            State::Speific => match self.cause {
                DeathCause::LightingStrike => {
                    self.pet_render.set_animation(PetAnimationSet::Sad);
                    self.lighting.clouds.anime().tick(args.delta);
                    if self.lighting.clouds.anime.frames() == assets::FRAMES_CLOUDS
                        && self.lighting.clouds.anime.current_frame_index()
                            == self.lighting.clouds.anime.frames().len() - 1
                    {
                        self.lighting.clouds = BasicAnimeSprite::new(
                            self.lighting.clouds.pos,
                            &assets::FRAMES_CLOUDS_RUBBING,
                        );
                        log::info!("{:?}", self.state_elapsed.as_millis());
                    }

                    if self.state_elapsed > Duration::from_millis(5500) {
                        self.state = State::Tombstone;
                        self.state_elapsed = Duration::ZERO;
                    }
                }
                DeathCause::Starvation => {
                    self.pet_render.set_animation(PetAnimationSet::Sad);

                    const SPEED: f32 = 10.;

                    let change = (SPEED * args.delta.as_secs_f32())
                        * (1. + self.state_elapsed.as_secs_f32());

                    self.starving.moving_left = if self.starving.stomach_x_offset + change > 2. {
                        true
                    } else if self.starving.stomach_x_offset - change < -2. {
                        false
                    } else {
                        self.starving.moving_left
                    };

                    self.starving.stomach_x_offset += if self.starving.moving_left {
                        -change
                    } else {
                        change
                    };

                    if self.state_elapsed > Duration::from_millis(5000) {
                        self.state = State::Tombstone;
                        self.state_elapsed = Duration::ZERO;
                    }
                }
                DeathCause::OldAge => {
                    self.pet_render.set_animation(PetAnimationSet::Sad);
                    self.pet_render.pos = CENTER_VEC + Vec2::new(0., 20.);

                    self.old_age.clock.pos = CENTER_VEC - Vec2::new(0., 20.);

                    self.old_age.speed_mul += (args.delta.as_millis_f32() * 3.) as i64;

                    (self.old_age.time, _) =
                        self.old_age
                            .time
                            .overflowing_add_signed(TimeDelta::microseconds(
                                args.delta.as_micros() as i64 * self.old_age.speed_mul,
                            ));

                    self.old_age.clock.update_time(&self.old_age.time);

                    if self.state_elapsed > Duration::from_secs(10) {
                        self.state = State::Tombstone;
                        self.state_elapsed = Duration::ZERO;
                    }
                }
                DeathCause::ToxicShock => {
                    self.pet_render.set_animation(PetAnimationSet::Sad);

                    for poop in &mut self.toxic_shock.around_poops {
                        poop.anime.tick(args.delta);
                    }

                    if self.state_elapsed > POOP_SPAWN_DURATION + PET_POOP_SPAWN_DURATION {
                        self.state = State::Tombstone;
                        self.state_elapsed = Duration::ZERO;
                    }
                }
                DeathCause::Illness => {
                    self.pet_render.set_animation(PetAnimationSet::Sad);
                    self.ilness.skull.anime().tick(args.delta);

                    const SPEED: f32 = 10.;

                    if self.state_elapsed > Duration::from_secs(3) {
                        self.pet_render.pos.y += SPEED * args.delta.as_secs_f32();
                    }

                    self.ilness.skull.pos = self.pet_render.pos
                        - Vec2::new(
                            0.,
                            self.pet_render.anime.current_frame().size.y as f32 / 2.
                                + assets::IMAGE_SKULL_0.size.y as f32 / 2.
                                + 5.,
                        );

                    if self.state_elapsed > ILLNESS_RUN_TIME {
                        self.state = State::Tombstone;
                        self.state_elapsed = Duration::ZERO;
                    }
                }
                DeathCause::Hypothermia => {
                    self.pet_render.set_animation(PetAnimationSet::Sad);

                    for flake in &mut self.hypothermia.snow_flakes {
                        flake.pos += flake.dir * args.delta.as_secs_f32();

                        if flake.pos.x > WIDTH_F32 + Snowflake::size().x
                            || flake.pos.x < -Snowflake::size().x
                            || flake.pos.y > HEIGHT_F32 + Snowflake::size().y
                        {
                            flake.reset(false, &mut args.game_ctx.rng);
                        }
                    }

                    self.hypothermia.shaking_duration += args.delta;
                    if self.hypothermia.shaking_duration > Duration::from_millis(200) {
                        self.hypothermia.shaking_duration = Duration::ZERO;
                        self.hypothermia.shaking_left = !self.hypothermia.shaking_left;
                    }

                    const SHAKE_AMOUNT_X: f32 = 10.;

                    self.pet_render.pos.x += if self.hypothermia.shaking_left {
                        SHAKE_AMOUNT_X
                    } else {
                        -SHAKE_AMOUNT_X
                    } * args.delta.as_secs_f32();

                    if self.state_elapsed > Duration::from_secs(20) {
                        self.state = State::Tombstone;
                        self.state_elapsed = Duration::ZERO;
                    }
                }
                DeathCause::Leaving => {}
            },
            State::Tombstone => {
                self.pet_render.pos =
                    Vec2::new(CENTER_X, assets::IMAGE_GHOST_CLOUD.size.y as f32 / 2.);
                self.pet_render.set_animation(PetAnimationSet::Sad);

                if args.input.any_pressed() {
                    return SceneOutput::new(SceneEnum::NewPet(NewPetScene::new(
                        args.game_ctx.rng.choice(PET_BABIES).unwrap(),
                        false,
                        None,
                        None,
                    )));
                }
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        match self.state {
            State::Intro => {
                display.render_sprite(&self.pet_render);
            }
            State::Speific => match self.cause {
                DeathCause::LightingStrike => {
                    display.render_sprite(&self.lighting.clouds);
                    display.render_sprite(&self.pet_render);

                    if self.state_elapsed > Duration::from_millis(5000) {
                        display.render_image_complex(
                            CENTER_X as i32 - 7,
                            self.pet_render.pos.y as i32,
                            &assets::IMAGE_LIGHTING_ONE,
                            ComplexRenderOption::new().with_white().with_bottom_left(),
                        );
                        display.invert();
                    }
                }
                DeathCause::Starvation => {
                    display.render_sprite(&self.pet_render);

                    display.render_image_complex(
                        (CENTER_X + self.starving.stomach_x_offset) as i32,
                        (CENTER_Y
                            - self.pet_render.image().size_vec2().y / 2.
                            - StomachRender::size().y / 2.) as i32,
                        &assets::IMAGE_STOMACH,
                        ComplexRenderOption::new().with_white().with_center(),
                    );

                    if self.state_elapsed > Duration::from_millis(4700) {
                        display.invert();
                    }
                }
                DeathCause::OldAge => {
                    display.render_complex(&self.old_age.clock);

                    display.render_sprite(&self.pet_render);
                }
                DeathCause::ToxicShock => {
                    display.render_sprite(&self.pet_render);

                    log::info!(
                        "{}",
                        self.state_elapsed.as_secs_f32() / POOP_SPAWN_DURATION.as_secs_f32()
                    );

                    let cutoff = (self.toxic_shock.around_poops.len() as f32
                        * (self.state_elapsed.as_secs_f32() / POOP_SPAWN_DURATION.as_secs_f32()))
                        as usize;

                    for (i, poop) in self.toxic_shock.around_poops.iter().enumerate() {
                        display.render_sprite(poop);

                        if i > cutoff {
                            break;
                        }
                    }

                    let cutoff = (self.toxic_shock.around_poops.len() as f32
                        * (self.state_elapsed.as_secs_f32() / POOP_SPAWN_DURATION.as_secs_f32()))
                        as usize;

                    for (i, poop) in self.toxic_shock.around_poops.iter().enumerate() {
                        if i >= cutoff {
                            break;
                        }

                        display.render_sprite(poop);
                    }

                    if self.state_elapsed > POOP_SPAWN_DURATION {
                        let cutoff = (self.toxic_shock.pet_poops.len() as f32
                            * (self.state_elapsed.as_secs_f32()
                                / (PET_POOP_SPAWN_DURATION + POOP_SPAWN_DURATION).as_secs_f32()))
                            as usize;

                        for (i, poop) in self.toxic_shock.pet_poops.iter().enumerate() {
                            if i >= cutoff {
                                break;
                            }

                            display.render_sprite(poop);
                        }
                    }
                }
                DeathCause::Illness => {
                    display.render_complex(&self.ilness.skull);
                    display.render_sprite(&self.pet_render);

                    const DOORS_CLOSE_TIME: Duration = Duration::from_secs(6);

                    let x_percent = (self.state_elapsed.as_millis_f32()
                        / DOORS_CLOSE_TIME.as_millis_f32())
                    .min(1.)
                        * 0.5;

                    let left_door = Rect::new_top_left(
                        Vec2::new(0., 0.),
                        Vec2::new(WIDTH_F32 * x_percent, HEIGHT_F32),
                    );
                    display.render_rect_solid(left_door, false);

                    let right_door = Rect::new_top_left(
                        Vec2::new(WIDTH_F32 - WIDTH_F32 * x_percent, 0.),
                        Vec2::new(WIDTH_F32, HEIGHT_F32),
                    );

                    display.render_rect_solid(right_door, false);
                }
                DeathCause::Hypothermia => {
                    display.render_sprite(&self.pet_render);

                    for flake in &self.hypothermia.snow_flakes {
                        display.render_sprite(flake);
                    }
                }
                DeathCause::Leaving => {}
            },
            State::Tombstone => {
                display.render_image_top_left(0, 0, &assets::IMAGE_GHOST_CLOUD);

                display.render_sprite(&self.pet_render);
                display.render_complex(&self.grave_stone);
            }
        }
    }
}
