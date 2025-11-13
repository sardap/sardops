use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use chrono::DateTime;
use chrono::Utc;
use euclid::Box2D;
use euclid::Point2D;
use flate2::write::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::Level;
use notan::draw::*;
use notan::prelude::*;
use sdop_game::ButtonState;
use sdop_game::Game;
use sdop_game::Timestamp;
use std::io::Write;
use std::time::Duration;
use wasm_cookies::*;

pub fn timestamp() -> Timestamp {
    Timestamp::new(chrono::Local::now().naive_local())
}

const SCALE: u32 = 5;
const GAME_WIDTH: u32 = sdop_game::WIDTH as u32 * SCALE;
const GAME_HEIGHT: u32 = sdop_game::HEIGHT as u32 * SCALE;
const CONTROL_HEIGHT: u32 = 70;
const COOKIE_NAME: &'static str = "sdop_save";

const LEFT_BUTTON: Box2D<f32, f32> = Box2D::new(
    Point2D::new(0., GAME_HEIGHT as f32 + 5.),
    Point2D::new(GAME_WIDTH as f32 / 3., CONTROL_HEIGHT as f32),
);

const MIDDLE_BUTTON: Box2D<f32, f32> = Box2D::new(
    Point2D::new(GAME_WIDTH as f32 / 3., GAME_HEIGHT as f32 + 5.),
    Point2D::new(GAME_WIDTH as f32 / 3., CONTROL_HEIGHT as f32),
);

const RIGHT_BUTTON: Box2D<f32, f32> = Box2D::new(
    Point2D::new((GAME_WIDTH as f32 / 3.) * 2., GAME_HEIGHT as f32 + 5.),
    Point2D::new(GAME_WIDTH as f32 / 3., CONTROL_HEIGHT as f32),
);

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
        .add_config(WindowConfig::new().set_size(GAME_WIDTH, GAME_HEIGHT + CONTROL_HEIGHT))
        .add_config(DrawConfig)
        .update(update)
        .draw(draw)
        .build()
}

fn setup(gfx: &mut Graphics) -> State {
    let mut game = Game::new(timestamp());

    let mut loaded = false;
    if let Some(cookie) = wasm_cookies::get(COOKIE_NAME) {
        if let Ok(encoded) = cookie {
            if let Ok(base64_decoded) = BASE64_STANDARD.decode(encoded) {
                let mut decompressed = vec![];
                {
                    let mut decoder = GzDecoder::new(&mut decompressed);
                    decoder.write_all(&base64_decoded).unwrap();
                }
                if let Ok((save, _)) =
                    bincode::decode_from_slice(&decompressed, bincode::config::standard())
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
    let (x, y) = mouse.position();
    let pos: Point2D<f32, f32> = Point2D::new(x, y);

    if mouse.left_is_down() {
        if pos.x > LEFT_BUTTON.min.x
            && pos.x < LEFT_BUTTON.min.x + LEFT_BUTTON.max.x
            && pos.y > LEFT_BUTTON.min.y
        {
            return [
                sdop_game::ButtonState::Down,
                sdop_game::ButtonState::Up,
                sdop_game::ButtonState::Up,
            ];
        }

        if pos.x > MIDDLE_BUTTON.min.x
            && pos.x < MIDDLE_BUTTON.min.x + MIDDLE_BUTTON.max.x
            && pos.y > MIDDLE_BUTTON.min.y
        {
            return [
                sdop_game::ButtonState::Up,
                sdop_game::ButtonState::Down,
                sdop_game::ButtonState::Up,
            ];
        }

        if pos.x > RIGHT_BUTTON.min.x
            && pos.x < RIGHT_BUTTON.min.x + RIGHT_BUTTON.max.x
            && pos.y > RIGHT_BUTTON.min.y
        {
            return [
                sdop_game::ButtonState::Up,
                sdop_game::ButtonState::Up,
                sdop_game::ButtonState::Down,
            ];
        }
    }

    return [
        sdop_game::ButtonState::Up,
        sdop_game::ButtonState::Up,
        sdop_game::ButtonState::Up,
    ];
}

fn update(app: &mut App, state: &mut State) {
    let delta = chrono::Utc::now() - state.last_update;
    state.last_update = chrono::Utc::now();

    let keyboard_input = buttons_to_input(&app.keyboard);
    let mouse_input = mouse_to_input(&app.mouse);

    let mut merged = keyboard_input;
    for (i, input) in mouse_input.into_iter().enumerate() {
        if input == ButtonState::Down {
            merged[i] = ButtonState::Down;
        }
    }

    state.game.update_input_states(merged);

    state.game.tick(delta.to_std().unwrap());

    if chrono::Utc::now() - state.last_save > chrono::Duration::seconds(5) {
        if let Some(save) = sdop_game::SaveFile::gen_save_bytes(timestamp(), &state.game) {
            match save {
                Ok(bytes) => {
                    let mut comprressed = vec![];
                    {
                        let mut encoder = GzEncoder::new(&mut comprressed, Compression::default());
                        encoder.write_all(&bytes).unwrap();
                    }
                    let base64_encoded = BASE64_STANDARD.encode(comprressed);
                    wasm_cookies::set(
                        COOKIE_NAME,
                        &base64_encoded,
                        &CookieOptions::default()
                            .with_same_site(SameSite::Strict)
                            .expires_after(Duration::from_secs(60 * 60 * 24 * 350)),
                    );
                    state.last_save = chrono::Utc::now();
                }
                Err(err) => {
                    panic!("Error wirting save {}", err);
                }
            }
        }
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

    draw.rect((0., GAME_HEIGHT as f32), (GAME_WIDTH as f32, 5.))
        .color(Color::WHITE);

    draw.rect(
        (LEFT_BUTTON.min.x, LEFT_BUTTON.min.y),
        (LEFT_BUTTON.max.x, LEFT_BUTTON.max.y),
    )
    .color(Color::new(0., 0.275, 0.54, 1.));

    draw.rect(
        (MIDDLE_BUTTON.min.x, MIDDLE_BUTTON.min.y),
        (MIDDLE_BUTTON.max.x, MIDDLE_BUTTON.max.y),
    )
    .color(Color::new(0.545, 0., 0., 1.));

    draw.rect(
        (RIGHT_BUTTON.min.x, RIGHT_BUTTON.min.y),
        (RIGHT_BUTTON.max.x, RIGHT_BUTTON.max.y),
    )
    .color(Color::new(0., 0.545, 0.275, 1.));

    gfx.render(&draw);
}
