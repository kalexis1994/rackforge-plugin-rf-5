//! Control-voltage-to-frequency laws for the dual-VCO voice.

use rf_5_contract::hardware::{
    OSCILLATOR_FREQUENCY_LOW_MAX_SEMITONES, OSCILLATOR_FREQUENCY_LOW_OFFSET_SEMITONES,
    OSCILLATOR_FREQUENCY_NORMAL_MAX_SEMITONES, quantize_analog_pot,
};

const A4_MIDI_NOTE: i32 = 69;
const A4_FREQUENCY_HZ: f32 = 440.0;
const SEMITONES_PER_OCTAVE: f32 = 12.0;
const TUNE_TABLE_MAX_SEMITONE: i32 = 108;

/// The five-octave keyboard contributes 0 V at its lowest C.
pub const LOWEST_KEY_MIDI_NOTE: u8 = 36;

/// The pitch components that the original control path keeps separate.
/// `output_semitones` includes the analog LO FREQ offset, while
/// `tune_dac_semitones` is the value presented to the individual oscillator
/// DAC before that offset. The automatic-tune lookup uses only the integer
/// coordinate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OscillatorPitch {
    output_semitones: f32,
    tune_dac_semitones: f32,
    tune_table_semitone: u8,
}

impl OscillatorPitch {
    pub const fn output_semitones(self) -> f32 {
        self.output_semitones
    }

    pub const fn tune_dac_semitones(self) -> f32 {
        self.tune_dac_semitones
    }

    pub const fn tune_table_semitone(self) -> u8 {
        self.tune_table_semitone
    }
}

pub fn note_frequency(note: u8) -> f32 {
    let octaves = (i32::from(note) - A4_MIDI_NOTE) as f32 / SEMITONES_PER_OCTAVE;
    A4_FREQUENCY_HZ * libm::exp2f(octaves)
}

pub fn oscillator_a_frequency(note: u8, coarse: f32) -> f32 {
    frequency_from_c0(oscillator_a_pitch(note, coarse).output_semitones)
}

/// Ideal oscillator-A CV position expressed in semitones above C0.
pub fn oscillator_a_tuning_semitones(note: u8, coarse: f32) -> f32 {
    oscillator_a_pitch(note, coarse).output_semitones
}

pub fn oscillator_a_pitch(note: u8, coarse: f32) -> OscillatorPitch {
    let coarse = i32::from(normal_frequency_code(coarse));
    pitch_control(note, coarse, true, 0.0, 0.0)
}

pub fn oscillator_b_frequency(
    note: u8,
    coarse: f32,
    fine: f32,
    keyboard_enabled: bool,
    low_frequency: bool,
) -> f32 {
    frequency_from_c0(
        oscillator_b_pitch(note, coarse, fine, keyboard_enabled, low_frequency).output_semitones,
    )
}

/// Ideal oscillator-B CV position expressed in semitones above C0.
pub fn oscillator_b_tuning_semitones(
    note: u8,
    coarse: f32,
    fine: f32,
    keyboard_enabled: bool,
    low_frequency: bool,
) -> f32 {
    oscillator_b_pitch(note, coarse, fine, keyboard_enabled, low_frequency).output_semitones
}

pub fn oscillator_b_pitch(
    note: u8,
    coarse: f32,
    fine: f32,
    keyboard_enabled: bool,
    low_frequency: bool,
) -> OscillatorPitch {
    let (coarse, analog_offset) = if low_frequency {
        (
            i32::from(low_frequency_code(coarse)),
            OSCILLATOR_FREQUENCY_LOW_OFFSET_SEMITONES,
        )
    } else {
        (i32::from(normal_frequency_code(coarse)), 0.0)
    };
    pitch_control(
        note,
        coarse,
        keyboard_enabled,
        fine_semitones(fine),
        analog_offset,
    )
}

fn pitch_control(
    note: u8,
    coarse_semitones: i32,
    keyboard_enabled: bool,
    fine_semitones: f32,
    analog_offset_semitones: f32,
) -> OscillatorPitch {
    let keyboard = if keyboard_enabled {
        i32::from(note) - i32::from(LOWEST_KEY_MIDI_NOTE)
    } else {
        0
    };
    let tune_table_semitone = (keyboard + coarse_semitones).clamp(0, TUNE_TABLE_MAX_SEMITONE);
    let tune_dac_semitones = tune_table_semitone as f32 + fine_semitones;
    OscillatorPitch {
        output_semitones: tune_dac_semitones + analog_offset_semitones,
        tune_dac_semitones,
        tune_table_semitone: tune_table_semitone as u8,
    }
}

fn normalized_pot_code(value: f32) -> u8 {
    libm::roundf(quantize_analog_pot(value) * 127.0) as u8
}

fn normal_frequency_code(value: f32) -> u8 {
    (normalized_pot_code(value) >> 1).min(OSCILLATOR_FREQUENCY_NORMAL_MAX_SEMITONES)
}

fn low_frequency_code(value: f32) -> u8 {
    normalized_pot_code(value).min(OSCILLATOR_FREQUENCY_LOW_MAX_SEMITONES)
}

/// OSC B FINE enters the common analog sum after the tune-table calculation.
/// The owner's manual specifies zero as no detuning and a one-semitone upward
/// range from the coarse frequency setting.
fn fine_semitones(value: f32) -> f32 {
    f32::from(normalized_pot_code(value)) / 127.0
}

fn frequency_from_c0(semitones: f32) -> f32 {
    let c0 = A4_FREQUENCY_HZ * libm::exp2f((12 - A4_MIDI_NOTE) as f32 / SEMITONES_PER_OCTAVE);
    c0 * libm::exp2f(semitones / SEMITONES_PER_OCTAVE)
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
    fn concert_coarse_controls_track_the_keyboard() {
        let concert = 48.0 / 127.0;
        let a = oscillator_a_frequency(60, concert);
        let b = oscillator_b_frequency(60, concert, 0.0, true, false);
        assert!((a - b).abs() < 0.001);
        assert!((oscillator_a_frequency(72, concert) / a - 2.0).abs() < 0.001);
    }

    #[test]
    fn tune_coordinates_share_the_frequency_control_law() {
        let concert = 48.0 / 127.0;
        let a_coordinate = oscillator_a_tuning_semitones(60, concert);
        let b_coordinate = oscillator_b_tuning_semitones(60, concert, 0.0, true, false);
        assert!((a_coordinate - b_coordinate).abs() < 1.0e-6);
        assert!((a_coordinate - 48.0).abs() < 0.1);
    }

    #[test]
    fn keyboard_defeat_holds_b_pitch_across_notes() {
        let low_note = oscillator_b_frequency(36, 48.0 / 127.0, 0.5, false, false);
        let high_note = oscillator_b_frequency(96, 48.0 / 127.0, 0.5, false, false);
        assert!((low_note - high_note).abs() < 0.001);
    }

    #[test]
    fn low_frequency_mode_enters_sub_audio_range() {
        let normal = oscillator_b_frequency(36, 48.0 / 127.0, 0.5, false, false);
        let low = oscillator_b_frequency(36, 0.5, 0.5, false, true);
        assert!(normal > 20.0);
        assert!(low < 5.0);
    }

    #[test]
    fn normal_frequency_control_matches_v81_integer_codes() {
        assert_eq!(normal_frequency_code(0.0), 0);
        assert_eq!(normal_frequency_code(48.0 / 127.0), 24);
        assert_eq!(normal_frequency_code(96.0 / 127.0), 48);
        assert_eq!(normal_frequency_code(1.0), 48);
        let mut seen = [false; 49];
        for raw in 0..=127 {
            seen[usize::from(normal_frequency_code(raw as f32 / 127.0))] = true;
        }
        assert!(seen.into_iter().all(|reachable| reachable));
    }

    #[test]
    fn low_frequency_control_uses_nine_octaves_then_analog_offset() {
        let bottom = oscillator_b_pitch(LOWEST_KEY_MIDI_NOTE, 0.0, 0.0, false, true);
        let top = oscillator_b_pitch(LOWEST_KEY_MIDI_NOTE, 1.0, 0.0, false, true);
        assert_eq!(bottom.tune_table_semitone(), 0);
        assert_eq!(top.tune_table_semitone(), 108);
        assert_eq!(bottom.output_semitones(), -90.0);
        assert_eq!(top.output_semitones(), 18.0);
    }

    #[test]
    fn b_fine_does_not_move_the_integer_tune_lookup() {
        let flat = oscillator_b_pitch(60, 48.0 / 127.0, 0.0, true, false);
        let sharp = oscillator_b_pitch(60, 48.0 / 127.0, 1.0, true, false);
        assert_eq!(flat.tune_table_semitone(), sharp.tune_table_semitone());
        assert!((sharp.tune_dac_semitones() - flat.tune_dac_semitones() - 1.0).abs() < 0.01);
    }

    #[test]
    fn b_fine_starts_at_unison_and_rises_one_semitone() {
        assert_eq!(fine_semitones(0.0), 0.0);
        assert_eq!(fine_semitones(1.0), 1.0);

        let concert = 48.0 / 127.0;
        let a = oscillator_a_frequency(60, concert);
        let b_flat = oscillator_b_frequency(60, concert, 0.0, true, false);
        let b_sharp = oscillator_b_frequency(60, concert, 1.0, true, false);
        assert!((b_flat / a - 1.0).abs() < 1.0e-6);
        assert!((b_sharp / b_flat - libm::exp2f(1.0 / 12.0)).abs() < 1.0e-6);
    }

    #[test]
    fn b_fine_exposes_all_128_one_sided_hardware_steps() {
        let values: [f32; 128] = core::array::from_fn(|code| fine_semitones(code as f32 / 127.0));
        assert_eq!(values[0], 0.0);
        assert_eq!(values[127], 1.0);
        assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
    }
}
