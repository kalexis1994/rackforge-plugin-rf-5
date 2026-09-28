//! Pulse-width control law shared by both CEM3340 candidates.
//!
//! The held PW CV (1/12 V per stored code) crosses two inverting summers:
//! SD334's PW MSUM at unity and the voice card's U432/U433-class summer with
//! R4163 100k in and R4162 52.3k feedback (R351/R350 for oscillator B). No
//! offset resistor is populated, so CEM3340 pin 5 is 0.523 times the held CV.
//! The data sheet's 0-5 V pin range spans 0-100% duty. Panel code 0 is
//! therefore DC, about code 57 is the square wave the owner's manual finds
//! "at approximately 5", and the top of the panel thins out to DC again, as
//! the manual also describes. Modulation sums at the same pin.

use rf_5_contract::hardware::{general_control_volts, quantize_analog_pot};

const PULSE_WIDTH_SUMMER_GAIN: f32 = 52_300.0 / 100_000.0;
const CEM3340_PULSE_WIDTH_RANGE_VOLTS: f32 = 5.0;

pub fn panel_duty_cycle(control: f32) -> f32 {
    (general_control_volts(quantize_analog_pot(control)) * PULSE_WIDTH_SUMMER_GAIN
        / CEM3340_PULSE_WIDTH_RANGE_VOLTS)
        .clamp(0.0, 1.0)
}

pub fn add_modulation(duty_cycle: f32, modulation: f32) -> f32 {
    (duty_cycle + modulation).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(code: u8) -> f32 {
        f32::from(code) / 127.0
    }

    #[test]
    fn panel_ends_thin_out_to_dc_and_code_57_is_square() {
        assert_eq!(panel_duty_cycle(0.0), 0.0);
        assert_eq!(panel_duty_cycle(code(120)), 1.0);
        assert!((panel_duty_cycle(code(57)) - 0.5).abs() < 0.005);
    }

    #[test]
    fn panel_codes_are_monotonic_until_the_pin_saturates() {
        let mut previous = panel_duty_cycle(0.0);
        for stored in 1..=114 {
            let current = panel_duty_cycle(code(stored));
            assert!(current > previous, "{stored}");
            previous = current;
        }
        for stored in 115..=127 {
            assert_eq!(panel_duty_cycle(code(stored)), 1.0);
        }
    }

    #[test]
    fn summed_modulation_can_reach_both_dc_endpoints() {
        assert_eq!(add_modulation(panel_duty_cycle(code(1)), -0.02), 0.0);
        assert_eq!(add_modulation(panel_duty_cycle(code(114)), 0.02), 1.0);
    }
}
