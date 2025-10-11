#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use core::time::Duration;

use agb::{
    display::{bitmap3::Bitmap3, busy_wait_for_vblank},
    input::{Button, ButtonController},
};
use time::{Date, Month, PrimitiveDateTime, Time};

const BASE_WIDTH: u32 = sdop_game::WIDTH as u32;
const BASE_HEIGHT: u32 = sdop_game::HEIGHT as u32;

const SCREEN_WIDTH: i32 = 240;
const SCREEN_HEIGHT: i32 = 160;
const OFFSET_X: i32 = (SCREEN_WIDTH - BASE_WIDTH as i32) / 2;
const OFFSET_Y: i32 = (SCREEN_HEIGHT - BASE_HEIGHT as i32) / 2;

const FRAME_TIME_MS: Duration = Duration::from_nanos(16666666);

fn buttons_to_input(controller: &ButtonController) -> sdop_game::ButtonStates {
    [
        if controller.is_pressed(Button::LEFT) {
            sdop_game::ButtonState::Down
        } else {
            sdop_game::ButtonState::Up
        },
        if controller.is_pressed(Button::UP) {
            sdop_game::ButtonState::Down
        } else {
            sdop_game::ButtonState::Up
        },
        if controller.is_pressed(Button::RIGHT) {
            sdop_game::ButtonState::Down
        } else {
            sdop_game::ButtonState::Up
        },
    ]
}

#[agb::entry]
fn entry(mut _gba: agb::Gba) -> ! {
    let mut game = sdop_game::Game::blank(None);

    let mut button_controller = ButtonController::new();
    let mut bitmap_gfx = unsafe { Bitmap3::new() };

    bitmap_gfx.clear(0x7C00);

    let mut color = 0x0;

    loop {
        button_controller.update();

        game.update_input_states(buttons_to_input(&button_controller));
        game.tick(FRAME_TIME_MS);
        game.refresh_display(FRAME_TIME_MS);

        // color += 1;

        // bitmap_gfx.clear(color);

        for (byte_index, byte_value) in game.get_display_image_data().iter().enumerate() {
            let start_x = (byte_index % (sdop_game::WIDTH as usize / 8)) * 8;
            let y = byte_index / (sdop_game::WIDTH as usize / 8);
            for bit_index in 0..8 {
                let x = start_x + bit_index;

                let rotated_x = x;
                let rotated_y = sdop_game::HEIGHT - 1 - y;

                let screen_x = rotated_x as i32 + OFFSET_X as i32;
                let screen_y = rotated_y as i32 + OFFSET_Y as i32;

                if screen_x >= 0
                    && screen_x + 2 < SCREEN_WIDTH as i32
                    && screen_y >= 0
                    && screen_y + 2 < SCREEN_HEIGHT as i32
                {
                    let screen_x = screen_x;
                    let screen_y = screen_y;

                    let is_set = (byte_value >> (7 - bit_index)) & 1 == 1;
                    let value = if is_set { 0xFFFF } else { 0x0000 };

                    bitmap_gfx.draw_point(screen_x, screen_y, value);
                }
            }
        }

        busy_wait_for_vblank();
    }
}
