//! One free-running modulation oscillator shared by all five voices.
//!
//! The Rev 3 service manual establishes the single-oscillator topology, the
//! three additive shapes and the square wave's 50% duty cycle. The populated
//! SD334 timing, reference and scale networks now establish the absolute
//! frequency law. The CEM3340's published finite timing-capacitor current
//! bounds retain the remaining populated-device uncertainty at the fast end.

use rf_5_contract::hardware::quantize_analog_pot;
use rf_5_voice::vco::cem3340_loaded_pulse_high_volts;

// SD334 does not use an arbitrary rate calibration. Its CEM3340 is populated
// with the data-sheet scale network, a 1 uF timing capacitor and a 2.21 Mohm
// reference-current feed. Two fixed currents establish the zero-code bias and
// the 0-10 V DAC adds the panel sweep through R3136.
const POPULATED_FREQUENCY_INPUT_OHMS: f32 = 110_000.0;
const PANEL_CONTROL_RANGE_VOLTS: f32 = 10.0;
const CEM3340_POSITIVE_SUPPLY_VOLTS: f32 = 15.0;
const CEM3340_REFERENCE_RESISTANCE_OHMS: f32 = 2_210_000.0;
const CEM3340_BASE_RESISTANCE_OHMS: f32 = 1_820.0;
const CEM3340_SCALE_ZERO_RESISTANCE_OHMS: f32 = 30_100.0;
const CEM3340_SCALE_TIMING_RESISTANCE_OHMS: f32 = 5_620.0;
const FIXED_POSITIVE_SUPPLY_INPUT_OHMS: f32 = 681_000.0;
const FIXED_FIVE_VOLT_INPUT_OHMS: f32 = 101_000.0;
const FIXED_REFERENCE_VOLTS: f32 = 5.0;
const POPULATED_TIMING_CAPACITANCE_FARADS: f32 = 1.0e-6;

// The data sheet gives 400/570/800 uA minimum/typical/maximum timing-capacitor
// current. A high-order continuous limiter reaches the nominal-device ceiling
// without producing duplicate panel steps or changing the accurate sub-100 uA
// portion of the law. Its order is deliberately isolated because the data
// sheet specifies the ceiling but not the overload-knee shape.
const CEM3340_TYPICAL_MAX_TIMING_CURRENT_AMPS: f32 = 570.0e-6;
const TIMING_CURRENT_KNEE_ORDER: f32 = 16.0;

// SD334 does not AC-centre the complete LFO bus. Saw and pulse remain
// positive-going through their 4016 switches, while only triangle crosses
// U380's level shifter. R3148/R3147 give the triangle a non-inverting gain of
// two around the measured 4.97 V reference, making its 0-5 V raw waveform
// approximately -4.97 to +5.03 V. One returned unit is five circuit volts at
// R3131's 160 kohm reference path; pulse is converted to the same current
// coordinate through its populated 200 kohm path.
const CEM3340_SAW_SPAN_VOLTS: f32 = 10.0;
const CEM3340_TRIANGLE_SPAN_VOLTS: f32 = 5.0;
const LFO_PULSE_PULLDOWN_VOLTS: f32 = 0.0;
const LFO_PULSE_PULLDOWN_RESISTANCE_OHMS: f32 = 10_000.0;
const CEM3340_PULSE_SPAN_VOLTS: f32 =
    cem3340_loaded_pulse_high_volts(LFO_PULSE_PULLDOWN_VOLTS, LFO_PULSE_PULLDOWN_RESISTANCE_OHMS);
const U380_REFERENCE_VOLTS: f32 = 4.97;
const U380_TRIANGLE_INPUT_OHMS: f32 = 100_000.0;
const U380_TRIANGLE_FEEDBACK_OHMS: f32 = 100_000.0;
const LFO_SAW_AND_TRIANGLE_INPUT_OHMS: f32 = 160_000.0;
const LFO_PULSE_INPUT_OHMS: f32 = 200_000.0;
const CD4016_TYPICAL_ON_RESISTANCE_OHMS: f32 = 300.0;
const LFO_REFERENCE_VOLTS_PER_UNIT: f32 = 5.0;
const U380_TRIANGLE_SIGNAL_GAIN: f32 = 1.0 + U380_TRIANGLE_FEEDBACK_OHMS / U380_TRIANGLE_INPUT_OHMS;
const SWITCHED_SAW_INPUT_OHMS: f32 =
    LFO_SAW_AND_TRIANGLE_INPUT_OHMS + CD4016_TYPICAL_ON_RESISTANCE_OHMS;
const SWITCHED_PULSE_INPUT_OHMS: f32 = LFO_PULSE_INPUT_OHMS + CD4016_TYPICAL_ON_RESISTANCE_OHMS;
const SAW_CURRENT_COORDINATE_RATIO: f32 = LFO_SAW_AND_TRIANGLE_INPUT_OHMS / SWITCHED_SAW_INPUT_OHMS;
const PULSE_CURRENT_COORDINATE_RATIO: f32 =
    LFO_SAW_AND_TRIANGLE_INPUT_OHMS / SWITCHED_PULSE_INPUT_OHMS;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LfoWaveSelection {
    pub saw: bool,
    pub triangle: bool,
    pub square: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Lfo {
    phase: f32,
}

impl Lfo {
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    pub fn next(
        &mut self,
        sample_rate: f32,
        frequency_control: f32,
        waves: LfoWaveSelection,
    ) -> f32 {
        let phase = self.phase;
        let saw_volts = phase * CEM3340_SAW_SPAN_VOLTS;
        let raw_triangle_volts = (1.0 - (phase * 2.0 - 1.0).abs()) * CEM3340_TRIANGLE_SPAN_VOLTS;
        let conditioned_triangle_volts =
            raw_triangle_volts * U380_TRIANGLE_SIGNAL_GAIN - U380_REFERENCE_VOLTS;
        let pulse_volts = if phase < 0.5 {
            CEM3340_PULSE_SPAN_VOLTS
        } else {
            0.0
        };
        let saw = saw_volts / LFO_REFERENCE_VOLTS_PER_UNIT * SAW_CURRENT_COORDINATE_RATIO;
        let triangle = conditioned_triangle_volts / LFO_REFERENCE_VOLTS_PER_UNIT;
        let square = pulse_volts / LFO_REFERENCE_VOLTS_PER_UNIT * PULSE_CURRENT_COORDINATE_RATIO;
        let mut output = 0.0;
        if waves.saw {
            output += saw;
        }
        if waves.triangle {
            output += triangle;
        }
        if waves.square {
            output += square;
        }

        let increment = frequency_hz(frequency_control) / sample_rate.max(1.0);
        let advanced = self.phase + increment;
        self.phase = advanced - libm::floorf(advanced);
        output
    }

    #[cfg(test)]
    pub(crate) fn phase(self) -> f32 {
        self.phase
    }
}

pub fn frequency_hz(control: f32) -> f32 {
    let control = quantize_analog_pot(control);
    let control_volts = control * PANEL_CONTROL_RANGE_VOLTS;
    let generator_current = bounded_exponential_generator_current_amps(control_volts);
    3.0 * generator_current
        / (2.0 * CEM3340_POSITIVE_SUPPLY_VOLTS * POPULATED_TIMING_CAPACITANCE_FARADS)
}

fn frequency_control_current_amps(control_volts: f32) -> f32 {
    CEM3340_POSITIVE_SUPPLY_VOLTS / FIXED_POSITIVE_SUPPLY_INPUT_OHMS
        + FIXED_REFERENCE_VOLTS / FIXED_FIVE_VOLT_INPUT_OHMS
        + control_volts / POPULATED_FREQUENCY_INPUT_OHMS
}

fn unbounded_exponential_generator_current_amps(control_volts: f32) -> f32 {
    let reference_current = CEM3340_POSITIVE_SUPPLY_VOLTS / CEM3340_REFERENCE_RESISTANCE_OHMS;
    let control_current = frequency_control_current_amps(control_volts);

    // CEM3340 data-sheet equations, combined:
    // I_OM = 22 V_T / R_T * (1 - I_C R_Z / 3 V)
    // V_B  = I_OM R_S
    // I_EG = I_REF exp(-V_B / V_T)
    // V_T cancels, leaving the populated resistors and summed control current.
    let exponent = -22.0 * CEM3340_BASE_RESISTANCE_OHMS / CEM3340_SCALE_TIMING_RESISTANCE_OHMS
        * (1.0 - control_current * CEM3340_SCALE_ZERO_RESISTANCE_OHMS / 3.0);
    reference_current * libm::expf(exponent)
}

fn bounded_exponential_generator_current_amps(control_volts: f32) -> f32 {
    let raw = unbounded_exponential_generator_current_amps(control_volts);
    let ratio = raw / CEM3340_TYPICAL_MAX_TIMING_CURRENT_AMPS;
    raw / libm::powf(
        1.0 + libm::powf(ratio, TIMING_CURRENT_KNEE_ORDER),
        1.0 / TIMING_CURRENT_KNEE_ORDER,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuit_frequency_mapping_is_monotonic_and_bounded() {
        assert!((frequency_hz(0.0) - 0.090_826_68).abs() < 1.0e-6);
        assert!((frequency_hz(1.0) - 55.803_21).abs() < 0.001);
        let mut previous = frequency_hz(0.0);
        for step in 1..=127 {
            let current = frequency_hz(step as f32 / 127.0);
            assert!(current > previous);
            previous = current;
        }
    }

    #[test]
    fn populated_scale_network_sets_the_unbounded_sweep() {
        let minimum = unbounded_exponential_generator_current_amps(0.0);
        let maximum = unbounded_exponential_generator_current_amps(PANEL_CONTROL_RANGE_VOLTS);
        let ratio = maximum / minimum;
        let expected_octaves =
            10.0 * CEM3340_BASE_RESISTANCE_OHMS * 22.0 * CEM3340_SCALE_ZERO_RESISTANCE_OHMS
                / (CEM3340_SCALE_TIMING_RESISTANCE_OHMS
                    * 3.0
                    * POPULATED_FREQUENCY_INPUT_OHMS
                    * core::f32::consts::LN_2);
        assert!((libm::log2f(ratio) - expected_octaves).abs() < 1.0e-5);
        assert!((expected_octaves - 9.375_293).abs() < 1.0e-5);
        assert!((ratio - 664.116_7).abs() < 0.01);
    }

    #[test]
    fn populated_reference_and_timing_network_set_absolute_endpoints() {
        let minimum_current = unbounded_exponential_generator_current_amps(0.0);
        let maximum_current =
            unbounded_exponential_generator_current_amps(PANEL_CONTROL_RANGE_VOLTS);
        assert!((minimum_current - 0.908_266_9e-6).abs() < 1.0e-12);
        assert!((maximum_current - 603.195_2e-6).abs() < 0.001e-6);
        assert!(minimum_current > 50.0e-9);
        assert!(maximum_current > CEM3340_TYPICAL_MAX_TIMING_CURRENT_AMPS);
    }

    #[test]
    fn finite_timing_current_only_rounds_the_fastest_steps() {
        let accurate_region = 100.0e-6;
        let bounded_accurate = bounded_exponential_generator_current_amps(7.0);
        let raw_accurate = unbounded_exponential_generator_current_amps(7.0);
        assert!(raw_accurate < accurate_region);
        assert!((bounded_accurate / raw_accurate - 1.0).abs() < 1.0e-5);

        let raw_max = unbounded_exponential_generator_current_amps(PANEL_CONTROL_RANGE_VOLTS);
        let bounded_max = bounded_exponential_generator_current_amps(PANEL_CONTROL_RANGE_VOLTS);
        assert!(bounded_max < raw_max);
        assert!(bounded_max < CEM3340_TYPICAL_MAX_TIMING_CURRENT_AMPS);
        assert!(bounded_max > 550.0e-6);
    }

    #[test]
    fn analog_panel_exposes_exactly_128_distinct_frequency_steps() {
        let frequencies: [f32; 128] =
            core::array::from_fn(|step| frequency_hz(step as f32 / 127.0));
        assert_eq!(frequencies.len(), 128);
        assert!(frequencies.windows(2).all(|pair| pair[0] < pair[1]));

        for (step, expected) in frequencies.iter().enumerate().take(127) {
            let midpoint = (step as f32 + 0.49) / 127.0;
            assert_eq!(frequency_hz(midpoint), *expected);
        }
    }

    #[test]
    fn square_has_equal_high_and_low_halves_without_negative_voltage() {
        let mut lfo = Lfo::default();
        let waves = LfoWaveSelection {
            square: true,
            ..LfoWaveSelection::default()
        };
        let sample_rate = frequency_hz(1.0) * 1_000.0;
        let mut high: i32 = 0;
        let mut low: i32 = 0;
        for _ in 0..1_000 {
            let sample = lfo.next(sample_rate, 1.0, waves);
            if sample > 0.0 {
                high += 1;
            } else {
                assert_eq!(sample, 0.0);
                low += 1;
            }
        }
        assert!((high - low).abs() <= 2);
    }

    #[test]
    fn enabled_shapes_are_summed_on_the_shared_bus() {
        let mut lfo = Lfo::default();
        let all = LfoWaveSelection {
            saw: true,
            triangle: true,
            square: true,
        };
        let expected = -U380_REFERENCE_VOLTS / LFO_REFERENCE_VOLTS_PER_UNIT
            + CEM3340_PULSE_SPAN_VOLTS / LFO_REFERENCE_VOLTS_PER_UNIT
                * PULSE_CURRENT_COORDINATE_RATIO;
        assert!((lfo.next(48_000.0, 0.5, all) - expected).abs() < 1.0e-6);
    }

    #[test]
    fn board_weighting_preserves_unipolar_saw_and_pulse_but_centres_triangle() {
        let mut saw = Lfo::default();
        let mut triangle = Lfo::default();
        let mut pulse = Lfo::default();
        assert_eq!(
            saw.next(
                48_000.0,
                0.5,
                LfoWaveSelection {
                    saw: true,
                    ..LfoWaveSelection::default()
                }
            ),
            0.0
        );
        assert!(
            (triangle.next(
                48_000.0,
                0.5,
                LfoWaveSelection {
                    triangle: true,
                    ..LfoWaveSelection::default()
                }
            ) + 0.994)
                .abs()
                < 1.0e-6
        );
        let pulse_high = pulse.next(
            48_000.0,
            0.5,
            LfoWaveSelection {
                square: true,
                ..LfoWaveSelection::default()
            },
        );
        assert!(pulse_high > 2.0);
        assert_eq!(U380_TRIANGLE_SIGNAL_GAIN, 2.0);
        assert!((SAW_CURRENT_COORDINATE_RATIO - 0.998_128_5).abs() < 1.0e-6);
        assert!((pulse_high - 2.078_3).abs() < 0.000_1);
    }

    #[test]
    fn u380_conditioned_triangle_is_nearly_symmetric_about_ground() {
        let triangle_units = |phase: f32| {
            let raw = (1.0 - (phase * 2.0 - 1.0).abs()) * CEM3340_TRIANGLE_SPAN_VOLTS;
            (raw * U380_TRIANGLE_SIGNAL_GAIN - U380_REFERENCE_VOLTS) / LFO_REFERENCE_VOLTS_PER_UNIT
        };
        let low = triangle_units(0.0);
        let high = triangle_units(0.5);
        assert!((low + 0.994).abs() < 1.0e-6);
        assert!((high - 1.006).abs() < 1.0e-6);
        assert!((0.5 * (low + high) - 0.006).abs() < 1.0e-6);
    }

    #[test]
    fn lfo_pulse_span_includes_the_populated_ground_pull_down() {
        let pull_down_current_amps = CEM3340_PULSE_SPAN_VOLTS / LFO_PULSE_PULLDOWN_RESISTANCE_OHMS;

        assert!(pull_down_current_amps > 0.6e-3);
        assert!((CEM3340_PULSE_SPAN_VOLTS - 13.008_85).abs() < 1.0e-5);
    }
}
