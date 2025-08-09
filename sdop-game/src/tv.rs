use glam::Vec2;

use crate::{
    anime::{Anime, HasAnime},
    assets::{self, Frame, StaticImage},
    display::{ComplexRender, ComplexRenderOption},
};

const SHOWS: &[&'static [Frame]] = &[
    &assets::FRAMES_TV_SHOW_NEWS,
    &assets::FRAMES_TV_SHOW_SPORT,
    &assets::FRAMES_TV_SHOW_SUBWAY,
];

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

    pub fn random_show(&mut self, rng: &mut fastrand::Rng) {
        let mut index = rng.usize(0..SHOWS.len()) as i32;
        if SHOWS[index as usize] == self.show.frames() {
            index = index as i32 + if rng.bool() { 1 } else { -1 };
            if index < 0 {
                index = SHOWS.len() as i32 - 1;
            } else if index >= SHOWS.len() as i32 {
                index = 0;
            }
        }

        self.change_show(SHOWS[index as usize], rng);
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
