extern crate sdl2;

use log::info;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdop_game::ButtonStates;
use sdop_game::SaveFile;
use sdop_game::Timestamp;
use std::io::Read;
use std::io::Write;
use std::time::{Duration, Instant};

const BASE_WIDTH: u32 = sdop_game::WIDTH as u32;
const BASE_HEIGHT: u32 = sdop_game::HEIGHT as u32;

const SAVE_FILE_NAME: &str = "sdop.sav";

pub fn timestamp() -> Timestamp {
    Timestamp::new(chrono::Local::now().naive_local())
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

    let mut game = sdop_game::Game::new(timestamp());
    let mut time_scale = 1.0f32;
    let mut loaded = false;
    if let Ok(mut file) = std::fs::File::open(SAVE_FILE_NAME) {
        let mut bytes = vec![];
        if file.read_to_end(&mut bytes).is_ok() {
            match SaveFile::load_from_bytes(&bytes, timestamp(), &mut game) {
                Ok(_) => {
                    loaded = true;
                    log::info!("Loadded save!")
                }
                Err(err) => log::error!("Error Loading save {}", err),
            }
        }
    }

    if !loaded {
        game = sdop_game::Game::blank(Some(timestamp()));
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
        let delta = last_frame_time.elapsed();
        last_frame_time = Instant::now();
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

        // HERE add weather input

        game.tick(delta);
        game.refresh_display(delta);
        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .load_texture_bytes(game.get_display_bmp())
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
        if since_save > Duration::from_secs(1)
            && let Some(save) = SaveFile::gen_save_bytes(timestamp(), &game)
        {
            match save {
                Ok(bytes) => {
                    let mut fs = std::fs::File::create(SAVE_FILE_NAME).unwrap();
                    let _ = fs.write_all(&bytes);
                    last_save_time = Instant::now();
                }
                Err(err) => {
                    panic!("Error wirting save {}", err);
                }
            }
        }

        // Frame timing - only sleep if we have time left in the frame
        let frame_elapsed = last_frame_time.elapsed();
        if frame_elapsed < FRAME_TIME {
            let sleep_time = FRAME_TIME - frame_elapsed;
            std::thread::sleep(sleep_time);
        }
    }
}
