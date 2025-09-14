use chrono::{Datelike, NaiveDateTime, Timelike};
use glam::Vec2;

use crate::{
    anime::{Anime, HasAnime},
    assets::{self, Frame, StaticImage},
    display::{ComplexRender, ComplexRenderOption},
};

pub type Show = &'static [Frame];

const SHOWS: &[Show] = &[
    &assets::FRAMES_TV_SHOW_NEWS,
    &assets::FRAMES_TV_SHOW_SPORT,
    &assets::FRAMES_TV_SHOW_SUBWAY,
    &assets::FRAMES_TV_SHOW_WEIGHT_LIFTING,
];

pub const SHOW_RUN_TIME: u8 = 15;

pub fn get_show_for_time(date_time: &NaiveDateTime) -> Show {
    let day = date_time.day() as u8;
    let month = date_time.month() as u8;
    let hour = date_time.hour() as u8;
    let miniutes = date_time.minute() as u8 / SHOW_RUN_TIME;

    let seed = u32::from_be_bytes([day, month, hour, miniutes]) as u64;

    let mut rng = fastrand::Rng::with_seed(seed);

    SHOWS[rng.usize(0..SHOWS.len())]
}

#[derive(Copy, Clone)]
pub enum TvKind {
    CRT,
    LCD,
}

pub struct TvRender {
    pub pos: Vec2,
    pub kind: TvKind,
    show: Anime,
}

impl TvRender {
    pub fn new(kind: TvKind, pos: Vec2, show: &'static [Frame]) -> Self {
        Self {
            kind,
            pos,
            show: Anime::new(show),
        }
    }

    pub fn change_show(&mut self, show: &'static [Frame], rng: &mut fastrand::Rng) {
        self.show = Anime::new(show);
        self.show.set_frame(rng.usize(0..show.len()));
    }

    fn image(&self) -> &'static StaticImage {
        match self.kind {
            TvKind::CRT => &assets::IMAGE_TV_CRT,
            TvKind::LCD => &assets::IMAGE_TV_LCD,
        }
    }

    pub fn size(&self) -> Vec2 {
        self.image().size.as_vec2()
    }
}

impl HasAnime for TvRender {
    fn anime(&mut self) -> &mut Anime {
        &mut self.show
    }
}

impl ComplexRender for TvRender {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        display.render_image_complex(
            self.pos.x as i32,
            self.pos.y as i32,
            self.image(),
            ComplexRenderOption::new()
                .with_white()
                .with_black()
                .with_center(),
        );

        let image_offset = match self.kind {
            TvKind::CRT => Vec2::new(2., -2.),
            TvKind::LCD => Vec2::new(0., 0.),
        };

        display.render_image_complex(
            self.pos.x as i32 + image_offset.x as i32,
            self.pos.y as i32 + image_offset.y as i32,
            self.show.current_frame(),
            ComplexRenderOption::new()
                .with_white()
                .with_black()
                .with_center(),
        );
    }
}
