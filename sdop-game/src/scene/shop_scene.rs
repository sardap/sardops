use chrono::{Datelike, Days, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Timelike};
use glam::Vec2;

use crate::{
    anime::HasAnime,
    assets,
    display::{ComplexRenderOption, GameDisplay, CENTER_VEC, CENTER_X},
    fonts::FONT_VARIABLE_SMALL,
    scene::{home_scene::HomeScene, RenderArgs, Scene, SceneEnum, SceneOutput, SceneTickArgs},
    shop::ShopItemSet,
    sprite::BasicAnimeSprite,
};

const OPEN_TIMES: [[NaiveTime; 2]; 7] = [
    // Monday
    [
        NaiveTime::from_hms_opt(10, 00, 00).unwrap(),
        NaiveTime::from_hms_opt(19, 00, 00).unwrap(),
    ],
    [
        NaiveTime::from_hms_opt(10, 00, 00).unwrap(),
        NaiveTime::from_hms_opt(19, 00, 00).unwrap(),
    ],
    [
        NaiveTime::from_hms_opt(10, 00, 00).unwrap(),
        NaiveTime::from_hms_opt(19, 00, 00).unwrap(),
    ],
    [
        NaiveTime::from_hms_opt(10, 00, 00).unwrap(),
        NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
    ],
    [
        NaiveTime::from_hms_opt(10, 00, 00).unwrap(),
        NaiveTime::from_hms_opt(21, 00, 00).unwrap(),
    ],
    // Sat
    [
        NaiveTime::from_hms_opt(11, 00, 00).unwrap(),
        NaiveTime::from_hms_opt(17, 00, 00).unwrap(),
    ],
    [
        NaiveTime::from_hms_opt(11, 00, 00).unwrap(),
        NaiveTime::from_hms_opt(15, 00, 00).unwrap(),
    ],
];

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

        let opening_times = OPEN_TIMES[args.timestamp.inner().date().weekday() as usize];
        if args.timestamp.inner().time() < opening_times[0]
            || args.timestamp.inner().time() > opening_times[1]
            || args.game_ctx.speical_days.is_non_trading_day()
        {
            self.state = State::Closed;
        }
    }

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        self.shop_keeper.anime().tick(args.delta);

        match self.state {
            State::Closed => {
                self.closed_sign.anime().tick(args.delta);
                if args.input.any_pressed() {
                    return SceneOutput::new(SceneEnum::Home(HomeScene::new()));
                }
                let opening_times = OPEN_TIMES[args.timestamp.inner().date().weekday() as usize];
                if args.timestamp.inner().time() > opening_times[0]
                    && args.timestamp.inner().time() < opening_times[1]
                    && !args.game_ctx.speical_days.is_non_trading_day()
                {
                    self.state = State::ShopKeeper;
                }
            }
            State::ShopKeeper => {
                if args.input.pressed(crate::Button::Right) {
                    self.state = State::Selected(0);
                }
                if args.input.pressed(crate::Button::Left) {
                    return SceneOutput::new(SceneEnum::Home(HomeScene::new()));
                }
            }
            State::Selected(selected) => {
                let current = self.for_sale[selected];
                if args.input.pressed(crate::Button::Middle)
                    && args.game_ctx.money > current.cost()
                    && !(current.unique() && args.game_ctx.inventory.has_item(current))
                {
                    args.game_ctx.inventory.add_item(current, 1);
                    args.game_ctx.money -= current.cost();
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
        SceneOutput::default()
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
                let y_offset = self.shop_keeper.pos.y
                    + assets::IMAGE_SHOP_SIGN_CLOSED_0.size.y as f32 / 2.
                    + 4.;

                if let Some(day) = args.game_ctx.speical_days.non_trading_day() {
                    display.render_text_complex(
                        Vec2::new(CENTER_X, y_offset),
                        "DUE TO",
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                    display.render_text_complex(
                        Vec2::new(CENTER_X, y_offset + 10.),
                        day.name(),
                        ComplexRenderOption::new()
                            .with_white()
                            .with_center()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                } else {
                    const X_OFFSET: f32 = 8.;
                    for (i, time) in OPEN_TIMES.iter().enumerate() {
                        let day = chrono::Weekday::try_from(i as u8).unwrap();
                        let open = time[0];
                        let closed = time[1];
                        let y = y_offset + i as f32 * 7.;
                        let str = fixedstr::str_format!(fixedstr::str16, "{}", day,);
                        display.render_text_complex(
                            Vec2::new(X_OFFSET + 0., y),
                            &str,
                            ComplexRenderOption::new()
                                .with_white()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        let str = fixedstr::str_format!(fixedstr::str16, "{:0>2}", open.hour(),);
                        display.render_text_complex(
                            Vec2::new(X_OFFSET + 20., y),
                            &str,
                            ComplexRenderOption::new()
                                .with_white()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );

                        display.render_text_complex(
                            Vec2::new(X_OFFSET + 31., y),
                            "-",
                            ComplexRenderOption::new()
                                .with_white()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );

                        let str = fixedstr::str_format!(fixedstr::str16, "{:0>2}", closed.hour(),);
                        display.render_text_complex(
                            Vec2::new(X_OFFSET + 37., y),
                            &str,
                            ComplexRenderOption::new()
                                .with_white()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                    }
                }
            }
            State::ShopKeeper => {
                const Y_BUFFER: f32 = 8.0;
                display.render_image_complex(
                    4,
                    (self.shop_keeper.pos.y
                        - self.shop_keeper.anime.current_frame().size.y as f32 * 0.4)
                        as i32,
                    &assets::IMAGE_SHOP_SIGN,
                    ComplexRenderOption::new().with_white().with_bottom_left(),
                );
                display.render_sprite(&self.shop_keeper);
                let mut y = self.shop_keeper.pos.y
                    + self.shop_keeper.anime.current_frame().size.y as f32 / 2.
                    + 5.;
                display.render_text_complex(
                    Vec2::new(CENTER_X, y),
                    "CHECK OUR",
                    ComplexRenderOption::new()
                        .with_center()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                y += Y_BUFFER;
                let str = fixedstr::str_format!(
                    fixedstr::str12,
                    "{} WARES",
                    self.for_sale.iter().filter(|i| i.is_some()).count()
                );
                display.render_text_complex(
                    Vec2::new(CENTER_X, y),
                    &str,
                    ComplexRenderOption::new()
                        .with_center()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                y += Y_BUFFER;
                display.render_text_complex(
                    Vec2::new(CENTER_X, y),
                    "NEW WARES IN",
                    ComplexRenderOption::new()
                        .with_center()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                y += Y_BUFFER;
                let tomrrow = match args.timestamp.inner().date().checked_add_days(Days::new(1)) {
                    Some(val) => val,
                    None => NaiveDate::MAX,
                };
                // Set time to open
                let open_time = OPEN_TIMES[tomrrow.weekday() as usize][0];
                let tomrrow = NaiveDateTime::new(tomrrow, open_time);
                let mut midnight_in = tomrrow - *args.timestamp.inner();
                let hours = midnight_in.num_hours();
                midnight_in -= TimeDelta::hours(hours);
                let miniutes = midnight_in.num_minutes();
                midnight_in -= TimeDelta::minutes(miniutes);
                let seconds = midnight_in.num_seconds();
                midnight_in -= TimeDelta::seconds(seconds);
                let str = fixedstr::str_format!(
                    fixedstr::str12,
                    "{:0>2}:{:0>2}:{:0>2}",
                    hours,
                    miniutes,
                    seconds
                );
                display.render_text_complex(
                    Vec2::new(CENTER_X, y),
                    &str,
                    ComplexRenderOption::new()
                        .with_center()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
            }
            State::Selected(selected) => {
                const BUFFER_Y: f32 = 8.;
                let item = self.for_sale[selected];
                let mut y = 20.;
                let str = fixedstr::str_format!(fixedstr::str12, "BANK ${}", args.game_ctx.money);
                display.render_text_complex(
                    Vec2::new(CENTER_X, y),
                    &str,
                    ComplexRenderOption::new()
                        .with_center()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                y += BUFFER_Y + 3.;
                let str = fixedstr::str_format!(
                    fixedstr::str12,
                    "#{} OWN {}",
                    selected + 1,
                    args.game_ctx.inventory.item_count(item)
                );
                display.render_text_complex(
                    Vec2::new(CENTER_X, y),
                    &str,
                    ComplexRenderOption::new()
                        .with_center()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                y += BUFFER_Y;
                display.render_text_complex(
                    Vec2::new(CENTER_X, y),
                    item.name(),
                    ComplexRenderOption::new()
                        .with_center()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                y += BUFFER_Y;
                let str = fixedstr::str_format!(fixedstr::str12, "CST ${}", item.cost());
                display.render_text_complex(
                    Vec2::new(CENTER_X, y),
                    &str,
                    ComplexRenderOption::new()
                        .with_center()
                        .with_white()
                        .with_font(&FONT_VARIABLE_SMALL),
                );
                y += item.image().size.y as f32;
                display.render_image_complex(
                    CENTER_X as i32,
                    y as i32,
                    item.image(),
                    ComplexRenderOption::new().with_white().with_center(),
                );
                y = 100.;
                if item.unique() && args.game_ctx.inventory.has_item(item) {
                    display.render_text_complex(
                        Vec2::new(CENTER_X, y),
                        "ALREADY OWN",
                        ComplexRenderOption::new()
                            .with_center()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                } else {
                    let too_much = args.game_ctx.money < item.cost();
                    if too_much {
                        let str = fixedstr::str_format!(fixedstr::str12, "TOO MUCH $");
                        display.render_text_complex(
                            Vec2::new(CENTER_X, y),
                            &str,
                            ComplexRenderOption::new()
                                .with_center()
                                .with_white()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        y += 10.;
                        let str = fixedstr::str_format!(
                            fixedstr::str12,
                            "NEED ${}",
                            item.cost() - args.game_ctx.money
                        );
                        display.render_text_complex(
                            Vec2::new(CENTER_X, y),
                            &str,
                            ComplexRenderOption::new()
                                .with_center()
                                .with_white()
                                .with_font(&FONT_VARIABLE_SMALL),
                        );
                        y += 10.;
                    }

                    let str = if too_much {
                        fixedstr::str_format!(fixedstr::str12, "BUY")
                    } else {
                        fixedstr::str_format!(fixedstr::str12, "CAN'T BUY")
                    };
                    display.render_text_complex(
                        Vec2::new(CENTER_X, y),
                        &str,
                        ComplexRenderOption::new()
                            .with_center()
                            .with_white()
                            .with_font(&FONT_VARIABLE_SMALL),
                    );
                }
            }
        }
    }
}
