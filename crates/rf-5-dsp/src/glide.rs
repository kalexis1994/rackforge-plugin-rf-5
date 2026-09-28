//! Common unison Glide circuit from SD334.
//!
//! The held Unison CV crosses U381, an unlinearized CA3280 (its ID terminal
//! is tied to -15 V) whose output current charges C376. D318/D319 bound its
//! differential input, so for any audible interval the OTA runs at its peak
//! output current and C376 slews linearly; only the last few tens of
//! millivolts follow the OTA's tanh law. Q309, an AD820 matched PNP pair,
//! sets that current: R3126 (30k from +15 V) feeds the common emitters, the
//! right base and collector are grounded, and the left base receives the
//! GLIDE CV through R3124 100k / R3125 2.7k. The left collector reaches U381's
//! IABC through R3123 100k. Every constant below is a populated part; no
//! panel-time anchor is fitted.

use rf_5_contract::hardware::general_control_volts;

const GLIDE_CV_SERIES_OHMS: f32 = 100_000.0;
const GLIDE_CV_SHUNT_OHMS: f32 = 2_700.0;
const MATCHED_PAIR_THERMAL_VOLTS: f32 = 0.025_85;
const TAIL_SUPPLY_VOLTS: f32 = 15.0;
const TAIL_RESISTOR_OHMS: f32 = 30_000.0;
const PNP_BASE_EMITTER_VOLTS: f32 = 0.65;
// U381's IABC terminal sits two junctions above its -15 V rail. The left
// collector can only fall through R3123 to that node until Q309 saturates
// (collector reaching roughly its base voltage).
const IABC_TERMINAL_VOLTS: f32 = -15.0 + 2.0 * 0.6;
const IABC_SERIES_OHMS: f32 = 100_000.0;
const PNP_SATURATION_MARGIN_VOLTS: f32 = 0.3;
// CA3280 data sheet: 410 uA peak output at 500 uA IABC.
const CA3280_PEAK_OUTPUT_CURRENT_RATIO: f32 = 410.0 / 500.0;
const GLIDE_CAPACITANCE_FARADS: f32 = 0.1e-6;
const SEMITONES_PER_VOLT: f32 = 12.0;

/// Peak slew of the Glide output in semitones per second for a panel value.
pub(crate) fn rate_semitones_per_second(amount: f32) -> f32 {
    CA3280_PEAK_OUTPUT_CURRENT_RATIO * u381_bias_current_amps(amount) / GLIDE_CAPACITANCE_FARADS
        * SEMITONES_PER_VOLT
}

/// Advance the C376 voltage, expressed in keyboard semitones, by one sample.
/// Large intervals slew at the peak rate; the OTA's tanh transfer closes the
/// final few tens of millivolts without overshoot.
pub(crate) fn advance_note(current: f64, target: f64, rate: f32, sample_rate: f32) -> f64 {
    if !current.is_finite() || !target.is_finite() {
        return if target.is_finite() { target } else { 0.0 };
    }
    if !rate.is_finite() || !sample_rate.is_finite() || sample_rate <= 0.0 {
        return target;
    }
    let difference = target - current;
    let differential_volts = difference / f64::from(SEMITONES_PER_VOLT);
    let shaped = libm::tanh(differential_volts / (2.0 * f64::from(MATCHED_PAIR_THERMAL_VOLTS)));
    let step = f64::from(rate) / f64::from(sample_rate) * shaped;
    if step.abs() >= difference.abs() {
        target
    } else {
        current + step
    }
}

fn u381_bias_current_amps(amount: f32) -> f32 {
    let base_volts = general_control_volts(amount) * GLIDE_CV_SHUNT_OHMS
        / (GLIDE_CV_SERIES_OHMS + GLIDE_CV_SHUNT_OHMS);
    let tail_amps = (TAIL_SUPPLY_VOLTS - PNP_BASE_EMITTER_VOLTS) / TAIL_RESISTOR_OHMS;
    let left_collector_amps =
        tail_amps / (1.0 + libm::expf(base_volts / MATCHED_PAIR_THERMAL_VOLTS));
    let saturation_limit_amps =
        (base_volts + PNP_SATURATION_MARGIN_VOLTS - IABC_TERMINAL_VOLTS) / IABC_SERIES_OHMS;
    left_collector_amps.min(saturation_limit_amps)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seconds_for_five_octaves(amount: f32) -> f32 {
        let sample_rate = 48_000.0;
        let rate = rate_semitones_per_second(amount);
        let mut note = 0.0_f64;
        let mut samples = 0_u32;
        while (60.0 - note) > 0.01 && samples < 48_000 * 120 {
            note = advance_note(note, 60.0, rate, sample_rate);
            samples += 1;
        }
        samples as f32 / sample_rate
    }

    #[test]
    fn populated_divider_sets_the_matched_pair_span() {
        // Code 120 is the 10 V panel ceiling of the ordinary DAC domain.
        let ceiling = general_control_volts(120.0 / 127.0) * GLIDE_CV_SHUNT_OHMS
            / (GLIDE_CV_SERIES_OHMS + GLIDE_CV_SHUNT_OHMS);
        assert!((ceiling - 0.262_902).abs() < 1.0e-5);
    }

    #[test]
    fn all_panel_steps_are_monotonic_and_finite() {
        let mut previous = f32::INFINITY;
        for raw in 0..=127 {
            let rate = rate_semitones_per_second(raw as f32 / 127.0);
            assert!(rate.is_finite());
            assert!(rate > 0.0);
            // Q309's saturation bound rises by microvolts with the base; the
            // tail steering dominates everywhere else.
            assert!(rate <= previous * 1.001);
            previous = rate;
        }
    }

    #[test]
    fn q309_saturation_bounds_the_fastest_glide() {
        let fastest = u381_bias_current_amps(0.0);
        assert!((135.0e-6..=150.0e-6).contains(&fastest), "{fastest}");
        // A few milliseconds for five octaves: effectively instant.
        assert!(seconds_for_five_octaves(0.0) < 0.01);
    }

    #[test]
    fn service_maximum_takes_longer_than_five_seconds_for_five_octaves() {
        // Service test 4-4: at least five seconds at 10 (code 120).
        let seconds = seconds_for_five_octaves(120.0 / 127.0);
        assert!((20.0..=45.0).contains(&seconds), "{seconds}");
    }

    #[test]
    fn dial_six_is_a_medium_glide() {
        let seconds = seconds_for_five_octaves(72.0 / 127.0);
        assert!((0.3..=0.9).contains(&seconds), "{seconds}");
    }

    #[test]
    fn final_approach_does_not_overshoot() {
        let rate = rate_semitones_per_second(0.0);
        let mut note = 0.0_f64;
        for _ in 0..4_800 {
            note = advance_note(note, 12.0, rate, 44_100.0);
            assert!(note <= 12.0);
        }
        assert!((note - 12.0).abs() < 1.0e-3);
    }

    #[test]
    fn invalid_inputs_cannot_poison_circuit_state() {
        assert_eq!(advance_note(f64::NAN, 60.0, 100.0, 48_000.0), 60.0);
        assert_eq!(advance_note(24.0, f64::NAN, 100.0, 48_000.0), 0.0);
        assert_eq!(advance_note(24.0, 60.0, f32::NAN, 48_000.0), 60.0);
        assert_eq!(advance_note(24.0, 60.0, 100.0, 0.0), 60.0);
    }
}
