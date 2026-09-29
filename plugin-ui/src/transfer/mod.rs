//! Programs in and out of the RF-5, the ways Prophet-5 owners pass them on:
//! the Rev 3's own cassette tapes (as WAV recordings), the Rev 4's MIDI
//! SysEx dumps, and RackForge's program documents.
//!
//! Every program travels as the Rev 3 kept it: the 24 bytes of its program
//! memory, each pot at its 128 positions in bits 0-6 and a switch in bit 7.
//! Nothing here touches the page; the CONFIG surface reads a file with
//! [`read`] and writes one with the medium's own writer.

pub mod document;
pub mod sysex;
pub mod tape;

use core::fmt;

/// One program's memory, as the Rev 3 stores it.
pub const PROGRAM_BYTES: usize = 24;
pub type Memory = [u8; PROGRAM_BYTES];

/// The longest program name RackForge keeps.
pub const MAX_NAME_CHARS: usize = 64;

/// What a file carried the programs on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Medium {
    /// A Rev 3 cassette recording.
    Tape,
    /// Rev 4 MIDI SysEx program dumps.
    Sysex,
    /// RackForge program documents.
    Document,
}

impl Medium {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Tape => "TAPE",
            Self::Sysex => "SYSEX",
            Self::Document => "FILE",
        }
    }
}

/// A program read from a file: where it sat there, its name when the file
/// gives one, and its memory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    /// Its place on the medium: "1-1" for a tape's first bank and program.
    pub place: String,
    pub name: Option<String>,
    pub memory: Memory,
}

/// Everything read from one file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reading {
    pub medium: Medium,
    pub programs: Vec<Program>,
    /// Something the reader could not vouch for, the programs read anyway:
    /// a tape whose checksum failed, SysEx messages that were skipped.
    pub warning: Option<String>,
}

/// Why nothing could be read from a file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadError(pub String);

impl fmt::Display for ReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub(crate) fn error<T>(message: impl Into<String>) -> Result<T, ReadError> {
    Err(ReadError(message.into()))
}

/// Reads programs from a file, by what it holds rather than its name.
pub fn read(bytes: &[u8]) -> Result<Reading, ReadError> {
    if bytes.starts_with(b"RIFF") {
        return tape::read(bytes);
    }
    if bytes.first() == Some(&0xF0) {
        return sysex::read(bytes);
    }
    let text = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    if text.iter().find(|byte| !byte.is_ascii_whitespace()) == Some(&b'{') {
        return document::read(text);
    }
    error("NOT A TAPE, SYSEX OR RF-5 PROGRAM FILE")
}

/// A name RackForge accepts: trimmed, no control characters, not too long.
pub fn clean_name(name: &str) -> Option<String> {
    let name: String = name
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect();
    let name: String = name
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MAX_NAME_CHARS)
        .collect();
    let name = name.trim_end().to_owned();
    (!name.is_empty()).then_some(name)
}

/// A place in a Rev 3 program file: bank 1-5, program 1-8.
pub fn bank_place(index: usize) -> String {
    format!("{}-{}", index / 8 + 1, index % 8 + 1)
}

/// The program a blank place is filled with, when fewer programs are
/// written than the medium holds: both oscillators' sawtooths through an open
/// filter, as a Prophet player would start one from nothing.
pub const INIT_PROGRAM: Memory = {
    const SWITCH: u8 = 0x80;
    let mut memory = [0_u8; PROGRAM_BYTES];
    memory[1] = SWITCH; // OSC A SAW; filter decay 0
    memory[2] = 127; // filter sustain
    memory[3] = SWITCH; // OSC B SAW; filter release 0
    memory[6] = 127; // amp sustain
    memory[7] = 10; // amp release
    memory[8] = 127; // cutoff
    memory[10] = 100; // mix OSC B
    memory[11] = 64; // OSC B pulse width
    memory[12] = 100; // mix OSC A
    memory[13] = 64; // OSC A pulse width
    memory[14] = SWITCH; // filter keyboard; no noise
    memory[17] = 60; // LFO frequency
    memory[18] = 64; // wheel source mix
    memory[21] = 24; // OSC A frequency
    memory[22] = 24; // OSC B frequency, fine at 0
    memory
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_is_read_by_what_it_holds() {
        assert!(read(b"hello").is_err());
        assert!(read(b"").is_err());
        assert_eq!(
            read(b"  {}").unwrap_err().0,
            "NO RF-5 PROGRAMS IN THIS FILE"
        );
    }

    #[test]
    fn names_are_cleaned_for_rackforge() {
        assert_eq!(
            clean_name("  BRASS \u{7}  II "),
            Some("BRASS II".to_owned())
        );
        assert_eq!(clean_name(" \t "), None);
        assert_eq!(clean_name(&"x".repeat(80)).unwrap().chars().count(), 64);
    }

    #[test]
    fn places_count_banks_of_eight() {
        assert_eq!(bank_place(0), "1-1");
        assert_eq!(bank_place(39), "5-8");
    }
}
