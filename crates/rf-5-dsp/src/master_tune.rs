//! Direct analog MASTER TUNE path on Rev 3 PCB 3 (SD334).
//!
//! R104 is a 100 kohm linear potentiometer from the +5 V analog rail to
//! ground. Its wiper reaches U369 through 1 Mohm; the 100 kohm feedback path
//! attenuates it by ten before the unity-gain A/B master summers. The
//! potentiometer's Thevenin resistance is retained here because loading by the
//! 1 Mohm input makes the two excursions around the centre detent slightly
//! asymmetric.

const POT_SUPPLY_VOLTS: f32 = 5.0;
const POT_RESISTANCE_OHMS: f32 = 100_000.0;
const INPUT_RESISTANCE_OHMS: f32 = 1_000_000.0;
const FIRST_SUMMER_FEEDBACK_OHMS: f32 = 100_000.0;
const FINAL_SUMMER_GAIN: f32 = 1.0;
const SEMITONES_PER_VOLT: f32 = 12.0;
const CENTRE_POSITION: f32 = 0.5;

/// Common pitch offset applied to oscillator A and B after automatic tuning.
pub fn offset_semitones(position: f32) -> f32 {
    if !position.is_finite() {
        return 0.0;
    }
    let position = position.clamp(0.0, 1.0);
    let centre_volts = loaded_wiper_volts(CENTRE_POSITION);
    let output_volts = (loaded_wiper_volts(position) - centre_volts) * FIRST_SUMMER_FEEDBACK_OHMS
        / INPUT_RESISTANCE_OHMS
        * FINAL_SUMMER_GAIN;
    output_volts * SEMITONES_PER_VOLT
}

fn loaded_wiper_volts(position: f32) -> f32 {
    let source_volts = POT_SUPPLY_VOLTS * position;
    let source_resistance = POT_RESISTANCE_OHMS * position * (1.0 - position);
    source_volts * INPUT_RESISTANCE_OHMS / (INPUT_RESISTANCE_OHMS + source_resistance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centre_detent_is_concert_pitch() {
        assert_eq!(offset_semitones(CENTRE_POSITION), 0.0);
        assert_eq!(offset_semitones(f32::NAN), 0.0);
    }

    #[test]
    fn populated_network_reaches_beyond_one_semitone_both_ways() {
        let flat = offset_semitones(0.0);
        let sharp = offset_semitones(1.0);
        assert!((flat - -2.926_829_3).abs() < 0.000_01);
        assert!((sharp - 3.073_170_7).abs() < 0.000_01);
        assert!(flat < -1.0);
        assert!(sharp > 1.0);
    }

    #[test]
    fn loaded_linear_pot_is_continuous_and_monotonic() {
        let mut previous = offset_semitones(0.0);
        for step in 1..=1_000 {
            let value = offset_semitones(step as f32 / 1_000.0);
            assert!(value > previous);
            previous = value;
        }
    }
}
