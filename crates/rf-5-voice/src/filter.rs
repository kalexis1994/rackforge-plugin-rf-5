//! Four-pole CEM3320 low-pass candidate with a bounded five-chip population.
//!
//! The reference voice wires the four independent cells as cascaded low-pass
//! stages and returns the fourth output through the chip's resonance VCA. This
//! model preserves that topology and evaluates it at the voice oversampling
//! rate. Published control-scale, resonance-transconductance, output-swing and
//! passband-distortion limits bound one deterministic profile per voice card.

const STAGE_COUNT: usize = 4;
const MAXIMUM_NORMALIZED_CUTOFF: f32 = 0.45;
const MAXIMUM_RESONANCE_FEEDBACK: f32 = 4.4;
const THERMAL_NOISE_LEVEL: f32 = 2.0e-7;
const NOMINAL_POLE_SENSITIVITY_MV_PER_DECADE: f32 = 60.0;
const CUTOFF_PROFILE_REFERENCE_HZ: f32 = 1_000.0;

// The normalized signal path is not yet calibrated to circuit volts. Two
// circuit volts per internal unit places the data-sheet 10-14 Vpp output swing
// around the overload region of the existing mixer candidate and keeps the
// conversion explicit for later measurements.
const CANDIDATE_CIRCUIT_VOLTS_PER_UNIT: f32 = 2.0;
const SPECIFIED_SIGNAL_FRACTION_OF_CLIP: f32 = 0.707_106_77;

#[derive(Clone, Copy, Debug)]
struct FilterProfile {
    pole_sensitivity_mv_per_decade: f32,
    resonance_gm_ratio: f32,
    output_clip_vpp: f32,
    passband_second_harmonic: f32,
}

// Deterministic validation population. Every entry stays within the CEM3320
// data-sheet limits: 57.5-62.5 mV/decade, 0.8-1.2 resonance Gm ratio and
// 10-14 Vpp clipping. The 0.1-0.3% figures model the stated predominantly
// second-harmonic passband distortion near the specified strong-signal point.
const FILTER_PROFILES: [FilterProfile; 5] = [
    FilterProfile {
        pole_sensitivity_mv_per_decade: 58.0,
        resonance_gm_ratio: 0.91,
        output_clip_vpp: 11.0,
        passband_second_harmonic: 0.0014,
    },
    FilterProfile {
        pole_sensitivity_mv_per_decade: 59.1,
        resonance_gm_ratio: 1.06,
        output_clip_vpp: 12.6,
        passband_second_harmonic: 0.0021,
    },
    FilterProfile {
        pole_sensitivity_mv_per_decade: 60.0,
        resonance_gm_ratio: 1.0,
        output_clip_vpp: 12.0,
        passband_second_harmonic: 0.0018,
    },
    FilterProfile {
        pole_sensitivity_mv_per_decade: 61.2,
        resonance_gm_ratio: 0.96,
        output_clip_vpp: 13.4,
        passband_second_harmonic: 0.0028,
    },
    FilterProfile {
        pole_sensitivity_mv_per_decade: 62.2,
        resonance_gm_ratio: 1.12,
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
}

#[derive(Clone, Copy, Debug)]
pub struct Cem3320Filter {
    stages: [TptStage; STAGE_COUNT],
    noise_state: u32,
    profile_index: usize,
}

impl Default for Cem3320Filter {
    fn default() -> Self {
        Self {
            stages: [TptStage::default(); STAGE_COUNT],
            noise_state: 0x51f1_5e5d,
            profile_index: 2,
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
        let sample_rate = sample_rate.max(1.0);
        let profile = FILTER_PROFILES[self.profile_index];
        let cutoff_hz = profiled_cutoff_hz(cutoff_hz, profile)
            .clamp(1.0, sample_rate * MAXIMUM_NORMALIZED_CUTOFF);
        let g = libm::tanf(core::f32::consts::PI * cutoff_hz / sample_rate);
        let coefficient = g / (1.0 + g);
        let feedback = resonance_feedback(resonance) * profile.resonance_gm_ratio;
        let thermal_noise = self.next_thermal_noise();
        let mut signal = cell_output(
            input + thermal_noise - self.stages[3].state * feedback,
            profile,
        );
        for stage in &mut self.stages {
            signal = stage.next(signal, coefficient, profile);
        }
        if signal.is_finite() {
            signal
        } else {
            self.stages = [TptStage::default(); STAGE_COUNT];
            0.0
        }
    }

    fn next_thermal_noise(&mut self) -> f32 {
        let mut state = self.noise_state;
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        self.noise_state = state;
        let normalized = state as f32 / u32::MAX as f32;
        (normalized * 2.0 - 1.0) * THERMAL_NOISE_LEVEL
    }

    #[cfg(test)]
    fn energy(self) -> f32 {
        self.stages.iter().map(|stage| stage.state.abs()).sum()
    }
}

fn resonance_feedback(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    // The CEM3320 resonance cell is described as modified-linear and its
    // transconductance curve flattens near maximum current. This gentle bend
    // retains a near-linear panel response while placing oscillation inside
    // the 7-9.5 service-manual dial window.
    let modified_linear = value * (1.08 - 0.08 * value);
    modified_linear * MAXIMUM_RESONANCE_FEEDBACK
}

fn profiled_cutoff_hz(cutoff_hz: f32, profile: FilterProfile) -> f32 {
    let ratio = cutoff_hz.max(1.0) / CUTOFF_PROFILE_REFERENCE_HZ;
    CUTOFF_PROFILE_REFERENCE_HZ
        * libm::powf(
            ratio,
            NOMINAL_POLE_SENSITIVITY_MV_PER_DECADE / profile.pole_sensitivity_mv_per_decade,
        )
}

fn cell_output(value: f32, profile: FilterProfile) -> f32 {
    let ceiling = profile.output_clip_vpp * 0.5 / CANDIDATE_CIRCUIT_VOLTS_PER_UNIT;
    let normalized = (value / ceiling).clamp(-64.0, 64.0);
    // A sixteenth-order soft knee remains nearly linear at the data sheet's
    // distortion test level, then approaches the published clipping swing.
    // Unlike tanh, it does not inject a large third harmonic before clipping.
    let symmetric =
        ceiling * normalized / libm::powf(1.0 + libm::powf(normalized.abs(), 16.0), 1.0 / 16.0);
    // y = x + k*x^2 has H2/fundamental ~= k*A/2. Calibrate k at the specified
    // signal amplitude (3 dB below clipping), so each profile's value maps to
    // the stated 0.1-0.3% passband measurement rather than to an arbitrary
    // normalized amplitude. The tiny DC term reflects operating-point shift.
    let reference_amplitude = ceiling * SPECIFIED_SIGNAL_FRACTION_OF_CLIP;
    let even_coefficient = 2.0 * profile.passband_second_harmonic / reference_amplitude;
    let even_harmonic = even_coefficient * symmetric * symmetric;
    (symmetric + even_harmonic).clamp(-ceiling, ceiling)
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
            assert!((10.0..=14.0).contains(&profile.output_clip_vpp));
            assert!((0.001..=0.003).contains(&profile.passband_second_harmonic));
        }
    }

    #[test]
    fn pole_scale_population_meets_at_the_calibration_reference() {
        for profile in FILTER_PROFILES {
            assert_eq!(profiled_cutoff_hz(1_000.0, profile), 1_000.0);
            let low = profiled_cutoff_hz(100.0, profile);
            let high = profiled_cutoff_hz(10_000.0, profile);
            assert!((85.0..=110.0).contains(&low));
            assert!((9_000.0..=11_800.0).contains(&high));
        }
    }

    #[test]
    fn clipping_span_and_even_harmonic_are_profiled() {
        for profile in FILTER_PROFILES {
            let ceiling = profile.output_clip_vpp * 0.5 / CANDIDATE_CIRCUIT_VOLTS_PER_UNIT;
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
    fn published_strong_signal_distortion_is_second_harmonic_dominant() {
        fn harmonic(profile: FilterProfile, harmonic: usize) -> f32 {
            const SAMPLE_COUNT: usize = 4_096;
            const CYCLES: usize = 17;
            let ceiling = profile.output_clip_vpp * 0.5 / CANDIDATE_CIRCUIT_VOLTS_PER_UNIT;
            let amplitude = ceiling * SPECIFIED_SIGNAL_FRACTION_OF_CLIP;
            let mut sine = 0.0_f32;
            let mut cosine = 0.0_f32;
            for index in 0..SAMPLE_COUNT {
                let phase = 2.0 * core::f32::consts::PI * CYCLES as f32 * index as f32
                    / SAMPLE_COUNT as f32;
                let output = cell_output(libm::sinf(phase) * amplitude, profile);
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
}
