use std::time::{Duration, Instant};

/// Bitmap Graphics example.
///
/// This example uses the CPU to render a simple bitmap image to the screen.
use ctru::prelude::*;
use ctru::services::gfx::{Flush, Screen, Swap};
use sdop_game::{HEIGHT, SaveFile, Timestamp};

const TOP_SCREEN_WIDTH: usize = 800;
const TOP_SCREEN_HEIGHT: usize = 240;
const OFFSET_X: usize = 10;
const OFFSET_Y: usize = 24;
const SAVE_FILE_NAME: &'static str = "sdop.sav";

fn draw_pixel(pixels: &mut [u8], x: usize, y: usize, r: u8, g: u8, b: u8) {
    let draw_y = y + TOP_SCREEN_HEIGHT;
    let draw_x = x;
    let i: usize = (draw_y + draw_x * TOP_SCREEN_HEIGHT) * 3;
    pixels[i] = b;
    pixels[i + 1] = g;
    pixels[i + 2] = r;
}

fn timestamp() -> Timestamp {
    Timestamp::new(chrono::Local::now().naive_local())
}

// Up -> Left Right -> Up Down -> Right
fn buttons_to_input(keys: &KeyPad) -> sdop_game::ButtonStates {
    [
        // LEFt
        if keys.contains(KeyPad::DPAD_UP) {
            sdop_game::ButtonState::Down
        } else {
            sdop_game::ButtonState::Up
        },
        // Middle
        if keys.contains(KeyPad::DPAD_RIGHT) {
            sdop_game::ButtonState::Down
        } else {
            sdop_game::ButtonState::Up
        },
        // Right
        if keys.contains(KeyPad::DPAD_DOWN) {
            sdop_game::ButtonState::Down
        } else {
            sdop_game::ButtonState::Up
        },
    ]
}

fn main() {
    let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
    let mut hid = Hid::new().expect("Couldn't obtain HID controller");
    let apt = Apt::new().expect("Couldn't obtain APT controller");
    let console = Console::new(gfx.bottom_screen.borrow_mut());

    let mut top_screen = gfx.top_screen.borrow_mut();

    top_screen.set_double_buffering(false);

    top_screen.swap_buffers();

    const PIXEL_COUNT: usize = TOP_SCREEN_WIDTH * TOP_SCREEN_HEIGHT;
    let mut pixel_data = [0; PIXEL_COUNT * 3];
    for i in 0..PIXEL_COUNT {
        pixel_data[i * 3] = 0xFF;
        pixel_data[i * 3 + 1] = 0x00;
        pixel_data[i * 3 + 2] = 0x00;
    }

    let mut game = sdop_game::Game::new(timestamp());

    if let Ok(save_bytes) = std::fs::read(SAVE_FILE_NAME) {
        if let Ok((save, _)) = bincode::decode_from_slice(&save_bytes, bincode::config::standard())
        {
            game.load_save(timestamp(), save);
        }
    }

    {
        let frame_buffer = top_screen.raw_framebuffer();
        unsafe {
            frame_buffer
                .ptr
                .copy_from(pixel_data.as_ptr(), pixel_data.len());
        }
    }
    let mut last_save_time = Instant::now();

    while apt.main_loop() {
        hid.scan_input();

        let keys = hid.keys_down();

        game.update_input_states(buttons_to_input(&keys));
        game.tick(timestamp());

        game.refresh_display(timestamp());
        // Center the game display within the top screen
        const SCALE: usize = 3;
        for (byte_index, byte_value) in game.get_display_image_data().iter().enumerate() {
            let start_x = (byte_index % (sdop_game::WIDTH as usize / 8)) * 8;
            let y = byte_index / (sdop_game::WIDTH as usize / 8);
            for bit_index in 0..8 {
                let x = start_x + bit_index;

                let rotated_x = y;
                let rotated_y = sdop_game::WIDTH - 1 - x;

                let screen_x = (rotated_x * SCALE) as i32 + OFFSET_X as i32;
                let screen_y = (rotated_y * SCALE) as i32 + OFFSET_Y as i32;

                if screen_x >= 0
                    && screen_x + 2 < TOP_SCREEN_WIDTH as i32
                    && screen_y >= 0
                    && screen_y + 2 < TOP_SCREEN_HEIGHT as i32
                {
                    let screen_x = screen_x as usize;
                    let screen_y = screen_y as usize;

                    let is_set = (byte_value >> (7 - bit_index)) & 1 == 1;
                    let (r, g, b) = if is_set {
                        (0xFF, 0xFF, 0xFF)
                    } else {
                        (0x00, 0x00, 0x00)
                    };

                    for dy in 0..SCALE {
                        for dx in 0..SCALE {
                            draw_pixel(&mut pixel_data, screen_x + dx, screen_y + dy, r, g, b);
                        }
                    }
                }
            }
        }

        // Update the framebuffer with new pixel data
        {
            let frame_buffer = top_screen.raw_framebuffer();
            unsafe {
                frame_buffer
                    .ptr
                    .copy_from(pixel_data.as_ptr(), pixel_data.len());
            }
        }

        top_screen.flush_buffers();

        if last_save_time.elapsed() > Duration::from_secs(10) {
            let save = game.get_save(timestamp());
            let save_bytes = bincode::encode_to_vec(save, bincode::config::standard()).unwrap();
            let mut file = std::fs::File::create(SAVE_FILE_NAME).unwrap();
            std::io::Write::write_all(&mut file, &save_bytes);
            last_save_time = Instant::now()
        }

        gfx.wait_for_vblank();
    }
}
