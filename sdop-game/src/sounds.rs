use core::time::Duration;

use bincode::{Decode, Encode};
use sdop_common::MelodyEntry;

include!(concat!(env!("OUT_DIR"), "/dist_sounds.rs"));

pub const CLEAR_SONG: Song = Song::new(&[], 85);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Song {
    melody: &'static [MelodyEntry],
    tempo: u16,
    whole_note: u32,
}

impl Song {
    pub const fn new(melody: &'static [MelodyEntry], tempo: u16) -> Self {
        Self {
            melody,
            tempo,
            whole_note: (60_000 * 4) / tempo as u32,
        }
    }

    pub const fn melody(&self) -> &'static [MelodyEntry] {
        self.melody
    }

    pub fn calc_note_duration(&self, divider: i16) -> Duration {
        if divider > 0 {
            Duration::from_millis((self.whole_note / divider as u32) as u64)
        } else {
            let duration = self.whole_note / divider.unsigned_abs() as u32;
            Duration::from_millis((duration as f64 * 1.5) as u64)
        }
    }
}

pub enum SoundKind {
    Music,
    Effect,
    Essential,
}

pub struct SongPlayOptions {
    kind: SoundKind,
}

impl Default for SongPlayOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl SongPlayOptions {
    pub const fn new() -> Self {
        Self {
            kind: SoundKind::Effect,
        }
    }

    pub const fn with_music(mut self) -> Self {
        self.kind = SoundKind::Music;
        self
    }

    pub const fn with_effect(mut self) -> Self {
        self.kind = SoundKind::Effect;
        self
    }

    pub const fn with_essential(mut self) -> Self {
        self.kind = SoundKind::Essential;
        self
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Copy, Clone)]
pub struct SoundOptions {
    pub play_music: bool,
    pub play_effect: bool,
    pub play_essential: bool,
}

impl Default for SoundOptions {
    fn default() -> Self {
        Self {
            play_music: true,
            play_effect: true,
            play_essential: true,
        }
    }
}

#[derive(Default)]
pub struct SoundSystem {
    pending: Option<Song>,
    playing: bool,
    options: SoundOptions,
}

impl SoundSystem {
    pub fn push_song(&mut self, song: Song, options: SongPlayOptions) {
        match options.kind {
            SoundKind::Music => {
                if !self.options.play_music {
                    return;
                }
            }
            SoundKind::Effect => {
                if !self.options.play_effect {
                    return;
                }
            }
            SoundKind::Essential => {
                if !self.options.play_essential {
                    return;
                }
            }
        }
        self.pending = Some(song);
    }

    pub fn pull_song(&mut self) -> Option<Song> {
        self.pending.take()
    }

    pub fn clear_song(&mut self) {
        self.push_song(CLEAR_SONG, SongPlayOptions::new());
    }

    pub fn song_queued(&self) -> bool {
        self.pending.is_some()
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.playing = playing;
    }

    pub fn get_playing(&self) -> bool {
        self.playing
    }

    pub fn sound_options(&self) -> &SoundOptions {
        &self.options
    }

    pub fn sound_options_mut(&mut self) -> &mut SoundOptions {
        &mut self.options
    }

    pub fn set_sound_options(&mut self, options: SoundOptions) {
        self.options = options;
    }
}
