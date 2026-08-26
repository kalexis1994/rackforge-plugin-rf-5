//! SD334 Wheel Mod destination network.
//!
//! The physical wheel attenuates one shared W-MOD source voltage before five
//! switches route it through three populated resistor networks. U378 and its
//! 10 kohm load now deliver circuit volts directly, so this module contains no
//! normalized-source calibration multiplier.

const OCTAVE_SEMITONES: f32 = 12.0;

// SD334 oscillator-frequency route: R3103/R3104 into U368, followed by the
// unity-gain A/B master summer. The complete oscillator path is 1 V/octave.
const OSCILLATOR_INPUT_OHMS: f32 = 182_000.0;
const OSCILLATOR_FEEDBACK_OHMS: f32 = 100_000.0;

// SD334 pulse-width route: R397/R398 and the 100 kohm first-stage feedback,
// followed by the 52.3/100 kohm common/voice summing scale. The CEM3340 PWM
// input covers the complete duty-cycle control range in 5 V.
const PULSE_WIDTH_INPUT_OHMS: f32 = 15_000.0;
const PULSE_WIDTH_FIRST_FEEDBACK_OHMS: f32 = 100_000.0;
const PULSE_WIDTH_SUM_INPUT_OHMS: f32 = 100_000.0;
const PULSE_WIDTH_SUM_FEEDBACK_OHMS: f32 = 52_300.0;
const CEM3340_PULSE_WIDTH_RANGE_VOLTS: f32 = 5.0;

// SD334 filter route: R399 into U367, followed by the unity FILT MSUM stage.
// The complete CEM3320 control path is calibrated to 1 V/octave.
const FILTER_INPUT_OHMS: f32 = 13_300.0;
const FILTER_FEEDBACK_OHMS: f32 = 100_000.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WheelModDestinations {
    pub oscillator_semitones: f32,
    pub pulse_width: f32,
    pub filter_octaves: f32,
}

pub fn destinations(source_volts: f32, wheel_amount: f32) -> WheelModDestinations {
    if !source_volts.is_finite() || !wheel_amount.is_finite() {
        return WheelModDestinations::default();
    }

    let wheel_amount = wheel_amount.clamp(0.0, 1.0);
    let source_volts = source_volts * wheel_amount;

    let oscillator_octaves = source_volts * OSCILLATOR_FEEDBACK_OHMS / OSCILLATOR_INPUT_OHMS;
    let pulse_width = source_volts * PULSE_WIDTH_FIRST_FEEDBACK_OHMS / PULSE_WIDTH_INPUT_OHMS
        * PULSE_WIDTH_SUM_FEEDBACK_OHMS
        / PULSE_WIDTH_SUM_INPUT_OHMS
        / CEM3340_PULSE_WIDTH_RANGE_VOLTS;
    let filter_octaves = source_volts * FILTER_FEEDBACK_OHMS / FILTER_INPUT_OHMS;

    WheelModDestinations {
        oscillator_semitones: oscillator_octaves * OCTAVE_SEMITONES,
        pulse_width,
        filter_octaves,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_source_volt_drives_every_destination() {
        let routed = destinations(1.0, 1.0);
        assert!((routed.oscillator_semitones - 6.593_406_7).abs() < 1.0e-5);
        assert!((routed.pulse_width - 0.697_333_34).abs() < 1.0e-5);
        assert!((routed.filter_octaves - 7.518_797).abs() < 1.0e-5);
    }

    #[test]
    fn destination_ratios_follow_the_populated_resistors() {
        let routed = destinations(0.37, 0.62);
        let oscillator_octaves = routed.oscillator_semitones / OCTAVE_SEMITONES;
        let expected_filter_ratio = (FILTER_FEEDBACK_OHMS / FILTER_INPUT_OHMS)
            / (OSCILLATOR_FEEDBACK_OHMS / OSCILLATOR_INPUT_OHMS);
        let expected_pulse_ratio = (PULSE_WIDTH_FIRST_FEEDBACK_OHMS / PULSE_WIDTH_INPUT_OHMS
            * PULSE_WIDTH_SUM_FEEDBACK_OHMS
            / PULSE_WIDTH_SUM_INPUT_OHMS
            / CEM3340_PULSE_WIDTH_RANGE_VOLTS)
            / (OSCILLATOR_FEEDBACK_OHMS / OSCILLATOR_INPUT_OHMS);

        assert!(
            (routed.filter_octaves / oscillator_octaves - expected_filter_ratio).abs() < 1.0e-5
        );
        assert!((routed.pulse_width / oscillator_octaves - expected_pulse_ratio).abs() < 1.0e-5);
    }

    #[test]
    fn wheel_is_passive_bounded_and_bipolar() {
        assert_eq!(destinations(1.0, 0.0), WheelModDestinations::default());
        assert_eq!(destinations(f32::NAN, 1.0), WheelModDestinations::default());
        assert_eq!(
            destinations(1.0, f32::INFINITY),
            WheelModDestinations::default()
        );
        assert_eq!(destinations(0.75, 2.0), destinations(0.75, 1.0));

        let positive = destinations(0.43, 0.78);
        let negative = destinations(-0.43, 0.78);
        assert_eq!(
            positive.oscillator_semitones,
            -negative.oscillator_semitones
        );
        assert_eq!(positive.pulse_width, -negative.pulse_width);
        assert_eq!(positive.filter_octaves, -negative.filter_octaves);
    }
}
