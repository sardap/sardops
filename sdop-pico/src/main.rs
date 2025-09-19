#![no_std]
#![no_main]

use core::cell::RefCell;
use core::time::Duration;

use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config as AdcConfig};
use embassy_rp::block::ImageDef;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::peripherals::{self, I2C0, I2C1};
use embassy_rp::{self as hal, bind_interrupts};
use embassy_rp::{adc, gpio, i2c};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Instant, Timer};
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use fixedstr::str_format;
use sdop_game::SaveFile;
use ssd1306::mode::{BufferedGraphicsMode, DisplayConfig};
use ssd1306::prelude::{DisplayRotation, I2CInterface};
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use {defmt_rtt as _, panic_probe as _};

/// Tell the Boot ROM about our application
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = hal::block::ImageDef::secure_exe();

const REFERENCE_VOLTAGE: f32 = 3.3;
const STEPS_12BIT: f32 = 4096 as f32;

fn adc_reading_to_voltage(reading_12bit: u16) -> f32 {
    (reading_12bit as f32 / STEPS_12BIT) * REFERENCE_VOLTAGE
}

fn temp36_c(adc_reading: u16) -> f32 {
    let voltage: f32 = adc_reading_to_voltage(adc_reading);
    let c = (100.0 * voltage) - 50.0;
    c
}

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => adc::InterruptHandler;
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

type Display = Ssd1306<
    I2CInterface<i2c::I2c<'static, peripherals::I2C0, i2c::Async>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

static SHARED_TEMPTURE: Mutex<ThreadModeRawMutex, f32> = Mutex::new(sdop_game::ROOM_TEMPTURE);

#[embassy_executor::task]
async fn tempeture_task(
    mut adc: Adc<'static, embassy_rp::adc::Async>,
    mut adc_channel: Channel<'static>,
) {
    const HISTORY: usize = 10;

    let mut buf = [0u16; HISTORY];
    let mut idx = 0;

    loop {
        {
            // let the ADC input settle
            Timer::after(embassy_time::Duration::from_micros(10)).await;

            let mut total: u32 = 0;
            for _ in 0..16 {
                total += (adc.read(&mut adc_channel).await.unwrap()) as u32;
            }
            let sample = (total / 16) as u16;
            buf[idx] = sample;
            idx = (idx + 1) % HISTORY;

            let avg_reading: u32 = buf.iter().map(|&v| v as u32).sum::<u32>() / HISTORY as u32;

            let c = temp36_c(avg_reading as u16);

            {
                let mut shared = SHARED_TEMPTURE.lock().await;
                *shared = c;
            }

            Timer::after(embassy_time::Duration::from_millis(200)).await;
        }
    }
}

#[embassy_executor::task]
async fn game_task(
    mut left_button: Input<'static>,
    mut middle_button: Input<'static>,
    mut right_button: Input<'static>,
    mut display: Display,
) {
    // Load save file
    let save_bytes = include_bytes!("../sdop.sav");
    let save_file = SaveFile::from_bytes(save_bytes).unwrap();
    let mut timestamp = save_file.last_timestamp;

    let mut game = sdop_game::Game::new(timestamp);
    SaveFile::load_from_bytes(save_bytes, timestamp, &mut game).unwrap();

    let mut tmp_sum = 0.;
    let mut tmp_count = 0;

    let mut last_time = Instant::now();
    let mut last_save = Instant::now();
    let mut last_tempture_update =
        Instant::now().saturating_sub(embassy_time::Duration::from_secs(10));

    loop {
        let loop_start = Instant::now();
        let delta = loop_start - last_time;
        last_time = loop_start;
        let delta = Duration::from_micros(delta.as_micros());
        timestamp = timestamp + delta;

        // Inputs
        let inputs = [
            if left_button.is_low() {
                sdop_game::ButtonState::Down
            } else {
                sdop_game::ButtonState::Up
            },
            if middle_button.is_low() {
                sdop_game::ButtonState::Down
            } else {
                sdop_game::ButtonState::Up
            },
            if right_button.is_low() {
                sdop_game::ButtonState::Down
            } else {
                sdop_game::ButtonState::Up
            },
        ];

        // Save every 5s
        if Duration::from_micros((loop_start - last_save).as_micros()) > Duration::from_secs(5) {
            last_save = loop_start;
            if let Some(save) = game.get_save(timestamp) {
                // TODO: persist save
                let _ = save;
            }
        }

        let tempture_delta = loop_start - last_tempture_update;
        if tempture_delta > embassy_time::Duration::from_millis(500) {
            last_tempture_update = loop_start;
            game.update_temperature(*SHARED_TEMPTURE.lock().await);
        }

        // Game logic
        game.update_input_states(inputs);
        game.tick(delta);
        game.refresh_display(delta);

        // Draw into display buffer
        game.drawable(|c| c).draw(&mut display).unwrap();
        display.flush().unwrap();

        // Target FPS pacing
        let target_fps: u64 = if game.low_power() { 5 } else { 60 };
        let frame_time: Duration = Duration::from_nanos(1_000_000_000 / target_fps);

        let frame_delta = Duration::from_micros((Instant::now() - last_time).as_micros());
        if frame_delta < frame_time {
            let sleep_time = frame_time - frame_delta;
            Timer::after_micros((sleep_time.as_micros() as u64).max(10)).await;
        } else {
            Timer::after_micros(10).await;
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut pins = embassy_rp::init(Default::default());
    // ADC to read the Vout value
    let mut adc = Adc::new(pins.ADC, Irqs, AdcConfig::default());
    let mut adc_channel_pin26 = Channel::new_pin(pins.PIN_26, Pull::Up);

    let mut config = i2c::Config::default();
    // config.frequency = 400_000; // fast mode
    config.frequency = 1_000_000; // super fast mode

    // Setting up I2C send text to OLED display
    let sda = pins.PIN_4;
    let scl = pins.PIN_5;
    let i2c0 = i2c::I2c::new_async(pins.I2C0, scl, sda, Irqs, config);
    let interface = I2CDisplayInterface::new(i2c0);

    let mut left_button = Input::new(pins.PIN_13, Pull::None);
    let mut middle_button = Input::new(pins.PIN_12, Pull::None);
    let mut right_button = Input::new(pins.PIN_11, Pull::None);

    // Make sure all the buttons are in a good state
    while left_button.is_low() && middle_button.is_low() && right_button.is_low() {
        Timer::after_millis(10).await;
    }

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate270)
        .into_buffered_graphics_mode();

    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    let mut sum: f32 = 0.;
    let mut count: usize = 0;
    let mut avg: f32 = 0.;
    let mut last = Instant::now();
    loop {
        display.clear(BinaryColor::Off).unwrap();
        let tmp36_voltage_24bit: u16 = adc.blocking_read(&mut adc_channel_pin26).unwrap();

        let cel = temp36_c(tmp36_voltage_24bit);

        if Instant::now() - last > embassy_time::Duration::from_secs(2) {
            last = Instant::now();
            if sum > 0. && count > 0 {
                avg = (libm::fabsf(sum) / count as f32);
            }
            sum = 0.;
            count = 0;
        }

        sum += cel;
        count += 1;

        let str = str_format!(fixedstr::str12, "{}", tmp36_voltage_24bit);
        Text::with_baseline(&str, Point::new(0, 06), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        let str = str_format!(fixedstr::str12, "{:.2}", sum);
        Text::with_baseline(&str, Point::new(0, 16), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        let str = str_format!(fixedstr::str12, "{:.2}", avg);
        Text::with_baseline(&str, Point::new(0, 26), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        Text::with_baseline(
            if left_button.is_low() { "LOW" } else { "HIGH" },
            Point::new(0, 66),
            text_style,
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();

        display.flush().unwrap();

        if left_button.is_low() {
            break;
        }

        // Timer::after_millis(1).await
    }

    spawner.spawn(tempeture_task(adc, adc_channel_pin26).unwrap());
    spawner.spawn(game_task(left_button, middle_button, right_button, display).unwrap());
}

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Blinky Example"),
    embassy_rp::binary_info::rp_program_description!(
        c"This example tests the RP Pico on board LED, connected to gpio 25"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

// End of file
