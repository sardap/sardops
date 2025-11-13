// These linker flags are only necessary with Northfear SDL2 fork

#[link(name = "SDL2", kind = "static")]
#[link(name = "vitaGL", kind = "static")]
#[link(name = "vitashark", kind = "static")]
#[link(name = "SceShaccCg_stub", kind = "static")]
#[link(name = "mathneon", kind = "static")]
#[link(name = "SceShaccCgExt", kind = "static")]
#[link(name = "taihen_stub", kind = "static")]
#[link(name = "SceKernelDmacMgr_stub", kind = "static")]
#[link(name = "SceIme_stub", kind = "static")]
unsafe extern "C" {}

use std::time::Instant;

use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use sdl2::{controller::Button, event::Event};
use sdop_game::{ButtonStates, SaveFile, Timestamp};

fn get_timestamp() -> Timestamp {
    return Timestamp::new(chrono::Local::now().naive_local());
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let controller_subsystem = sdl_context.game_controller()?;

    let available = controller_subsystem
        .num_joysticks()
        .map_err(|e| format!("can't enumerate joysticks: {}", e))?;
    let _controller = (0..available)
        .find_map(|id| {
            if !controller_subsystem.is_game_controller(id) {
                return None;
            }

            controller_subsystem.open(id).ok()
        })
        .expect("Couldn't open any controller");

    let mut game = sdop_game::Game::blank(Some(get_timestamp()));
    let save_bytes = include_bytes!("../sdop.sav");
    if let Ok(mut save) = SaveFile::from_bytes(save_bytes) {
        save.last_timestamp = get_timestamp();
        game.load_save(get_timestamp(), save);
    }

    let window = video_subsystem
        .window("Sardops", 960, 544)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    let texture_creator: TextureCreator<_> = canvas.texture_creator();
    let mut input: ButtonStates = [sdop_game::ButtonState::Up; 3];
    let mut last_frame_time = Instant::now();
    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        let delta = last_frame_time.elapsed();
        last_frame_time = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::ControllerButtonDown { button, .. } => match button {
                    Button::A => {
                        input[sdop_game::Button::Left.index()] = sdop_game::ButtonState::Down
                    }
                    Button::B => {}
                    Button::X => {
                        input[sdop_game::Button::Middle.index()] = sdop_game::ButtonState::Down
                    }
                    Button::Y => {
                        input[sdop_game::Button::Right.index()] = sdop_game::ButtonState::Down
                    }
                    _ => {}
                },
                Event::ControllerButtonUp { button, .. } => match button {
                    Button::A => {
                        input[sdop_game::Button::Left.index()] = sdop_game::ButtonState::Up
                    }
                    Button::B => {}
                    Button::X => {
                        input[sdop_game::Button::Middle.index()] = sdop_game::ButtonState::Up
                    }
                    Button::Y => {
                        input[sdop_game::Button::Right.index()] = sdop_game::ButtonState::Up
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        const SCALE: u32 = 7;
        let texture = texture_creator
            .load_texture_bytes(game.get_display_bmp())
            .unwrap();
        canvas.set_draw_color(Color::RGB(10, 90, 89));
        canvas.clear();
        canvas
            .copy_ex(
                &texture,
                None,
                Some(Rect::new(
                    255,
                    -175,
                    sdop_game::WIDTH as u32 * SCALE,
                    sdop_game::HEIGHT as u32 * SCALE,
                )),
                -90.0, // rotation angle in degrees
                // Some(Point::new(0, 0)), // rotation center (None = center of dest rect)
                None,
                false, // flip horizontally
                false, // flip vertically
            )
            .unwrap();
        canvas.present();

        game.update_input_states(input);

        game.tick(delta);
        game.refresh_display(delta);
    }

    Ok(())
}
