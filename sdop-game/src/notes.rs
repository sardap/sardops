use core::time::Duration;

use sdop_common::Note;

const NOTE_A1: &[u8] = include_bytes!("../../assets/notes/NOTE_A1.ogg");
const NOTE_A2: &[u8] = include_bytes!("../../assets/notes/NOTE_A2.ogg");
const NOTE_A3: &[u8] = include_bytes!("../../assets/notes/NOTE_A3.ogg");
const NOTE_A4: &[u8] = include_bytes!("../../assets/notes/NOTE_A4.ogg");
const NOTE_A5: &[u8] = include_bytes!("../../assets/notes/NOTE_A5.ogg");
const NOTE_A6: &[u8] = include_bytes!("../../assets/notes/NOTE_A6.ogg");
const NOTE_A7: &[u8] = include_bytes!("../../assets/notes/NOTE_A7.ogg");

const NOTE_AS1: &[u8] = include_bytes!("../../assets/notes/NOTE_AS1.ogg");
const NOTE_AS2: &[u8] = include_bytes!("../../assets/notes/NOTE_AS2.ogg");
const NOTE_AS3: &[u8] = include_bytes!("../../assets/notes/NOTE_AS3.ogg");
const NOTE_AS4: &[u8] = include_bytes!("../../assets/notes/NOTE_AS4.ogg");
const NOTE_AS5: &[u8] = include_bytes!("../../assets/notes/NOTE_AS5.ogg");
const NOTE_AS6: &[u8] = include_bytes!("../../assets/notes/NOTE_AS6.ogg");
const NOTE_AS7: &[u8] = include_bytes!("../../assets/notes/NOTE_AS7.ogg");

const NOTE_B0: &[u8] = include_bytes!("../../assets/notes/NOTE_B0.ogg");
const NOTE_B1: &[u8] = include_bytes!("../../assets/notes/NOTE_B1.ogg");
const NOTE_B2: &[u8] = include_bytes!("../../assets/notes/NOTE_B2.ogg");
const NOTE_B3: &[u8] = include_bytes!("../../assets/notes/NOTE_B3.ogg");
const NOTE_B4: &[u8] = include_bytes!("../../assets/notes/NOTE_B4.ogg");
const NOTE_B5: &[u8] = include_bytes!("../../assets/notes/NOTE_B5.ogg");
const NOTE_B6: &[u8] = include_bytes!("../../assets/notes/NOTE_B6.ogg");
const NOTE_B7: &[u8] = include_bytes!("../../assets/notes/NOTE_B7.ogg");

const NOTE_C1: &[u8] = include_bytes!("../../assets/notes/NOTE_C1.ogg");
const NOTE_C2: &[u8] = include_bytes!("../../assets/notes/NOTE_C2.ogg");
const NOTE_C3: &[u8] = include_bytes!("../../assets/notes/NOTE_C3.ogg");
const NOTE_C4: &[u8] = include_bytes!("../../assets/notes/NOTE_C4.ogg");
const NOTE_C5: &[u8] = include_bytes!("../../assets/notes/NOTE_C5.ogg");
const NOTE_C6: &[u8] = include_bytes!("../../assets/notes/NOTE_C6.ogg");
const NOTE_C7: &[u8] = include_bytes!("../../assets/notes/NOTE_C7.ogg");
const NOTE_C8: &[u8] = include_bytes!("../../assets/notes/NOTE_C8.ogg");

const NOTE_CS1: &[u8] = include_bytes!("../../assets/notes/NOTE_CS1.ogg");
const NOTE_CS2: &[u8] = include_bytes!("../../assets/notes/NOTE_CS2.ogg");
const NOTE_CS3: &[u8] = include_bytes!("../../assets/notes/NOTE_CS3.ogg");
const NOTE_CS4: &[u8] = include_bytes!("../../assets/notes/NOTE_CS4.ogg");
const NOTE_CS5: &[u8] = include_bytes!("../../assets/notes/NOTE_CS5.ogg");
const NOTE_CS6: &[u8] = include_bytes!("../../assets/notes/NOTE_CS6.ogg");
const NOTE_CS7: &[u8] = include_bytes!("../../assets/notes/NOTE_CS7.ogg");
const NOTE_CS8: &[u8] = include_bytes!("../../assets/notes/NOTE_CS8.ogg");

const NOTE_D1: &[u8] = include_bytes!("../../assets/notes/NOTE_D1.ogg");
const NOTE_D2: &[u8] = include_bytes!("../../assets/notes/NOTE_D2.ogg");
const NOTE_D3: &[u8] = include_bytes!("../../assets/notes/NOTE_D3.ogg");
const NOTE_D4: &[u8] = include_bytes!("../../assets/notes/NOTE_D4.ogg");
const NOTE_D5: &[u8] = include_bytes!("../../assets/notes/NOTE_D5.ogg");
const NOTE_D6: &[u8] = include_bytes!("../../assets/notes/NOTE_D6.ogg");
const NOTE_D7: &[u8] = include_bytes!("../../assets/notes/NOTE_D7.ogg");
const NOTE_D8: &[u8] = include_bytes!("../../assets/notes/NOTE_D8.ogg");

const NOTE_DS1: &[u8] = include_bytes!("../../assets/notes/NOTE_DS1.ogg");
const NOTE_DS2: &[u8] = include_bytes!("../../assets/notes/NOTE_DS2.ogg");
const NOTE_DS3: &[u8] = include_bytes!("../../assets/notes/NOTE_DS3.ogg");
const NOTE_DS4: &[u8] = include_bytes!("../../assets/notes/NOTE_DS4.ogg");
const NOTE_DS5: &[u8] = include_bytes!("../../assets/notes/NOTE_DS5.ogg");
const NOTE_DS6: &[u8] = include_bytes!("../../assets/notes/NOTE_DS6.ogg");
const NOTE_DS7: &[u8] = include_bytes!("../../assets/notes/NOTE_DS7.ogg");
const NOTE_DS8: &[u8] = include_bytes!("../../assets/notes/NOTE_DS8.ogg");

const NOTE_E1: &[u8] = include_bytes!("../../assets/notes/NOTE_E1.ogg");
const NOTE_E2: &[u8] = include_bytes!("../../assets/notes/NOTE_E2.ogg");
const NOTE_E3: &[u8] = include_bytes!("../../assets/notes/NOTE_E3.ogg");
const NOTE_E4: &[u8] = include_bytes!("../../assets/notes/NOTE_E4.ogg");
const NOTE_E5: &[u8] = include_bytes!("../../assets/notes/NOTE_E5.ogg");
const NOTE_E6: &[u8] = include_bytes!("../../assets/notes/NOTE_E6.ogg");
const NOTE_E7: &[u8] = include_bytes!("../../assets/notes/NOTE_E7.ogg");

const NOTE_F1: &[u8] = include_bytes!("../../assets/notes/NOTE_F1.ogg");
const NOTE_F2: &[u8] = include_bytes!("../../assets/notes/NOTE_F2.ogg");
const NOTE_F3: &[u8] = include_bytes!("../../assets/notes/NOTE_F3.ogg");
const NOTE_F4: &[u8] = include_bytes!("../../assets/notes/NOTE_F4.ogg");
const NOTE_F5: &[u8] = include_bytes!("../../assets/notes/NOTE_F5.ogg");
const NOTE_F6: &[u8] = include_bytes!("../../assets/notes/NOTE_F6.ogg");
const NOTE_F7: &[u8] = include_bytes!("../../assets/notes/NOTE_F7.ogg");

const NOTE_FS1: &[u8] = include_bytes!("../../assets/notes/NOTE_FS1.ogg");
const NOTE_FS2: &[u8] = include_bytes!("../../assets/notes/NOTE_FS2.ogg");
const NOTE_FS3: &[u8] = include_bytes!("../../assets/notes/NOTE_FS3.ogg");
const NOTE_FS4: &[u8] = include_bytes!("../../assets/notes/NOTE_FS4.ogg");
const NOTE_FS5: &[u8] = include_bytes!("../../assets/notes/NOTE_FS5.ogg");
const NOTE_FS6: &[u8] = include_bytes!("../../assets/notes/NOTE_FS6.ogg");
const NOTE_FS7: &[u8] = include_bytes!("../../assets/notes/NOTE_FS7.ogg");

const NOTE_G1: &[u8] = include_bytes!("../../assets/notes/NOTE_G1.ogg");
const NOTE_G2: &[u8] = include_bytes!("../../assets/notes/NOTE_G2.ogg");
const NOTE_G3: &[u8] = include_bytes!("../../assets/notes/NOTE_G3.ogg");
const NOTE_G4: &[u8] = include_bytes!("../../assets/notes/NOTE_G4.ogg");
const NOTE_G5: &[u8] = include_bytes!("../../assets/notes/NOTE_G5.ogg");
const NOTE_G6: &[u8] = include_bytes!("../../assets/notes/NOTE_G6.ogg");
const NOTE_G7: &[u8] = include_bytes!("../../assets/notes/NOTE_G7.ogg");

const NOTE_GS1: &[u8] = include_bytes!("../../assets/notes/NOTE_GS1.ogg");
const NOTE_GS2: &[u8] = include_bytes!("../../assets/notes/NOTE_GS2.ogg");
const NOTE_GS3: &[u8] = include_bytes!("../../assets/notes/NOTE_GS3.ogg");
const NOTE_GS4: &[u8] = include_bytes!("../../assets/notes/NOTE_GS4.ogg");
const NOTE_GS5: &[u8] = include_bytes!("../../assets/notes/NOTE_GS5.ogg");
const NOTE_GS6: &[u8] = include_bytes!("../../assets/notes/NOTE_GS6.ogg");
const NOTE_GS7: &[u8] = include_bytes!("../../assets/notes/NOTE_GS7.ogg");

const REST: &[u8] = include_bytes!("../../assets/notes/REST.ogg");

pub fn note_sound_file(note: &Note) -> &'static [u8] {
    match note {
        Note::B0 => NOTE_B0,
        Note::C1 => NOTE_C1,
        Note::Cs1 => NOTE_CS1,
        Note::D1 => NOTE_D1,
        Note::Ds1 => NOTE_DS1,
        Note::E1 => NOTE_E1,
        Note::F1 => NOTE_F1,
        Note::Fs1 => NOTE_FS1,
        Note::G1 => NOTE_G1,
        Note::Gs1 => NOTE_GS1,
        Note::A1 => NOTE_A1,
        Note::As1 => NOTE_AS1,
        Note::B1 => NOTE_B1,
        Note::C2 => NOTE_C2,
        Note::Cs2 => NOTE_CS2,
        Note::D2 => NOTE_D2,
        Note::Ds2 => NOTE_DS2,
        Note::E2 => NOTE_E2,
        Note::F2 => NOTE_F2,
        Note::Fs2 => NOTE_FS2,
        Note::G2 => NOTE_G2,
        Note::Gs2 => NOTE_GS2,
        Note::A2 => NOTE_A2,
        Note::As2 => NOTE_AS2,
        Note::B2 => NOTE_B2,
        Note::C3 => NOTE_C3,
        Note::Cs3 => NOTE_CS3,
        Note::D3 => NOTE_D3,
        Note::Ds3 => NOTE_DS3,
        Note::E3 => NOTE_E3,
        Note::F3 => NOTE_F3,
        Note::Fs3 => NOTE_FS3,
        Note::G3 => NOTE_G3,
        Note::Gs3 => NOTE_GS3,
        Note::A3 => NOTE_A3,
        Note::As3 => NOTE_AS3,
        Note::B3 => NOTE_B3,
        Note::C4 => NOTE_C4,
        Note::Cs4 => NOTE_CS4,
        Note::D4 => NOTE_D4,
        Note::Ds4 => NOTE_DS4,
        Note::E4 => NOTE_E4,
        Note::F4 => NOTE_F4,
        Note::Fs4 => NOTE_FS4,
        Note::G4 => NOTE_G4,
        Note::Gs4 => NOTE_GS4,
        Note::A4 => NOTE_A4,
        Note::As4 => NOTE_AS4,
        Note::B4 => NOTE_B4,
        Note::C5 => NOTE_C5,
        Note::Cs5 => NOTE_CS5,
        Note::D5 => NOTE_D5,
        Note::Ds5 => NOTE_DS5,
        Note::E5 => NOTE_E5,
        Note::F5 => NOTE_F5,
        Note::Fs5 => NOTE_FS5,
        Note::G5 => NOTE_G5,
        Note::Gs5 => NOTE_GS5,
        Note::A5 => NOTE_A5,
        Note::As5 => NOTE_AS5,
        Note::B5 => NOTE_B5,
        Note::C6 => NOTE_C6,
        Note::Cs6 => NOTE_CS6,
        Note::D6 => NOTE_D6,
        Note::Ds6 => NOTE_DS6,
        Note::E6 => NOTE_E6,
        Note::F6 => NOTE_F6,
        Note::Fs6 => NOTE_FS6,
        Note::G6 => NOTE_G6,
        Note::Gs6 => NOTE_GS6,
        Note::A6 => NOTE_A6,
        Note::As6 => NOTE_AS6,
        Note::B6 => NOTE_B6,
        Note::C7 => NOTE_C7,
        Note::Cs7 => NOTE_CS7,
        Note::D7 => NOTE_D7,
        Note::Ds7 => NOTE_DS7,
        Note::E7 => NOTE_E7,
        Note::F7 => NOTE_F7,
        Note::Fs7 => NOTE_FS7,
        Note::G7 => NOTE_G7,
        Note::Gs7 => NOTE_GS7,
        Note::A7 => NOTE_A7,
        Note::As7 => NOTE_AS7,
        Note::B7 => NOTE_B7,
        Note::C8 => NOTE_C8,
        Note::Cs8 => NOTE_CS8,
        Note::D8 => NOTE_D8,
        Note::Ds8 => NOTE_DS8,
        Note::Rest => REST,
    }
}
