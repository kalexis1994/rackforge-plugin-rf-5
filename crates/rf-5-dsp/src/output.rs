//! Common audio path after the five voice cards.
//!
//! Five equal-value input resistors feed the inverting voice summer. Its
//! output reaches a linearized CA3280 master VCA, the C4189 coupling network
//! and the loaded NE5534 voltage follower on SD430. Every value through the
//! output jack is expressed in circuit volts; one explicit adapter-boundary
//! constant maps those volts to the host's dimensionless domain without
//! adding a second, fictitious saturation stage.

use rf_5_voice::vca;

// SD430 joins five low-impedance voice outputs through equal 39 kohm
// resistors into the high-impedance U480 follower. Active and inactive cards
// therefore form an exact passive five-input average, not an arbitrary gain.
const VOICE_SUMMER_EQUAL_RESISTOR_GAIN: f32 = 1.0 / 5.0;
// External load and interface reference level are not specified by the
// instrument. PCB3's shared oscillator-level current sources divide across
// the five voice-card IABC inputs; two circuit volts per host unit recovers a
// practical nominal plugin level after admitting that physical fanout while
// retaining headroom for the strongest resonant programs.
// This conversion remains strictly linear and downstream of every analog
// overload or interaction, so it cannot restore drive removed inside the
// modeled circuit. A reference output sweep can replace this one boundary.
const CANDIDATE_CIRCUIT_VOLTS_PER_HOST_UNIT: f32 = 2.0;

// SD430: C4189 couples the U479 master VCA to the U481 output buffer. The
// following node sees R4562 and R4541 to ground.
const COUPLING_CAPACITANCE_FARADS: f32 = 2.2e-6;
const COUPLING_LOAD_A_OHMS: f32 = 20_000.0;
const COUPLING_LOAD_B_OHMS: f32 = 100_000.0;

// SD430 buffers the C4189 node with U481, an NE5534 voltage follower on
// +/-15 V rails. R4544 permanently loads its output with 1 kohm before R4543
// adds 560 ohm of jack isolation. The manufacturer guarantees at least 24 Vpp
// at 600 ohm and gives 26 Vpp, 38 mA and 13 V/us as typical values. The high-Z
// RackForge boundary leaves the series resistor unloaded, but the permanent
// one-kilohm board load and the device limits remain explicit here.
const OUTPUT_BUFFER_LOAD_OHMS: f32 = 1_000.0;
const OUTPUT_ISOLATION_OHMS: f32 = 560.0;
#[cfg(test)]
const OUTPUT_BUFFER_GUARANTEED_SWING_VOLTS: f32 = 12.0;
const OUTPUT_BUFFER_TYPICAL_SWING_VOLTS: f32 = 13.0;
const OUTPUT_BUFFER_SHORT_CIRCUIT_CURRENT_AMPS: f32 = 38.0e-3;
const OUTPUT_BUFFER_SLEW_VOLTS_PER_SECOND: f32 = 13.0e6;

// PCB1 R113 is a 10 kohm linear panel pot. SD430 loads its wiper with R4555
// and C4184 before U480 buffers it into the Q411 current converter. The
// five-volt analog control rail is the panel reference. Loading and the
// position-dependent Thevenin resistance are retained explicitly.
const MASTER_VOLUME_REFERENCE_VOLTS: f32 = 5.0;
const MASTER_VOLUME_POT_OHMS: f32 = 10_000.0;
const MASTER_VOLUME_LOAD_OHMS: f32 = 100_000.0;
const MASTER_VOLUME_CAPACITANCE_FARADS: f32 = 0.22e-6;

#[derive(Clone, Copy, Debug)]
pub struct OutputStage {
    // Voltage stored across C4189. The 36.7 ms time constant needs more state
    // precision than the audio path to settle without a float-rounding
    // residue.
    coupling_capacitor_voltage: f64,
    master_volume_cv_volts: f64,
    output_buffer_voltage: f64,
    master_volume_panel: f32,
    master_volume_sample_rate: f32,
    master_volume_target: f32,
    master_volume_coefficient: f64,
    master_volume_snap: bool,
    master_control_cv: f32,
    master_control_ratio: f32,
    coupling_sample_rate: f32,
    coupling_retained: f64,
}

impl Default for OutputStage {
    fn default() -> Self {
        Self {
            coupling_capacitor_voltage: 0.0,
            master_volume_cv_volts: 0.0,
            output_buffer_voltage: 0.0,
            master_volume_panel: 0.0,
            master_volume_sample_rate: 0.0,
            master_volume_target: 0.0,
            master_volume_coefficient: 0.0,
            master_volume_snap: false,
            master_control_cv: 0.0,
            master_control_ratio: 0.0,
            coupling_sample_rate: 0.0,
            coupling_retained: 0.0,
        }
    }
}

impl OutputStage {
    pub fn reset(&mut self) {
        self.coupling_capacitor_voltage = 0.0;
        self.master_volume_cv_volts = 0.0;
        self.output_buffer_voltage = 0.0;
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
        let jack_volts = self.output_buffer(coupled_volts, sample_rate);
        host_from_jack_volts(jack_volts)
    }

    fn master_volume_control(&mut self, panel: f32, sample_rate: f32) -> f32 {
        if self.master_volume_panel.to_bits() != panel.to_bits()
            || self.master_volume_sample_rate.to_bits() != sample_rate.to_bits()
        {
            let (target, resistance) = master_volume_wiper(panel);
            self.master_volume_target = target;
            self.master_volume_snap = resistance <= f32::EPSILON;
            self.master_volume_coefficient = if self.master_volume_snap {
                0.0
            } else {
                let time_constant = f64::from(resistance * MASTER_VOLUME_CAPACITANCE_FARADS);
                1.0 - libm::exp(-1.0 / (f64::from(sample_rate) * time_constant))
            };
            self.master_volume_panel = panel;
            self.master_volume_sample_rate = sample_rate;
        }
        if self.master_volume_snap {
            self.master_volume_cv_volts = f64::from(self.master_volume_target);
        } else {
            self.master_volume_cv_volts += (f64::from(self.master_volume_target)
                - self.master_volume_cv_volts)
                * self.master_volume_coefficient;
        }
        let control_cv = self.master_volume_cv_volts as f32;
        if self.master_control_cv.to_bits() != control_cv.to_bits() {
            self.master_control_ratio = vca::master_volume_control_from_cv(control_cv);
            self.master_control_cv = control_cv;
        }
        self.master_control_ratio
    }

    fn ac_couple(&mut self, input: f32, sample_rate: f32) -> f32 {
        let input = f64::from(input);
        if self.coupling_sample_rate.to_bits() != sample_rate.to_bits() {
            let time_constant = f64::from(coupling_load_ohms() * COUPLING_CAPACITANCE_FARADS);
            let sample_period = 1.0 / f64::from(sample_rate);
            self.coupling_retained = libm::exp(-sample_period / time_constant);
            self.coupling_sample_rate = sample_rate;
        }
        self.coupling_capacitor_voltage =
            input + (self.coupling_capacitor_voltage - input) * self.coupling_retained;
        (input - self.coupling_capacitor_voltage) as f32
    }

    fn output_buffer(&mut self, input: f32, sample_rate: f32) -> f32 {
        // The 1 kohm board load asks for at most 13 mA at the typical swing,
        // comfortably inside the 38 mA typical current capability. Keep the
        // current boundary in the calculation so a future measured load can
        // replace the present high-impedance interface without changing the
        // device model.
        let current_limited_swing =
            OUTPUT_BUFFER_SHORT_CIRCUIT_CURRENT_AMPS * OUTPUT_BUFFER_LOAD_OHMS;
        let swing = OUTPUT_BUFFER_TYPICAL_SWING_VOLTS.min(current_limited_swing);
        let target = input.clamp(-swing, swing);
        let maximum_step = f64::from(OUTPUT_BUFFER_SLEW_VOLTS_PER_SECOND / sample_rate);
        let delta =
            (f64::from(target) - self.output_buffer_voltage).clamp(-maximum_step, maximum_step);
        self.output_buffer_voltage += delta;
        jack_voltage_for_load(self.output_buffer_voltage as f32, f32::INFINITY)
    }
}

fn host_from_jack_volts(jack_volts: f32) -> f32 {
    jack_volts / CANDIDATE_CIRCUIT_VOLTS_PER_HOST_UNIT
}

fn jack_voltage_for_load(buffer_volts: f32, external_load_ohms: f32) -> f32 {
    if external_load_ohms.is_infinite() && external_load_ohms.is_sign_positive() {
        return buffer_volts;
    }
    if !external_load_ohms.is_finite() || external_load_ohms <= 0.0 {
        return 0.0;
    }
    buffer_volts * external_load_ohms / (external_load_ohms + OUTPUT_ISOLATION_OHMS)
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
    fn rc_coefficient_caches_are_sample_exact_when_rebuilt() {
        let mut cached = OutputStage::default();
        for sample in 0..256 {
            let input = libm::sinf(sample as f32 * 0.037) * 2.0;
            let _ = cached.next(input, 0.64, SAMPLE_RATE);
        }
        let mut rebuilt = cached;
        for sample in 0..2_048 {
            rebuilt.master_volume_sample_rate = 0.0;
            rebuilt.coupling_sample_rate = 0.0;
            rebuilt.master_control_cv = 0.0;
            let input = libm::sinf(sample as f32 * 0.053) * 3.0;
            let volume = if sample < 1_024 { 0.64 } else { 0.37 };
            let cached_output = cached.next(input, volume, SAMPLE_RATE);
            let rebuilt_output = rebuilt.next(input, volume, SAMPLE_RATE);
            assert_eq!(cached_output.to_bits(), rebuilt_output.to_bits());
            assert_eq!(
                cached.master_volume_cv_volts.to_bits(),
                rebuilt.master_volume_cv_volts.to_bits()
            );
            assert_eq!(
                cached.coupling_capacitor_voltage.to_bits(),
                rebuilt.coupling_capacitor_voltage.to_bits()
            );
        }
    }

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
    fn output_is_symmetric_finite_and_physically_bounded() {
        for index in -20_000..=20_000 {
            let input = index as f32 * 0.01;
            let mut positive_stage = OutputStage::default();
            let mut negative_stage = OutputStage::default();
            let positive = positive_stage.next(input, 1.0, SAMPLE_RATE);
            let negative = negative_stage.next(-input, 1.0, SAMPLE_RATE);
            assert!(positive.is_finite());
            assert!(
                positive.abs()
                    <= OUTPUT_BUFFER_TYPICAL_SWING_VOLTS / CANDIDATE_CIRCUIT_VOLTS_PER_HOST_UNIT
            );
            assert!((positive + negative).abs() < 1.0e-6);
        }
    }

    #[test]
    fn ne5534_follower_is_linear_inside_its_guaranteed_swing() {
        for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
            for input in [
                -OUTPUT_BUFFER_GUARANTEED_SWING_VOLTS,
                -4.0,
                0.0,
                4.0,
                OUTPUT_BUFFER_GUARANTEED_SWING_VOLTS,
            ] {
                let mut stage = OutputStage::default();
                assert_eq!(stage.output_buffer(input, sample_rate), input);
            }
        }
    }

    #[test]
    fn ne5534_follower_obeys_swing_current_and_slew_boundaries() {
        let maximum_load_current = OUTPUT_BUFFER_TYPICAL_SWING_VOLTS / OUTPUT_BUFFER_LOAD_OHMS;
        assert!(maximum_load_current < OUTPUT_BUFFER_SHORT_CIRCUIT_CURRENT_AMPS);

        let mut stage = OutputStage::default();
        assert_eq!(
            stage.output_buffer(100.0, SAMPLE_RATE),
            OUTPUT_BUFFER_TYPICAL_SWING_VOLTS
        );
        assert_eq!(
            stage.output_buffer(-100.0, SAMPLE_RATE),
            -OUTPUT_BUFFER_TYPICAL_SWING_VOLTS
        );
    }

    #[test]
    fn output_isolation_resistor_respects_external_load() {
        assert_eq!(jack_voltage_for_load(4.0, f32::INFINITY), 4.0);
        let loaded = jack_voltage_for_load(4.0, 100_000.0);
        assert!((loaded - 3.977_724_8).abs() < 1.0e-6);
        assert_eq!(jack_voltage_for_load(4.0, 0.0), 0.0);
    }

    #[test]
    fn host_boundary_is_linear_and_does_not_replace_analog_overload() {
        assert_eq!(host_from_jack_volts(0.0), 0.0);
        assert_eq!(host_from_jack_volts(2.0), 1.0);
        assert_eq!(host_from_jack_volts(4.0), 2.0);
        assert_eq!(host_from_jack_volts(8.0), 4.0);
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
