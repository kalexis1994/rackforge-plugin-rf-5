//! Common audio path after the five voice cards.
//!
//! Five equal-value input resistors feed the inverting voice summer. Its
//! output reaches a linearized CA3280 master VCA, the C4189 coupling network
//! and an NE5534 output buffer. Every value through the coupling capacitor is
//! expressed in circuit volts; one explicit adapter-boundary constant maps
//! those volts to the host's dimensionless full-scale domain.

use rf_5_voice::vca;

// SD430 joins five low-impedance voice outputs through equal 39 kohm
// resistors into the high-impedance U480 follower. Active and inactive cards
// therefore form an exact passive five-input average, not an arbitrary gain.
const VOICE_SUMMER_EQUAL_RESISTOR_GAIN: f32 = 1.0 / 5.0;
// External load and interface headroom are not specified by the instrument.
// Preserve the accepted 2 V-per-host-unit listening calibration only here,
// after the complete analog circuit model, until a reference output sweep is
// available. This value cannot alter any analog overload or interaction.
const CANDIDATE_CIRCUIT_VOLTS_PER_HOST_UNIT: f32 = 2.0;
const HOST_OUTPUT_CEILING: f32 = 0.98;

// SD430: C4189 couples the U479 master VCA to the U481 output buffer. The
// following node sees R4562 and R4541 to ground.
const COUPLING_CAPACITANCE_FARADS: f32 = 2.2e-6;
const COUPLING_LOAD_A_OHMS: f32 = 20_000.0;
const COUPLING_LOAD_B_OHMS: f32 = 100_000.0;

// PCB1 R113 is a 10 kohm linear panel pot. SD430 loads its wiper with R4555
// and C4184 before U480 buffers it into the Q411 current converter. The
// five-volt analog control rail is the panel reference. Loading and the
// position-dependent Thevenin resistance are retained explicitly.
const MASTER_VOLUME_REFERENCE_VOLTS: f32 = 5.0;
const MASTER_VOLUME_POT_OHMS: f32 = 10_000.0;
const MASTER_VOLUME_LOAD_OHMS: f32 = 100_000.0;
const MASTER_VOLUME_CAPACITANCE_FARADS: f32 = 0.22e-6;

#[derive(Clone, Copy, Debug, Default)]
pub struct OutputStage {
    // Voltage stored across C4189. The 36.7 ms time constant needs more state
    // precision than the audio path to settle without a float-rounding
    // residue.
    coupling_capacitor_voltage: f64,
    master_volume_cv_volts: f64,
}

impl OutputStage {
    pub fn reset(&mut self) {
        self.coupling_capacitor_voltage = 0.0;
        self.master_volume_cv_volts = 0.0;
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

        let summer = voice_sum * VOICE_SUMMER_EQUAL_RESISTOR_GAIN;
        let master_control = self.master_volume_control(master_volume, sample_rate);
        let master_vca = vca::master_output(summer, master_control);
        let coupled_volts = self.ac_couple(master_vca, sample_rate);
        let host_input = coupled_volts / CANDIDATE_CIRCUIT_VOLTS_PER_HOST_UNIT;
        HOST_OUTPUT_CEILING * libm::tanhf(host_input / HOST_OUTPUT_CEILING)
    }

    fn master_volume_control(&mut self, panel: f32, sample_rate: f32) -> f32 {
        let (target, resistance) = master_volume_wiper(panel);
        if resistance <= f32::EPSILON {
            self.master_volume_cv_volts = f64::from(target);
        } else {
            let time_constant = f64::from(resistance * MASTER_VOLUME_CAPACITANCE_FARADS);
            let coefficient = 1.0 - libm::exp(-1.0 / (f64::from(sample_rate) * time_constant));
            self.master_volume_cv_volts +=
                (f64::from(target) - self.master_volume_cv_volts) * coefficient;
        }
        vca::master_volume_control_from_cv(self.master_volume_cv_volts as f32)
    }

    fn ac_couple(&mut self, input: f32, sample_rate: f32) -> f32 {
        let input = f64::from(input);
        let time_constant = f64::from(coupling_load_ohms() * COUPLING_CAPACITANCE_FARADS);
        let sample_period = 1.0 / f64::from(sample_rate);
        let retained = libm::exp(-sample_period / time_constant);
        self.coupling_capacitor_voltage =
            input + (self.coupling_capacitor_voltage - input) * retained;
        (input - self.coupling_capacitor_voltage) as f32
    }
}

fn coupling_load_ohms() -> f32 {
    1.0 / (1.0 / COUPLING_LOAD_A_OHMS + 1.0 / COUPLING_LOAD_B_OHMS)
}

fn master_volume_wiper(panel: f32) -> (f32, f32) {
    let position = panel.clamp(0.0, 1.0);
    let thevenin_resistance = MASTER_VOLUME_POT_OHMS * position * (1.0 - position);
    let load_ratio = MASTER_VOLUME_LOAD_OHMS / (MASTER_VOLUME_LOAD_OHMS + thevenin_resistance);
    let loaded_voltage = MASTER_VOLUME_REFERENCE_VOLTS * position * load_ratio;
    let loaded_resistance = if thevenin_resistance <= f32::EPSILON {
        0.0
    } else {
        1.0 / (1.0 / thevenin_resistance + 1.0 / MASTER_VOLUME_LOAD_OHMS)
    };
    (loaded_voltage, loaded_resistance)
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
    fn linear_volume_pot_has_the_populated_loaded_wiper_law() {
        let (closed_voltage, closed_resistance) = master_volume_wiper(0.0);
        let (middle_voltage, middle_resistance) = master_volume_wiper(0.5);
        let (open_voltage, open_resistance) = master_volume_wiper(1.0);
        assert_eq!(closed_voltage, 0.0);
        assert_eq!(closed_resistance, 0.0);
        assert!((middle_voltage - 2.439_024_4).abs() < 1.0e-6);
        assert!((middle_resistance - 2_439.024_4).abs() < 0.001);
        assert_eq!(open_voltage, MASTER_VOLUME_REFERENCE_VOLTS);
        assert_eq!(open_resistance, 0.0);
    }

    #[test]
    fn q411_turns_the_linear_pot_into_an_audio_taper() {
        let mut stage = OutputStage::default();
        let mut control = 0.0;
        for _ in 0..(SAMPLE_RATE as usize / 100) {
            control = stage.master_volume_control(0.5, SAMPLE_RATE);
        }
        assert!((0.40..0.44).contains(&control));
        assert!((stage.master_volume_control(1.0, SAMPLE_RATE) - 1.0).abs() < 1.0e-6);
        assert_eq!(stage.master_volume_control(0.0, SAMPLE_RATE), 0.0);
    }

    #[test]
    fn c4184_smoothing_is_monotonic_and_sample_rate_stable() {
        fn after_two_milliseconds(sample_rate: f32) -> f32 {
            let mut stage = OutputStage::default();
            let samples = (sample_rate * 0.002) as usize;
            let mut control = 0.0;
            for _ in 0..samples {
                control = stage.master_volume_control(0.5, sample_rate);
            }
            control
        }

        let mut stage = OutputStage::default();
        let first = stage.master_volume_control(0.5, SAMPLE_RATE);
        let second = stage.master_volume_control(0.5, SAMPLE_RATE);
        assert!(first > 0.0);
        assert!(second > first);

        let at_48k = after_two_milliseconds(48_000.0);
        let at_96k = after_two_milliseconds(96_000.0);
        assert!((at_48k - at_96k).abs() < 1.0e-5);
        assert!((0.38..0.43).contains(&at_48k));
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
    fn coupling_capacitor_decay_is_exact_and_sample_rate_invariant() {
        let duration_seconds = 0.1;
        let time_constant = f64::from(coupling_load_ohms() * COUPLING_CAPACITANCE_FARADS);
        let expected = libm::exp(-duration_seconds / time_constant);

        for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
            let mut stage = OutputStage::default();
            let samples = (sample_rate * duration_seconds as f32) as usize;
            let mut output = 0.0;
            for _ in 0..samples {
                output = stage.ac_couple(1.0, sample_rate);
            }
            assert!(
                (f64::from(output) - expected).abs() < 1.0e-7,
                "rate={sample_rate}, output={output}, expected={expected}"
            );
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
                let output = stage.next(
                    input * CANDIDATE_CIRCUIT_VOLTS_PER_HOST_UNIT
                        / (VOICE_SUMMER_EQUAL_RESISTOR_GAIN * vca::MASTER_VCA_VOLTAGE_GAIN),
                    1.0,
                    sample_rate,
                );
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
