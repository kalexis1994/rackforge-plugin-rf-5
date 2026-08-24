//! Common unison Glide circuit from SD334.
//!
//! The held Unison CV crosses a CA3280 whose output current charges C376.
//! Q309 is a matched differential pair: the divided GLIDE CV steers its tail
//! current away from the OTA bias input, so panel position controls slew rate
//! through a bounded transistor law instead of an arbitrary exponential map.

use crate::cv::COMMON_CV_SPAN_VOLTS;

const GLIDE_CV_SERIES_OHMS: f32 = 100_000.0;
const GLIDE_CV_SHUNT_OHMS: f32 = 2_700.0;
const MATCHED_PAIR_THERMAL_VOLTS: f32 = 0.025_85;

// Service test 4-4 requires at least five seconds to slew five octaves at
// panel 10. The active candidate uses the fastest compliant boundary; a
// measured serviced instrument can replace this single absolute anchor.
const FULL_GLIDE_RATE_SEMITONES_PER_SECOND: f32 = 12.0;

pub fn advance_note(current: f32, target: f32, amount: f32, sample_rate: f32) -> f32 {
    if !current.is_finite() || !target.is_finite() {
        return if target.is_finite() { target } else { 0.0 };
    }
    if !amount.is_finite() || !sample_rate.is_finite() || sample_rate <= 0.0 {
        return target;
    }
    let maximum_step = rate_semitones_per_second(amount) / sample_rate;
    current + (target - current).clamp(-maximum_step, maximum_step)
}

fn rate_semitones_per_second(amount: f32) -> f32 {
    let slowest_bias = matched_pair_bias_fraction(1.0);
    FULL_GLIDE_RATE_SEMITONES_PER_SECOND * matched_pair_bias_fraction(amount) / slowest_bias
}

fn matched_pair_bias_fraction(amount: f32) -> f32 {
    let exponent = amount.clamp(0.0, 1.0) * glide_cv_node_span_volts() / MATCHED_PAIR_THERMAL_VOLTS;
    1.0 / (1.0 + libm::expf(exponent))
}

fn glide_cv_node_span_volts() -> f32 {
    COMMON_CV_SPAN_VOLTS * GLIDE_CV_SHUNT_OHMS / (GLIDE_CV_SERIES_OHMS + GLIDE_CV_SHUNT_OHMS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populated_divider_sets_the_matched_pair_span() {
        assert!((glide_cv_node_span_volts() - 0.131_450_83).abs() < 1.0e-7);
    }

    #[test]
    fn all_panel_steps_are_monotonic_and_finite() {
        let mut previous = f32::INFINITY;
        for raw in 0..=127 {
            let rate = rate_semitones_per_second(raw as f32 / 127.0);
            assert!(rate.is_finite());
            assert!(rate > 0.0);
            assert!(rate < previous || raw == 0);
            previous = rate;
        }
    }

    #[test]
    fn matched_pair_sets_the_full_physical_rate_ratio() {
        let fastest = rate_semitones_per_second(0.0);
        let slowest = rate_semitones_per_second(1.0);
        assert!((fastest / slowest - 81.301_15).abs() < 0.001);
        assert_eq!(slowest, FULL_GLIDE_RATE_SEMITONES_PER_SECOND);
    }

    #[test]
    fn service_maximum_traverses_five_octaves_in_five_seconds() {
        let sample_rate = 48_000.0;
        let mut note = 0.0;
        for _ in 0..(sample_rate as usize * 5) {
            note = advance_note(note, 60.0, 1.0, sample_rate);
        }
        assert!((note - 60.0).abs() < 0.001, "five-octave result: {note}");
    }

    #[test]
    fn dial_six_is_a_medium_glide() {
        let seconds_for_five_octaves = 60.0 / rate_semitones_per_second(0.6);
        assert!((seconds_for_five_octaves - 0.680_75).abs() < 0.001);
    }

    #[test]
    fn minimum_glide_is_fast_but_not_a_digital_bypass() {
        let first = advance_note(0.0, 60.0, 0.0, 48_000.0);
        assert!(first > 0.0);
        assert!(first < 60.0);
        let seconds_for_five_octaves = 60.0 / rate_semitones_per_second(0.0);
        assert!((0.06..0.07).contains(&seconds_for_five_octaves));
    }

    #[test]
    fn invalid_inputs_cannot_poison_circuit_state() {
        assert_eq!(advance_note(f32::NAN, 60.0, 0.5, 48_000.0), 60.0);
        assert_eq!(advance_note(24.0, f32::NAN, 0.5, 48_000.0), 0.0);
        assert_eq!(advance_note(24.0, 60.0, f32::NAN, 48_000.0), 60.0);
        assert_eq!(advance_note(24.0, 60.0, 0.5, 0.0), 60.0);
    }
}
