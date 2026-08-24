//! Control-voltage-to-frequency laws for the dual-VCO voice.

use rf_5_contract::hardware::quantize_analog_pot;

const A4_MIDI_NOTE: i32 = 69;
const A4_FREQUENCY_HZ: f32 = 440.0;
const SEMITONES_PER_OCTAVE: f32 = 12.0;
const QUANTIZED_POT_CENTER: f32 = 64.0 / 127.0;

/// The five-octave keyboard contributes 0 V at its lowest C.
pub const LOWEST_KEY_MIDI_NOTE: u8 = 36;

pub fn note_frequency(note: u8) -> f32 {
    let octaves = (i32::from(note) - A4_MIDI_NOTE) as f32 / SEMITONES_PER_OCTAVE;
    A4_FREQUENCY_HZ * libm::exp2f(octaves)
}

pub fn oscillator_a_frequency(note: u8, coarse: f32) -> f32 {
    tracked_frequency(note, coarse_octaves(coarse), 0.0)
}

pub fn oscillator_b_frequency(
    note: u8,
    coarse: f32,
    fine: f32,
    keyboard_enabled: bool,
    low_frequency: bool,
) -> f32 {
    let note = if keyboard_enabled {
        note
    } else {
        LOWEST_KEY_MIDI_NOTE
    };
    let coarse = if low_frequency {
        low_frequency_octaves(coarse)
    } else {
        coarse_octaves(coarse)
    };
    tracked_frequency(note, coarse, fine_octaves(fine))
}

fn tracked_frequency(note: u8, coarse_octaves: f32, fine_octaves: f32) -> f32 {
    note_frequency(note) * libm::exp2f(coarse_octaves + fine_octaves)
}

/// The documented frequency pot contributes 0-4 V; the initial-frequency
/// trim establishes the center-reference offset used by RF-5.
fn coarse_octaves(value: f32) -> f32 {
    (quantize_analog_pot(value) - QUANTIZED_POT_CENTER) * 4.0
}

/// In LO FREQ the documented initial-frequency range expands to 9 V and the
/// hardware inserts a -7.5 V offset.
fn low_frequency_octaves(value: f32) -> f32 {
    (quantize_analog_pot(value) - QUANTIZED_POT_CENTER) * 9.0 - 7.5
}

/// Current evidence bounds the B fine control provisionally to +/-50 cents.
fn fine_octaves(value: f32) -> f32 {
    (quantize_analog_pot(value) - QUANTIZED_POT_CENTER) / 12.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_temperament_reference_notes_are_stable() {
        assert!((note_frequency(69) - 440.0).abs() < 0.001);
        assert!((note_frequency(57) - 220.0).abs() < 0.001);
        assert!((note_frequency(81) - 880.0).abs() < 0.001);
    }

    #[test]
    fn centered_coarse_controls_track_the_keyboard() {
        let a = oscillator_a_frequency(60, 0.5);
        let b = oscillator_b_frequency(60, 0.5, 0.5, true, false);
        assert!((a - b).abs() < 0.001);
        assert!((oscillator_a_frequency(72, 0.5) / a - 2.0).abs() < 0.001);
    }

    #[test]
    fn keyboard_defeat_holds_b_pitch_across_notes() {
        let low_note = oscillator_b_frequency(36, 0.5, 0.5, false, false);
        let high_note = oscillator_b_frequency(96, 0.5, 0.5, false, false);
        assert!((low_note - high_note).abs() < 0.001);
    }

    #[test]
    fn low_frequency_mode_enters_sub_audio_range() {
        let normal = oscillator_b_frequency(36, 0.5, 0.5, false, false);
        let low = oscillator_b_frequency(36, 0.5, 0.5, false, true);
        assert!(normal > 20.0);
        assert!(low < 1.0);
    }
}
