use std::time::Instant;

/// Bitmap Graphics example.
///
/// This example uses the CPU to render a simple bitmap image to the screen.
use ctru::prelude::*;
use ctru::services::gfx::{Flush, Screen, Swap};
use sdop_game::{HEIGHT, Timestamp};

/// Ferris image taken from <https://rustacean.net> and scaled down to 320x240px.
/// To regenerate the data, you will need to install `imagemagick` and run this
/// command from the `examples` directory:
///
/// ```sh
/// magick assets/ferris.png -channel-fx "red<=>blue" -rotate 90 assets/ferris.rgb
/// ```
///
/// This creates an image appropriate for the default frame buffer format of
/// [`Bgr8`](ctru::services::gspgpu::FramebufferFormat::Bgr8)
/// and rotates the image 90° to account for the portrait mode screen.
// static IMAGE: &[u8] = include_bytes!("assets/ferris.rgb");

const TOP_SCREEN_WIDTH: usize = 800;
const TOP_SCREEN_HEIGHT: usize = 240;
const OFFSET_X: usize = 0; //(TOP_SCREEN_WIDTH - sdop_game::WIDTH) / 2;
const OFFSET_Y: usize = 0; //(TOP_SCREEN_HEIGHT - sdop_game::HEIGHT) / 2;

fn draw_pixel(pixels: &mut [u8], x: usize, y: usize, r: u8, g: u8, b: u8) {
    let draw_y = y + TOP_SCREEN_HEIGHT;
    let draw_x = x;
    let i: usize = (draw_y + draw_x * TOP_SCREEN_HEIGHT) * 3;
    pixels[i] = b;
    pixels[i + 1] = g;
    pixels[i + 2] = r;
}

fn main() {
    let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
    let mut hid = Hid::new().expect("Couldn't obtain HID controller");
    let apt = Apt::new().expect("Couldn't obtain APT controller");
    let _console = Console::new(gfx.bottom_screen.borrow_mut());

    println!("\x1b[21;4HPress A to flip the image.");
    println!("\x1b[29;16HPress Start to exit");

    let mut bottom_screen = gfx.top_screen.borrow_mut();

    // We don't need double buffering in this example.
    // In this way we can draw our image only once on screen.
    bottom_screen.set_double_buffering(false);
    // Swapping buffers commits the change from the line above.
    bottom_screen.swap_buffers();

    const PIXEL_COUNT: usize = TOP_SCREEN_WIDTH * TOP_SCREEN_HEIGHT;
    let mut pixel_data = [0; PIXEL_COUNT * 3];
    for i in 0..PIXEL_COUNT {
        pixel_data[i * 3] = 0xFF;
        pixel_data[i * 3 + 1] = 0x00;
        pixel_data[i * 3 + 2] = 0x00;
    }

    // Draw solid cube in the top left coner
    for x in 0..100 {
        for y in 0..100 {
            draw_pixel(&mut pixel_data, x, y, 0, 0xFF, 0);
        }
    }

    let mut game = sdop_game::Game::new(Timestamp::from_millis(
        Instant::now().elapsed().as_millis() as u64,
    ));

    // We assume the image is the correct size already, so we drop width + height.
    // We copy the initial image to the framebuffer.
    {
        let frame_buffer = bottom_screen.raw_framebuffer();
        unsafe {
            frame_buffer
                .ptr
                .copy_from(pixel_data.as_ptr(), pixel_data.len());
        }
    }

    while apt.main_loop() {
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            break;
        }

        game.tick(Timestamp::from_millis(
            Instant::now().elapsed().as_millis() as u64
        ));

        let display = game.display();
        // Center the game display within the top screen
        for (byte_index, byte_value) in display.iter().enumerate() {
            let start_x = (byte_index % (sdop_game::WIDTH as usize / 8)) * 8;
            let y = byte_index / (sdop_game::WIDTH as usize / 8);
            for bit_index in 0..8 {
                let x = start_x + bit_index;
                let screen_x = x as i32 + OFFSET_X as i32;
                let screen_y = y as i32 + OFFSET_Y as i32;

                // Only draw if within screen bounds and game area
                if screen_x >= 0
                    && screen_x < TOP_SCREEN_WIDTH as i32
                    && screen_y >= 0
                    && screen_y < TOP_SCREEN_HEIGHT as i32
                {
                    let screen_x = screen_x as usize;
                    let screen_y = screen_y as usize;

                    if (byte_value >> bit_index) & 1 == 1 {
                        // Draw white pixel (0xFFFF) for set bits
                        draw_pixel(&mut pixel_data, screen_x, screen_y, 0xFF, 0xFF, 0xFF);
                    } else {
                        // Draw black pixel (0x0000) for unset bits
                        draw_pixel(&mut pixel_data, screen_x, screen_y, 0x00, 0x00, 0x00);
                    }
                }
            }
        }

        // Update the framebuffer with new pixel data
        {
            let frame_buffer = bottom_screen.raw_framebuffer();
            unsafe {
                frame_buffer
                    .ptr
                    .copy_from(pixel_data.as_ptr(), pixel_data.len());
            }
        }

        // Flush framebuffers. Since we're not using double buffering,
        // this will render the pixels immediately
        bottom_screen.flush_buffers();

        gfx.wait_for_vblank();
    }
}
