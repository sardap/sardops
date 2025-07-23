#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use agb::{
    display::{bitmap3::Bitmap3, busy_wait_for_vblank},
    input::{Button, ButtonController},
};
use sdop_game::Timestamp;

const BASE_WIDTH: u32 = sdop_game::WIDTH as u32;
const BASE_HEIGHT: u32 = sdop_game::HEIGHT as u32;

const SCREEN_WIDTH: i32 = 240;
const SCREEN_HEIGHT: i32 = 160;
const OFFSET_X: i32 = (SCREEN_WIDTH - BASE_WIDTH as i32) / 2;
const OFFSET_Y: i32 = (SCREEN_HEIGHT - BASE_HEIGHT as i32) / 2;

const FRAME_TIME_MS: f32 = 16.667 * 2.0;

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
    let mut game = sdop_game::Game::new(Timestamp::default());

    let mut elapsed_time = 0f32;

    let mut button_controller = ButtonController::new();
    let mut gfx = unsafe { Bitmap3::new() };

    gfx.clear(0x7C00);

    loop {
        elapsed_time += FRAME_TIME_MS;
        button_controller.update();

        game.update_input_states(buttons_to_input(&button_controller));
        game.tick(Timestamp::from_millis(elapsed_time as u64));

        agb::println!("time elapsed {}", elapsed_time);

        let display = game.display(Timestamp::from_millis(elapsed_time as u64));
        for (byte_index, byte_value) in display.raw().iter().enumerate() {
            let start_x = (byte_index % (BASE_WIDTH as usize / 8)) * 8;
            let y = byte_index / (BASE_WIDTH as usize / 8);
            for bit_index in 0..8 {
                let x = start_x + bit_index;
                let screen_x = x as i32 + OFFSET_X;
                let screen_y = y as i32 + OFFSET_Y;

                // Only draw if within screen bounds and game area
                if screen_x >= 0
                    && screen_x < SCREEN_WIDTH
                    && screen_y >= 0
                    && screen_y < SCREEN_HEIGHT
                {
                    if (byte_value >> bit_index) & 1 == 1 {
                        // Draw white pixel (0xFFFF) for set bits
                        gfx.draw_point(screen_x, screen_y, 0xFFFF);
                    } else {
                        // Draw black pixel (0x0000) for unset bits
                        gfx.draw_point(screen_x, screen_y, 0x0000);
                    }
                }
            }
        }

        busy_wait_for_vblank();
    }
}
