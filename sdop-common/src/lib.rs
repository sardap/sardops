#![no_std]

use const_for::const_for;
use strum_macros::{EnumCount, EnumIter};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Note {
    B0,
    C1,
    Cs1,
    D1,
    Ds1,
    E1,
    F1,
    Fs1,
    G1,
    Gs1,
    A1,
    As1,
    B1,
    C2,
    Cs2,
    D2,
    Ds2,
    E2,
    F2,
    Fs2,
    G2,
    Gs2,
    A2,
    As2,
    B2,
    C3,
    Cs3,
    D3,
    Ds3,
    E3,
    F3,
    Fs3,
    G3,
    Gs3,
    A3,
    As3,
    B3,
    C4,
    Cs4,
    D4,
    Ds4,
    E4,
    F4,
    Fs4,
    G4,
    Gs4,
    A4,
    As4,
    B4,
    C5,
    Cs5,
    D5,
    Ds5,
    E5,
    F5,
    Fs5,
    G5,
    Gs5,
    A5,
    As5,
    B5,
    C6,
    Cs6,
    D6,
    Ds6,
    E6,
    F6,
    Fs6,
    G6,
    Gs6,
    A6,
    As6,
    B6,
    C7,
    Cs7,
    D7,
    Ds7,
    E7,
    F7,
    Fs7,
    G7,
    Gs7,
    A7,
    As7,
    B7,
    C8,
    Cs8,
    D8,
    Ds8,
    Rest,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MelodyEntry {
    pub note: Note,
    pub duration: i16,
}

impl MelodyEntry {
    pub const fn new(note: Note, duration: i16) -> Self {
        Self { note, duration }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemCategory {
    Misc,
    Furniture,
    PlayThing,
    Usable,
    Book,
    Software,
    Food,
    Map,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount)]
pub enum LifeStage {
    Baby = 0b100,
    Child = 0b010,
    Adult = 0b001,
}

pub type LifeStageMask = u8;

impl LifeStage {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Baby,
            1 => Self::Child,
            2 => Self::Adult,
            _ => unreachable!(),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            LifeStage::Baby => "BABY",
            LifeStage::Child => "CHILD",
            LifeStage::Adult => "ADULT",
        }
    }

    pub const fn create_bitmask(stages: &[LifeStage]) -> LifeStageMask {
        let mut result = 0;
        const_for!(i in 0..stages.len() => {
            result |= stages[i].bitmask();
        });
        result
    }

    pub const fn bitmask(&self) -> LifeStageMask {
        *self as u8
    }
}
