use core::time::Duration;

use chrono::{Datelike, Days, NaiveDate, NaiveDateTime, TimeDelta, Timelike};
use glam::{IVec2, Vec2};

use crate::{
    anime::HasAnime,
    assets::{self},
    display::{CENTER_VEC, CENTER_X_I32, ComplexRenderOption, GameDisplay},
    fonts::FONT_VARIABLE_SMALL,
    game_consts::SHOP_OPEN_TIMES,
    geo::RectVec2,
    particle_system::{ParticleSpawnArgs, ParticleSystem, ParticleTemplate, TemplateCullTatic},
    scene::{RenderArgs, Scene, SceneOutput, SceneTickArgs},
    shop::ShopItemSet,
    sounds::{self, SONG_SHOP, SONG_SHOP_CLOSED, SongPlayOptions},
    sprite::BasicAnimeSprite,
};

const SIGN_SHAKE_DURATION: Duration = Duration::from_millis(500);

enum State {
    Closed,
    ShopKeeper,
    Selected(usize),
}

pub struct ShopScene {
    for_sale: ShopItemSet,
    state: State,
    shop_keeper: BasicAnimeSprite,
    closed_sign: BasicAnimeSprite,
    sign_shake_remaining: Duration,
    particle_system: ParticleSystem<50, 1>,
}

impl Default for ShopScene {
    fn default() -> Self {
        Self::new()
    }
}

impl ShopScene {
    pub fn new() -> Self {
        Self {
            for_sale: Default::default(),
            state: State::ShopKeeper,
            shop_keeper: BasicAnimeSprite::new(
                CENTER_VEC + Vec2::new(0., -7.),
                &assets::FRAMES_SHOP_KEEPER,
            ),
            closed_sign: BasicAnimeSprite::new(
                CENTER_VEC + Vec2::new(0., -7.),
                &assets::FRAMES_SHOP_SIGN_CLOSED,
            ),
            sign_shake_remaining: Duration::ZERO,
            particle_system: ParticleSystem::default(),
        }
    }

    pub fn item_count(&self) -> usize {
        self.for_sale.iter().filter(|i| i.is_some()).count()
    }
}

impl Scene for ShopScene {
    fn setup(&mut self, args: &mut SceneTickArgs) {
        self.for_sale = args.game_ctx.shop.item_set(args.timestamp);

        self.shop_keeper
            .anime()
            .set_random_frame(&mut args.game_ctx.rng);

        let opening_times = SHOP_OPEN_TIMES[args.timestamp.inner().date().weekday() as usize];
        if args.timestamp.inner().time() < opening_times[0]
            || args.timestamp.inner().time() > opening_times[1]
            || args.game_ctx.speical_days.is_non_trading_day()
        {
            // self.state = State::Closed;
        }
    }

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.sound_system.clear_song();
    }

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        self.shop_keeper.anime().tick(args.delta);

        self.sign_shake_remaining = self
            .sign_shake_remaining
            .checked_sub(args.delta)
            .unwrap_or_default();

        self.particle_system.tick(&mut ParticleSpawnArgs::new(
            args.delta,
            &mut args.game_ctx.rng,
            &Vec2::default(),
        ));

        match self.state {
            State::Closed => {
                self.closed_sign.anime().tick(args.delta);
                if args.input.any_pressed() {
                    output.set_home();
                    return;
                }
                let opening_times =
                    SHOP_OPEN_TIMES[args.timestamp.inner().date().weekday() as usize];
                if args.timestamp.inner().time() > opening_times[0]
                    && args.timestamp.inner().time() < opening_times[1]
                    && !args.game_ctx.speical_days.is_non_trading_day()
                {
                    self.state = State::ShopKeeper;
                }

                if !args.game_ctx.sound_system.get_playing() {
                    args.game_ctx
                        .sound_system
                        .push_song(SONG_SHOP_CLOSED, SongPlayOptions::new().with_music());
                }
            }
            State::ShopKeeper => {
                if !args.game_ctx.sound_system.get_playing() {
                    args.game_ctx
                        .sound_system
                        .push_song(SONG_SHOP, SongPlayOptions::new().with_music());
                }

                if args.input.pressed(crate::Button::Right) {
                    args.game_ctx.sound_system.clear_song();
                    self.state = State::Selected(0);
                }
                if args.input.pressed(crate::Button::Left) {
                    output.set_home();
                    return;
                }
            }
            State::Selected(selected) => {
                let current = self.for_sale[selected];
                if args.input.pressed(crate::Button::Middle) {
                    if args.game_ctx.money > current.cost()
                        && !(current.unique() && args.game_ctx.inventory.has_item(current))
                    {
                        args.game_ctx.inventory.add_item(current, 1);
                        args.game_ctx.money -= current.cost();
                        args.game_ctx.sound_system.push_song(
                            sounds::SONG_BUY_CHIME,
                            SongPlayOptions::new().with_effect(),
                        );
                        self.particle_system.run_once_spawner(
                            |particles, args| {
                                static TEMPLATE: ParticleTemplate = ParticleTemplate::new(
                                    TemplateCullTatic::Remaning(
                                        Duration::from_millis(1000)..Duration::from_millis(2000),
                                    ),
                                    RectVec2::new_top_left(
                                        Vec2::new(10., 80.),
                                        Vec2::new(40., 40.),
                                    ),
                                    Vec2::new(-50.0, -50.0)..Vec2::new(50.0, 50.0),
                                    &[&assets::IMAGE_MONEY_PARTICLE],
                                );

                                for _ in 0..20 {
                                    particles.add(TEMPLATE.instantiate(&mut args.rng));
                                }
                            },
                            &mut ParticleSpawnArgs::new(
                                args.delta,
                                &mut args.game_ctx.rng,
                                &Vec2::default(),
                            ),
                        );
                        self.sign_shake_remaining = SIGN_SHAKE_DURATION;
                    } else {
                        args.game_ctx
                            .sound_system
                            .push_song(sounds::SONG_ERROR, SongPlayOptions::new().with_effect());
                        self.sign_shake_remaining = SIGN_SHAKE_DURATION;
                    }
                }

                let mut selected = selected as isize;
                if args.input.pressed(crate::Button::Right) {
                    selected += 1;
                }
                if args.input.pressed(crate::Button::Left) {
                    selected -= 1;
                }

                if selected < 0 || selected >= self.item_count() as isize {
                    self.state = State::ShopKeeper;
                } else {
                    self.state = State::Selected(selected as usize)
                }
            }
        }
    }

    fn render(&self, display: &mut GameDisplay, args: &mut RenderArgs) {
        match self.state {
            State::Closed => {
                display.render_image_complex(
                    4,
                    (self.shop_keeper.pos.y
                        - self.shop_keeper.anime.current_frame().size.y as f32 * 0.4)
                        as i32,
                    &assets::IMAGE_SHOP_SIGN,
                    ComplexRenderOption::new().with_white().with_bottom_left(),
                );
                display.render_sprite(&self.closed_sign);
                let y_offset = self.shop_keeper.pos.y as i32
                    + assets::IMAGE_SHOP_SIGN_CLOSED_0.isize.y / 2
                    + 4;

                if let Some(day) = args.game_ctx.speical_days.non_trading_day() {
                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, y_offset),
                        "DUE TO",
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    display.render_text_complex(
                        &IVec2::new(CENTER_X_I32, y_offset + 10),
                        day.name(),
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                } else {
                    const X_OFFSET: i32 = 8;
                    for (i, time) in SHOP_OPEN_TIMES.iter().enumerate() {
                        let day = chrono::Weekday::try_from(i as u8).unwrap();
                        let open = time[0];
                        let closed = time[1];
                        let y = y_offset + (i as i32) * 7;
                        let str = fixedstr::str_format!(fixedstr::str16, "{}", day,);
                        display.render_text_complex(
                            &IVec2::new(X_OFFSET + 0, y),
                            &str,
                            ComplexRenderOption::new()
                                .with_white()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        let str = fixedstr::str_format!(fixedstr::str16, "{:0>2}", open.hour(),);
                        display.render_text_complex(
                            &IVec2::new(X_OFFSET + 20, y),
                            &str,
                            ComplexRenderOption::new()
                                .with_white()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );

                        display.render_text_complex(
                            &IVec2::new(X_OFFSET + 31, y),
                            "-",
                            ComplexRenderOption::new()
                                .with_white()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );

                        let str = fixedstr::str_format!(fixedstr::str16, "{:0>2}", closed.hour(),);
                        display.render_text_complex(
                            &IVec2::new(X_OFFSET + 37, y),
                            &str,
                            ComplexRenderOption::new()
                                .with_white()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                    }
                }
            }
            State::ShopKeeper => {
                const Y_BUFFER: i32 = 8;
                display.render_image_complex(
                    4,
                    (self.shop_keeper.pos.y
                        - self.shop_keeper.anime.current_frame().size.y as f32 * 0.4)
                        as i32,
                    &assets::IMAGE_SHOP_SIGN,
                    ComplexRenderOption::new().with_white().with_bottom_left(),
                );
                display.render_sprite(&self.shop_keeper);
                let y = self.shop_keeper.pos.y as i32
                    + self.shop_keeper.anime.current_frame().isize.y / 2
                    + 5;

                let mut render_pos = IVec2::new(CENTER_X_I32, y);

                display.render_text_complex(
                    &render_pos,
                    "CHECK OUR",
                    ComplexRenderOption::new()
                        .with_center()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                render_pos.y += Y_BUFFER;
                let str = fixedstr::str_format!(
                    fixedstr::str12,
                    "{} WARES",
                    self.for_sale.iter().filter(|i| i.is_some()).count()
                );
                display.render_text_complex(
                    &render_pos,
                    &str,
                    ComplexRenderOption::new()
                        .with_center()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                render_pos.y += Y_BUFFER;
                display.render_text_complex(
                    &render_pos,
                    "NEW WARES IN",
                    ComplexRenderOption::new()
                        .with_center()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                render_pos.y += Y_BUFFER;
                let tomorrow = match args.timestamp.inner().date().checked_add_days(Days::new(1)) {
                    Some(val) => val,
                    None => NaiveDate::MAX,
                };
                // Set time to open
                let open_time = SHOP_OPEN_TIMES[tomorrow.weekday() as usize][0];
                let tomorrow = NaiveDateTime::new(tomorrow, open_time);
                let mut midnight_in = tomorrow - *args.timestamp.inner();
                let hours = midnight_in.num_hours();
                midnight_in -= TimeDelta::hours(hours);
                let mins = midnight_in.num_minutes();
                midnight_in -= TimeDelta::minutes(mins);
                let seconds = midnight_in.num_seconds();
                midnight_in -= TimeDelta::seconds(seconds);
                let str = fixedstr::str_format!(
                    fixedstr::str12,
                    "{:0>2}:{:0>2}:{:0>2}",
                    hours,
                    mins,
                    seconds
                );
                display.render_text_complex(
                    &render_pos,
                    &str,
                    ComplexRenderOption::new()
                        .with_center()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
            }
            State::Selected(selected) => {
                const BUFFER_Y: i32 = 8;
                let item = self.for_sale[selected];
                let y = 4;

                let mut render_pos = IVec2::new(CENTER_X_I32, y);

                let str = fixedstr::str_format!(fixedstr::str12, "BANK ${}", args.game_ctx.money);
                display.render_text_complex(
                    &render_pos,
                    &str,
                    ComplexRenderOption::new()
                        .with_center()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                render_pos.y += BUFFER_Y;
                display.render_text_complex(
                    &render_pos,
                    item.name(),
                    ComplexRenderOption::new()
                        .with_center()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                render_pos.y += BUFFER_Y;
                let str = fixedstr::str_format!(fixedstr::str12, "CST ${}", item.cost());
                display.render_text_complex(
                    &render_pos,
                    &str,
                    ComplexRenderOption::new()
                        .with_center()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                display.render_image_complex(
                    CENTER_X_I32,
                    render_pos.y + 30,
                    item.image(),
                    ComplexRenderOption::new().with_white().with_center(),
                );
                render_pos.y = 80;
                let own_count = args.game_ctx.inventory.item_count(item);
                if !item.unique() {
                    let str = fixedstr::str_format!(fixedstr::str12, "OWN {}", own_count);
                    display.render_text_complex(
                        &render_pos,
                        &str,
                        ComplexRenderOption::new()
                            .with_center()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    render_pos.y += BUFFER_Y;
                }
                if item.unique() && own_count > 0 {
                    display.render_image_center(
                        CENTER_X_I32
                            + if self.sign_shake_remaining > Duration::ZERO {
                                args.game_ctx.rng.i32(-3..=3)
                            } else {
                                0
                            },
                        render_pos.y + assets::IMAGE_ALREADY_OWN_SIGN.isize.y / 2,
                        &assets::IMAGE_ALREADY_OWN_SIGN,
                    );
                } else {
                    let too_much = args.game_ctx.money < item.cost();
                    if too_much {
                        let str = fixedstr::str_format!(
                            fixedstr::str12,
                            "NEED ${}",
                            item.cost() - args.game_ctx.money
                        );
                        display.render_text_complex(
                            &render_pos,
                            &str,
                            ComplexRenderOption::new()
                                .with_center()
                                .with_white()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        render_pos.y += 10;
                    }

                    if !too_much {
                        display.render_image_center(
                            CENTER_X_I32
                                + if self.sign_shake_remaining > Duration::ZERO {
                                    args.game_ctx.rng.i32(-3..=3)
                                } else {
                                    0
                                },
                            render_pos.y + assets::IMAGE_BUY_SIGN.isize.y / 2,
                            &assets::IMAGE_BUY_SIGN,
                        );
                    } else {
                        display.render_image_center(
                            CENTER_X_I32
                                + if self.sign_shake_remaining > Duration::ZERO {
                                    args.game_ctx.rng.i32(-3..=3)
                                } else {
                                    0
                                },
                            render_pos.y + assets::IMAGE_CANT_BUY_SIGN.isize.y / 2,
                            &assets::IMAGE_CANT_BUY_SIGN,
                        );
                    };
                }
            }
        }

        display.render_complex(&self.particle_system);
    }
}
