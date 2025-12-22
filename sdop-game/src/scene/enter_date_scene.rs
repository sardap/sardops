use chrono::{Datelike, Days, Months, NaiveDate, NaiveTime, Timelike};
use fixedstr::str_format;
use glam::{IVec2, Vec2};

use crate::{
    Button, assets,
    date_utils::{END_YEAR, START_YEAR},
    display::{CENTER_X, CENTER_X_I32, ComplexRenderOption, GameDisplay},
    fonts::FONT_VARIABLE_SMALL,
    geo::RectVec2,
    scene::{RenderArgs, Scene, SceneOutput, SceneTickArgs},
};

enum State {
    SelectField,
    SelectValue,
}

pub enum Required {
    Date,
    Time,
    DateTime,
}

enum FieldOption {
    Year,
    Month,
    Day,
    Hour,
    Miniute,
    Second,
    Submit,
}

impl FieldOption {
    fn next(&self, date: bool, time: bool) -> Self {
        match self {
            FieldOption::Year => Self::Month,
            FieldOption::Month => Self::Day,
            FieldOption::Day => {
                if time {
                    Self::Hour
                } else {
                    Self::Submit
                }
            }
            FieldOption::Hour => Self::Miniute,
            FieldOption::Miniute => Self::Second,
            FieldOption::Second => Self::Submit,
            FieldOption::Submit => {
                if date {
                    Self::Year
                } else {
                    Self::Hour
                }
            }
        }
    }

    fn prev(&self, date: bool, time: bool) -> Self {
        match self {
            FieldOption::Year => Self::Submit,
            FieldOption::Month => Self::Year,
            FieldOption::Day => Self::Month,
            FieldOption::Hour => {
                if time {
                    Self::Submit
                } else {
                    Self::Day
                }
            }
            FieldOption::Miniute => Self::Hour,
            FieldOption::Second => Self::Miniute,
            FieldOption::Submit => {
                if date {
                    Self::Year
                } else {
                    Self::Miniute
                }
            }
        }
    }
}

pub struct EnterDateScene {
    state: State,
    date: NaiveDate,
    time: NaiveTime,
    selected: FieldOption,
    need_date: bool,
    need_time: bool,
    display_text: fixedstr::str12,
}

impl EnterDateScene {
    pub fn new(required: Required, display_text: fixedstr::str12) -> Self {
        Self {
            state: State::SelectField,
            date: NaiveDate::from_ymd_opt(2025, 01, 01).unwrap(),
            time: NaiveTime::default(),
            selected: match required {
                Required::Date | Required::DateTime => FieldOption::Year,
                Required::Time => FieldOption::Hour,
            },
            need_date: matches!(required, Required::Date) || matches!(required, Required::DateTime),
            need_time: matches!(required, Required::Time) || matches!(required, Required::DateTime),
            display_text,
        }
    }

    pub fn with_time(mut self, time: NaiveTime) -> Self {
        self.time = time;
        self
    }

    pub fn with_date(mut self, date: NaiveDate) -> Self {
        self.date = date;
        self
    }
}

impl Scene for EnterDateScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, _args: &mut SceneTickArgs) {}

    fn tick(&mut self, args: &mut SceneTickArgs, output: &mut SceneOutput) {
        match self.state {
            State::SelectField => {
                if args.input.pressed(Button::Left) {
                    self.selected = self.selected.prev(self.need_date, self.need_time)
                }
                if args.input.pressed(Button::Right) {
                    self.selected = self.selected.next(self.need_date, self.need_time)
                }

                if args.input.pressed(Button::Middle) {
                    if matches!(self.selected, FieldOption::Submit) {
                        args.game_ctx.shared_out.date_out = self.date;
                        args.game_ctx.shared_out.time_out = self.time;
                        output.set(args.last_scene.take().unwrap());
                        return;
                    }
                    self.state = State::SelectValue;
                }
            }
            State::SelectValue => {
                if args.input.pressed(Button::Left) {
                    match self.selected {
                        FieldOption::Year => {
                            self.date = NaiveDate::from_ymd_opt(
                                self.date.year() - 1,
                                self.date.month(),
                                self.date.day(),
                            )
                            .unwrap_or(
                                NaiveDate::from_ymd_opt(self.date.year() - 1, 1, 1).unwrap(),
                            );
                        }
                        FieldOption::Month => {
                            self.date = self
                                .date
                                .checked_sub_months(Months::new(1))
                                .unwrap_or(self.date);
                        }
                        FieldOption::Day => {
                            self.date = self
                                .date
                                .checked_sub_days(Days::new(1))
                                .unwrap_or(self.date);
                        }
                        FieldOption::Hour => {
                            self.time -= chrono::TimeDelta::hours(1);
                        }
                        FieldOption::Miniute => {
                            self.time -= chrono::TimeDelta::minutes(1);
                        }
                        FieldOption::Second => {
                            self.time -= chrono::TimeDelta::seconds(1);
                        }
                        FieldOption::Submit => unreachable!(),
                    }
                }

                if args.input.pressed(Button::Right) {
                    match self.selected {
                        FieldOption::Year => {
                            self.date = NaiveDate::from_ymd_opt(
                                self.date.year() + 1,
                                self.date.month(),
                                self.date.day(),
                            )
                            .unwrap_or(
                                NaiveDate::from_ymd_opt(self.date.year() + 1, 1, 1).unwrap(),
                            );
                        }
                        FieldOption::Month => {
                            self.date = self
                                .date
                                .checked_add_months(Months::new(1))
                                .unwrap_or(self.date);
                        }
                        FieldOption::Day => {
                            self.date = self
                                .date
                                .checked_add_days(Days::new(1))
                                .unwrap_or(self.date);
                        }
                        FieldOption::Hour => {
                            self.time += chrono::TimeDelta::hours(1);
                        }
                        FieldOption::Miniute => {
                            self.time += chrono::TimeDelta::minutes(1);
                        }
                        FieldOption::Second => {
                            self.time += chrono::TimeDelta::seconds(1);
                        }
                        FieldOption::Submit => unreachable!(),
                    }
                }

                if self.date.year() < START_YEAR {
                    self.date = NaiveDate::from_ymd_opt(START_YEAR, 1, 1).unwrap();
                }

                if self.date.year() > END_YEAR {
                    self.date = NaiveDate::from_ymd_opt(END_YEAR, 1, 1).unwrap();
                }

                if args.input.pressed(Button::Middle) {
                    self.state = State::SelectField;
                }
            }
        }
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        display.render_text_complex(
            &IVec2::new(CENTER_X_I32, 30),
            &self.display_text,
            ComplexRenderOption::new()
                .with_white()
                .with_center()
                .with_font(&FONT_VARIABLE_SMALL),
        );

        let mut y = 50;

        // Seprate each part so I can better point to a section
        const SLASH_WIDTH: i32 = 5;
        const X_START: i32 = 6;
        const YEAR_WIDTH: i32 = 4 * 5;
        const X_MONTH: i32 = X_START + YEAR_WIDTH + SLASH_WIDTH;
        const MONTH_WIDTH: i32 = 2 * 5;
        const X_DAY: i32 = X_MONTH + MONTH_WIDTH + SLASH_WIDTH;
        const DAY_WIDTH: i32 = 2 * 5;

        let y_date = y;

        if self.need_date {
            let str = str_format!(fixedstr::str5, "{}", self.date.year());
            display.render_text_complex(
                &IVec2::new(X_START, y),
                &str,
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );

            display.render_text_complex(
                &IVec2::new(X_START + YEAR_WIDTH + 1, y),
                "/",
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );

            let str = str_format!(fixedstr::str5, "{:0>2}", self.date.month());
            display.render_text_complex(
                &IVec2::new(X_MONTH, y),
                &str,
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );

            display.render_text_complex(
                &IVec2::new(X_MONTH + MONTH_WIDTH + 1, y),
                "/",
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );

            let str = str_format!(fixedstr::str5, "{:0>2}", self.date.day());
            display.render_text_complex(
                &IVec2::new(X_DAY, y),
                &str,
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );

            y += 25;
        }

        let y_time = y;

        const X_HOUR: i32 = 12;
        const HOUR_WIDTH: i32 = 2 * 5;
        const X_MINIUTE: i32 = X_HOUR + HOUR_WIDTH + SLASH_WIDTH;
        const MINIUTE_WIDTH: i32 = 2 * 5;
        const X_SECOND: i32 = X_MINIUTE + MINIUTE_WIDTH + SLASH_WIDTH;
        const SECOND_WIDTH: i32 = 2 * 5;

        if self.need_time {
            let str = str_format!(fixedstr::str5, "{:0>2}", self.time.hour());
            display.render_text_complex(
                &IVec2::new(X_HOUR, y),
                &str,
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );
            display.render_text_complex(
                &IVec2::new(X_HOUR + HOUR_WIDTH + 1, y),
                ":",
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );

            let str = str_format!(fixedstr::str5, "{:0>2}", self.time.minute());
            display.render_text_complex(
                &IVec2::new(X_MINIUTE, y),
                &str,
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );
            display.render_text_complex(
                &IVec2::new(X_MINIUTE + MINIUTE_WIDTH + 1, y),
                ":",
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );

            let str = str_format!(fixedstr::str5, "{:0>2}", self.time.second());
            display.render_text_complex(
                &IVec2::new(X_SECOND, y),
                &str,
                ComplexRenderOption::new()
                    .with_white()
                    .with_font(&FONT_VARIABLE_SMALL),
            );

            y += 25;
        }

        display.render_image_complex(
            CENTER_X as i32,
            y as i32,
            &assets::IMAGE_SUBMIT_BUTTON,
            ComplexRenderOption::new().with_white().with_center(),
        );

        let (x, y) = match self.selected {
            FieldOption::Year => (X_START + (YEAR_WIDTH / 2), y_date),
            FieldOption::Month => (X_MONTH + (MONTH_WIDTH / 2), y_date),
            FieldOption::Day => (X_DAY + (DAY_WIDTH / 2), y_date),
            FieldOption::Hour => (X_HOUR + (HOUR_WIDTH / 2), y_time),
            FieldOption::Miniute => (X_MINIUTE + (MINIUTE_WIDTH / 2), y_time),
            FieldOption::Second => (X_SECOND + (SECOND_WIDTH / 2), y_time),
            FieldOption::Submit => (CENTER_X_I32, y),
        };

        match self.state {
            State::SelectField => {
                if matches!(self.selected, FieldOption::Submit) {
                    let rect = RectVec2::new_center(
                        Vec2::new(x as f32, y as f32),
                        assets::IMAGE_SUBMIT_BUTTON.size.as_vec2(),
                    )
                    .grow(4.);

                    display.render_rect_outline(rect, true);
                } else {
                    display.render_image_complex(
                        x as i32,
                        y as i32 + assets::IMAGE_NAME_ARROW.size.y as i32,
                        &assets::IMAGE_NAME_ARROW,
                        ComplexRenderOption::new().with_center().with_white(),
                    );
                }
            }
            State::SelectValue => {
                let width = match self.selected {
                    FieldOption::Year => YEAR_WIDTH,
                    FieldOption::Month => MONTH_WIDTH,
                    FieldOption::Day => DAY_WIDTH,
                    FieldOption::Hour => HOUR_WIDTH,
                    FieldOption::Miniute => MINIUTE_WIDTH,
                    FieldOption::Second => SECOND_WIDTH,
                    FieldOption::Submit => unreachable!(),
                };

                let rect = RectVec2::new_center(
                    Vec2::new(x as f32, y as f32 + 10.),
                    Vec2::new(width as f32, 2.),
                );

                display.render_rect_solid(rect, true);
            }
        }
    }
}
