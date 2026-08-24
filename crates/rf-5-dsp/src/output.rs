//! Common audio path after the five voice cards.
//!
//! Five equal-value input resistors feed the inverting voice summer. Its
//! output reaches a linearized CA3280 master VCA, the C4189 coupling network
//! and an NE5534 output buffer. Circuit-to-host scaling remains a calibration
//! candidate.

use rf_5_voice::vca;

const VOICE_SUMMER_TO_HOST_GAIN: f32 = 0.18;
const HOST_OUTPUT_CEILING: f32 = 0.98;

// SD430: C4189 couples the U479 master VCA to the U481 output buffer. The
// following node sees R4562 and R4541 to ground.
const COUPLING_CAPACITANCE_FARADS: f32 = 2.2e-6;
const COUPLING_LOAD_A_OHMS: f32 = 20_000.0;
const COUPLING_LOAD_B_OHMS: f32 = 100_000.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct OutputStage {
    // The 36.7 ms time constant needs more state precision than the audio path
    // to settle to zero without a float-rounding residue.
    previous_input: f64,
    previous_output: f64,
}

impl OutputStage {
    pub fn reset(&mut self) {
        self.previous_input = 0.0;
        self.previous_output = 0.0;
    }

    pub fn next(&mut self, voice_sum: f32, master_volume: f32, sample_rate: f32) -> f32 {
        if !voice_sum.is_finite()
            || !master_volume.is_finite()
            || !sample_rate.is_finite()
            || sample_rate <= 0.0
        {
            self.reset();
            return 0.0;
        }

        let summer = voice_sum * VOICE_SUMMER_TO_HOST_GAIN;
        let master_vca = vca::master_output(summer, master_volume);
        let coupled = self.ac_couple(master_vca, sample_rate);
        HOST_OUTPUT_CEILING * libm::tanhf(coupled / HOST_OUTPUT_CEILING)
    }

    fn ac_couple(&mut self, input: f32, sample_rate: f32) -> f32 {
        let input = f64::from(input);
        let time_constant = f64::from(coupling_load_ohms() * COUPLING_CAPACITANCE_FARADS);
        let sample_period = 1.0 / f64::from(sample_rate);
        let coefficient = time_constant / (time_constant + sample_period);
        let output = coefficient * (self.previous_output + input - self.previous_input);
        self.previous_input = input;
        self.previous_output = output;
        output as f32
    }
}

fn coupling_load_ohms() -> f32 {
    1.0 / (1.0 / COUPLING_LOAD_A_OHMS + 1.0 / COUPLING_LOAD_B_OHMS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f32::consts::PI;

    const SAMPLE_RATE: f32 = 48_000.0;

    #[test]
    fn schematic_network_has_expected_corner() {
        let corner = 1.0 / (2.0 * PI * coupling_load_ohms() * COUPLING_CAPACITANCE_FARADS);
        assert!((corner - 4.3406).abs() < 0.001);
    }

    #[test]
    fn closed_master_vca_is_silent_from_rest() {
        for input in [-10.0, -1.0, 0.0, 1.0, 10.0] {
            let mut stage = OutputStage::default();
            assert_eq!(stage.next(input, 0.0, SAMPLE_RATE), 0.0);
        }
    }

    #[test]
    fn output_is_symmetric_finite_and_bounded() {
        for index in -20_000..=20_000 {
            let input = index as f32 * 0.01;
            let mut positive_stage = OutputStage::default();
            let mut negative_stage = OutputStage::default();
            let positive = positive_stage.next(input, 1.0, SAMPLE_RATE);
            let negative = negative_stage.next(-input, 1.0, SAMPLE_RATE);
            assert!(positive.is_finite());
            assert!(positive.abs() <= HOST_OUTPUT_CEILING);
            assert!((positive + negative).abs() < 1.0e-6);
        }
    }

    #[test]
    fn common_stage_has_headroom_for_five_voice_sum() {
        let mut one_voice_stage = OutputStage::default();
        let mut five_voice_stage = OutputStage::default();
        let one_voice = one_voice_stage.next(1.0, 1.0, SAMPLE_RATE);
        let five_voices = five_voice_stage.next(5.0, 1.0, SAMPLE_RATE);
        assert!(five_voices > one_voice * 3.0);
        assert!(five_voices < one_voice * 5.0);
    }

    #[test]
    fn coupling_capacitor_rejects_steady_dc() {
        for sample_rate in [44_100.0, 48_000.0, 96_000.0] {
            let mut stage = OutputStage::default();
            let mut output = 0.0;
            for _ in 0..(sample_rate as usize * 2) {
                output = stage.next(1.0, 1.0, sample_rate);
            }
            assert!(output.abs() < 1.0e-6, "rate={sample_rate}, output={output}");
        }
    }

    #[test]
    fn coupling_network_preserves_twenty_hertz_bass() {
        for sample_rate in [44_100.0, 48_000.0, 96_000.0] {
            let mut stage = OutputStage::default();
            let duration = sample_rate as usize * 2;
            let settle = sample_rate as usize;
            let mut input_energy = 0.0;
            let mut output_energy = 0.0;
            for index in 0..duration {
                let input = libm::sinf(2.0 * PI * 20.0 * index as f32 / sample_rate) * 0.01;
                let output = stage.next(input / VOICE_SUMMER_TO_HOST_GAIN, 1.0, sample_rate);
                if index >= settle {
                    input_energy += input * input;
                    output_energy += output * output;
                }
            }
            let measured = libm::sqrtf(output_energy / input_energy);
            let corner = 1.0 / (2.0 * PI * coupling_load_ohms() * COUPLING_CAPACITANCE_FARADS);
            let expected = 20.0 / libm::sqrtf(20.0 * 20.0 + corner * corner);
            assert!(
                (measured - expected).abs() < 0.002,
                "rate={sample_rate}, measured={measured}, expected={expected}"
            );
        }
    }

    #[test]
    fn reset_discards_coupling_capacitor_history() {
        let mut stage = OutputStage::default();
        for _ in 0..1_000 {
            let _ = stage.next(1.0, 1.0, SAMPLE_RATE);
        }
        stage.reset();
        assert_eq!(stage.next(0.0, 1.0, SAMPLE_RATE), 0.0);
    }

    #[test]
    fn invalid_sample_resets_stage_and_returns_silence() {
        let mut stage = OutputStage::default();
        let _ = stage.next(1.0, 1.0, SAMPLE_RATE);
        assert_eq!(stage.next(f32::NAN, 1.0, SAMPLE_RATE), 0.0);
        assert_eq!(stage.next(0.0, 1.0, SAMPLE_RATE), 0.0);
    }
}
