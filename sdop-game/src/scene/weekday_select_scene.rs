use chrono::{Datelike, Days, Months, NaiveDate, NaiveTime, Timelike, Weekday, WeekdaySet};
use fixedstr::str_format;
use glam::Vec2;

use crate::{
    Button, assets,
    date_utils::{END_YEAR, START_YEAR},
    display::{CENTER_X, ComplexRenderOption, GameDisplay},
    fonts::FONT_VARIABLE_SMALL,
    geo::Rect,
    scene::{RenderArgs, Scene, SceneOutput, SceneTickArgs},
    sounds::{SONG_ERROR, SongPlayOptions},
};

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

const WEEKDAYS: &[Weekday] = &[
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
    Weekday::Sat,
    Weekday::Sun,
];

pub struct WeekdaySelectScene {
    current: i8,
    days: WeekdaySet,
    min_select: u8,
    display_text: fixedstr::str12,
}

impl WeekdaySelectScene {
    pub fn new(min_select: u8, display_text: fixedstr::str12) -> Self {
        Self {
            current: 0,
            days: WeekdaySet::EMPTY,
            min_select,
            display_text,
        }
    }

    pub fn with_days(mut self, days: WeekdaySet) -> Self {
        self.days = days;
        self
    }
}

impl Scene for WeekdaySelectScene {
    fn setup(&mut self, _args: &mut SceneTickArgs) {}

    fn teardown(&mut self, args: &mut SceneTickArgs) {
        args.game_ctx.shared_out.weekday_out = self.days;
    }

    fn tick(&mut self, args: &mut SceneTickArgs) -> SceneOutput {
        if args.input.pressed(Button::Left) {
            self.current -= 1;
        }
        if args.input.pressed(Button::Right) {
            self.current += 1;
        }

        if self.current >= WEEKDAYS.len() as i8 {
            self.current = -1;
        } else if self.current < -1 {
            self.current = (WEEKDAYS.len() - 1) as i8;
        }

        if args.input.pressed(Button::Middle) {
            if self.current >= 0 && self.current < WEEKDAYS.len() as i8 {
                let current = WEEKDAYS[self.current as usize];
                if self.days.contains(current) {
                    self.days.remove(current);
                } else {
                    self.days.insert(current);
                }
            } else {
                let selected_count = WEEKDAYS
                    .iter()
                    .filter(|day| self.days.contains(**day))
                    .count();
                if selected_count < self.min_select as usize {
                    args.game_ctx
                        .sound_system
                        .push_song(SONG_ERROR, SongPlayOptions::new().with_effect());
                } else {
                    return SceneOutput::new(args.last_scene.take().unwrap());
                }
            }
        }

        SceneOutput::default()
    }

    fn render(&self, display: &mut GameDisplay, _args: &mut RenderArgs) {
        display.render_text_complex(
            Vec2::new(CENTER_X, 30.),
            &self.display_text,
            ComplexRenderOption::new()
                .with_white()
                .with_center()
                .with_font(&FONT_VARIABLE_SMALL),
        );

        let mut y = 50.;

        const BUFFER_X: f32 = 5.;
        const GAP_X: f32 = 20.;
        const GAP_Y: f32 = 12.;

        for (i, day) in WEEKDAYS.iter().enumerate() {
            let (x, y) = if i < 3 {
                (BUFFER_X + (i as f32 * GAP_X), y)
            } else if i < 6 {
                (BUFFER_X + ((i - 3) as f32 * GAP_X), y + GAP_Y)
            } else {
                (BUFFER_X + GAP_X, y + GAP_Y * 2.)
            };

            let str = str_format!(fixedstr::str12, "{}", day);
            let width = display.render_text_complex(
                Vec2::new(x, y),
                &str,
                ComplexRenderOption::new()
                    .with_font(&FONT_VARIABLE_SMALL)
                    .with_white()
                    .with_bottom_left(),
            );

            if self.days.contains(*day) {
                display.render_rect_solid(
                    Rect::new_bottom_left(Vec2::new(x - 2., y + 4.), Vec2::new(width + 4., 1.)),
                    true,
                );
            }

            if self.current >= 0 {
                let selected = WEEKDAYS.get(self.current as usize).unwrap_or(&Weekday::Mon);
                if day == selected {
                    display.render_rect_outline_dashed(
                        Rect::new_bottom_left(
                            Vec2::new(x - 2., y + 2.),
                            Vec2::new(width + 4., 10.),
                        ),
                        true,
                        1,
                    );
                }
            }
        }

        display.render_image_complex(
            CENTER_X as i32,
            (y + GAP_Y * 4.) as i32,
            &assets::IMAGE_SUBMIT_BUTTON,
            ComplexRenderOption::new().with_white().with_center(),
        );

        if self.current < 0 {
            let rect = Rect::new_center(
                Vec2::new(CENTER_X, y + GAP_Y * 4.),
                assets::IMAGE_SUBMIT_BUTTON.size.as_vec2(),
            )
            .grow(4.);
            display.render_rect_outline(rect, true);
        }
    }
}
