#![no_std]
#![no_main]

use core::time::Duration;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::InputPin;
use embedded_sdmmc::{TimeSource, Timestamp};
use hal::block::ImageDef;
use panic_halt as _;
use rp235x_hal::{self as hal};

/// Tell the Boot ROM about our application
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = hal::block::ImageDef::secure_exe();

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

use embedded_graphics::prelude::*;
use hal::fugit::RateExtU32;
use hal::gpio::{FunctionI2C, Pin};
use sdop_game::SaveFile;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

/// A dummy timesource, which is mostly important for creating files.
#[derive(Default)]
pub struct DummyTimesource();

impl TimeSource for DummyTimesource {
    // In theory you could use the RTC of the rp2040 here, if you had
    // any external time synchronizing device.
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

#[hal::entry]
fn main() -> ! {
    let save_bytes = include_bytes!("../sdop.sav");

    let mut pac = hal::pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut left_button = pins.gpio13.into_pull_up_input();
    let mut middle_button = pins.gpio12.into_pull_up_input();
    let mut right_button = pins.gpio11.into_pull_up_input();

    let mut timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    // The logic for the I2C & OLED starts here

    let sda_pin: Pin<_, FunctionI2C, _> = pins.gpio18.reconfigure();
    let scl_pin: Pin<_, FunctionI2C, _> = pins.gpio19.reconfigure();

    let i2c = hal::I2C::i2c1(
        pac.I2C1,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        &clocks.system_clock,
    );

    let interface = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate270)
        .into_buffered_graphics_mode();

    display.init().unwrap();

    let save_file = SaveFile::from_bytes(save_bytes).unwrap();
    let mut timestamp = save_file.last_timestamp;

    let mut game = sdop_game::Game::new(timestamp);

    SaveFile::load_from_bytes(save_bytes, timestamp, &mut game).unwrap();

    display.flush().unwrap();

    let mut last_time = timer.get_counter();
    let mut last_save = timer.get_counter();

    loop {
        let now = timer.get_counter();
        let delta = now - last_time;
        last_time = now;
        let delta = Duration::from_micros(delta.to_micros() as u64);
        timestamp = timestamp + delta;

        let inputs = [
            if left_button.is_low().unwrap_or(false) {
                sdop_game::ButtonState::Down
            } else {
                sdop_game::ButtonState::Up
            },
            if middle_button.is_low().unwrap_or(false) {
                sdop_game::ButtonState::Down
            } else {
                sdop_game::ButtonState::Up
            },
            if right_button.is_low().unwrap_or(false) {
                sdop_game::ButtonState::Down
            } else {
                sdop_game::ButtonState::Up
            },
        ];

        if Duration::from_micros((now - last_save).to_micros()) > Duration::from_secs(5) {
            last_save = now;
            // TODO
            if let Some(save) = game.get_save(timestamp) {}
        }

        game.update_input_states(inputs);
        game.tick(delta);
        game.refresh_display(delta);

        if game.drawable(|c| c).draw(&mut display).is_err() || display.flush().is_err() {
            while display.init().is_err() {}
        }

        let target_fps: u64 = if game.low_power() { 5 } else { 30 };
        let frame_time: Duration = Duration::from_nanos(1_000_000_000 / target_fps);

        let now = timer.get_counter();
        let delta = now - last_time;
        let delta = Duration::from_micros(delta.to_micros() as u64);
        if delta < frame_time {
            let sleep_time = frame_time - delta;
            timer.delay_ns(sleep_time.as_nanos() as u32);
        }
    }
}

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"PWM Blinky Example"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];
