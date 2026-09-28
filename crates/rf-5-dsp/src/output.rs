//! Common audio path after the five voice cards.
//!
//! Five voice cards and the A-440 network meet at the passive summing node
//! buffered by U480. Its output reaches a linearized CA3280 master VCA, the C4189 coupling network
//! and the loaded NE5534 voltage follower on SD430. Every value through the
//! output jack is expressed in circuit volts; one explicit adapter-boundary
//! constant maps those volts to the host's dimensionless domain without
//! adding a second, fictitious saturation stage.

use rf_5_voice::vca;

// SD430 joins the five voice cards at U480's high-impedance input through
// R4565-R4569 (39 kohm). Each card is a Thevenin source behind its 25 kohm
// VOL trimmer plus that resistor (`vca::VOICE_SUMMER_SOURCE_OHMS`); idle cards
// still load the node. The A-440 network hangs on the same node: R4519 20 kohm
// to C4183 (0.1 uF to ground), which R4498 10 kohm returns to TP401 - the 0/5 V
// reference when selected, otherwise grounded by U460. C4183 is the node's
// only state: a gentle low-frequency shelf and the physical A-440 injection.
const VOICE_CARD_COUNT: f64 = 5.0;
const REFERENCE_SUMMER_OHMS: f64 = 20_000.0;
const REFERENCE_SERIES_OHMS: f64 = 10_000.0;
const REFERENCE_CAPACITANCE_FARADS: f64 = 0.1e-6;

// External load and interface reference level are not specified by the
// instrument, so one linear constant maps jack volts to host units. It sits
// downstream of every analog overload and cannot restore drive removed inside
// the modeled circuit. The Rev 3 schematic audit left the circuit about 2 dB
// hotter than earlier releases (1/12 V DAC steps, U451's triangle, the C4183
// shelf), so one voice above the shelf reaches the host 2 dB below the
// earlier per-voice level (0.2 summer x 0.80 master / 2 V per unit). That
// keeps the original headroom for the strongest resonant programs. A
// reference output sweep can replace this one boundary.
const EARLIER_RELEASE_VOICE_TO_HOST_GAIN: f32 = 0.2 * 0.80 / 2.0;
const HEADROOM_RESTORATION_GAIN: f32 = 0.794_328_2; // -2 dB
const CANDIDATE_CIRCUIT_VOLTS_PER_HOST_UNIT: f32 =
    vca::FINAL_VCA_THEVENIN_GAIN * midband_summer_gain_per_voice() * vca::MASTER_VCA_VOLTAGE_GAIN
        / (EARLIER_RELEASE_VOICE_TO_HOST_GAIN * HEADROOM_RESTORATION_GAIN);

const fn midband_summer_gain_per_voice() -> f32 {
    // Above the shelf C4183 is a short: R4519 alone loads the node.
    let voice_conductance = 1.0 / vca::VOICE_SUMMER_SOURCE_OHMS as f64;
    (voice_conductance / (VOICE_CARD_COUNT * voice_conductance + 1.0 / REFERENCE_SUMMER_OHMS))
        as f32
}

// SD430: C4189 couples the U479 master VCA to the U481 output buffer. R4562
// loads the OTA side of the capacitor and R4543 the U481 side. Because U479 is
// a current source, the midband load is R4562 || R4543 while the capacitor
// charges through their series sum: a 0.60 Hz corner, not the 4.34 Hz of the
// parallel pair.
const COUPLING_CAPACITANCE_FARADS: f32 = 2.2e-6;
const COUPLING_LOAD_A_OHMS: f32 = 20_000.0;
const COUPLING_LOAD_B_OHMS: f32 = 100_000.0;

// SD430 buffers the C4189 node with U481, an NE5534 voltage follower on
// +/-15 V rails. R4544 permanently loads its output with 1 kohm before R4545
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

// PCB1 R113 is a 10 kohm linear panel pot (SD131: "all pots 10K, LIN"; SD334
// repeats it as 100K in a cross-reference). Its top is fed from the +5 V
// analog rail through R345 (100 ohm) and the normalled AMPLIFIER CV IN jack.
// SD430 loads the wiper with R4535 and C4184 before U480 buffers it into the
// Q411 current converter. Loading and the position-dependent Thevenin
// resistance are retained explicitly.
const MASTER_VOLUME_REFERENCE_VOLTS: f32 = 5.0;
const MASTER_VOLUME_FEED_OHMS: f32 = 100.0;
const MASTER_VOLUME_POT_OHMS: f32 = 10_000.0;
const MASTER_VOLUME_LOAD_OHMS: f32 = 100_000.0;
const MASTER_VOLUME_CAPACITANCE_FARADS: f32 = 0.22e-6;

#[derive(Clone, Copy, Debug)]
pub struct OutputStage {
    // Voltage stored across C4189. The 264 ms time constant needs more state
    // precision than the audio path to settle without a float-rounding
    // residue.
    coupling_capacitor_voltage: f64,
    reference_capacitor_voltage: f64,
    summer_sample_rate: f32,
    summer_retained: f64,
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
            reference_capacitor_voltage: 0.0,
            summer_sample_rate: 0.0,
            summer_retained: 0.0,
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
        self.reference_capacitor_voltage = 0.0;
        self.master_volume_cv_volts = 0.0;
        self.output_buffer_voltage = 0.0;
    }

    /// `voice_sum` is the sum of the five cards' Thevenin voltages and
    /// `reference_volts` the mean TP401 voltage over this sample.
    pub fn next(
        &mut self,
        voice_sum: f32,
        reference_volts: f32,
        master_volume: f32,
        sample_rate: f32,
    ) -> f32 {
        if !voice_sum.is_finite()
            || !reference_volts.is_finite()
            || !master_volume.is_finite()
            || !sample_rate.is_finite()
            || sample_rate <= 0.0
        {
            self.reset();
            return 0.0;
        }

        let summer = self.summing_node(voice_sum, reference_volts, sample_rate);
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

    /// Advance C4183 exactly for inputs held over the sample, then return the
    /// U480 node voltage. With node voltage eliminated algebraically,
    /// C dVx/dt = Gs (Vtp - Vx) + K (S - N Vx), K = Gr Gv / (N Gv + Gr).
    fn summing_node(&mut self, voice_sum: f32, reference_volts: f32, sample_rate: f32) -> f32 {
        let voice_conductance = 1.0 / f64::from(vca::VOICE_SUMMER_SOURCE_OHMS);
        let summer_conductance = 1.0 / REFERENCE_SUMMER_OHMS;
        let series_conductance = 1.0 / REFERENCE_SERIES_OHMS;
        let node_conductance = VOICE_CARD_COUNT * voice_conductance + summer_conductance;
        let coupling = summer_conductance * voice_conductance / node_conductance;
        let total_conductance = series_conductance + VOICE_CARD_COUNT * coupling;
        if self.summer_sample_rate.to_bits() != sample_rate.to_bits() {
            let time_constant = REFERENCE_CAPACITANCE_FARADS / total_conductance;
            self.summer_retained = libm::exp(-1.0 / (f64::from(sample_rate) * time_constant));
            self.summer_sample_rate = sample_rate;
        }
        let voice_sum = f64::from(voice_sum);
        let target = (series_conductance * f64::from(reference_volts) + coupling * voice_sum)
            / total_conductance;
        self.reference_capacitor_voltage =
            target + (self.reference_capacitor_voltage - target) * self.summer_retained;
        ((voice_conductance * voice_sum + summer_conductance * self.reference_capacitor_voltage)
            / node_conductance) as f32
    }

    fn ac_couple(&mut self, input: f32, sample_rate: f32) -> f32 {
        let input = f64::from(input);
        if self.coupling_sample_rate.to_bits() != sample_rate.to_bits() {
            let time_constant = f64::from(coupling_charge_ohms() * COUPLING_CAPACITANCE_FARADS);
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

fn coupling_charge_ohms() -> f32 {
    COUPLING_LOAD_A_OHMS + COUPLING_LOAD_B_OHMS
}

fn master_volume_wiper(panel: f32) -> (f32, f32) {
    let position = panel.clamp(0.0, 1.0);
    let lower_ohms = MASTER_VOLUME_POT_OHMS * position;
    let upper_ohms = MASTER_VOLUME_FEED_OHMS + MASTER_VOLUME_POT_OHMS * (1.0 - position);
    let open_voltage = MASTER_VOLUME_REFERENCE_VOLTS * lower_ohms / (lower_ohms + upper_ohms);
    let thevenin_resistance = lower_ohms * upper_ohms / (lower_ohms + upper_ohms);
    let load_ratio = MASTER_VOLUME_LOAD_OHMS / (MASTER_VOLUME_LOAD_OHMS + thevenin_resistance);
    let loaded_voltage = open_voltage * load_ratio;
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
            let _ = cached.next(input, 0.0, 0.64, SAMPLE_RATE);
        }
        let mut rebuilt = cached;
        for sample in 0..2_048 {
            rebuilt.master_volume_sample_rate = 0.0;
            rebuilt.coupling_sample_rate = 0.0;
            rebuilt.master_control_cv = 0.0;
            let input = libm::sinf(sample as f32 * 0.053) * 3.0;
            let volume = if sample < 1_024 { 0.64 } else { 0.37 };
            let cached_output = cached.next(input, 0.0, volume, SAMPLE_RATE);
            let rebuilt_output = rebuilt.next(input, 0.0, volume, SAMPLE_RATE);
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
        let corner = 1.0 / (2.0 * PI * coupling_charge_ohms() * COUPLING_CAPACITANCE_FARADS);
        assert!((corner - 0.602_86).abs() < 0.001);
    }

    #[test]
    fn linear_volume_pot_has_the_populated_loaded_wiper_law() {
        let (closed_voltage, closed_resistance) = master_volume_wiper(0.0);
        let (middle_voltage, middle_resistance) = master_volume_wiper(0.5);
        let (open_voltage, open_resistance) = master_volume_wiper(1.0);
        assert_eq!(closed_voltage, 0.0);
        assert_eq!(closed_resistance, 0.0);
        // R345's 100 ohm feed trims the top by about 1% and the midpoint by
        // half of that.
        assert!(
            (middle_voltage - 2.414_3).abs() < 1.0e-4,
            "{middle_voltage}"
        );
        assert!(
            (middle_resistance - 2_462.6).abs() < 1.0,
            "{middle_resistance}"
        );
        assert!((open_voltage - 4.945_6).abs() < 1.0e-3, "{open_voltage}");
        assert!((open_resistance - 98.9).abs() < 0.5, "{open_resistance}");
    }

    #[test]
    fn q411_turns_the_linear_pot_into_an_audio_taper() {
        let mut stage = OutputStage::default();
        let mut control = 0.0;
        for _ in 0..(SAMPLE_RATE as usize / 100) {
            control = stage.master_volume_control(0.5, SAMPLE_RATE);
        }
        assert!((0.40..0.44).contains(&control));
        // R345 leaves the fully open wiper at about 4.95 V, not the 5 V
        // at which Q411's nominal current is quoted.
        let mut full = 0.0;
        for _ in 0..(SAMPLE_RATE as usize / 10) {
            full = stage.master_volume_control(1.0, SAMPLE_RATE);
        }
        assert!((full - full_volume_control_ratio()).abs() < 1.0e-4);
        assert!((0.98..1.0).contains(&full), "{full}");
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
            assert_eq!(stage.next(input, 0.0, 0.0, SAMPLE_RATE), 0.0);
        }
    }

    #[test]
    fn output_is_symmetric_finite_and_physically_bounded() {
        for index in -20_000..=20_000 {
            let input = index as f32 * 0.01;
            let mut positive_stage = OutputStage::default();
            let mut negative_stage = OutputStage::default();
            let positive = positive_stage.next(input, 0.0, 1.0, SAMPLE_RATE);
            let negative = negative_stage.next(-input, 0.0, 1.0, SAMPLE_RATE);
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
        let unit = CANDIDATE_CIRCUIT_VOLTS_PER_HOST_UNIT;
        assert_eq!(host_from_jack_volts(0.0), 0.0);
        assert!((host_from_jack_volts(unit) - 1.0).abs() < 1.0e-6);
        assert!((host_from_jack_volts(2.0 * unit) - 2.0).abs() < 1.0e-6);
        assert!((host_from_jack_volts(4.0 * unit) - 4.0).abs() < 1.0e-6);
    }

    #[test]
    fn common_stage_has_headroom_for_five_voice_sum() {
        let mut one_voice_stage = OutputStage::default();
        let mut five_voice_stage = OutputStage::default();
        let one_voice = one_voice_stage.next(1.0, 0.0, 1.0, SAMPLE_RATE);
        let five_voices = five_voice_stage.next(5.0, 0.0, 1.0, SAMPLE_RATE);
        // U479's linear range (+/-ID x 28 kohm, about 11.9 V) passes five
        // ordinary voices proportionally; only a grossly overdriven node
        // reaches its rounded current limit.
        assert!((five_voices / one_voice - 5.0).abs() < 0.05);
        let mut overdriven_stage = OutputStage::default();
        let overdriven = overdriven_stage.next(500.0, 0.0, 1.0, SAMPLE_RATE);
        assert!(overdriven < one_voice * 500.0 * 0.5);
    }

    #[test]
    fn coupling_capacitor_rejects_steady_dc() {
        for sample_rate in [44_100.0, 48_000.0, 96_000.0] {
            let mut stage = OutputStage::default();
            let mut output = 0.0;
            // About 23 time constants of the 264 ms C4189 network.
            for _ in 0..(sample_rate as usize * 6) {
                output = stage.next(1.0, 0.0, 1.0, sample_rate);
            }
            assert!(output.abs() < 1.0e-6, "rate={sample_rate}, output={output}");
        }
    }

    #[test]
    fn coupling_capacitor_decay_is_exact_and_sample_rate_invariant() {
        let duration_seconds = 0.1;
        let time_constant = f64::from(coupling_charge_ohms() * COUPLING_CAPACITANCE_FARADS);
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

    fn full_volume_control_ratio() -> f32 {
        vca::master_volume_control_from_cv(master_volume_wiper(1.0).0)
    }

    /// |node gain(f)| / midband node gain for the R4519/C4183/R4498 branch.
    fn summer_shelf_relative_to_midband(frequency: f32) -> f32 {
        let omega = 2.0 * core::f64::consts::PI * f64::from(frequency);
        let voice = 1.0 / f64::from(vca::VOICE_SUMMER_SOURCE_OHMS);
        // Branch impedance: R4519 + (R4498 || C4183), as a complex pair.
        let parallel_denominator =
            1.0 + (omega * REFERENCE_SERIES_OHMS * REFERENCE_CAPACITANCE_FARADS).powi(2);
        let parallel_real = REFERENCE_SERIES_OHMS / parallel_denominator;
        let parallel_imaginary =
            -omega * REFERENCE_SERIES_OHMS * REFERENCE_SERIES_OHMS * REFERENCE_CAPACITANCE_FARADS
                / parallel_denominator;
        let branch_real = REFERENCE_SUMMER_OHMS + parallel_real;
        let branch_magnitude_squared =
            branch_real * branch_real + parallel_imaginary * parallel_imaginary;
        let admittance_real = VOICE_CARD_COUNT * voice + branch_real / branch_magnitude_squared;
        let admittance_imaginary = -parallel_imaginary / branch_magnitude_squared;
        let low = voice
            / libm::sqrt(
                admittance_real * admittance_real + admittance_imaginary * admittance_imaginary,
            );
        (low / f64::from(midband_summer_gain_per_voice())) as f32
    }

    #[test]
    fn summer_shelf_lifts_bass_by_the_c4183_branch() {
        let bass = summer_shelf_relative_to_midband(5.0);
        assert!((1.14..1.16).contains(&bass), "{bass}");
        assert!((summer_shelf_relative_to_midband(10_000.0) - 1.0).abs() < 1.0e-3);
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
                        / (midband_summer_gain_per_voice() * vca::MASTER_VCA_VOLTAGE_GAIN),
                    0.0,
                    1.0,
                    sample_rate,
                );
                if index >= settle {
                    input_energy += input * input;
                    output_energy += output * output;
                }
            }
            let measured = libm::sqrtf(output_energy / input_energy);
            let corner = 1.0 / (2.0 * PI * coupling_charge_ohms() * COUPLING_CAPACITANCE_FARADS);
            let coupling = 20.0 / libm::sqrtf(20.0 * 20.0 + corner * corner);
            let expected =
                coupling * summer_shelf_relative_to_midband(20.0) * full_volume_control_ratio();
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
            let _ = stage.next(1.0, 0.0, 1.0, SAMPLE_RATE);
        }
        stage.reset();
        assert_eq!(stage.next(0.0, 0.0, 1.0, SAMPLE_RATE), 0.0);
    }

    #[test]
    fn invalid_sample_resets_stage_and_returns_silence() {
        let mut stage = OutputStage::default();
        let _ = stage.next(1.0, 0.0, 1.0, SAMPLE_RATE);
        assert_eq!(stage.next(f32::NAN, 0.0, 1.0, SAMPLE_RATE), 0.0);
        assert_eq!(stage.next(0.0, 0.0, 1.0, SAMPLE_RATE), 0.0);
    }
}
