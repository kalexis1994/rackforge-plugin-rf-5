//! Pulse-width control law shared by both CEM3340 candidates.
//!
//! The panel control covers approximately 1-99% duty cycle. Modulation is
//! summed afterwards at the board CV node and can drive the oscillator to the
//! 0% or 100% DC endpoints described by the owner's manual.

use rf_5_contract::hardware::quantize_analog_pot;

pub const PANEL_MINIMUM_DUTY: f32 = 0.01;
pub const PANEL_MAXIMUM_DUTY: f32 = 0.99;

pub fn panel_duty_cycle(control: f32) -> f32 {
    PANEL_MINIMUM_DUTY + quantize_analog_pot(control) * (PANEL_MAXIMUM_DUTY - PANEL_MINIMUM_DUTY)
}

pub fn add_modulation(duty_cycle: f32, modulation: f32) -> f32 {
    (duty_cycle + modulation).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_reaches_the_documented_one_and_ninety_nine_percent_limits() {
        assert_eq!(panel_duty_cycle(0.0), PANEL_MINIMUM_DUTY);
        assert_eq!(panel_duty_cycle(1.0), PANEL_MAXIMUM_DUTY);
        assert!((panel_duty_cycle(0.5) - 0.5).abs() < 0.005);
    }

    #[test]
    fn all_128_panel_codes_are_distinct_and_monotonic() {
        let mut previous = panel_duty_cycle(0.0);
        for code in 1..=127 {
            let current = panel_duty_cycle(code as f32 / 127.0);
            assert!(current > previous);
            previous = current;
        }
    }

    #[test]
    fn summed_modulation_can_reach_both_dc_endpoints() {
        assert_eq!(add_modulation(panel_duty_cycle(0.0), -0.02), 0.0);
        assert_eq!(add_modulation(panel_duty_cycle(1.0), 0.02), 1.0);
    }
}
