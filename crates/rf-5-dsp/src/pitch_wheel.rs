//! SD334 pitch-wheel deadband and serviced performance span.
//!
//! The centred R1 wiper is trimmed to 0 V at P301-7. Anti-parallel 1N914
//! diodes D315/D316 isolate that node from the tune summer until its magnitude
//! exceeds one forward drop. The downstream 1 Mohm / 100 kohm path attenuates
//! the remaining voltage by ten before it reaches both oscillator summers.

pub const RANGE_SEMITONES: f32 = 7.0;

const DIODE_FORWARD_VOLTS: f32 = 0.6;
const MASTER_TUNE_INPUT_OHMS: f32 = 1_000_000.0;
const MASTER_TUNE_FEEDBACK_OHMS: f32 = 100_000.0;
const VOLTS_PER_OCTAVE: f32 = 1.0;
const SEMITONES_PER_OCTAVE: f32 = 12.0;
#[cfg(test)]
const SERVICE_CENTER_TOLERANCE_VOLTS: f32 = 0.05;

const DOWNSTREAM_GAIN: f32 = MASTER_TUNE_FEEDBACK_OHMS / MASTER_TUNE_INPUT_OHMS;
const ENDPOINT_SUMMER_VOLTS: f32 = RANGE_SEMITONES / SEMITONES_PER_OCTAVE * VOLTS_PER_OCTAVE;
const ENDPOINT_WIPER_VOLTS: f32 = ENDPOINT_SUMMER_VOLTS / DOWNSTREAM_GAIN + DIODE_FORWARD_VOLTS;
const DEADBAND_NORMALIZED: f32 = DIODE_FORWARD_VOLTS / ENDPOINT_WIPER_VOLTS;

pub fn normalized_output(value: u16) -> f32 {
    let input = midi_normalized(value);
    let magnitude = input.abs();
    if magnitude <= DEADBAND_NORMALIZED {
        0.0
    } else {
        input.signum() * (magnitude - DEADBAND_NORMALIZED) / (1.0 - DEADBAND_NORMALIZED)
    }
}

fn midi_normalized(value: u16) -> f32 {
    let value = value.min(16_383);
    if value < 8_192 {
        (f32::from(value) - 8_192.0) / 8_192.0
    } else {
        (f32::from(value) - 8_192.0) / 8_191.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anti_parallel_diodes_create_a_symmetric_center_deadband() {
        assert!((0.09..0.10).contains(&DEADBAND_NORMALIZED));
        assert_eq!(normalized_output(8_192), 0.0);
        assert_eq!(normalized_output(8_192 + 700), 0.0);
        assert_eq!(normalized_output(8_192 - 700), 0.0);
        assert!(normalized_output(8_192 + 800) > 0.0);
        assert!(normalized_output(8_192 - 800) < 0.0);
    }

    #[test]
    fn serviced_center_tolerance_cannot_reach_the_summers() {
        let normalized_tolerance = SERVICE_CENTER_TOLERANCE_VOLTS / ENDPOINT_WIPER_VOLTS;
        assert!(normalized_tolerance < DEADBAND_NORMALIZED);
        let midi_tolerance = (normalized_tolerance * 8_191.0).ceil() as u16;
        assert_eq!(normalized_output(8_192 + midi_tolerance), 0.0);
        assert_eq!(normalized_output(8_192 - midi_tolerance), 0.0);
    }

    #[test]
    fn full_midi_range_preserves_the_documented_fifth_endpoints() {
        assert_eq!(normalized_output(0), -1.0);
        assert_eq!(normalized_output(16_383), 1.0);
        assert_eq!(normalized_output(0) * RANGE_SEMITONES, -7.0);
        assert_eq!(normalized_output(16_383) * RANGE_SEMITONES, 7.0);
    }

    #[test]
    fn circuit_transfer_is_monotonic_and_bounded() {
        let mut previous = -1.0;
        for value in 0..=16_383 {
            let output = normalized_output(value);
            assert!((-1.0..=1.0).contains(&output));
            assert!(output >= previous);
            previous = output;
        }
    }
}
