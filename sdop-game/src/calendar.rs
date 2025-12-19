use chrono::{Datelike, NaiveDate, Weekday};
use fixedstr::str_format;
use glam::{IVec2, Vec2};

use crate::{
    assets,
    display::{ComplexRender, ComplexRenderOption},
    fonts::FONT_VARIABLE_SMALL,
};

pub struct CalendarRender {
    pub pos: Vec2,
    date: NaiveDate,
}

impl CalendarRender {
    pub fn new(pos: Vec2, date: NaiveDate) -> Self {
        let mut result = Self { date, pos };

        result.set_date(date);

        result
    }

    pub fn set_date(&mut self, date: NaiveDate) {
        self.date = date;
    }

    pub const fn size() -> Vec2 {
        Vec2::new(
            assets::IMAGE_CALENDAR.size.x as f32,
            assets::IMAGE_CALENDAR.size.y as f32,
        )
    }
}

impl ComplexRender for CalendarRender {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            &assets::IMAGE_CALENDAR,
            ComplexRenderOption::new()
                .with_white()
                .with_black()
                .with_center(),
        );

        const MONTH_OFFSET: IVec2 = IVec2::new(9, 5);

        let top_left = IVec2::new(
            self.pos.x as i32 - Self::size().x as i32 / 2,
            self.pos.y as i32 - Self::size().y as i32 / 2,
        );
        // Render month
        let str = str_format!(fixedstr::str4, "{}", self.date.month());
        display.render_text_complex(
            &(top_left + MONTH_OFFSET),
            &str,
            ComplexRenderOption::new()
                .with_white()
                .with_center()
                .with_font(&FONT_VARIABLE_SMALL),
        );

        const ROWS_Y: &[i32] = &[10, 12, 14, 16, 18];
        const X_OFFSET: i32 = 2;

        let mut top = NaiveDate::from_ymd_opt(self.date.year(), self.date.month(), 1).unwrap();

        let mut col = top.weekday().days_since(Weekday::Mon) as usize;
        let mut row = 0;
        while top <= self.date {
            display.render_point(
                top_left.x as i32 + 3 + (X_OFFSET * col as i32),
                top_left.y as i32 + 1 + ROWS_Y[row],
                true,
            );

            top = top.succ_opt().unwrap();

            if top.weekday() == Weekday::Mon {
                row += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
    }
}
