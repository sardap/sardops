use core::time::Duration;

use glam::Vec2;

use crate::{
    anime::Anime,
    assets::{self, Frame},
    display::{ComplexRender, ComplexRenderOption},
};

pub type Program = &'static [Frame];

pub const ALL_PROGRAMS: &[Program] = &[
    &assets::FRAMES_PC_PROGRAM_OS,
    &assets::FRAMES_PC_PROGRAM_TIC_TAC_TOE,
    &assets::FRAMES_PC_PROGRAM_RTS,
];

pub enum PcKind {
    Desktop,
}

pub struct PcRender {
    pub pos: Vec2,
    pub kind: PcKind,
    pub pc_anime: Anime,
    pub program_anime: Anime,
}

impl PcRender {
    pub fn new(kind: PcKind, pos: Vec2, program: Program) -> Self {
        Self {
            kind,
            pc_anime: Anime::new(&assets::FRAMES_PC_DESKTOP),
            pos,
            program_anime: Anime::new(program),
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        self.pc_anime.tick(delta);
        self.program_anime.tick(delta);
    }

    pub fn change_random_program(&mut self, rng: &mut fastrand::Rng) {
        self.program_anime = Anime::new(rng.choice(ALL_PROGRAMS.iter()).unwrap());
    }
}

impl ComplexRender for PcRender {
    fn render(&self, display: &mut crate::display::GameDisplay) {
        let pc_x = self.pos.x as i32 - self.pc_anime.current_frame().size.x as i32 / 2;

        display.render_image_complex(
            pc_x,
            self.pos.y as i32,
            self.pc_anime.current_frame(),
            ComplexRenderOption::new().with_white().with_center(),
        );

        let screen_x = self.pos.x as i32 + assets::IMAGE_PC_DESKTOP_SCREEN.size.x as i32 / 2 + 2;
        let screen_y = self.pos.y as i32
            - (assets::IMAGE_PC_DESKTOP_0.size.y - assets::IMAGE_PC_DESKTOP_SCREEN.size.y) as i32;

        display.render_image_complex(
            screen_x,
            screen_y,
            &assets::IMAGE_PC_DESKTOP_SCREEN,
            ComplexRenderOption::new().with_white().with_center(),
        );
        display.render_image_complex(
            screen_x,
            screen_y,
            &assets::IMAGE_PC_DESKTOP_SCREEN_MASK,
            ComplexRenderOption::new().with_black().with_center(),
        );

        let screen_x_top_left = screen_x - assets::IMAGE_PC_DESKTOP_SCREEN.size.x as i32 / 2;
        let screen_y_top_left = screen_y - assets::IMAGE_PC_DESKTOP_SCREEN.size.y as i32 / 2;

        display.render_image_complex(
            screen_x_top_left + 2,
            screen_y_top_left + 2,
            self.program_anime.current_frame(),
            ComplexRenderOption::new().with_white().with_black(),
        );

        display.render_image_complex(
            screen_x_top_left,
            screen_y + assets::IMAGE_PC_DESKTOP_SCREEN.size.y as i32 / 2 + 2,
            &assets::IMAGE_PC_KEYBOARD,
            ComplexRenderOption::new().with_white().with_black(),
        );
        display.render_image_complex(
            screen_x,
            screen_y,
            &assets::IMAGE_PC_DESKTOP_SCREEN_MASK,
            ComplexRenderOption::new().with_black().with_center(),
        );
    }
}
