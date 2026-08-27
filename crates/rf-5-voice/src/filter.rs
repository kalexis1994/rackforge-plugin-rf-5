//! Four-pole CEM3320 low-pass candidate with a bounded five-chip population.
//!
//! The reference voice wires the four independent cells as cascaded low-pass
//! stages and returns the fourth output through the chip's resonance VCA. This
//! model preserves that topology and evaluates it at the voice oversampling
//! rate. Published control-scale, resonance-transconductance, output-swing and
//! passband-distortion limits bound one deterministic profile per voice card.

const STAGE_COUNT: usize = 4;
const MAXIMUM_NORMALIZED_CUTOFF: f32 = 0.45;
const THERMAL_NOISE_LEVEL_VOLTS: f32 = 4.0e-7;
const NOMINAL_POLE_SENSITIVITY_MV_PER_DECADE: f32 = 60.0;
const SERVICE_FILTER_REFERENCE_HZ: f32 = 440.0;
const FILTER_WARMUP_UPDATE_RATE_HZ: f32 = 10.0;
const TYPICAL_FILTER_WARMUP_DRIFT_RATIO: f32 = 0.005;
const MAXIMUM_FILTER_WARMUP_DRIFT_RATIO: f32 = 0.015;
const FILTER_WARMUP_TARGETS: [f32; 5] = [-0.82, 0.47, -0.31, 0.73, -0.58];
const FILTER_WARMUP_TIME_CONSTANT_SECONDS: [f32; 5] = [210.0, 330.0, 270.0, 390.0, 240.0];

// SD431 populates all four pole capacitors with 150 pF polystyrene parts. The
// 100 kohm feedback resistor sees the CEM3320 buffer's nominal 1 megohm output
// impedance in parallel, producing 90.909 kohm: effectively unity against the
// populated 91 kohm interstage coupling resistors.
#[cfg(test)]
const POLE_CAPACITANCE_FARADS: f32 = 150.0e-12;
const CELL_FEEDBACK_OHMS: f32 = 100_000.0;
const BUFFER_OUTPUT_IMPEDANCE_OHMS: f32 = 1_000_000.0;
const INTERSTAGE_COUPLING_OHMS: f32 = 91_000.0;

// Filter Resonance is a 0-10 V common S/H destination and reaches the CEM3320
// current input through R4414's 200 kohm. The data-sheet graph is represented
// by a saturating rational law fixed by its 1 mmho-at-100 uA typical point and
// 2.2 mmho maximum-Gm line. Unlike the former exponential, that law also stays
// within the read uncertainty of Figure 6's modified-linear trace.
const FILTER_RESONANCE_CV_SPAN_VOLTS: f32 = 10.0;
const RESONANCE_CONTROL_RESISTOR_OHMS: f32 = 200_000.0;
const RESONANCE_GM_REFERENCE_AMPS: f32 = 100.0e-6;
const RESONANCE_GM_AT_REFERENCE_MHOS: f32 = 1.0e-3;
const RESONANCE_GM_LIMIT_MHOS: f32 = 2.2e-3;
const RESONANCE_GM_HALF_SATURATION_AMPS: f32 =
    RESONANCE_GM_REFERENCE_AMPS * (RESONANCE_GM_LIMIT_MHOS / RESONANCE_GM_AT_REFERENCE_MHOS - 1.0);
#[cfg(test)]
const FOUR_POLE_OSCILLATION_FEEDBACK: f32 = 4.0;
const FEEDBACK_SOLVER_ITERATIONS: usize = 3;

// SD431 does not connect OUT D directly to Q IN. C4164 AC-couples the final
// pole into a 68 kohm load, U474 applies a non-inverting gain of 3.4, and
// R4416 returns that signal to pin 8. The pin is loaded both by the CEM3320's
// published 2.7-4.5 kohm input impedance and by R4415/C4145. Keeping both
// capacitor memories makes the resonance return disappear at DC and recover
// the populated audio-band loop gain without a service-normalized constant.
const OUTPUT_COUPLING_CAPACITANCE_FARADS: f32 = 2.2e-6;
const OUTPUT_COUPLING_LOAD_OHMS: f32 = 68_000.0;
const OUTPUT_BUFFER_FEEDBACK_OHMS: f32 = 240_000.0;
const OUTPUT_BUFFER_GROUND_OHMS: f32 = 100_000.0;
#[cfg(test)]
const OUTPUT_BUFFER_SWING_MINIMUM_VOLTS: f32 = 12.0;
#[cfg(test)]
const OUTPUT_BUFFER_SWING_TYPICAL_VOLTS: f32 = 13.5;
#[cfg(test)]
const OUTPUT_BUFFER_SLEW_MINIMUM_VOLTS_PER_SECOND: f32 = 8.0e6;
#[cfg(test)]
const OUTPUT_BUFFER_SLEW_TYPICAL_VOLTS_PER_SECOND: f32 = 13.0e6;
const OUTPUT_BUFFER_SOFT_KNEE_ORDER: f32 = 32.0;
#[cfg(test)]
const FINAL_VCA_INPUT_OHMS: f32 = 20_000.0;
const RESONANCE_RETURN_OHMS: f32 = 51_000.0;
const RESONANCE_SHUNT_OHMS: f32 = 3_000.0;
const RESONANCE_SHUNT_CAPACITANCE_FARADS: f32 = 10.0e-6;
#[cfg(test)]
const RESONANCE_INPUT_MINIMUM_OHMS: f32 = 2_700.0;
#[cfg(test)]
const RESONANCE_INPUT_MAXIMUM_OHMS: f32 = 4_500.0;

// Audio entering, crossing and leaving the four cells is expressed in circuit
// volts. The published 10-14 Vpp output population therefore bounds the
// nonlinear cells directly, without a hidden normalized-unit conversion.
const SPECIFIED_SIGNAL_FRACTION_OF_CLIP: f32 = 0.707_106_77;
const CELL_SECOND_HARMONIC_SHARE: f32 = 1.0 / STAGE_COUNT as f32;

#[derive(Clone, Copy, Debug)]
struct FilterProfile {
    pole_sensitivity_mv_per_decade: f32,
    resonance_gm_ratio: f32,
    resonance_input_ohms: f32,
    output_buffer_swing_volts: f32,
    output_buffer_slew_volts_per_second: f32,
    output_clip_vpp: f32,
    passband_second_harmonic: f32,
}

// Deterministic validation population. Every entry stays within the CEM3320
// data-sheet limits: 57.5-62.5 mV/decade, 0.8-1.2 resonance Gm ratio,
// 2.7-4.5 kohm Q-input impedance, 12-13.5 V U474 swing and 10-14 Vpp filter
// clipping. Q-input and Gm tolerances are paired so the populated voice still
// meets the service-manual oscillation window without renormalizing any chip.
const FILTER_PROFILES: [FilterProfile; 5] = [
    FilterProfile {
        pole_sensitivity_mv_per_decade: 58.0,
        resonance_gm_ratio: 0.82,
        resonance_input_ohms: 3_600.0,
        output_buffer_swing_volts: 12.2,
        output_buffer_slew_volts_per_second: 8.4e6,
        output_clip_vpp: 11.0,
        passband_second_harmonic: 0.0014,
    },
    FilterProfile {
        pole_sensitivity_mv_per_decade: 59.1,
        resonance_gm_ratio: 0.95,
        resonance_input_ohms: 2_700.0,
        output_buffer_swing_volts: 13.1,
        output_buffer_slew_volts_per_second: 10.1e6,
        output_clip_vpp: 12.6,
        passband_second_harmonic: 0.0021,
    },
    FilterProfile {
        pole_sensitivity_mv_per_decade: 60.0,
        resonance_gm_ratio: 0.87,
        resonance_input_ohms: 3_300.0,
        output_buffer_swing_volts: 12.8,
        output_buffer_slew_volts_per_second: 11.2e6,
        output_clip_vpp: 12.0,
        passband_second_harmonic: 0.0018,
    },
    FilterProfile {
        pole_sensitivity_mv_per_decade: 61.2,
        resonance_gm_ratio: 0.84,
        resonance_input_ohms: 3_600.0,
        output_buffer_swing_volts: 13.4,
        output_buffer_slew_volts_per_second: 12.9e6,
        output_clip_vpp: 13.4,
        passband_second_harmonic: 0.0028,
    },
    FilterProfile {
        pole_sensitivity_mv_per_decade: 62.2,
        resonance_gm_ratio: 0.96,
        resonance_input_ohms: 2_700.0,
        output_buffer_swing_volts: 12.5,
        output_buffer_slew_volts_per_second: 9.3e6,
        output_clip_vpp: 10.5,
        passband_second_harmonic: 0.0011,
    },
];

#[derive(Clone, Copy, Debug, Default)]
struct TptStage {
    state: f32,
}

impl TptStage {
    fn next(&mut self, input: f32, coefficient: f32, profile: FilterProfile) -> f32 {
        let delta = (input - self.state) * coefficient;
        let output = delta + self.state;
        self.state = output + delta;
        cell_output(output, profile)
    }

    fn predict(self, input: f32, coefficient: f32, profile: FilterProfile) -> (f32, f32) {
        let output = (input - self.state) * coefficient + self.state;
        let (shaped, shaping_slope) = cell_output_with_slope(output, profile);
        (shaped, coefficient * shaping_slope)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct TptMemory {
    state: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct OutputBuffer {
    voltage: f32,
}

impl OutputBuffer {
    fn predict(self, input: f32, sample_rate: f32, profile: FilterProfile) -> (f32, f32) {
        let (target, target_slope) = output_buffer_with_slope(input, profile);
        let maximum_step = profile.output_buffer_slew_volts_per_second / sample_rate.max(1.0);
        let delta = target - self.voltage;
        if delta.abs() <= maximum_step {
            (target, target_slope)
        } else {
            (self.voltage + delta.signum() * maximum_step, 0.0)
        }
    }

    fn next(&mut self, input: f32, sample_rate: f32, profile: FilterProfile) -> f32 {
        let output = self.predict(input, sample_rate, profile).0;
        self.voltage = output;
        output
    }
}

impl TptMemory {
    fn predict(self, input: f32, coefficient: f32) -> (f32, f32) {
        ((input - self.state) * coefficient + self.state, coefficient)
    }

    fn next(&mut self, input: f32, coefficient: f32) -> f32 {
        let delta = (input - self.state) * coefficient;
        let output = delta + self.state;
        self.state = output + delta;
        output
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ResonanceReturn {
    output_coupling: TptMemory,
    output_buffer: OutputBuffer,
    input_shunt: TptMemory,
}

impl ResonanceReturn {
    fn predict(self, output: f32, sample_rate: f32, profile: FilterProfile) -> (f32, f32) {
        let coupling_coefficient = one_pole_coefficient(output_coupling_corner_hz(), sample_rate);
        let (coupling_lowpass, coupling_lowpass_slope) =
            self.output_coupling.predict(output, coupling_coefficient);
        let buffer_drive = (output - coupling_lowpass) * output_buffer_gain();
        let buffer_drive_slope = (1.0 - coupling_lowpass_slope) * output_buffer_gain();
        let (buffer_output, buffer_shape_slope) =
            self.output_buffer
                .predict(buffer_drive, sample_rate, profile);
        let buffer_output_slope = buffer_drive_slope * buffer_shape_slope;

        let shunt_coefficient = one_pole_coefficient(resonance_input_pole_hz(profile), sample_rate);
        let (shunt_lowpass, shunt_lowpass_slope) =
            self.input_shunt.predict(buffer_output, shunt_coefficient);
        let dc_gain = resonance_input_dc_gain(profile);
        let high_frequency_gain = resonance_input_high_frequency_gain(profile);
        let pin_voltage =
            high_frequency_gain * buffer_output + (dc_gain - high_frequency_gain) * shunt_lowpass;
        let pin_slope = (high_frequency_gain
            + (dc_gain - high_frequency_gain) * shunt_lowpass_slope)
            * buffer_output_slope;
        (pin_voltage, pin_slope)
    }

    fn next(&mut self, output: f32, sample_rate: f32, profile: FilterProfile) -> (f32, f32) {
        let coupling_coefficient = one_pole_coefficient(output_coupling_corner_hz(), sample_rate);
        let coupling_lowpass = self.output_coupling.next(output, coupling_coefficient);
        let buffer_drive = (output - coupling_lowpass) * output_buffer_gain();
        let buffer_output = self.output_buffer.next(buffer_drive, sample_rate, profile);

        let shunt_coefficient = one_pole_coefficient(resonance_input_pole_hz(profile), sample_rate);
        let shunt_lowpass = self.input_shunt.next(buffer_output, shunt_coefficient);
        let dc_gain = resonance_input_dc_gain(profile);
        let high_frequency_gain = resonance_input_high_frequency_gain(profile);
        let pin_voltage =
            high_frequency_gain * buffer_output + (dc_gain - high_frequency_gain) * shunt_lowpass;
        (buffer_output, pin_voltage)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cem3320Filter {
    stages: [TptStage; STAGE_COUNT],
    resonance_return: ResonanceReturn,
    noise_state: u32,
    profile_index: usize,
    warmup_position: f32,
    warmup_sample_phase: f64,
    last_output: f32,
}

impl Default for Cem3320Filter {
    fn default() -> Self {
        Self {
            stages: [TptStage::default(); STAGE_COUNT],
            resonance_return: ResonanceReturn::default(),
            noise_state: 0x51f1_5e5d,
            profile_index: 2,
            warmup_position: 0.0,
            warmup_sample_phase: 0.0,
            last_output: 0.0,
        }
    }
}

impl Cem3320Filter {
    pub fn with_profile(profile_index: usize) -> Self {
        Self {
            profile_index: profile_index % FILTER_PROFILES.len(),
            noise_state: 0x51f1_5e5d ^ (profile_index as u32).wrapping_mul(0x9e37_79b9),
            ..Self::default()
        }
    }

    pub fn next(&mut self, input: f32, cutoff_hz: f32, resonance: f32, sample_rate: f32) -> f32 {
        self.next_with_character(input, cutoff_hz, resonance, sample_rate, 0.0)
    }

    pub fn next_with_character(
        &mut self,
        input: f32,
        cutoff_hz: f32,
        resonance: f32,
        sample_rate: f32,
        character: f32,
    ) -> f32 {
        let sample_rate = sample_rate.max(1.0);
        let profile = FILTER_PROFILES[self.profile_index];
        self.advance_warmup(sample_rate);
        let cutoff_hz = (service_trimmed_cutoff_hz(cutoff_hz, profile)
            * self.warmup_frequency_ratio(character))
        .clamp(1.0, sample_rate * MAXIMUM_NORMALIZED_CUTOFF);
        let g = libm::tanf(core::f32::consts::PI * cutoff_hz / sample_rate);
        let coefficient = g / (1.0 + g);
        let resonance_drive = resonance_drive_gain(resonance, profile);
        let thermal_noise = self.next_thermal_noise();
        let open_input = input + thermal_noise;
        let feedback_output = self.solve_feedback(
            open_input,
            coefficient,
            resonance_drive,
            sample_rate,
            profile,
        );
        let (resonance_voltage, _) =
            self.resonance_return
                .predict(feedback_output, sample_rate, profile);
        let mut signal = open_input - resonance_voltage * resonance_drive;
        for (index, stage) in self.stages.iter_mut().enumerate() {
            if index > 0 {
                signal *= interstage_passband_gain();
            }
            signal = stage.next(signal, coefficient, profile);
        }
        if signal.is_finite() {
            let (buffered_output, _) = self.resonance_return.next(signal, sample_rate, profile);
            self.last_output = signal;
            buffered_output
        } else {
            self.stages = [TptStage::default(); STAGE_COUNT];
            self.resonance_return = ResonanceReturn::default();
            self.last_output = 0.0;
            0.0
        }
    }

    fn solve_feedback(
        &self,
        input: f32,
        coefficient: f32,
        resonance_drive: f32,
        sample_rate: f32,
        profile: FilterProfile,
    ) -> f32 {
        if resonance_drive <= 0.0 {
            return self.predict_path(input, coefficient, profile).0;
        }
        let ceiling = output_ceiling(profile);
        let mut estimate = self.last_output.clamp(-ceiling, ceiling);
        for _ in 0..FEEDBACK_SOLVER_ITERATIONS {
            let (resonance_voltage, resonance_slope) =
                self.resonance_return
                    .predict(estimate, sample_rate, profile);
            let (predicted, path_slope) = self.predict_path(
                input - resonance_voltage * resonance_drive,
                coefficient,
                profile,
            );
            let residual = estimate - predicted;
            let derivative = 1.0 + resonance_drive * resonance_slope.max(0.0) * path_slope.max(0.0);
            estimate = (estimate - residual / derivative.max(1.0)).clamp(-ceiling, ceiling);
        }
        estimate
    }

    fn predict_path(&self, input: f32, coefficient: f32, profile: FilterProfile) -> (f32, f32) {
        let mut signal = input;
        let mut slope = 1.0;
        for (index, stage) in self.stages.into_iter().enumerate() {
            if index > 0 {
                signal *= interstage_passband_gain();
                slope *= interstage_passband_gain();
            }
            let (output, stage_slope) = stage.predict(signal, coefficient, profile);
            signal = output;
            slope *= stage_slope;
        }
        (signal, slope)
    }

    fn advance_warmup(&mut self, sample_rate: f32) {
        self.warmup_sample_phase += f64::from(FILTER_WARMUP_UPDATE_RATE_HZ);
        while self.warmup_sample_phase >= f64::from(sample_rate) {
            self.warmup_sample_phase -= f64::from(sample_rate);
            self.step_warmup();
        }
    }

    fn step_warmup(&mut self) {
        let time_constant = FILTER_WARMUP_TIME_CONSTANT_SECONDS[self.profile_index];
        let alpha = 1.0 - libm::expf(-1.0 / (FILTER_WARMUP_UPDATE_RATE_HZ * time_constant));
        self.warmup_position +=
            (FILTER_WARMUP_TARGETS[self.profile_index] - self.warmup_position) * alpha;
        self.warmup_position = self.warmup_position.clamp(-1.0, 1.0);
    }

    fn warmup_frequency_ratio(&self, character: f32) -> f32 {
        let limit = TYPICAL_FILTER_WARMUP_DRIFT_RATIO
            + character.clamp(0.0, 1.0)
                * (MAXIMUM_FILTER_WARMUP_DRIFT_RATIO - TYPICAL_FILTER_WARMUP_DRIFT_RATIO);
        1.0 + self.warmup_position * limit
    }

    fn next_thermal_noise(&mut self) -> f32 {
        let mut state = self.noise_state;
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        self.noise_state = state;
        let normalized = state as f32 / u32::MAX as f32;
        (normalized * 2.0 - 1.0) * THERMAL_NOISE_LEVEL_VOLTS
    }

    #[cfg(test)]
    fn energy(self) -> f32 {
        self.stages.iter().map(|stage| stage.state.abs()).sum()
    }
}

fn resonance_drive_gain(value: f32, profile: FilterProfile) -> f32 {
    resonance_gm(value, profile) * equivalent_feedback_ohms()
}

fn resonance_gm(value: f32, profile: FilterProfile) -> f32 {
    nominal_resonance_gm(value) * profile.resonance_gm_ratio
}

fn nominal_resonance_gm(value: f32) -> f32 {
    resonance_gm_from_current(resonance_control_current_amps(value))
}

fn resonance_gm_from_current(current: f32) -> f32 {
    let current = current.max(0.0);
    RESONANCE_GM_LIMIT_MHOS * current / (current + RESONANCE_GM_HALF_SATURATION_AMPS)
}

fn resonance_control_current_amps(value: f32) -> f32 {
    value.clamp(0.0, 1.0) * FILTER_RESONANCE_CV_SPAN_VOLTS / RESONANCE_CONTROL_RESISTOR_OHMS
}

fn one_pole_coefficient(frequency_hz: f32, sample_rate: f32) -> f32 {
    let g = libm::tanf(core::f32::consts::PI * frequency_hz / sample_rate.max(1.0));
    g / (1.0 + g)
}

fn output_coupling_corner_hz() -> f32 {
    1.0 / (2.0
        * core::f32::consts::PI
        * OUTPUT_COUPLING_LOAD_OHMS
        * OUTPUT_COUPLING_CAPACITANCE_FARADS)
}

fn output_buffer_gain() -> f32 {
    1.0 + OUTPUT_BUFFER_FEEDBACK_OHMS / OUTPUT_BUFFER_GROUND_OHMS
}

#[cfg(test)]
fn output_buffer(value: f32, profile: FilterProfile) -> f32 {
    output_buffer_with_slope(value, profile).0
}

fn output_buffer_with_slope(value: f32, profile: FilterProfile) -> (f32, f32) {
    let ceiling = profile.output_buffer_swing_volts;
    let unclamped = value / ceiling;
    let normalized = unclamped.clamp(-64.0, 64.0);
    let squared = normalized * normalized;
    let fourth = squared * squared;
    let eighth = fourth * fourth;
    let sixteenth = eighth * eighth;
    let thirty_second = sixteenth * sixteenth;
    let soft_base = 1.0 + thirty_second;
    let reciprocal_root = 1.0 / libm::powf(soft_base, 1.0 / OUTPUT_BUFFER_SOFT_KNEE_ORDER);
    let curved = ceiling * normalized * reciprocal_root;
    let output = curved.clamp(-ceiling, ceiling);
    let slope = if unclamped == normalized && output == curved {
        reciprocal_root / soft_base
    } else {
        0.0
    };
    (output, slope)
}

fn parallel_ohms(left: f32, right: f32) -> f32 {
    left * right / (left + right)
}

#[cfg(test)]
fn output_buffer_audio_load_ohms(profile: FilterProfile) -> f32 {
    let resonance_load =
        RESONANCE_RETURN_OHMS + parallel_ohms(profile.resonance_input_ohms, RESONANCE_SHUNT_OHMS);
    let feedback_load = OUTPUT_BUFFER_FEEDBACK_OHMS + OUTPUT_BUFFER_GROUND_OHMS;
    parallel_ohms(
        parallel_ohms(FINAL_VCA_INPUT_OHMS, resonance_load),
        feedback_load,
    )
}

fn resonance_input_dc_gain(profile: FilterProfile) -> f32 {
    profile.resonance_input_ohms / (RESONANCE_RETURN_OHMS + profile.resonance_input_ohms)
}

fn resonance_input_high_frequency_gain(profile: FilterProfile) -> f32 {
    let load = parallel_ohms(profile.resonance_input_ohms, RESONANCE_SHUNT_OHMS);
    load / (RESONANCE_RETURN_OHMS + load)
}

fn resonance_input_pole_hz(profile: FilterProfile) -> f32 {
    let conductance_sum = 1.0 / RESONANCE_RETURN_OHMS
        + 1.0 / profile.resonance_input_ohms
        + 1.0 / RESONANCE_SHUNT_OHMS;
    let shunt_state_share = (1.0 / RESONANCE_SHUNT_OHMS) / conductance_sum;
    (1.0 - shunt_state_share)
        / (2.0 * core::f32::consts::PI * RESONANCE_SHUNT_OHMS * RESONANCE_SHUNT_CAPACITANCE_FARADS)
}

#[cfg(test)]
fn resonance_audio_band_feedback(value: f32, profile: FilterProfile) -> f32 {
    resonance_drive_gain(value, profile)
        * output_buffer_gain()
        * resonance_input_high_frequency_gain(profile)
}

fn equivalent_feedback_ohms() -> f32 {
    CELL_FEEDBACK_OHMS * BUFFER_OUTPUT_IMPEDANCE_OHMS
        / (CELL_FEEDBACK_OHMS + BUFFER_OUTPUT_IMPEDANCE_OHMS)
}

fn interstage_passband_gain() -> f32 {
    equivalent_feedback_ohms() / INTERSTAGE_COUPLING_OHMS
}

fn service_trimmed_cutoff_hz(cutoff_hz: f32, profile: FilterProfile) -> f32 {
    let target_hz = cutoff_hz.max(1.0);
    let low_calibration_hz = resonance_calibrated_pole_hz(SERVICE_FILTER_REFERENCE_HZ, profile);
    let high_calibration_hz =
        resonance_calibrated_pole_hz(SERVICE_FILTER_REFERENCE_HZ * 2.0, profile);
    let calibrated_exponent =
        libm::logf(high_calibration_hz / low_calibration_hz) / core::f32::consts::LN_2;
    let untrimmed_exponent =
        NOMINAL_POLE_SENSITIVITY_MV_PER_DECADE / profile.pole_sensitivity_mv_per_decade;
    let untrimmed_ratio = libm::powf(target_hz / SERVICE_FILTER_REFERENCE_HZ, untrimmed_exponent);
    let populated_scale_trim = calibrated_exponent / untrimmed_exponent;
    low_calibration_hz * libm::powf(untrimmed_ratio, populated_scale_trim)
}

fn resonance_calibrated_pole_hz(target_hz: f32, profile: FilterProfile) -> f32 {
    let coupling_phase = libm::atanf(output_coupling_corner_hz() / target_hz);
    let normalized_frequency = target_hz / resonance_input_pole_hz(profile);
    let transition =
        resonance_input_dc_gain(profile) - resonance_input_high_frequency_gain(profile);
    let denominator = 1.0 + normalized_frequency * normalized_frequency;
    let input_real = resonance_input_high_frequency_gain(profile) + transition / denominator;
    let input_imaginary = -transition * normalized_frequency / denominator;
    let input_phase = libm::atan2f(input_imaginary, input_real);
    let required_cell_phase =
        (core::f32::consts::PI + coupling_phase + input_phase) / STAGE_COUNT as f32;
    target_hz / libm::tanf(required_cell_phase)
}

fn cell_output(value: f32, profile: FilterProfile) -> f32 {
    cell_output_with_slope(value, profile).0
}

fn output_ceiling(profile: FilterProfile) -> f32 {
    profile.output_clip_vpp * 0.5
}

fn cell_output_with_slope(value: f32, profile: FilterProfile) -> (f32, f32) {
    let ceiling = output_ceiling(profile);
    let unclamped = value / ceiling;
    let normalized = (value / ceiling).clamp(-64.0, 64.0);
    // A sixteenth-order soft knee remains nearly linear at the data sheet's
    // distortion test level, then approaches the published clipping swing.
    // Unlike tanh, it does not inject a large third harmonic before clipping.
    let squared = normalized * normalized;
    let fourth = squared * squared;
    let eighth = fourth * fourth;
    let sixteenth = eighth * eighth;
    let soft_base = 1.0 + sixteenth;
    let reciprocal_root = 1.0 / libm::powf(soft_base, 1.0 / 16.0);
    let symmetric = ceiling * normalized * reciprocal_root;
    let symmetric_slope = if unclamped == normalized {
        reciprocal_root / soft_base
    } else {
        0.0
    };
    // y = x + k*x^2 has H2/fundamental ~= k*A/2. Calibrate k at the specified
    // signal amplitude (3 dB below clipping), so each profile's value maps to
    // the stated 0.1-0.3% passband measurement rather than to an arbitrary
    // normalized amplitude. The tiny DC term reflects operating-point shift.
    let reference_amplitude = ceiling * SPECIFIED_SIGNAL_FRACTION_OF_CLIP;
    let even_coefficient =
        2.0 * profile.passband_second_harmonic * CELL_SECOND_HARMONIC_SHARE / reference_amplitude;
    let even_harmonic = even_coefficient * symmetric * symmetric;
    let curved = symmetric + even_harmonic;
    let output = curved.clamp(-ceiling, ceiling);
    let slope = if output == curved {
        symmetric_slope * (1.0 + 2.0 * even_coefficient * symmetric)
    } else {
        0.0
    };
    (output, slope.max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_pole_candidate_is_finite_across_supported_rates() {
        for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
            for profile in 0..FILTER_PROFILES.len() {
                let mut filter = Cem3320Filter::with_profile(profile);
                for index in 0..32_768 {
                    let input = libm::sinf(index as f32 * 0.173) * 4.0;
                    let cutoff = 10.0 + (index % 20_000) as f32;
                    let resonance = (index % 128) as f32 / 127.0;
                    assert!(
                        filter
                            .next(input, cutoff, resonance, sample_rate)
                            .is_finite()
                    );
                }
            }
        }
    }

    #[test]
    fn five_chip_population_stays_inside_published_limits() {
        for profile in FILTER_PROFILES {
            assert!((57.5..=62.5).contains(&profile.pole_sensitivity_mv_per_decade));
            assert!((0.8..=1.2).contains(&profile.resonance_gm_ratio));
            assert!(
                (RESONANCE_INPUT_MINIMUM_OHMS..=RESONANCE_INPUT_MAXIMUM_OHMS)
                    .contains(&profile.resonance_input_ohms)
            );
            assert!(
                (OUTPUT_BUFFER_SWING_MINIMUM_VOLTS..=OUTPUT_BUFFER_SWING_TYPICAL_VOLTS)
                    .contains(&profile.output_buffer_swing_volts)
            );
            assert!(
                (OUTPUT_BUFFER_SLEW_MINIMUM_VOLTS_PER_SECOND
                    ..=OUTPUT_BUFFER_SLEW_TYPICAL_VOLTS_PER_SECOND)
                    .contains(&profile.output_buffer_slew_volts_per_second)
            );
            assert!((10.0..=14.0).contains(&profile.output_clip_vpp));
            assert!((0.001..=0.003).contains(&profile.passband_second_harmonic));
        }
    }

    #[test]
    fn populated_pole_network_is_four_matched_150_pf_unity_cells() {
        assert_eq!(STAGE_COUNT, 4);
        assert_eq!(POLE_CAPACITANCE_FARADS, 150.0e-12);
        assert!((equivalent_feedback_ohms() - 90_909.09).abs() < 0.01);
        assert!((0.999..=1.0).contains(&interstage_passband_gain()));
    }

    #[test]
    fn predicted_signal_path_contains_exactly_four_nonlinear_cells() {
        let filter = Cem3320Filter::default();
        let profile = FILTER_PROFILES[filter.profile_index];
        let input = output_ceiling(profile) * 0.9;
        let (predicted, _) = filter.predict_path(input, 1.0, profile);

        let mut four_cells = input;
        for index in 0..STAGE_COUNT {
            if index > 0 {
                four_cells *= interstage_passband_gain();
            }
            four_cells = cell_output(four_cells, profile);
        }
        let five_cells = cell_output(four_cells * interstage_passband_gain(), profile);

        assert!((predicted - four_cells).abs() < 1.0e-6);
        assert!((predicted - five_cells).abs() > 1.0e-4);
    }

    #[test]
    fn populated_resonance_control_reaches_fifty_microamps() {
        assert_eq!(resonance_control_current_amps(0.0), 0.0);
        assert!((resonance_control_current_amps(1.0) - 50.0e-6).abs() < 1.0e-10);
    }

    #[test]
    fn modified_linear_resonance_cell_matches_published_anchor_and_flattens() {
        let gm_at_reference = resonance_gm_from_current(RESONANCE_GM_REFERENCE_AMPS);
        assert!((gm_at_reference - RESONANCE_GM_AT_REFERENCE_MHOS).abs() < 1.0e-8);
        assert!((RESONANCE_GM_HALF_SATURATION_AMPS - 120.0e-6).abs() < 1.0e-10);
        assert!(
            (resonance_gm_from_current(RESONANCE_GM_HALF_SATURATION_AMPS)
                - RESONANCE_GM_LIMIT_MHOS * 0.5)
                .abs()
                < 1.0e-8
        );

        let first_25_ua = nominal_resonance_gm(0.5) - nominal_resonance_gm(0.0);
        let second_25_ua = nominal_resonance_gm(1.0) - nominal_resonance_gm(0.5);
        assert!(second_25_ua < first_25_ua);
        assert!(nominal_resonance_gm(1.0) < RESONANCE_GM_LIMIT_MHOS);
    }

    #[test]
    fn rational_resonance_law_tracks_digitized_figure_six_landmarks() {
        // The thick scanned trace was normalized to the tabulated 1 mmho at
        // 100 uA anchor. A six-percent band is wider than the pixel/read error
        // at all three landmarks and avoids treating the graph as a table.
        for (current_amps, digitized_mhos) in [
            (50.0e-6, 625.0e-6),
            (150.0e-6, 1.210e-3),
            (300.0e-6, 1.540e-3),
        ] {
            let modeled = resonance_gm_from_current(current_amps);
            assert!(
                ((modeled - digitized_mhos) / digitized_mhos).abs() <= 0.06,
                "current={current_amps}, modeled={modeled}, digitized={digitized_mhos}"
            );
        }
    }

    #[test]
    fn populated_resonance_return_matches_sd431_and_the_data_sheet_input() {
        let typical = FILTER_PROFILES[0];
        assert!((output_coupling_corner_hz() - 1.063_87).abs() < 1.0e-5);
        assert!((output_buffer_gain() - 3.4).abs() < 1.0e-6);
        assert!((resonance_input_dc_gain(typical) - 0.065_934_07).abs() < 1.0e-7);
        assert!((resonance_input_high_frequency_gain(typical) - 0.031_088_084).abs() < 1.0e-7);
        assert!((resonance_input_pole_hz(typical) - 2.501_399).abs() < 1.0e-5);
    }

    #[test]
    fn u474_load_and_output_stay_inside_the_tl082_characterized_region() {
        for profile in FILTER_PROFILES {
            assert!(output_buffer_audio_load_ohms(profile) >= 10_000.0);
            assert!(output_buffer(100.0, profile) <= profile.output_buffer_swing_volts);
            assert!(output_buffer(-100.0, profile) >= -profile.output_buffer_swing_volts);
            let ten_volt_output = output_buffer(10.0, profile);
            assert!((ten_volt_output / 10.0 - 1.0).abs() < 0.0001);
        }
    }

    #[test]
    fn u474_late_knee_preserves_the_published_ten_volt_linearity() {
        const SAMPLE_COUNT: usize = 4_096;
        const CYCLES: usize = 17;
        for profile in FILTER_PROFILES {
            let mut fundamental_sine = 0.0;
            let mut fundamental_cosine = 0.0;
            let mut third_sine = 0.0;
            let mut third_cosine = 0.0;
            for index in 0..SAMPLE_COUNT {
                let phase = 2.0 * core::f32::consts::PI * CYCLES as f32 * index as f32
                    / SAMPLE_COUNT as f32;
                let output = output_buffer(libm::sinf(phase) * 10.0, profile);
                fundamental_sine += output * libm::sinf(phase);
                fundamental_cosine += output * libm::cosf(phase);
                third_sine += output * libm::sinf(phase * 3.0);
                third_cosine += output * libm::cosf(phase * 3.0);
            }
            let fundamental = libm::sqrtf(
                fundamental_sine * fundamental_sine + fundamental_cosine * fundamental_cosine,
            );
            let third = libm::sqrtf(third_sine * third_sine + third_cosine * third_cosine);
            assert!(third / fundamental < 0.0002);
        }
    }

    #[test]
    fn u474_slew_is_inside_the_published_population_and_stateful() {
        for profile in FILTER_PROFILES {
            let sample_rate = 192_000.0 * 4.0;
            let maximum_step = profile.output_buffer_slew_volts_per_second / sample_rate;
            let mut buffer = OutputBuffer::default();
            let positive = buffer.next(100.0, sample_rate, profile);
            assert!(positive > 0.0 && positive <= maximum_step);
            let negative = buffer.next(-100.0, sample_rate, profile);
            assert!(negative < positive);
            assert!((negative - positive).abs() <= maximum_step + 1.0e-6);
        }
    }

    #[test]
    fn u474_prediction_matches_the_committed_slew_step() {
        for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
            for profile in FILTER_PROFILES {
                let internal_rate = sample_rate * 4.0;
                let mut buffer = OutputBuffer::default();
                for input in [10.0, -20.0, 4.0, 100.0, -100.0] {
                    let predicted = buffer.predict(input, internal_rate, profile).0;
                    let committed = buffer.next(input, internal_rate, profile);
                    assert_eq!(predicted, committed);
                }
            }
        }
    }

    #[test]
    fn resonance_return_blocks_dc_but_passes_audio() {
        let profile = FILTER_PROFILES[0];
        let mut return_path = ResonanceReturn::default();
        let (initial_audio, initial_pin) = return_path.next(1.0, 48_000.0, profile);
        let mut settled_audio = initial_audio;
        let mut settled_pin = initial_pin;
        for _ in 0..240_000 {
            (settled_audio, settled_pin) = return_path.next(1.0, 48_000.0, profile);
        }
        assert!((3.39..3.4).contains(&initial_audio));
        assert!(initial_pin > 0.09);
        assert!(settled_audio.abs() < initial_audio * 0.002);
        assert!(
            settled_pin.abs() < initial_pin * 0.002,
            "DC residue: {settled_pin}"
        );
    }

    #[test]
    fn physical_resonance_loop_crosses_four_inside_the_service_window() {
        for (index, profile) in FILTER_PROFILES.into_iter().enumerate() {
            let below = resonance_audio_band_feedback(0.65, profile);
            let inside = resonance_audio_band_feedback(0.95, profile);
            assert!(
                below < FOUR_POLE_OSCILLATION_FEEDBACK,
                "profile {index} crossed early: {below}"
            );
            assert!(
                inside > FOUR_POLE_OSCILLATION_FEEDBACK,
                "profile {index} missed service window: {inside}"
            );
        }
    }

    #[test]
    fn service_scale_trim_compensates_the_return_phase_at_440_and_880_hz() {
        for profile in FILTER_PROFILES {
            let low_pole_hz = service_trimmed_cutoff_hz(440.0, profile);
            let high_pole_hz = service_trimmed_cutoff_hz(880.0, profile);
            assert!((440.0..445.0).contains(&low_pole_hz));
            assert!((880.0..885.0).contains(&high_pole_hz));
            let trim =
                profile.pole_sensitivity_mv_per_decade / NOMINAL_POLE_SENSITIVITY_MV_PER_DECADE;
            assert!((0.95..=1.05).contains(&trim));
        }
    }

    #[test]
    fn five_minute_warmup_motion_is_distinct_and_inside_published_limits() {
        let mut typical = [0.0; 5];
        let mut maximum = [0.0; 5];
        for profile in 0..5 {
            let mut filter = Cem3320Filter::with_profile(profile);
            for _ in 0..(300 * FILTER_WARMUP_UPDATE_RATE_HZ as usize) {
                filter.step_warmup();
            }
            typical[profile] = filter.warmup_frequency_ratio(0.0);
            maximum[profile] = filter.warmup_frequency_ratio(1.0);
            assert!((typical[profile] - 1.0).abs() <= TYPICAL_FILTER_WARMUP_DRIFT_RATIO);
            assert!((maximum[profile] - 1.0).abs() <= MAXIMUM_FILTER_WARMUP_DRIFT_RATIO);
            assert!((maximum[profile] - 1.0).abs() > (typical[profile] - 1.0).abs());
        }
        for index in 0..5 {
            assert!(
                typical[..index]
                    .iter()
                    .all(|previous| (previous - typical[index]).abs() > 1.0e-6)
            );
        }
    }

    #[test]
    fn warmup_elapsed_time_is_independent_of_audio_rate() {
        let mut low_rate = Cem3320Filter::with_profile(3);
        let mut high_rate = Cem3320Filter::with_profile(3);
        for _ in 0..441_000 {
            low_rate.advance_warmup(44_100.0);
        }
        for _ in 0..960_000 {
            high_rate.advance_warmup(96_000.0);
        }
        assert_eq!(low_rate.warmup_position, high_rate.warmup_position);
    }

    #[test]
    fn clipping_span_and_even_harmonic_are_profiled() {
        for profile in FILTER_PROFILES {
            let ceiling = profile.output_clip_vpp * 0.5;
            assert!(cell_output(100.0, profile) <= ceiling);
            assert!(cell_output(-100.0, profile) >= -ceiling);
            let positive = cell_output(1.0, profile);
            let negative = cell_output(-1.0, profile);
            assert!(positive + negative > 0.0, "missing even-order asymmetry");
            assert!(
                positive + negative < 0.02,
                "asymmetry exceeds candidate bound"
            );
        }
    }

    #[test]
    fn four_cell_passband_distortion_matches_the_published_filter_bound() {
        fn harmonic(profile: FilterProfile, harmonic: usize) -> f32 {
            const SAMPLE_COUNT: usize = 4_096;
            const CYCLES: usize = 17;
            let ceiling = profile.output_clip_vpp * 0.5;
            let amplitude = ceiling * SPECIFIED_SIGNAL_FRACTION_OF_CLIP;
            let mut sine = 0.0_f32;
            let mut cosine = 0.0_f32;
            for index in 0..SAMPLE_COUNT {
                let phase = 2.0 * core::f32::consts::PI * CYCLES as f32 * index as f32
                    / SAMPLE_COUNT as f32;
                let mut output = libm::sinf(phase) * amplitude;
                for index in 0..STAGE_COUNT {
                    if index > 0 {
                        output *= interstage_passband_gain();
                    }
                    output = cell_output(output, profile);
                }
                sine += output * libm::sinf(phase * harmonic as f32);
                cosine += output * libm::cosf(phase * harmonic as f32);
            }
            2.0 * libm::sqrtf(sine * sine + cosine * cosine) / SAMPLE_COUNT as f32
        }

        for profile in FILTER_PROFILES {
            let fundamental = harmonic(profile, 1);
            let second = harmonic(profile, 2) / fundamental;
            let third = harmonic(profile, 3) / fundamental;
            assert!(
                (profile.passband_second_harmonic * 0.8..=profile.passband_second_harmonic * 1.2)
                    .contains(&second),
                "H2 ratio {second} missed profile target"
            );
            assert!(
                second > third,
                "third harmonic dominated: H2={second}, H3={third}"
            );
        }
    }

    #[test]
    fn nonlinear_feedback_solver_closes_the_instantaneous_loop() {
        let mut filter = Cem3320Filter::default();
        for index in 0..4_096 {
            let input = libm::sinf(index as f32 * 0.071) * 4.0;
            let _ = filter.next(input, 1_700.0, 0.92, 192_000.0);
        }
        let profile = FILTER_PROFILES[filter.profile_index];
        for cutoff_hz in [50.0, 1_000.0, 12_000.0, 60_000.0] {
            let g = libm::tanf(core::f32::consts::PI * cutoff_hz / 192_000.0);
            let coefficient = g / (1.0 + g);
            for resonance in [0.1, 0.5, 0.8, 1.0] {
                let resonance_drive = resonance_drive_gain(resonance, profile);
                for input in [-16.0, -4.0, 0.0, 4.0, 16.0] {
                    let solved = filter.solve_feedback(
                        input,
                        coefficient,
                        resonance_drive,
                        192_000.0,
                        profile,
                    );
                    let (resonance_voltage, _) =
                        filter.resonance_return.predict(solved, 192_000.0, profile);
                    let (predicted, _) = filter.predict_path(
                        input - resonance_voltage * resonance_drive,
                        coefficient,
                        profile,
                    );
                    assert!(
                        (solved - predicted).abs() < 4.0e-4,
                        "cutoff={cutoff_hz}, resonance={resonance}, input={input}, residual={}",
                        solved - predicted
                    );
                }
            }
        }
    }

    #[test]
    fn low_cutoff_rejects_more_high_frequency_energy() {
        fn render(cutoff: f32) -> f32 {
            let mut filter = Cem3320Filter::default();
            let mut energy = 0.0;
            for index in 0..16_384 {
                let input =
                    libm::sinf(2.0 * core::f32::consts::PI * 6_000.0 * index as f32 / 48_000.0);
                let output = filter.next(input, cutoff, 0.0, 48_000.0);
                if index > 4_096 {
                    energy += output * output;
                }
            }
            energy
        }
        assert!(render(8_000.0) > render(500.0) * 100.0);
    }

    #[test]
    fn resonance_extends_the_impulse_decay() {
        fn tail_energy(resonance: f32) -> f32 {
            let mut filter = Cem3320Filter::default();
            let mut energy = 0.0;
            for index in 0..24_000 {
                let input = if index == 0 { 0.1 } else { 0.0 };
                let output = filter.next(input, 1_000.0, resonance, 48_000.0);
                if index > 1_000 {
                    energy += output * output;
                }
            }
            energy
        }
        assert!(tail_energy(0.9) > tail_energy(0.0) * 100.0);
    }

    #[test]
    fn zero_input_and_zero_resonance_decay_to_rest() {
        let mut filter = Cem3320Filter::default();
        let _ = filter.next(1.0, 4_000.0, 0.0, 48_000.0);
        for _ in 0..32_768 {
            let _ = filter.next(0.0, 4_000.0, 0.0, 48_000.0);
        }
        assert!(filter.energy() < 2.0e-6);
    }

    #[test]
    fn self_oscillation_begins_inside_the_service_manual_window() {
        fn late_peak(profile: usize, resonance: f32) -> f32 {
            let mut filter = Cem3320Filter::with_profile(profile);
            let mut peak = 0.0_f32;
            for index in 0..192_000 {
                let output = filter.next(0.0, 1_000.0, resonance, 48_000.0);
                if index > 144_000 {
                    peak = peak.max(output.abs());
                }
            }
            peak
        }

        for profile in 0..FILTER_PROFILES.len() {
            let below_window = late_peak(profile, 0.65);
            let inside_window = late_peak(profile, 0.95);
            assert!(
                below_window < 0.001,
                "profile {profile} oscillated too early: {below_window}"
            );
            assert!(
                inside_window > 0.01,
                "profile {profile} did not sustain: {inside_window}"
            );
        }
    }

    #[test]
    fn self_oscillation_frequency_tracks_cutoff_across_supported_rates() {
        const CUTOFF_HZ: f32 = 1_000.0;
        let mut measurements = [0.0; 4];
        for (index, sample_rate) in [44_100.0, 48_000.0, 96_000.0, 192_000.0]
            .into_iter()
            .enumerate()
        {
            let mut filter = Cem3320Filter::default();
            let settle_samples = sample_rate as usize;
            for _ in 0..settle_samples {
                let _ = filter.next(0.0, CUTOFF_HZ, 1.0, sample_rate);
            }

            let mut previous = filter.next(0.0, CUTOFF_HZ, 1.0, sample_rate);
            let mut rising_crossings = 0;
            for _ in 0..settle_samples {
                let output = filter.next(0.0, CUTOFF_HZ, 1.0, sample_rate);
                rising_crossings += usize::from(previous <= 0.0 && output > 0.0);
                previous = output;
            }
            measurements[index] = rising_crossings as f32;
        }
        for (sample_rate, measured_hz) in [44_100.0, 48_000.0, 96_000.0, 192_000.0]
            .into_iter()
            .zip(measurements)
        {
            assert!(
                (measured_hz - CUTOFF_HZ).abs() <= CUTOFF_HZ * 0.001,
                "rate={sample_rate}, measured={measured_hz}, all={measurements:?}"
            );
        }
    }

    #[test]
    fn self_oscillation_honors_the_service_calibration_pair() {
        for sample_rate in [48_000.0, 192_000.0] {
            for expected_hz in [440.0, 880.0] {
                let mut filter = Cem3320Filter::default();
                for _ in 0..sample_rate as usize {
                    let _ = filter.next(0.0, expected_hz, 1.0, sample_rate);
                }

                let mut previous = filter.next(0.0, expected_hz, 1.0, sample_rate);
                let mut rising_crossings = 0;
                for _ in 0..(sample_rate as usize * 2) {
                    let output = filter.next(0.0, expected_hz, 1.0, sample_rate);
                    rising_crossings += usize::from(previous <= 0.0 && output > 0.0);
                    previous = output;
                }
                let measured_hz = rising_crossings as f32 * 0.5;
                assert!(
                    (measured_hz - expected_hz).abs() <= 1.0,
                    "rate={sample_rate}, expected={expected_hz}, measured={measured_hz}"
                );
            }
        }
    }
}
