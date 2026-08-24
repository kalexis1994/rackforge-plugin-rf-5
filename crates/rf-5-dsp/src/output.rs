//! Common audio path after the five voice cards.
//!
//! Five equal-value input resistors feed the inverting voice summer. Its
//! output reaches a linearized CA3280 master VCA and then an NE5534 output
//! buffer. Circuit-to-host scaling remains a calibration candidate.

use rf_5_voice::vca;

const VOICE_SUMMER_TO_HOST_GAIN: f32 = 0.18;
const HOST_OUTPUT_CEILING: f32 = 0.98;

pub fn render(voice_sum: f32, master_volume: f32) -> f32 {
    let summer = voice_sum * VOICE_SUMMER_TO_HOST_GAIN;
    let master_vca = vca::master_output(summer, master_volume);
    HOST_OUTPUT_CEILING * libm::tanhf(master_vca / HOST_OUTPUT_CEILING)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_master_vca_is_silent() {
        for input in [-10.0, -1.0, 0.0, 1.0, 10.0] {
            assert_eq!(render(input, 0.0), 0.0);
        }
    }

    #[test]
    fn output_is_symmetric_finite_and_bounded() {
        for index in -20_000..=20_000 {
            let input = index as f32 * 0.01;
            let positive = render(input, 1.0);
            let negative = render(-input, 1.0);
            assert!(positive.is_finite());
            assert!(positive.abs() <= HOST_OUTPUT_CEILING);
            assert!((positive + negative).abs() < 1.0e-6);
        }
    }

    #[test]
    fn common_stage_has_headroom_for_five_voice_sum() {
        let one_voice = render(1.0, 1.0);
        let five_voices = render(5.0, 1.0);
        assert!(five_voices > one_voice * 3.0);
        assert!(five_voices < one_voice * 5.0);
    }
}
