#![no_std]
#![no_main]

use core::time::Duration;

use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::{Baseline, Text};
use embedded_hal::digital::InputPin;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{
    BlockDevice, SdCard, TimeSource, Timestamp, Volume, VolumeIdx, VolumeManager,
};
use embedded_text::alignment::HorizontalAlignment;
use embedded_text::style::{HeightMode, TextBoxStyleBuilder};
use embedded_text::TextBox;
use hal::block::ImageDef;
use panic_halt as _;
use rp235x_hal::{self as hal, Clock};

/// Tell the Boot ROM about our application
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: ImageDef = hal::block::ImageDef::secure_exe();

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

use embedded_graphics::prelude::*;
use hal::fugit::RateExtU32;
use hal::gpio::{FunctionI2C, Pin};
use rp235x_hal::{timer::TimerDevice, Timer};
use sdop_game::{Game, SaveFile};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

fn timestamp<D: TimerDevice>(
    timestamp: sdop_game::Timestamp,
    timer: &Timer<D>,
) -> sdop_game::Timestamp {
    let ticks = timer.get_counter().ticks();
    let micros = ticks / 1; // 1 tick = 1 µs at 1 MHz

    timestamp + Duration::from_micros(micros as u64)
}

fn save_game<D, T, const MAX_DIRS: usize, const MAX_FILES: usize, const MAX_VOLUMES: usize>(
    vol: &mut Volume<D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    game: &Game,
) where
    D: BlockDevice,
    T: TimeSource,
    <D as BlockDevice>::Error: core::fmt::Debug,
{
    if let Some(save) = SaveFile::gen_save_bytes(sdop_game::Timestamp::default(), &game) {
        if let Ok(bytes) = save {
            if let Ok(mut root_dir) = vol.open_root_dir() {
                if let Ok(mut file) = root_dir
                    .open_file_in_dir("sdop.sav", embedded_sdmmc::Mode::ReadWriteCreateOrAppend)
                {
                    file.seek_from_start(0);
                    file.write(&bytes);
                    file.close();
                }
            }
        }
    }
}

// fn load_game<D, T>(volume_mgr: &mut VolumeManager<D, T>) -> Option<SaveFile>
// where
//     D: BlockDevice,
//     T: TimeSource,
//     <D as BlockDevice>::Error: core::fmt::Debug,
// {
//     if let Ok(mut vol) = volume_mgr.open_volume(VolumeIdx(0)) {
//         if let Ok(mut root_dir) = vol.open_root_dir() {
//             if let Ok(mut file) =
//                 root_dir.open_file_in_dir("sdop.sav", embedded_sdmmc::Mode::ReadOnly)
//             {
//                 const SAVE_FILE_LENGTH: usize = SaveFile::size();
//                 let mut buffer = [0; SAVE_FILE_LENGTH];
//                 file.read(&mut buffer).unwrap();

//                 if let Ok(save) = SaveFile::from_bytes(&buffer) {
//                     return Some(save);
//                 }
//             }
//         }
//     }

//     None
// }

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

    let mut left_button = pins.gpio13.into_pull_up_input();
    let mut middle_button = pins.gpio12.into_pull_up_input();
    let mut right_button = pins.gpio11.into_pull_up_input();

    let spi_cs = pins.gpio1.into_push_pull_output();
    let spi_sck = pins.gpio2.into_function::<hal::gpio::FunctionSpi>();
    let spi_mosi = pins.gpio3.into_function::<hal::gpio::FunctionSpi>();
    let spi_miso = pins.gpio4.into_function::<hal::gpio::FunctionSpi>();
    let spi_bus = hal::spi::Spi::<_, _, _, 8>::new(pac.SPI0, (spi_mosi, spi_miso, spi_sck));

    let spi = spi_bus.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        400.kHz(), // card initialization happens at low baud rate
        embedded_hal::spi::MODE_0,
    );

    let spi = ExclusiveDevice::new(spi, spi_cs, timer).unwrap();

    let sdcard = SdCard::new(spi, timer);
    let mut volume_mgr = VolumeManager::new(sdcard, DummyTimesource::default());

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
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let mut vol = match volume_mgr.open_volume(VolumeIdx(0)) {
        Ok(vol) => Some(vol),
        Err(err) => {
            Text::with_baseline("Device", Point::new(0, 16), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            let str = fixedstr::str_format!(fixedstr::str32, "{:?}", err);

            Text::with_baseline(&str, Point::new(0, 30), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();

            display.flush().unwrap();

            None
        }
    };

    let mut root_dir = match &mut vol {
        Some(vol) => Some(vol.open_root_dir().unwrap()),
        None => None,
    };

    if let Some(root_dir) = &mut root_dir {
        match root_dir.open_file_in_dir("sdop.sav", embedded_sdmmc::Mode::ReadWriteCreateOrAppend) {
            Ok(mut file) => {
                file.seek_from_start(0);
                file.write(&[1, 2, 3, 4]);
                file.close();

                Text::with_baseline("success", Point::new(0, 16), text_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();
            }
            Err(err) => {
                Text::with_baseline("file", Point::new(0, 16), text_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();

                let str = fixedstr::str_format!(fixedstr::str32, "{:?}", err);

                Text::with_baseline(&str, Point::new(0, 30), text_style, Baseline::Top)
                    .draw(&mut display)
                    .unwrap();
            }
        }
    }

    display.flush().unwrap();
    loop {
        if left_button.is_low().unwrap() {
            break;
        }
    }

    let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Justified)
        .paragraph_spacing(6)
        .build();

    let mut timestamp = sdop_game::Timestamp::from_parts(2025, 1, 1, 1, 1, 1, 1).unwrap();

    let mut game = sdop_game::Game::blank(None);

    display.clear(BinaryColor::Off);
    if let Some(root_dir) = &mut root_dir {
        if let Ok(mut file) = root_dir.open_file_in_dir("sdop.sav", embedded_sdmmc::Mode::ReadOnly)
        {
            const SAVE_FILE_LENGTH: usize = SaveFile::size();
            let mut buffer = [0; SAVE_FILE_LENGTH];
            file.read(&mut buffer).unwrap();

            match SaveFile::from_bytes(&buffer) {
                Ok(save) => {
                    timestamp = save.last_timestamp;
                    game.load_save(save.last_timestamp, save);
                    Text::with_baseline("Loaded", Point::new(0, 50), text_style, Baseline::Top)
                        .draw(&mut display)
                        .unwrap();
                }
                Err(err) => {
                    Text::with_baseline("Load Error", Point::new(0, 0), text_style, Baseline::Top)
                        .draw(&mut display)
                        .unwrap();

                    let str = fixedstr::str_format!(fixedstr::str32, "{:?}", err);

                    // Specify the bounding box. Note the 0px height. The `FitToText` height mode will
                    // measure and adjust the height of the text box in `into_styled()`.
                    let bounds = Rectangle::new(Point::new(0, 10), Size::new(64, 128));

                    // Create the text box and apply styling options.
                    let text_box =
                        TextBox::with_textbox_style(&str, bounds, character_style, textbox_style);

                    text_box.draw(&mut display).unwrap();
                }
            }
        } else {
            Text::with_baseline("File Error", Point::new(0, 50), text_style, Baseline::Top)
                .draw(&mut display)
                .unwrap();
        }
    } else {
        Text::with_baseline("Root Error", Point::new(0, 50), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
    }

    display.flush().unwrap();
    loop {
        if right_button.is_low().unwrap() {
            break;
        }
    }

    let mut last_time = timer.get_counter();
    let mut last_save = timer.get_counter();

    loop {
        let now = timer.get_counter();
        let delta = now - last_time;
        last_time = now;
        let delta = Duration::from_micros(delta.to_micros() as u64);
        timestamp = timestamp + delta;

        let inputs = [
            if left_button.is_low().unwrap() {
                sdop_game::ButtonState::Down
            } else {
                sdop_game::ButtonState::Up
            },
            if middle_button.is_low().unwrap() {
                sdop_game::ButtonState::Down
            } else {
                sdop_game::ButtonState::Up
            },
            if right_button.is_low().unwrap() {
                sdop_game::ButtonState::Down
            } else {
                sdop_game::ButtonState::Up
            },
        ];

        if let Some(root_dir) = &mut root_dir {
            if Duration::from_micros((timer.get_counter() - last_save).to_micros() as u64)
                > Duration::from_secs(5)
            {
                let mut error = false;

                if let Some(save) = SaveFile::gen_save_bytes(timestamp, &game) {
                    if let Ok(bytes) = save {
                        match root_dir.open_file_in_dir(
                            "sdop.sav",
                            embedded_sdmmc::Mode::ReadWriteCreateOrAppend,
                        ) {
                            Ok(mut file) => {
                                file.seek_from_start(0);
                                file.write(&bytes);
                                file.flush();
                                last_save = timer.get_counter();

                                Text::with_baseline(
                                    "success",
                                    Point::new(0, 16),
                                    text_style,
                                    Baseline::Top,
                                )
                                .draw(&mut display)
                                .unwrap();
                            }
                            Err(err) => {
                                error = true;
                                Text::with_baseline(
                                    "file",
                                    Point::new(0, 16),
                                    text_style,
                                    Baseline::Top,
                                )
                                .draw(&mut display)
                                .unwrap();

                                let str = fixedstr::str_format!(fixedstr::str32, "{:?}", err);

                                Text::with_baseline(
                                    &str,
                                    Point::new(0, 30),
                                    text_style,
                                    Baseline::Top,
                                )
                                .draw(&mut display)
                                .unwrap();
                            }
                        }
                    } else {
                        error = true;
                        Text::with_baseline("root", Point::new(0, 16), text_style, Baseline::Top)
                            .draw(&mut display)
                            .unwrap();
                    }
                }

                if error {
                    display.flush().unwrap();
                    loop {
                        if left_button.is_low().unwrap() {
                            break;
                        }
                    }
                }
            }
        }

        game.update_input_states(inputs);
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
