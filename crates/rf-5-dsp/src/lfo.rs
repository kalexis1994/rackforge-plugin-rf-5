//! One free-running modulation oscillator shared by all five voices.
//!
//! The Rev 3 service manual establishes the single-oscillator topology, the
//! three additive shapes and the square wave's 50% duty cycle. The populated
//! 110 kohm frequency-CV input establishes the panel sweep width; the absolute
//! upper-frequency anchor remains isolated until measurements can narrow it.

use rf_5_contract::hardware::quantize_analog_pot;
use rf_5_voice::vco::cem3340_loaded_pulse_high_volts;

// A standard 100 kohm CEM3340 control input produces one octave per volt. The
// SD334 LFO instead populates 110 kohm and the control DAC traverses 0-10 V,
// giving 10 * 100/110 = 9.0909 octaves. This fixes the sweep ratio from the
// circuit even though no accepted source specifies either absolute endpoint.
const STANDARD_ONE_VOLT_PER_OCTAVE_INPUT_OHMS: f32 = 100_000.0;
const POPULATED_FREQUENCY_INPUT_OHMS: f32 = 110_000.0;
const PANEL_CONTROL_RANGE_VOLTS: f32 = 10.0;
const CIRCUIT_SWEEP_OCTAVES: f32 = PANEL_CONTROL_RANGE_VOLTS
    * STANDARD_ONE_VOLT_PER_OCTAVE_INPUT_OHMS
    / POPULATED_FREQUENCY_INPUT_OHMS;

// Twenty hertz remains an explicit calibration hypothesis, not a measurement.
// SD334 populates C381 at 0.1 uF and the CEM3340 data sheet gives
// f = 3 I_EG / (2 V_CC C_F). The resulting 20 uA high-end generator current
// and 36.7 nA low-end current keep that uncertainty explicit and testable.
const CANDIDATE_MAXIMUM_HZ: f32 = 20.0;
#[cfg(test)]
const CEM3340_POSITIVE_SUPPLY_VOLTS: f32 = 15.0;
#[cfg(test)]
const POPULATED_TIMING_CAPACITANCE_FARADS: f32 = 0.1e-6;

// SD334 sends saw and U380-conditioned triangle through 160 kohm paths and
// pulse through 200 kohm. U380's equal 100 kohm input/feedback resistors give
// the triangle a signal gain of two around its reference, so its original 5 V
// span becomes the same 10 V span as saw before both reach the OTA input.
const CEM3340_SAW_SPAN_VOLTS: f32 = 10.0;
const CEM3340_TRIANGLE_SPAN_VOLTS: f32 = 5.0;
const LFO_PULSE_PULLDOWN_VOLTS: f32 = 0.0;
const LFO_PULSE_PULLDOWN_RESISTANCE_OHMS: f32 = 10_000.0;
const CEM3340_PULSE_SPAN_VOLTS: f32 =
    cem3340_loaded_pulse_high_volts(LFO_PULSE_PULLDOWN_VOLTS, LFO_PULSE_PULLDOWN_RESISTANCE_OHMS);
const U380_TRIANGLE_INPUT_OHMS: f32 = 100_000.0;
const U380_TRIANGLE_FEEDBACK_OHMS: f32 = 100_000.0;
const LFO_SAW_AND_TRIANGLE_INPUT_OHMS: f32 = 160_000.0;
const LFO_PULSE_INPUT_OHMS: f32 = 200_000.0;
const U380_TRIANGLE_SIGNAL_GAIN: f32 = 1.0 + U380_TRIANGLE_FEEDBACK_OHMS / U380_TRIANGLE_INPUT_OHMS;
const SAW_INPUT_CURRENT_SPAN_AMPS: f32 = CEM3340_SAW_SPAN_VOLTS / LFO_SAW_AND_TRIANGLE_INPUT_OHMS;
const SAW_GAIN: f32 = 1.0;
const TRIANGLE_GAIN: f32 = (CEM3340_TRIANGLE_SPAN_VOLTS * U380_TRIANGLE_SIGNAL_GAIN
    / LFO_SAW_AND_TRIANGLE_INPUT_OHMS)
    / SAW_INPUT_CURRENT_SPAN_AMPS;
const PULSE_GAIN: f32 =
    (CEM3340_PULSE_SPAN_VOLTS / LFO_PULSE_INPUT_OHMS) / SAW_INPUT_CURRENT_SPAN_AMPS;

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
        let saw = (phase * 2.0 - 1.0) * SAW_GAIN;
        let triangle = (1.0 - 4.0 * (phase - 0.5).abs()) * TRIANGLE_GAIN;
        let square = if phase < 0.5 { PULSE_GAIN } else { -PULSE_GAIN };
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
    CANDIDATE_MAXIMUM_HZ * libm::powf(2.0, CIRCUIT_SWEEP_OCTAVES * (control - 1.0))
}

#[cfg(test)]
fn exponential_generator_current_amps(frequency_hz: f32) -> f32 {
    frequency_hz.max(0.0)
        * 2.0
        * CEM3340_POSITIVE_SUPPLY_VOLTS
        * POPULATED_TIMING_CAPACITANCE_FARADS
        / 3.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuit_frequency_mapping_is_monotonic_and_bounded() {
        let minimum = CANDIDATE_MAXIMUM_HZ / libm::powf(2.0, CIRCUIT_SWEEP_OCTAVES);
        assert!((frequency_hz(0.0) - minimum).abs() < 1.0e-6);
        assert!((frequency_hz(1.0) - CANDIDATE_MAXIMUM_HZ).abs() < 1.0e-4);
        let mut previous = frequency_hz(0.0);
        for step in 1..=127 {
            let current = frequency_hz(step as f32 / 127.0);
            assert!(current > previous);
            previous = current;
        }
    }

    #[test]
    fn populated_frequency_input_sets_the_full_sweep_ratio() {
        let ratio = frequency_hz(1.0) / frequency_hz(0.0);
        let expected = libm::powf(2.0, 10.0 / 1.1);
        assert!((ratio - expected).abs() < 0.001);
        assert!((ratio - 545.30).abs() < 0.1);
    }

    #[test]
    fn populated_timing_capacitance_bounds_the_candidate_generator_current() {
        let minimum_hz = frequency_hz(0.0);
        let maximum_current = exponential_generator_current_amps(frequency_hz(1.0));
        let minimum_current = exponential_generator_current_amps(minimum_hz);

        assert!((maximum_current - 20.0e-6).abs() < 1.0e-10);
        assert!((minimum_current - 36.675e-9).abs() < 0.01e-9);
        assert!(maximum_current < 100.0e-6);
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
    fn square_has_equal_positive_and_negative_halves() {
        let mut lfo = Lfo::default();
        let waves = LfoWaveSelection {
            square: true,
            ..LfoWaveSelection::default()
        };
        let sample_rate = 20_000.0;
        let mut positive: i32 = 0;
        let mut negative: i32 = 0;
        for _ in 0..1_000 {
            let sample = lfo.next(sample_rate, 1.0, waves);
            if sample > 0.0 {
                positive += 1;
            } else {
                negative += 1;
            }
        }
        assert!((positive - negative).abs() <= 2);
    }

    #[test]
    fn enabled_shapes_are_summed_on_the_shared_bus() {
        let mut lfo = Lfo::default();
        let all = LfoWaveSelection {
            saw: true,
            triangle: true,
            square: true,
        };
        let expected = -SAW_GAIN - TRIANGLE_GAIN + PULSE_GAIN;
        assert!((lfo.next(48_000.0, 0.5, all) - expected).abs() < 1.0e-6);
    }

    #[test]
    fn board_weighting_includes_u380_triangle_conditioning() {
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
            -SAW_GAIN
        );
        assert_eq!(
            triangle.next(
                48_000.0,
                0.5,
                LfoWaveSelection {
                    triangle: true,
                    ..LfoWaveSelection::default()
                }
            ),
            -TRIANGLE_GAIN
        );
        assert_eq!(
            pulse.next(
                48_000.0,
                0.5,
                LfoWaveSelection {
                    square: true,
                    ..LfoWaveSelection::default()
                }
            ),
            PULSE_GAIN
        );
        assert_eq!(U380_TRIANGLE_SIGNAL_GAIN, 2.0);
        assert_eq!(TRIANGLE_GAIN, SAW_GAIN);
        assert!((PULSE_GAIN - 1.040_708).abs() < 1.0e-6);
    }

    #[test]
    fn lfo_pulse_span_includes_the_populated_ground_pull_down() {
        let pull_down_current_amps = CEM3340_PULSE_SPAN_VOLTS / LFO_PULSE_PULLDOWN_RESISTANCE_OHMS;

        assert!(pull_down_current_amps > 0.6e-3);
        assert!((CEM3340_PULSE_SPAN_VOLTS - 13.008_85).abs() < 1.0e-5);
    }
}
