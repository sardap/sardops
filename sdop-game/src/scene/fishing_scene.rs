use core::time::Duration;

use fixedstr::{str_format, str32};
use glam::{IVec2, Vec2};

use crate::{
    Button,
    anime::{HasAnime, MaskedAnimeSprite},
    assets::{
        self, FRAMES_FISHING_POND_MOVING, IMAGE_FISHING_POND_LINE_MASK_0, IMAGE_FISHING_POND_STILL,
    },
    display::{
        CENTER_X, CENTER_X_I32, ComplexRenderOption, GameDisplay, HEIGHT_F32, WIDTH_F32, WIDTH_I32,
    },
    fonts::FONT_VARIABLE_SMALL,
    geo::RectIVec2,
    input::random_button,
    items::{FISHING_ITEM_ODDS, ItemKind, pick_item_from_set},
    money::Money,
    pet::{definition::PetAnimationSet, render::PetRender},
    scene::{RenderArgs, Scene, SceneOutput, SceneTickArgs},
    sounds::{SONG_FAN_FARE, SONG_FISHING_IDLE, SONG_FISHING_PULLING, SONG_LOST, SongPlayOptions},
    sprite::BasicAnimeSprite,
};

struct HitSeqEntry {
    button: Button,
    elasped: Duration,
    hit: bool,
}

type HitSeq = [Option<HitSeqEntry>; 4];

enum State {
    Waiting { duration: Duration },
    Pulling,
    PullOut,
    FanFare,
}

#[derive(PartialEq, Eq)]
enum Winning {
    Garbage,
    Item(ItemKind),
    Money(Money),
}

pub struct FishingScene {
    state: State,
    state_elasped: Duration,
    seq: HitSeq,
    pet_render: PetRender,
    fishing_pond_moving: BasicAnimeSprite,
    fishing_line: MaskedAnimeSprite,
    fishing_line_pulled: MaskedAnimeSprite,
    fishing_line_item: MaskedAnimeSprite,
    fishing_line_nothing: MaskedAnimeSprite,
    winning: Option<Winning>,
    garbage: MaskedAnimeSprite,
}

const FISHING_POND_POS_CENTER: Vec2 = Vec2::new(
    WIDTH_F32 - IMAGE_FISHING_POND_STILL.size.x as f32 / 2.,
    70. - IMAGE_FISHING_POND_STILL.size.y as f32 / 2.,
);

const FISHING_ROD_POS_CENTER: Vec2 = Vec2::new(
    IMAGE_FISHING_POND_LINE_MASK_0.size.x as f32 / 2.,
    70. - IMAGE_FISHING_POND_LINE_MASK_0.size.y as f32 / 2.,
);

const PULL_TIME: Duration = Duration::from_millis(5000);

impl Default for FishingScene {
    fn default() -> Self {
        Self::new()
    }
}

impl FishingScene {
    pub fn new() -> Self {
        let fishing_line = MaskedAnimeSprite::new(
            FISHING_ROD_POS_CENTER,
            &assets::FRAMES_FISHING_POND_LINE,
            &assets::FRAMES_FISHING_POND_LINE_MASK,
        );

        let fishing_line_pulled = MaskedAnimeSprite::new(
            FISHING_ROD_POS_CENTER,
            &assets::FRAMES_FISHING_POND_LINE_PULLED,
            &assets::FRAMES_FISHING_POND_LINE_PULLED_MASK,
        );

        Self {
            state: State::Waiting {
                duration: Duration::ZERO,
            },
            fishing_pond_moving: BasicAnimeSprite::new(
                FISHING_POND_POS_CENTER,
                &FRAMES_FISHING_POND_MOVING,
            ),
            seq: [None, None, None, None],
            state_elasped: Duration::ZERO,
            pet_render: PetRender::default(),
            fishing_line,
            fishing_line_pulled,
            fishing_line_item: MaskedAnimeSprite::new(
                FISHING_ROD_POS_CENTER,
                &assets::FRAMES_FISHING_POND_LINE_PULLOUT_ITEM,
                &assets::FRAMES_FISHING_POND_LINE_PULLOUT_ITEM_MASK,
            ),
            fishing_line_nothing: MaskedAnimeSprite::new(
                FISHING_ROD_POS_CENTER,
                &assets::FRAMES_FISHING_POND_LINE_PULLOUT_NOTHIN,
                &assets::FRAMES_FISHING_POND_LINE_PULLOUT_NOTHIN_MASK,
            ),
            winning: None,
            garbage: MaskedAnimeSprite::new(
                Vec2::new(CENTER_X, 20.),
                &assets::FRAMES_GARBAGE,
                &assets::FRAMES_GARBAGE_MASK,
            ),
        }
    }
}

impl Scene for FishingScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.pet_render.set_def_id(args.game_ctx.pet.def_id);
        self.pet_render.pos = Vec2::new(
            (self.pet_render.anime.current_frame().size.x as f32 / 2.) + 2.,
            40.,
        );

        let rng = args.game_ctx.rng.f32();

        let mut current = Duration::from_millis(500);
        current += Duration::from_millis(args.game_ctx.rng.u64(0..2000));
        self.seq[0] = Some(HitSeqEntry {
            button: random_button(&mut args.game_ctx.rng),
            elasped: current,
            hit: false,
        });

        if rng < 0.7 {
            current += Duration::from_millis(args.game_ctx.rng.u64(500..1500));
            self.seq[1] = Some(HitSeqEntry {
                button: random_button(&mut args.game_ctx.rng),
                elasped: current,
                hit: false,
            });
        }

        if rng < 0.5 {
            current += Duration::from_millis(args.game_ctx.rng.u64(500..1500));
            self.seq[2] = Some(HitSeqEntry {
                button: random_button(&mut args.game_ctx.rng),
                elasped: current,
                hit: false,
            });
        }

        self.state = State::Waiting {
            duration: Duration::from_millis(args.game_ctx.rng.u64(1000..10000)),
        }
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.sound_system.clear_song();
    }

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        self.fishing_line.anime().tick(args.delta);
        self.fishing_line_pulled.anime().tick(args.delta);
        self.fishing_pond_moving.anime().tick(args.delta);
        self.pet_render.tick(args.delta);
        self.garbage.anime().tick(args.delta);
        self.state_elasped += args.delta;

        match self.state {
            State::Waiting { duration } => {
                if !args.game_ctx.sound_system.get_playing() {
                    args.game_ctx
                        .sound_system
                        .push_song(SONG_FISHING_IDLE, SongPlayOptions::new().with_music());
                }

                if self.state_elasped > duration {
                    self.state_elasped = Duration::ZERO;
                    args.game_ctx.sound_system.clear_song();
                    if args.game_ctx.rng.f32() < 0.2 {
                        self.pet_render.set_animation(PetAnimationSet::Sad);
                        self.state = State::PullOut;
                    } else {
                        self.pet_render.set_animation(PetAnimationSet::Happy);
                        self.state = State::Pulling;
                    }
                }
            }
            State::Pulling => {
                if !args.game_ctx.sound_system.get_playing() {
                    args.game_ctx
                        .sound_system
                        .push_song(SONG_FISHING_PULLING, SongPlayOptions::new().with_music());
                }

                if args.input.any_pressed() {
                    let current_percent =
                        self.state_elasped.as_secs_f32() / PULL_TIME.as_secs_f32() * 100.;

                    for entry in self.seq.iter_mut().flatten() {
                        let entry_percent =
                            entry.elasped.as_secs_f32() / PULL_TIME.as_secs_f32() * 100.;

                        let delta = (entry_percent - current_percent).abs();
                        if delta < 5. {
                            entry.hit = true;
                        }
                    }
                }

                if self.state_elasped > PULL_TIME {
                    let hits = self
                        .seq
                        .iter()
                        .filter(|i| i.is_some() && i.as_ref().unwrap().hit)
                        .count() as f32;
                    let possible = self.seq.iter().filter(|i| i.is_some()).count() as f32;
                    let percent = if hits == 0. { 0.05 } else { possible / hits };
                    if args.game_ctx.rng.f32() < percent {
                        let percent = args.game_ctx.rng.f32();
                        self.winning = Some(if percent < 0.3 {
                            Winning::Garbage
                        } else if percent < 0.6 {
                            Winning::Money(args.game_ctx.rng.i32(100..5000))
                        } else {
                            Winning::Item(pick_item_from_set(
                                args.game_ctx.rng.f32(),
                                FISHING_ITEM_ODDS,
                            ))
                        });
                    } else {
                        self.winning = None;
                    }

                    self.pet_render.set_animation(
                        if self.winning.is_some() && self.winning != Some(Winning::Garbage) {
                            PetAnimationSet::Happy
                        } else {
                            PetAnimationSet::Sad
                        },
                    );

                    self.state_elasped = Duration::ZERO;
                    self.state = State::PullOut;
                }
            }
            State::PullOut => {
                let anime = if self.winning.is_some() {
                    self.fishing_line_item.anime()
                } else {
                    self.fishing_line_nothing.anime()
                };

                anime.tick(args.delta);

                if self.state_elasped > anime.total_duration() {
                    self.pet_render.pos = Vec2::new(
                        CENTER_X,
                        HEIGHT_F32 - self.pet_render.anime.current_frame().size.y as f32 / 2. - 20.,
                    );
                    self.state_elasped = Duration::ZERO;
                    self.state = State::FanFare;

                    args.game_ctx.sound_system.push_song(
                        if self.winning.is_some() && self.winning != Some(Winning::Garbage) {
                            SONG_FAN_FARE
                        } else {
                            SONG_LOST
                        },
                        SongPlayOptions::new().with_effect(),
                    );
                }
            }
            State::FanFare => {
                if self.state_elasped > Duration::from_secs(5) {
                    if let Some(winnigs) = self.winning.take() {
                        match winnigs {
                            Winning::Garbage => {}
                            Winning::Item(item_kind) => {
                                args.game_ctx.inventory.add_item(item_kind, 1);
                            }
                            Winning::Money(money) => args.game_ctx.money += money,
                        }
                    }

                    output.set_home();
                    return;
                }
            }
        }
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        match self.state {
            State::Waiting { duration: _ } => {
                display.render_sprite(&self.pet_render);

                display.render_image_complex(
                    FISHING_POND_POS_CENTER.x as i32,
                    FISHING_POND_POS_CENTER.y as i32,
                    &assets::IMAGE_FISHING_POND_STILL,
                    ComplexRenderOption::new().with_center().with_white(),
                );

                display.render_sprite(&self.fishing_line);
            }
            State::Pulling => {
                display.render_sprite(&self.pet_render);

                display.render_sprite(&self.fishing_pond_moving);

                display.render_sprite(&self.fishing_line_pulled);

                const HIT_RECTNAGLE: RectIVec2 =
                    RectIVec2::new_top_left(IVec2::new(10, 90), IVec2::new(WIDTH_I32 - 20, 20));

                display.render_rect_outline(&HIT_RECTNAGLE, true);

                let percent_of_hit = self.state_elasped.as_secs_f32() / PULL_TIME.as_secs_f32();
                let position_rec = RectIVec2::new_top_left(
                    HIT_RECTNAGLE.pos_top_left()
                        + IVec2::new((HIT_RECTNAGLE.size.x as f32 * percent_of_hit) as i32 - 1, 0),
                    IVec2::new(3, 20),
                );
                display.render_rect_outline(&position_rec, true);

                for (i, entry) in self.seq.iter().enumerate() {
                    if let Some(entry) = entry {
                        let percent_of_hit = entry.elasped.as_secs_f32() / PULL_TIME.as_secs_f32();
                        let position_rec = RectIVec2::new_top_left(
                            HIT_RECTNAGLE.pos_top_left()
                                + IVec2::new(
                                    (HIT_RECTNAGLE.size.x as f32 * percent_of_hit) as i32,
                                    if i % 2 == 0 { 0 } else { 10 },
                                ),
                            IVec2::new(2, 10),
                        );
                        if entry.hit {
                            display.render_rect_outline_dashed(&position_rec, true, 1);
                        } else {
                            display.render_rect_outline(&position_rec, true);
                        }
                    }
                }

                log::info!("{}", percent_of_hit);
            }
            State::PullOut => {
                display.render_sprite(&self.pet_render);

                display.render_sprite(&self.fishing_pond_moving);

                if self.winning.is_some() {
                    display.render_sprite(&self.fishing_line_item);
                } else {
                    display.render_sprite(&self.fishing_line_nothing);
                }
            }
            State::FanFare => {
                if let Some(winnings) = &self.winning {
                    match winnings {
                        Winning::Garbage => {
                            display.render_sprite(&self.garbage);
                        }
                        Winning::Item(item_kind) => {
                            display.render_image_complex(
                                (item_kind.image().size.x as f32 / 2.) as i32,
                                20,
                                item_kind.image(),
                                ComplexRenderOption::new().with_white(),
                            );
                        }
                        Winning::Money(money) => {
                            let total = str_format!(str32, "${}", args.game_ctx.money);
                            display.render_text_complex(
                                &IVec2::new(10, 20),
                                &total,
                                ComplexRenderOption::new().with_white(),
                            );
                            let winnings = str_format!(str32, "+${}", money);
                            display.render_text_complex(
                                &IVec2::new(10, 30),
                                &winnings,
                                ComplexRenderOption::new().with_white(),
                            );
                        }
                    }
                } else {
                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, 10),
                        "NOTHING",
                        ComplexRenderOption::new()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL)
                            .with_center(),
                    );
                }

                display.render_sprite(&self.pet_render);
            }
        }
    }
}
