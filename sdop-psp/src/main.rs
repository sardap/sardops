#![feature(generic_const_exprs)]
#![feature(const_index)]
#![feature(const_trait_impl)]
#![no_std]
#![no_main]

use core::ffi::c_void;
use core::mem::MaybeUninit;
use core::time::Duration;

use embedded_graphics::image::Image;
use psp::sys::{self, sceIoMkdir, sceIoOpen, CtrlButtons, SceCtrlData, ScePspDateTime};
use psp::{SCREEN_HEIGHT, SCREEN_WIDTH};
use sdop_game::{SaveFile, Timestamp, HEIGHT, WIDTH};

use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;

use psp::embedded_graphics::Framebuffer;
use tinybmp::Bmp;

const SAVE_PATH: &[u8] = b"ms0:/PSP/SAVEDATA/SARDOPS/sardops.sav\0";
const PATHS: &[&[u8]] = &[b"ms0:/PSP/SAVEDATA/SARDOPS\0"];
psp::module!("sample_module", 1, 1);

fn get_timestamp() -> Timestamp {
    let mut tick = 0;
    unsafe {
        psp::sys::sceRtcGetCurrentTick(&mut tick);
        let mut date = MaybeUninit::uninit();
        psp::sys::sceRtcSetTick(date.as_mut_ptr(), &tick);
        let date: ScePspDateTime = date.assume_init();

        let naive_date =
            chrono::NaiveDate::from_ymd_opt(date.year as i32, date.month as u32, date.day as u32)
                .unwrap();
        let navie_time = chrono::NaiveTime::from_hms_micro_opt(
            date.hour as u32,
            date.minutes as u32,
            date.seconds as u32,
            date.microseconds,
        )
        .unwrap();
        return Timestamp::new(chrono::NaiveDateTime::new(naive_date, navie_time));
    }
}

fn buttons_to_input(controller: &mut SceCtrlData) -> sdop_game::ButtonStates {
    [
        if controller.buttons.contains(CtrlButtons::CROSS) {
            sdop_game::ButtonState::Down
        } else {
            sdop_game::ButtonState::Up
        },
        if controller.buttons.contains(CtrlButtons::SQUARE) {
            sdop_game::ButtonState::Down
        } else {
            sdop_game::ButtonState::Up
        },
        if controller.buttons.contains(CtrlButtons::TRIANGLE) {
            sdop_game::ButtonState::Down
        } else {
            sdop_game::ButtonState::Up
        },
    ]
}

fn psp_main() {
    psp::enable_home_button();
    let mut disp = Framebuffer::new();

    let style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb888::BLACK)
        .build();
    let black_backdrop = Rectangle::new(Point::new(0, 0), Size::new(160, 80)).into_styled(style);
    black_backdrop.draw(&mut disp).unwrap();

    let pad_data = &mut SceCtrlData::default();

    psp::enable_home_button();
    unsafe {
        sys::sceDisplaySetMode(
            sys::DisplayMode::Lcd,
            SCREEN_WIDTH as usize,
            SCREEN_HEIGHT as usize,
        );
    }

    let background_bmp = Bmp::from_slice(include_bytes!("../assets/background.bmp")).unwrap();
    let background_image = Image::new(&background_bmp, Point::zero());

    let background_top_bmp =
        Bmp::from_slice(include_bytes!("../assets/background_top.bmp")).unwrap();
    let background_top_image = Image::new(&background_top_bmp, Point::zero());

    let save_icon_bmp = Bmp::from_slice(include_bytes!("../assets/save_icon.bmp")).unwrap();
    let save_icon_image = Image::new(&save_icon_bmp, Point::zero());

    let loading_icon_bmp = Bmp::from_slice(include_bytes!("../assets/loading_screen.bmp")).unwrap();
    let loading_image = Image::new(&loading_icon_bmp, Point::zero());

    loading_image.draw(&mut disp).unwrap();

    // Load save
    let mut existing_save: Option<[u8; sdop_game::SAVE_SIZE]> = None;
    unsafe {
        // Create save folder
        for path in PATHS {
            sceIoMkdir(path.as_ptr(), 0o777);
        }

        let fd = sceIoOpen(SAVE_PATH.as_ptr(), sys::IoOpenFlags::RD_ONLY, 0o666);
        if fd.0 >= 0 {
            let mut buffer = [0u8; sdop_game::SAVE_SIZE];
            sys::sceIoRead(
                fd,
                &mut buffer as *mut _ as *mut c_void,
                sdop_game::SAVE_SIZE as u32,
            );
            existing_save = Some(buffer);
            sys::sceIoClose(fd);
        }
    }

    let mut game = sdop_game::Game::blank(Some(get_timestamp()));

    if let Some(save_bytes) = existing_save {
        if let Ok(save) = SaveFile::from_bytes(&save_bytes) {
            game.load_save(get_timestamp(), save);
        }
    }
    background_image.draw(&mut disp).unwrap();

    let mut background_pending = true;
    let mut showing_save = true;
    let mut last_frame = get_timestamp();
    let mut last_save = get_timestamp();
    unsafe {
        loop {
            sys::sceDisplayWaitVblankStart();
            sys::sceCtrlReadBufferPositive(pad_data, 1);

            // Convert the tick to an instance of `ScePspDateTime`
            let delta = get_timestamp() - last_frame;
            last_frame = get_timestamp();

            game.update_input_states(buttons_to_input(pad_data));
            game.tick(delta);
            game.refresh_display(delta);

            if background_pending {
                background_top_image.draw(&mut disp).unwrap();
                background_pending = false;
            }
            let save_delta = last_frame - last_save;
            if save_delta < Duration::from_secs(5) {
                save_icon_image.draw(&mut disp).unwrap();
                showing_save = true;
            } else if showing_save {
                background_pending = true;
                showing_save = false;
            }
            if save_delta > Duration::from_secs(60) {
                if let Some(save_file) = game.get_save(last_frame) {
                    if let Ok(bytes) = save_file.to_bytes() {
                        let fd = sceIoOpen(
                            SAVE_PATH.as_ptr(),
                            sys::IoOpenFlags::WR_ONLY | sys::IoOpenFlags::CREAT,
                            0o666,
                        );
                        if fd.0 > 0 {
                            sys::sceIoWrite(
                                fd,
                                bytes.as_ptr() as *const _ as *const c_void,
                                sdop_game::SAVE_SIZE,
                            );
                            sys::sceIoClose(fd);
                        }
                    }
                }

                last_save = last_frame;
            }

            const OFFSET_X: usize = 42;
            const OFFSET_Y: usize = 40;
            const SCALE: usize = 3;
            for (byte_index, byte_value) in game.get_display_image_data().iter().enumerate() {
                let start_x = (byte_index % (sdop_game::WIDTH as usize / 8)) * 8;
                let y = byte_index / (sdop_game::WIDTH as usize / 8);
                for bit_index in 0..8 {
                    let x = start_x + bit_index;

                    let rotated_x = HEIGHT - 1 - y;
                    let rotated_y = WIDTH - 1 - x;

                    let screen_x = (rotated_x * SCALE) as i32 + OFFSET_X as i32;
                    let screen_y = (rotated_y * SCALE) as i32 + OFFSET_Y as i32;

                    let screen_x = screen_x as usize;
                    let screen_y = screen_y as usize;

                    let is_set = (byte_value >> (7 - bit_index)) & 1 == 1;

                    for dy in 0..SCALE {
                        for dx in 0..SCALE {
                            let pixel = Pixel(
                                Point::new((screen_x + dx) as i32, (screen_y + dy) as i32),
                                if is_set { Rgb888::WHITE } else { Rgb888::BLACK },
                            );
                            pixel.draw(&mut disp).unwrap();
                        }
                    }
                }
            }
        }
    }
}
