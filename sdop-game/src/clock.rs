use chrono::{NaiveTime, Timelike};
use glam::{U8Vec2, Vec2};

use crate::{
    assets::{self, StaticImage},
    display::{ComplexRender, ComplexRenderOption, CENTER_VEC},
};

pub enum ClockKind {
    Clock41,
    Clock21,
}

impl ClockKind {
    const fn image(&self) -> &'static StaticImage {
        match self {
            ClockKind::Clock41 => &assets::IMAGE_CLOCK_41_EMPTY,
            ClockKind::Clock21 => &assets::IMAGE_CLOCK_21_EMPTY,
        }
    }

    const fn image_mask(&self) -> &'static StaticImage {
        match self {
            ClockKind::Clock41 => &assets::IMAGE_CLOCK_41_EMPTY_MASK,
            ClockKind::Clock21 => &assets::IMAGE_CLOCK_21_EMPTY_MASK,
        }
    }

    const fn radius_hour(&self) -> f32 {
        match self {
            ClockKind::Clock41 => 14.,
            ClockKind::Clock21 => 14. / 2.,
        }
    }

    const fn radius_miniute(&self) -> f32 {
        match self {
            ClockKind::Clock41 => 12.,
            ClockKind::Clock21 => 12. / 2.,
        }
    }

    const fn radius_second(&self) -> f32 {
        match self {
            ClockKind::Clock41 => 10.,
            ClockKind::Clock21 => 10. / 2.,
        }
    }
}

pub struct RenderClock {
    pub pos: Vec2,
    pub hour: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub kind: ClockKind,

    pub hour_hand: bool,
    pub minute_hand: bool,
    pub second_hand: bool,
}

impl RenderClock {
    pub const fn size(&self) -> Vec2 {
        Vec2::new(
            self.kind.image().size.x as f32,
            self.kind.image().size.y as f32,
        )
    }

    pub fn update_time(&mut self, now: &NaiveTime) {
        self.hour = now.hour() as u8;
        self.minutes = now.minute() as u8;
        self.seconds = now.second() as u8;
    }

    pub fn new(kind: ClockKind, pos: Vec2, now: NaiveTime) -> Self {
        Self {
            pos: pos,
            hour: now.hour() as u8,
            minutes: now.minute() as u8,
            seconds: now.second() as u8,
            kind,
            hour_hand: true,
            minute_hand: true,
            second_hand: true,
        }
    }

    pub fn without_second_hand(mut self) -> Self {
        self.second_hand = false;
        self
    }

    fn get_hour_angle_deg(&self) -> f32 {
        let min_percent = self.minutes as f32 / 60.0;
        let hour_percent = (self.hour % 12) as f32 + min_percent;

        30.0 * hour_percent - 90.0
    }

    fn get_minute_angle_deg(&self) -> f32 {
        let sec_percent = self.seconds as f32 / 60.0;
        let minute_percent = self.minutes as f32 + sec_percent;
        6.0 * minute_percent - 90.0
    }

    fn get_second_angle_deg(&self) -> f32 {
        6.0 * self.seconds as f32 - 90.0
    }

    fn get_hand_position(&self, radius: f32, angle_deg: f32) -> Vec2 {
        let angle_rad = angle_deg.to_radians();

        Vec2::new(
            self.pos.x + radius * libm::cosf(angle_rad),
            self.pos.y + radius * libm::sinf(angle_rad),
        )
    }
}

impl Default for RenderClock {
    fn default() -> Self {
        Self::new(ClockKind::Clock41, CENTER_VEC, NaiveTime::default())
    }
}

impl ComplexRender for RenderClock {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            self.kind.image(),
            ComplexRenderOption::new().with_white().with_center(),
        );

        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            self.kind.image_mask(),
            ComplexRenderOption::new().with_black().with_center(),
        );

        if self.hour_hand {
            display.render_line(
                self.pos,
                self.get_hand_position(self.kind.radius_hour(), self.get_hour_angle_deg()),
                true,
            );
        }

        if self.minute_hand {
            display.render_line(
                self.pos,
                self.get_hand_position(self.kind.radius_miniute(), self.get_minute_angle_deg()),
                true,
            );
        }

        if self.second_hand {
            display.render_line(
                self.pos,
                self.get_hand_position(self.kind.radius_second(), self.get_second_angle_deg()),
                true,
            );
        }
    }
}
