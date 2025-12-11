#![no_std]
#![no_main]

use core::time::Duration;

mod fram;
mod notes;

use embassy_executor::Spawner;
use embassy_rp::{
    self as hal, adc,
    adc::{Adc, Channel as AdcChannel, Config as AdcConfig},
    bind_interrupts,
    block::ImageDef,
    gpio::{Input, Level, Output, Pull},
    i2c,
    peripherals::{self, I2C0, SPI0},
    pwm::{Config as PwmConfig, Pwm, SetDutyCycle},
    spi::Spi,
};
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex},
    channel::Channel,
    mutex::Mutex,
};
use embassy_time::{Instant, Timer};
use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    mono_font::ascii::FONT_6X10,
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use fixed::types::extra::U4;
use fixedstr::str_format;
use sdop_game::{SaveFile, Timestamp};
use ssd1306::{
    I2CDisplayInterface, Ssd1306,
    mode::{BufferedGraphicsMode, DisplayConfig},
    prelude::{DisplayRotation, I2CInterface},
    size::DisplaySize128x64,
};

use crate::notes::freq;

use {defmt_rtt as _, panic_probe as _};

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
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
    c - 8.
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

static PENDING_SAVE: Channel<CriticalSectionRawMutex, sdop_game::SaveFile, 1> = Channel::new();

#[embassy_executor::task]
async fn save_task(mut spi: Spi<'static, SPI0, embassy_rp::spi::Async>, mut cs: Output<'static>) {
    loop {
        let save = PENDING_SAVE.receive().await;

        if let Ok(save_bytes) = save.to_bytes() {
            fram::write(&mut spi, &mut cs, sdop_game::SAVE_SIZE as u16, &save_bytes)
                .await
                .unwrap();
        }
    }
}

static CHANNEL: Channel<CriticalSectionRawMutex, sdop_game::Song, 2> = Channel::new();
static SOUND_PLAYING: Mutex<ThreadModeRawMutex, bool> = Mutex::new(false);

#[embassy_executor::task]
async fn play_sound_task(mut buzzer_b: Pwm<'static>) {
    let divider = fixed::FixedU16::<U4>::from_num(200.0);
    let mut cfg = PwmConfig::default();
    cfg.divider = divider;
    let receiver = CHANNEL.receiver();
    loop {
        {
            let mut shared = SOUND_PLAYING.lock().await;
            *shared = false;
        }
        let song = receiver.receive().await;
        {
            let mut shared = SOUND_PLAYING.lock().await;
            *shared = true;
        }
        for entry in song.melody() {
            if !receiver.is_empty() {
                break;
            }

            let top = notes::get_top(freq(&entry.note), divider.to_num::<f64>());
            cfg.top = top;
            buzzer_b.set_config(&cfg);

            let note_duration = song.calc_note_duration(entry.duration);
            let pause_duration = note_duration / 10; // 10% of note_duration

            buzzer_b.set_duty_cycle_percent(50).unwrap(); // Set duty cycle to 50% to play the note

            Timer::after_millis((note_duration - pause_duration).as_millis() as u64).await;
            buzzer_b.set_duty_cycle(0).unwrap(); // Stop tone
            Timer::after_millis(pause_duration.as_millis() as u64).await;
        }
        buzzer_b.set_duty_cycle(0).unwrap();
    }
}

static SHARED_TEMPTURE: Mutex<ThreadModeRawMutex, f32> = Mutex::new(sdop_game::ROOM_TEMPTURE);

#[embassy_executor::task]
async fn tempeture_task(
    mut adc: Adc<'static, embassy_rp::adc::Async>,
    mut adc_channel: AdcChannel<'static>,
) {
    const HISTORY: usize = 10;

    let mut buf = [0u16; HISTORY];
    let mut idx = 0;
    let start = Instant::now();

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

            if Instant::now() - start > embassy_time::Duration::from_secs(5) {
                let mut shared = SHARED_TEMPTURE.lock().await;
                *shared = c;
            }

            Timer::after(embassy_time::Duration::from_millis(200)).await;
        }
    }
}

#[embassy_executor::task]
async fn game_task(
    left_button: Input<'static>,
    middle_button: Input<'static>,
    right_button: Input<'static>,
    mut display: Display,
    mut spi: Spi<'static, SPI0, embassy_rp::spi::Async>,
    mut cs: Output<'static>,
    save_file: Option<SaveFile>,
) {
    let sender = CHANNEL.sender();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    // Probably need some enter time screen seperated from the game that I can loop though

    // Load save file
    let (mut game, mut timestamp) = if let Some(save) = save_file {
        let timestamp = save.last_timestamp;
        let mut game = sdop_game::Game::new(timestamp);
        game.load_save(timestamp, save);
        (game, timestamp)
    } else {
        (sdop_game::Game::blank(None), Timestamp::default())
    };

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

        if (loop_start - last_save) > embassy_time::Duration::from_secs(60) {
            last_save = loop_start;
            if let Some(save) = game.get_save(game.get_time()) {
                if let Ok(save_bytes) = save.to_bytes() {
                    if let Err(err) =
                        fram::write(&mut spi, &mut cs, fram::SDOP_SAVE_ADDR, &save_bytes).await
                    {
                        loop {
                            display.clear(BinaryColor::Off);
                            display.flush().unwrap();
                        }
                    }
                }
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

        if let Some(song) = game.pull_song() {
            sender.send(song).await;
        }
        {
            let playing = SOUND_PLAYING.lock().await;
            game.set_playing_song(*playing);
        }

        game.refresh_display(delta);

        // Draw into display buffer
        game.drawable(|c| c).draw(&mut display).unwrap();

        display.flush().unwrap();

        // Target FPS pacing
        let target_fps: u64 = if game.low_power() { 5 } else { 60 };
        let frame_time: Duration = Duration::from_nanos(1_000_000_000 / target_fps);

        // Always give oxygen to other tasks
        Timer::after_micros(10).await;
        let frame_delta = Duration::from_micros((Instant::now() - last_time).as_micros());
        if frame_delta < frame_time {
            let mut sleep_time = (frame_time - frame_delta).as_micros();
            while sleep_time > 0 {
                Timer::after_micros(100.min(sleep_time) as u64).await;
                if left_button.is_high() || middle_button.is_high() || right_button.is_high() {
                    break;
                }
                sleep_time = sleep_time.checked_sub(100).unwrap_or(0);
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut pins = embassy_rp::init(Default::default());
    // ADC to read the Vout value
    let mut adc = Adc::new(pins.ADC, Irqs, AdcConfig::default());
    let mut adc_channel_pin26 = AdcChannel::new_pin(pins.PIN_27, Pull::Up);

    let mut buzzer_a = Pwm::new_output_a(
        pins.PWM_SLICE6,
        pins.PIN_28,
        embassy_rp::pwm::Config::default(),
    );

    let mut config = i2c::Config::default();
    // config.frequency = 400_000; // fast mode
    config.frequency = 1_000_000; // super fast mode

    // Setting up I2C send text to OLED display
    let sda = pins.PIN_4;
    let scl = pins.PIN_5;
    let i2c0 = i2c::I2c::new_async(pins.I2C0, scl, sda, Irqs, config);
    let interface = I2CDisplayInterface::new(i2c0);

    let left_button = Input::new(pins.PIN_26, Pull::None);
    let middle_button = Input::new(pins.PIN_15, Pull::None);
    let right_button = Input::new(pins.PIN_14, Pull::None);

    let fram_sck = pins.PIN_6;
    let fram_miso = pins.PIN_0;
    let fram_mosi = pins.PIN_3;
    let fram_cs = pins.PIN_2;

    // SPI setup
    let cfg = embassy_rp::spi::Config::default();

    // Create the SPI peripheral
    let mut spi = Spi::new(
        pins.SPI0,
        fram_sck,
        fram_mosi,
        fram_miso,
        pins.DMA_CH0,
        pins.DMA_CH1,
        cfg,
    );
    // Chip Select pin
    let mut cs = Output::new(fram_cs, Level::High);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate270)
        .into_buffered_graphics_mode();

    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    // Make sure all the buttons are in a good state
    loop {
        display.clear(BinaryColor::Off).unwrap();
        let str = str_format!(
            fixedstr::str12,
            "{}, {}, {}",
            left_button.is_high() as u8,
            middle_button.is_high() as u8,
            right_button.is_high() as u8
        );
        Text::with_baseline(&str, Point::new(0, 06), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        display.flush().unwrap();
        Timer::after_millis(100).await;
        if left_button.is_high() && middle_button.is_high() && right_button.is_high() {
            break;
        }
    }

    enum SaveMode {
        Clear,
        Restore,
        Load,
    }
    let save_mode = {
        let mut save_mode = SaveMode::Load;
        loop {
            display.clear(BinaryColor::On).unwrap();
            display.flush().unwrap();
            if left_button.is_low() && right_button.is_low() {
                save_mode = SaveMode::Clear;
                break;
            } else if left_button.is_low() {
                save_mode = SaveMode::Restore;
                break;
            } else if middle_button.is_low() {
                break;
            }
        }

        save_mode
    };

    // Load Save
    let save_file = match save_mode {
        SaveMode::Clear => None,
        SaveMode::Restore => SaveFile::from_bytes(include_bytes!("../sdop.sav")).ok(),
        SaveMode::Load => {
            let mut buf = [0u8; sdop_game::SAVE_SIZE];
            fram::read(&mut spi, &mut cs, fram::SDOP_SAVE_ADDR, &mut buf)
                .await
                .unwrap();

            SaveFile::from_bytes(&buf).ok()
        }
    };

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
                avg = libm::fabsf(sum) / count as f32;
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

        Timer::after_millis(16).await
    }

    spawner.spawn(tempeture_task(adc, adc_channel_pin26).unwrap());
    spawner.spawn(play_sound_task(buzzer_a).unwrap());
    // spawner.spawn(save_task(spi, cs).unwrap());
    spawner.spawn(
        game_task(
            left_button,
            middle_button,
            right_button,
            display,
            spi,
            cs,
            save_file,
        )
        .unwrap(),
    );
}

// Program metadata for `picotool info`.
// This isn't needed, but it's recomended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
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
