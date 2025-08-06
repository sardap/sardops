#![no_std]
#![no_main]

use core::time::Duration;

use hal::block::ImageDef;
use panic_halt as _;
use rp235x_hal as hal;

/// Tell the Boot ROM about our application
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = hal::block::ImageDef::secure_exe();

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

use embedded_graphics::prelude::*;
use hal::fugit::RateExtU32;
use hal::gpio::{FunctionI2C, Pin};
use rp235x_hal::{timer::TimerDevice, Timer};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

fn timestamp<D: TimerDevice>(
    timestamp: sdop_game::Timestamp,
    timer: &Timer<D>,
) -> sdop_game::Timestamp {
    let ticks = timer.get_counter().ticks();
    let micros = ticks / 1; // 1 tick = 1 µs at 1 MHz

    timestamp + Duration::from_micros(micros as u64)
}

#[hal::entry]
fn main() -> ! {
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

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate90)
        .into_buffered_graphics_mode();

    display.init().unwrap();

    let start_timestamp = sdop_game::Timestamp::from_parts(1991, 12, 20, 10, 0, 0, 0).unwrap();

    let mut game = sdop_game::Game::new(timestamp(start_timestamp, &timer));

    loop {
        let ticks = timer.get_counter().ticks();
        let micros = ticks / 1; // 1 tick = 1 µs at 1 MHz
        let delta = Duration::from_micros(micros as u64);

        game.tick(delta);
        game.refresh_display(delta);

        game.drawable(|c| c).draw(&mut display).unwrap();
        display.flush().unwrap();
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
