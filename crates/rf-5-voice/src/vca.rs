//! CA3280 transfer candidates used at the physical OTA boundaries.
//!
//! The oscillator mixer OTAs operate with their linearizing-diode terminal
//! cut off, while the envelope-controlled final VCA drives that terminal.
//! These functions preserve that distinction without claiming that the
//! normalized input scale is a measured circuit voltage.

const UNLINEARIZED_INPUT_DRIVE: f32 = 0.55;
const LINEARIZED_INPUT_DRIVE: f32 = 0.12;

pub fn unlinearized(input: f32, control: f32) -> f32 {
    ota_transfer(input, control, UNLINEARIZED_INPUT_DRIVE)
}

pub fn linearized(input: f32, control: f32) -> f32 {
    ota_transfer(input, control, LINEARIZED_INPUT_DRIVE)
}

fn ota_transfer(input: f32, control: f32, drive: f32) -> f32 {
    if control <= 0.0 || !input.is_finite() {
        return 0.0;
    }
    let current = libm::tanhf(input * drive) / drive;
    current * control.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_bias_current_closes_the_vca() {
        for input in [-8.0, -1.0, 0.0, 1.0, 8.0] {
            assert_eq!(unlinearized(input, 0.0), 0.0);
            assert_eq!(linearized(input, 0.0), 0.0);
        }
    }

    #[test]
    fn control_current_changes_gain_monotonically() {
        let low = unlinearized(0.5, 0.25).abs();
        let middle = unlinearized(0.5, 0.5).abs();
        let high = unlinearized(0.5, 1.0).abs();
        assert!(low < middle && middle < high);
    }

    #[test]
    fn active_linearizing_diodes_extend_the_input_range() {
        let input = 3.0;
        let unlinearized_output = unlinearized(input, 1.0);
        let linearized_output = linearized(input, 1.0);
        assert!(linearized_output > unlinearized_output * 1.35);
        assert!(linearized_output < input);
    }

    #[test]
    fn transfer_is_odd_symmetric_and_finite() {
        for index in 0..10_000 {
            let input = index as f32 * 0.002;
            let positive = unlinearized(input, 1.0);
            let negative = unlinearized(-input, 1.0);
            assert!(positive.is_finite());
            assert!((positive + negative).abs() < 1.0e-6);
        }
    }
}
