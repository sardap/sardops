use core::time::Duration;

use glam::Vec2;
use heapless::Vec;

use crate::{
    anime::Anime,
    assets::{self, Frame},
    display::{ComplexRender, ComplexRenderOption},
    items::{ALL_PROGRAMS, Inventory, ItemKind, PROGRAM_COUNT},
    pet::PetInstance,
};

const KEY_X_LOCATIONS: &[i8] = &[2, 4, 6, 8, 10, 11, 13, 15, 17, 19, 20, 21, 23];

pub type Program = &'static [Frame];

pub enum PcKind {
    Desktop,
}

pub struct PcRender {
    pub pos: Vec2,
    pub kind: PcKind,
    pub pc_anime: Anime,
    pub program_anime: Anime,
    pub since_key_change: Duration,
    pub key_change: Duration,
    pub pressed_key: u8,
}

impl PcRender {
    pub fn new(kind: PcKind, pos: Vec2, program: Program) -> Self {
        Self {
            kind,
            pc_anime: Anime::new(&assets::FRAMES_PC_DESKTOP),
            pos,
            program_anime: Anime::new(program),
            since_key_change: Duration::ZERO,
            key_change: Duration::ZERO,
            pressed_key: 0,
        }
    }

    pub fn tick(&mut self, delta: Duration, rng: &mut fastrand::Rng) {
        self.pc_anime.tick(delta);
        self.program_anime.tick(delta);

        self.since_key_change += delta;
        if self.since_key_change > self.key_change {
            self.key_change = Duration::from_millis(rng.u32(250..500) as u64);
            self.since_key_change = Duration::ZERO;
            self.pressed_key = rng.u8(0..(KEY_X_LOCATIONS.len() as u8));
        }
    }

    pub fn change_program(
        &mut self,
        rng: &mut fastrand::Rng,
        inventory: &Inventory,
        pet: &PetInstance,
    ) {
        let mut owned: Vec<Program, PROGRAM_COUNT> = Vec::new();
        for item in ALL_PROGRAMS {
            if inventory.has_item(item)
                && (item != ItemKind::ProgramCCompiler
                    || pet.book_history.get_read(item).completed())
            {
                let _ = owned.push(item.program().unwrap_or_default());
            }
        }
        if owned.is_empty() {
            let _ = owned.push(&assets::FRAMES_PC_PROGRAM_OS);
        }
        self.program_anime = Anime::new(rng.choice(owned.iter()).unwrap());
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

        let keyboard_y = screen_y + assets::IMAGE_PC_DESKTOP_SCREEN.size.y as i32 / 2 + 2;
        display.render_image_complex(
            screen_x_top_left,
            keyboard_y,
            &assets::IMAGE_PC_KEYBOARD,
            ComplexRenderOption::new().with_white().with_black(),
        );

        let key_pressed_x = KEY_X_LOCATIONS[self.pressed_key as usize] as i32 + screen_x_top_left;
        display.render_image_complex(
            key_pressed_x,
            keyboard_y + 8,
            &assets::IMAGE_PC_KEYBOARD_KEY_PRESSED,
            ComplexRenderOption::new()
                .with_white()
                .with_black()
                .with_bottom_left(),
        );
    }
}
