//! Rev 3 V8.1 programmable intonation.
//!
//! Scale Mode reuses twelve panel pots as one signed pitch offset for each
//! chromatic note. The operating ROM doubles the 7-bit pot code and subtracts
//! `0x80` in its internal 256-units-per-semitone pitch word. Consequently code
//! 64 is equal temperament and each code step is exactly 1/128 semitone.

use rf_5_contract::{SCALE_NOTE_COUNT, hardware::quantize_analog_pot};

pub const EQUAL_TEMPERAMENT_CODE: u8 = 64;
const INTERNAL_CODES_PER_SEMITONE: f32 = 256.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScaleProgram {
    codes: [u8; SCALE_NOTE_COUNT],
}

impl Default for ScaleProgram {
    fn default() -> Self {
        Self::equal_temperament()
    }
}

impl ScaleProgram {
    pub const fn equal_temperament() -> Self {
        Self {
            codes: [EQUAL_TEMPERAMENT_CODE; SCALE_NOTE_COUNT],
        }
    }

    pub fn from_normalized(values: [f32; SCALE_NOTE_COUNT]) -> Self {
        Self {
            codes: values.map(raw_pot_code),
        }
    }

    pub const fn from_codes(codes: [u8; SCALE_NOTE_COUNT]) -> Option<Self> {
        let mut index = 0;
        while index < SCALE_NOTE_COUNT {
            if codes[index] > 127 {
                return None;
            }
            index += 1;
        }
        Some(Self { codes })
    }

    pub const fn codes(self) -> [u8; SCALE_NOTE_COUNT] {
        self.codes
    }

    pub fn offset_semitones(self, midi_note: u8) -> f32 {
        let raw = self.codes[usize::from(midi_note % SCALE_NOTE_COUNT as u8)];
        f32::from(scale_internal_offset(raw)) / INTERNAL_CODES_PER_SEMITONE
    }
}

fn raw_pot_code(value: f32) -> u8 {
    libm::roundf(quantize_analog_pot(value) * 127.0) as u8
}

fn scale_internal_offset(raw: u8) -> i16 {
    i16::from(raw) * 2 - 0x80
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_temperament_is_exactly_code_64() {
        let scale = ScaleProgram::default();
        assert_eq!(scale.codes(), [64; SCALE_NOTE_COUNT]);
        for note in 0..=127 {
            assert_eq!(scale.offset_semitones(note), 0.0);
        }
    }

    #[test]
    fn v81_scale_word_has_the_asymmetric_half_semitone_limits() {
        let low = ScaleProgram::from_codes([0; SCALE_NOTE_COUNT]).unwrap();
        let high = ScaleProgram::from_codes([127; SCALE_NOTE_COUNT]).unwrap();
        assert_eq!(low.offset_semitones(60), -0.5);
        assert_eq!(high.offset_semitones(60), 126.0 / 256.0);
    }

    #[test]
    fn offsets_repeat_by_chromatic_note_class() {
        let mut codes = [64; SCALE_NOTE_COUNT];
        codes[4] = 46;
        let scale = ScaleProgram::from_codes(codes).unwrap();
        assert_eq!(scale.offset_semitones(40), -36.0 / 256.0);
        assert_eq!(scale.offset_semitones(52), scale.offset_semitones(40));
        assert_eq!(scale.offset_semitones(41), 0.0);
    }

    #[test]
    fn normalized_panel_positions_quantize_to_all_raw_codes() {
        let mut reached = [false; 128];
        for raw in 0..=127 {
            let mut values = [64.0 / 127.0; SCALE_NOTE_COUNT];
            values[0] = raw as f32 / 127.0;
            let scale = ScaleProgram::from_normalized(values);
            reached[usize::from(scale.codes()[0])] = true;
        }
        assert!(reached.into_iter().all(|value| value));
    }
}
