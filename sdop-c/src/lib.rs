use core::time::Duration;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use std::boxed::Box;

pub struct TimestampInner(sdop_game::Timestamp);

#[repr(C)]
pub struct SdopTimestamp {
    _private: [u8; 0],
}

#[no_mangle]
pub extern "C" fn sdop_timestamp_new(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    miniute: u32,
    second: u32,
) -> *mut SdopTimestamp {
    let ts = sdop_game::Timestamp::new(NaiveDateTime::new(
        NaiveDate::from_ymd_opt(year, month, day).unwrap(),
        NaiveTime::from_hms_opt(hour, miniute, second).unwrap(),
    ));
    Box::into_raw(Box::new(ts)) as *mut SdopTimestamp
}

#[no_mangle]
pub extern "C" fn sdop_timestamp_free(ptr: *mut SdopTimestamp) {
    if !ptr.is_null() {
        unsafe {
            Box::from_raw(ptr);
        }
    }
}

pub struct GameInner(sdop_game::Game);

#[repr(C)]
pub struct SdopGame {
    _private: [u8; 0],
}

#[no_mangle]
pub extern "C" fn sdop_game_blank(ts_ptr: *mut SdopTimestamp) -> *mut SdopGame {
    let ts = if ts_ptr.is_null() {
        None
    } else {
        unsafe { Some((&mut *(ts_ptr as *mut TimestampInner)).0) }
    };
    let game = sdop_game::Game::blank(ts);
    Box::into_raw(Box::new(game)) as *mut SdopGame
}

#[no_mangle]
pub extern "C" fn sdop_game_tick(ptr: *mut SdopGame, delta_nanos: u64) {
    let game = unsafe { &mut *(ptr as *mut GameInner) };
    game.0.tick(Duration::from_nanos(delta_nanos));
}

#[no_mangle]
pub extern "C" fn sdop_game_refresh_display(ptr: *mut SdopGame, delta_nanos: u64) {
    let game = unsafe { &mut *(ptr as *mut GameInner) };
    game.0.refresh_display(Duration::from_nanos(delta_nanos));
}

#[no_mangle]
pub extern "C" fn sdop_game_free(ptr: *mut SdopGame) {
    if !ptr.is_null() {
        unsafe {
            Box::from_raw(ptr);
        }
    }
}
