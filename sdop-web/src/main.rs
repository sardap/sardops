use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::DateTime;
use chrono::Utc;
use log::Level;
use notan::draw::*;
use notan::prelude::*;
use sdop_game::Game;
use sdop_game::Timestamp;
use std::time::Duration;
use wasm_cookies::*;

pub fn timestamp() -> Timestamp {
    Timestamp::new(chrono::Local::now().naive_local())
}

const SCALE: u32 = 5;
const COOKIE_NAME: &'static str = "sdop_save";

#[derive(AppState)]
struct State {
    game: Game,
    last_save: DateTime<Utc>,
    last_update: DateTime<Utc>,
    last_render: DateTime<Utc>,
}

#[notan_main]
fn main() -> Result<(), String> {
    console_log::init_with_level(Level::Debug);

    notan::init_with(setup)
        .add_config(WindowConfig::new().set_size(
            sdop_game::WIDTH as u32 * SCALE,
            sdop_game::HEIGHT as u32 * SCALE,
        ))
        .add_config(DrawConfig)
        .update(update)
        .draw(draw)
        .build()
}

fn setup(gfx: &mut Graphics) -> State {
    let mut game = Game::new(timestamp());

    let mut loaded = false;
    #[cfg(target_arch = "wasm32")]
    if let Some(cookie) = wasm_cookies::get(COOKIE_NAME) {
        if let Ok(encoded) = cookie {
            if let Ok(base64_decoded) = BASE64_STANDARD.decode(encoded) {
                if let Ok((save, _)) =
                    bincode::decode_from_slice(&base64_decoded, bincode::config::standard())
                {
                    game.load_save(timestamp(), save);
                    loaded = true;
                }
            }
        }
    }

    if !loaded {
        game = Game::blank(Some(timestamp()));
    }

    State {
        game,
        last_save: chrono::Utc::now(),
        last_render: chrono::Utc::now(),
        last_update: chrono::Utc::now(),
    }
}

fn buttons_to_input(keyboard: &Keyboard) -> sdop_game::ButtonStates {
    [
        if keyboard.is_down(KeyCode::Q) {
            sdop_game::ButtonState::Down
        } else {
            sdop_game::ButtonState::Up
        },
        if keyboard.is_down(KeyCode::W) {
            sdop_game::ButtonState::Down
        } else {
            sdop_game::ButtonState::Up
        },
        if keyboard.is_down(KeyCode::E) {
            sdop_game::ButtonState::Down
        } else {
            sdop_game::ButtonState::Up
        },
    ]
}

// Do all this mouse shit when you can be bothered

fn mouse_to_input(mouse: &Mouse) -> sdop_game::ButtonStates {
    todo!()
}

fn update(app: &mut App, state: &mut State) {
    let delta = chrono::Utc::now() - state.last_update;
    state.last_update = chrono::Utc::now();

    state
        .game
        .update_input_states(buttons_to_input(&app.keyboard));
    // state.game.update_input_states(mouse_to_input(&app.mouse));

    state.game.tick(delta.to_std().unwrap());

    #[cfg(target_arch = "wasm32")]
    if chrono::Utc::now() - state.last_save > chrono::Duration::seconds(1) {
        let save = state.game.get_save(timestamp());
        let encoded_save = bincode::encode_to_vec(save, bincode::config::standard()).unwrap();
        let base64_encoded = BASE64_STANDARD.encode(encoded_save);
        wasm_cookies::set(
            COOKIE_NAME,
            &base64_encoded,
            &CookieOptions::default()
                .with_same_site(SameSite::Strict)
                .expires_after(Duration::from_secs(60 * 60 * 24 * 350)),
        );
        state.last_save = chrono::Utc::now();
    }
}

fn draw(gfx: &mut Graphics, state: &mut State) {
    let delta = chrono::Utc::now() - state.last_render;
    state.last_render = chrono::Utc::now();

    let mut draw = gfx.create_draw();
    draw.clear(Color::BLACK);

    state.game.refresh_display(delta.to_std().unwrap());
    let texture = gfx
        .create_texture()
        .from_image(&state.game.get_display_bmp())
        .build()
        .unwrap();

    draw.image(&texture).size(
        sdop_game::WIDTH as f32 * SCALE as f32,
        sdop_game::HEIGHT as f32 * SCALE as f32,
    );

    gfx.render(&draw);
}
