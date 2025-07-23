extern crate sdl2;

use log::info;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdop_game::ButtonStates;
use sdop_game::Timestamp;
use std::time::{Duration, Instant};

const BASE_WIDTH: u32 = sdop_game::WIDTH as u32;
const BASE_HEIGHT: u32 = sdop_game::HEIGHT as u32;

const SAVE_FILE_NAME: &str = "sdop.sav";

pub fn timestamp() -> Timestamp {
    let now = chrono::Utc::now();
    let duration = Duration::from_secs(now.timestamp() as u64)
        + Duration::from_nanos(now.timestamp_subsec_nanos() as u64);
    Timestamp::from_duration(duration)
}

pub fn main() {
    env_logger::init();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let current = video_subsystem.current_display_mode(0).unwrap();

    let scale_w = (current.w as f32 / BASE_WIDTH as f32).floor() as u32;
    let scale_h = (current.h as f32 / BASE_HEIGHT as f32).floor() as u32;

    let mut scale: i32 = (scale_w.min(scale_h) as i32 - 2).max(1);

    let window = video_subsystem
        .window(
            "rust-sdl2 demo",
            BASE_WIDTH * scale as u32,
            BASE_HEIGHT * scale as u32,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    // Create a texture for the display buffer
    let mut texture = texture_creator
        .create_texture_target(PixelFormatEnum::RGB24, BASE_WIDTH, BASE_HEIGHT)
        .unwrap();

    let mut game = sdop_game::Game::new(timestamp());
    let mut time_scale = 1.0f32;
    if let Ok(file) = std::fs::File::open(SAVE_FILE_NAME) {
        use std::io::BufReader;
        let buf_reader = BufReader::new(file);
        if let Ok(save_file) = bincode::decode_from_reader(buf_reader, bincode::config::standard())
        {
            game.load_save(timestamp(), save_file);
        }
    }

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    const TARGET_FPS: u64 = 60;
    const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS);
    let mut last_frame_time = Instant::now();

    let mut last_save_time = Instant::now();

    let mut input: ButtonStates = [sdop_game::ButtonState::Up; 3];
    'running: loop {
        let loop_timestamp = timestamp();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Q => {
                        input[sdop_game::Button::Left.index()] = sdop_game::ButtonState::Down
                    }
                    Keycode::W => {
                        input[sdop_game::Button::Middle.index()] = sdop_game::ButtonState::Down
                    }
                    Keycode::E => {
                        input[sdop_game::Button::Right.index()] = sdop_game::ButtonState::Down
                    }
                    _ => {}
                },
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::EQUALS => {
                        time_scale += 1.;
                        info!("Time scale changed {time_scale}");
                    }
                    Keycode::MINUS => {
                        time_scale = (time_scale - 1.0).max(1.0);
                        info!("Time scale changed {time_scale}");
                    }
                    Keycode::P => {
                        scale = (scale + 1).max(1);
                        let _ = canvas
                            .window_mut()
                            .set_size(BASE_WIDTH * scale as u32, BASE_HEIGHT * scale as u32);
                    }
                    Keycode::O => {
                        scale = (scale - 1).max(1);
                        let _ = canvas
                            .window_mut()
                            .set_size(BASE_WIDTH * scale as u32, BASE_HEIGHT * scale as u32);
                    }
                    Keycode::Q => {
                        input[sdop_game::Button::Left.index()] = sdop_game::ButtonState::Up
                    }
                    Keycode::W => {
                        input[sdop_game::Button::Middle.index()] = sdop_game::ButtonState::Up
                    }
                    Keycode::E => {
                        input[sdop_game::Button::Right.index()] = sdop_game::ButtonState::Up
                    }
                    _ => {}
                },

                _ => {}
            }
        }
        game.set_sim_time_scale(time_scale);
        game.update_input_states(input);

        game.tick(loop_timestamp);

        let display = game.display(loop_timestamp);

        let mut pixel_data = vec![0u8; (BASE_WIDTH * BASE_HEIGHT * 3) as usize]; // RGB24 format
        for bit_index in 0..display.bit_count() {
            if display.get_bit(bit_index) {
                let x = bit_index % BASE_WIDTH as usize;
                let y = bit_index / BASE_WIDTH as usize;

                let pixel_index = (y * BASE_WIDTH as usize + x) * 3;
                if pixel_index + 2 < pixel_data.len() {
                    pixel_data[pixel_index] = 255; // R
                    pixel_data[pixel_index + 1] = 255; // G
                    pixel_data[pixel_index + 2] = 255; // B
                }
            }
        }
        texture
            .update(None, &pixel_data, (BASE_WIDTH * 3) as usize)
            .unwrap();
        canvas
            .copy(
                &texture,
                None,
                Some(Rect::new(
                    0,
                    0,
                    BASE_WIDTH * scale as u32,
                    BASE_HEIGHT * scale as u32,
                )),
            )
            .unwrap();
        canvas.present();

        let since_save = last_save_time.elapsed();
        if since_save > Duration::from_secs(1) {
            let save = game.get_save(loop_timestamp);
            let mut fs = std::fs::File::create(SAVE_FILE_NAME).unwrap();
            bincode::encode_into_std_write(save, &mut fs, bincode::config::standard()).unwrap();
            last_save_time = Instant::now();
        }

        // Frame timing - only sleep if we have time left in the frame
        let frame_elapsed = last_frame_time.elapsed();
        if frame_elapsed < FRAME_TIME {
            let sleep_time = FRAME_TIME - frame_elapsed;
            std::thread::sleep(sleep_time);
        }
        last_frame_time = Instant::now();
    }
}
