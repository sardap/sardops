#![feature(generic_const_exprs)]
#![feature(const_index)]
#![feature(const_trait_impl)]
#![no_std]
#![no_main]

use core::mem::MaybeUninit;
use core::time::Duration;

use chrono::NaiveDateTime;
use psp::sys::{self, CtrlButtons, SceCtrlData, ScePspDateTime};
use psp::{SCREEN_HEIGHT, SCREEN_WIDTH};
use sdop_game::Timestamp;

use embedded_graphics::pixelcolor::{BinaryColor, Rgb888};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;

use psp::embedded_graphics::Framebuffer;

psp::module!("sample_module", 1, 1);

const FRAME_TIME_MS: f32 = 16.667;
const OFFSET_X: usize = 0;
const OFFSET_Y: usize = 0;

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
        let now = NaiveDateTime::new(naive_date, navie_time).and_utc();
        let duration = Duration::from_secs(now.timestamp() as u64)
            + Duration::from_nanos(now.timestamp_subsec_nanos() as u64);
        return Timestamp::from_duration(duration);
    }
}

fn buttons_to_input(controller: &mut SceCtrlData) -> sdop_game::ButtonStates {
    [
        if controller.buttons.contains(CtrlButtons::LEFT) {
            sdop_game::ButtonState::Down
        } else {
            sdop_game::ButtonState::Up
        },
        if controller.buttons.contains(CtrlButtons::UP) {
            sdop_game::ButtonState::Down
        } else {
            sdop_game::ButtonState::Up
        },
        if controller.buttons.contains(CtrlButtons::RIGHT) {
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

    let mut game = sdop_game::Game::new(get_timestamp());

    let pad_data = &mut SceCtrlData::default();

    psp::enable_home_button();
    unsafe {
        sys::sceDisplaySetMode(
            sys::DisplayMode::Lcd,
            SCREEN_WIDTH as usize,
            SCREEN_HEIGHT as usize,
        );
    }

    unsafe {
        loop {
            sys::sceDisplayWaitVblankStart();
            psp::sys::sceCtrlReadBufferPositive(pad_data, 1);
            // Convert the tick to an instance of `ScePspDateTime`
            let tick_timestamp = get_timestamp();

            game.update_input_states(buttons_to_input(pad_data));
            game.tick(tick_timestamp);
            game.refresh_display(tick_timestamp);

            game.drawable(|c| match c {
                BinaryColor::On => Rgb888::WHITE,
                BinaryColor::Off => Rgb888::BLACK,
            })
            .draw(&mut disp)
            .unwrap();
        }
    }
}
