use chrono::{NaiveTime, Timelike};
use glam::Vec2;

use crate::{
    assets::{self, IMAGE_DIGITAL_CLOCK_EMPTY, StaticImage},
    display::{CENTER_VEC, ComplexRender, ComplexRenderOption},
};

pub enum AnalogueClockKind {
    Clock41,
    Clock21,
}

impl AnalogueClockKind {
    const fn image(&self) -> &'static StaticImage {
        match self {
            AnalogueClockKind::Clock41 => &assets::IMAGE_CLOCK_41_EMPTY,
            AnalogueClockKind::Clock21 => &assets::IMAGE_CLOCK_21_EMPTY,
        }
    }

    const fn image_mask(&self) -> &'static StaticImage {
        match self {
            AnalogueClockKind::Clock41 => &assets::IMAGE_CLOCK_41_EMPTY_MASK,
            AnalogueClockKind::Clock21 => &assets::IMAGE_CLOCK_21_EMPTY_MASK,
        }
    }

    const fn radius_hour(&self) -> f32 {
        match self {
            AnalogueClockKind::Clock41 => 14.,
            AnalogueClockKind::Clock21 => 14. / 2.,
        }
    }

    const fn radius_miniute(&self) -> f32 {
        match self {
            AnalogueClockKind::Clock41 => 12.,
            AnalogueClockKind::Clock21 => 12. / 2.,
        }
    }

    const fn radius_second(&self) -> f32 {
        match self {
            AnalogueClockKind::Clock41 => 10.,
            AnalogueClockKind::Clock21 => 10. / 2.,
        }
    }

    pub const fn size(&self) -> Vec2 {
        Vec2::new(self.image().size.x as f32, self.image().size.y as f32)
    }
}

pub struct AnalogueRenderClock {
    pub pos: Vec2,
    pub hour: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub kind: AnalogueClockKind,
    pub hour_hand: bool,
    pub minute_hand: bool,
    pub second_hand: bool,
}

impl AnalogueRenderClock {
    pub fn update_time(&mut self, now: &NaiveTime) {
        self.hour = now.hour() as u8;
        self.minutes = now.minute() as u8;
        self.seconds = now.second() as u8;
    }

    pub fn new(kind: AnalogueClockKind, pos: Vec2, now: NaiveTime) -> Self {
        Self {
            pos,
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

impl Default for AnalogueRenderClock {
    fn default() -> Self {
        Self::new(AnalogueClockKind::Clock41, CENTER_VEC, NaiveTime::default())
    }
}

impl ComplexRender for AnalogueRenderClock {
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

pub struct DigitalClockRender {
    pub pos: Vec2,
    pub hour: u8,
    pub minutes: u8,
}

impl DigitalClockRender {
    pub const fn size() -> Vec2 {
        Vec2::new(
            IMAGE_DIGITAL_CLOCK_EMPTY.size.x as f32,
            IMAGE_DIGITAL_CLOCK_EMPTY.size.y as f32,
        )
    }

    pub fn update_time(&mut self, now: &NaiveTime) {
        self.hour = now.hour() as u8;
        self.minutes = now.minute() as u8;
    }

    pub fn new(pos: Vec2, now: NaiveTime) -> Self {
        Self {
            pos,
            hour: now.hour() as u8,
            minutes: now.minute() as u8,
        }
    }
}

fn get_number_image(num: u8) -> &'static StaticImage {
    match num {
        0 => &assets::IMAGE_DGC_0,
        1 => &assets::IMAGE_DGC_1,
        2 => &assets::IMAGE_DGC_2,
        3 => &assets::IMAGE_DGC_3,
        4 => &assets::IMAGE_DGC_4,
        5 => &assets::IMAGE_DGC_5,
        6 => &assets::IMAGE_DGC_6,
        7 => &assets::IMAGE_DGC_7,
        8 => &assets::IMAGE_DGC_8,
        9 => &assets::IMAGE_DGC_9,
        _ => &assets::IMAGE_DGC_0,
    }
}

fn get_images(num: u8) -> [&'static StaticImage; 2] {
    let mut result = [&assets::IMAGE_DGC_EMPTY, &assets::IMAGE_DGC_EMPTY];

    if num < 10 {
        result[1] = get_number_image(num % 10);
    } else {
        result[1] = get_number_image(num % 10);
        result[0] = get_number_image((num / 10) % 10);
    }

    result
}

impl ComplexRender for DigitalClockRender {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            &assets::IMAGE_DIGITAL_CLOCK_EMPTY,
            ComplexRenderOption::new()
                .with_center()
                .with_white()
                .with_black(),
        );

        let number_top_left = Vec2::new(
            self.pos.x - Self::size().x / 2. + 3.,
            self.pos.y - Self::size().y / 2. + 3.,
        );

        let hours = get_images(self.hour);
        let mins = get_images(self.minutes);

        for (i, image) in hours.iter().chain(mins.iter()).enumerate() {
            display.render_image_complex(
                number_top_left.x as i32 + (i as i32 * 5) + if i >= 2 { 2 } else { 0 },
                number_top_left.y as i32,
                *image,
                ComplexRenderOption::new().with_white(),
            );
        }
    }
}
