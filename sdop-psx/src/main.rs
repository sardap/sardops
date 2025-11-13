#![no_std]
#![no_main]

use core::time::Duration;
use psx::constants::*;
use psx::format::tim::{Bitmap, TIM};
use psx::gpu::{link_list, Bpp, Clut, Color, Packet, TexCoord, Vertex};
use psx::gpu::{primitives::*, TexPage};
use psx::include_tim;
use psx::math::{f16, rotate_x, rotate_y, rotate_z, Rad};
use psx::sys::gamepad::Gamepad;
use psx::sys::kernel;
use psx::{dma, dprintln, Framebuffer};
use sdop_game::{ButtonState, Game};

psx::no_heap!();

const WIDTH: usize = sdop_game::WIDTH as usize;
const HEIGHT: usize = sdop_game::HEIGHT as usize;
const TEXTURE_SIZE: usize = WIDTH * HEIGHT;

fn buttons_to_input(gamepad: &mut Gamepad) -> [ButtonState; 3] {
    let mut result = [ButtonState::Up; 3];
    let mut buttons = gamepad.poll_p1();
    for button in buttons {
        let index = match button {
            psx::sys::gamepad::Button::Square => 0,
            psx::sys::gamepad::Button::Circle => 1,
            psx::sys::gamepad::Button::Triangle => 2,
            _ => {
                continue;
            }
        };
        result[index] = ButtonState::Down;
    }
    result
}

#[no_mangle]
fn main() {
    let mut fb = Framebuffer::default();
    fb.set_bg_color(BLUE);
    let mut gpu_dma = dma::GPU::new();

    let mut polys_a = [const { Packet::new(PolyG4::new()) }; sdop_game::WIDTH * sdop_game::HEIGHT];
    let mut polys_b = [const { Packet::new(PolyG4::new()) }; sdop_game::WIDTH * sdop_game::HEIGHT];
    link_list(&mut polys_a);
    link_list(&mut polys_b);

    let mut game = sdop_game::Game::blank(None);

    let mut swapped = false;
    loop {
        let delta = Duration::from_nanos(16666666);
        // let mut gamepad = Gamepad::new();

        // game.update_input_states(buttons_to_input(&mut gamepad));
        game.tick(delta);
        game.refresh_display(delta);

        const OFFSET_X: u32 = 100;
        const OFFSET_Y: u32 = 50;

        swapped = !swapped;
        let (draw_polys, disp_polys) = if swapped {
            (&mut polys_a, &mut polys_b)
        } else {
            (&mut polys_b, &mut polys_a)
        };
        gpu_dma.send_list_and(disp_polys, || {
            for (byte_index, byte_value) in game.get_display_image_data().iter().enumerate() {
                let start_x = (byte_index % (sdop_game::WIDTH as usize / 8)) * 8;
                let y = byte_index / (sdop_game::WIDTH as usize / 8);
                for bit_index in 0..8 {
                    let x = start_x + bit_index;

                    let rotated_x = x;
                    let rotated_y = HEIGHT - 1 - y;

                    let screen_x = rotated_x as i32 + OFFSET_X as i32;
                    let screen_y = rotated_y as i32 + OFFSET_Y as i32;

                    let screen_x = screen_x;
                    let screen_y = screen_y;

                    let is_set = (byte_value >> (7 - bit_index)) & 1 == 1;
                    let value = if is_set { 0xFFFF } else { 0x0000 };

                    let size: i16 = 1;
                    let v0 = Vertex(screen_x as i16, screen_y as i16);
                    let v1 = Vertex(screen_x as i16 + size, screen_y as i16);
                    let v2 = Vertex(screen_x as i16, screen_y as i16 + size);
                    let v3 = Vertex(screen_x as i16 + size, screen_y as i16 + size);

                    let color = if is_set { WHITE } else { BLACK };

                    let poly = &mut draw_polys[byte_index * 8 + bit_index].contents;
                    poly.set_colors([color, color, color, color]);
                    poly.set_vertices([v0, v1, v2, v3]);
                }
            }
        });

        fb.draw_sync();
        fb.wait_vblank();
        fb.dma_swap(&mut gpu_dma);
    }
}

fn project_face(face: [[f16; 3]; 4]) -> [Vertex; 4] {
    face.map(|[x, y, z]| {
        let xp = x / (z + 64);
        let yp = y / (z + 64);
        Vertex(xp.0, yp.0) + Vertex(160, 120)
    })
}
