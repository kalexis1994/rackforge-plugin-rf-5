//! Physical CA3280 boundaries in the RF-5 audio path.
//!
//! Each voice card uses both halves of one CA3280 for oscillator A/B level,
//! with the linearizing-diode terminal cut off. A separate linearized half is
//! the final voice VCA. The common noise level and master volume each have
//! their own physical CA3280 stage. Balance and voice-volume trimmers are
//! treated as serviced, so they cancel zero-input feed-through and equalize
//! the five final-VCA small-signal gains.

const UNLINEARIZED_INPUT_DRIVE: f32 = 0.55;
const LINEARIZED_INPUT_DRIVE: f32 = 0.05;
// TM1000D.2 section 2-5 gives approximately 100 kohm for a CA3280 input with
// its linearizing-diode terminal cut off. SD431 feeds saw/triangle through
// 150 kohm and pulse through 200 kohm. Conductances are normalized to the
// populated 150 kohm path so the existing single-saw calibration remains the
// sole circuit-to-host level anchor.
const UNLINEARIZED_INPUT_RESISTANCE_OHMS: f32 = 100_000.0;
const MIXER_REFERENCE_SOURCE_RESISTANCE_OHMS: f32 = 150_000.0;
const MIXER_INPUT_CONDUCTANCE_RATIO: f32 =
    MIXER_REFERENCE_SOURCE_RESISTANCE_OHMS / UNLINEARIZED_INPUT_RESISTANCE_OHMS;
#[cfg(test)]
const DATASHEET_MINIMUM_PEAK_CURRENT_RATIO: f32 = 0.70;
#[cfg(test)]
const DATASHEET_MAXIMUM_PEAK_CURRENT_RATIO: f32 = 1.30;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MixerChannel {
    OscillatorA,
    OscillatorB,
}

#[derive(Clone, Copy, Debug)]
struct OtaHalfProfile {
    transconductance_ratio: f32,
    input_drive_ratio: f32,
}

#[derive(Clone, Copy, Debug)]
struct MixerProfile {
    oscillator_a: OtaHalfProfile,
    oscillator_b: OtaHalfProfile,
}

#[derive(Clone, Copy, Debug)]
struct EnvelopeAmountProfile {
    direct_filter: OtaHalfProfile,
    poly_mod: OtaHalfProfile,
}

// One dual OTA per voice card. The conservative deterministic spread stays
// well inside the data-sheet 0.70-1.30 peak-output-current ratio. Both halves
// of a package remain close, while no two physical packages collapse to the
// same transfer.
const MIXER_PROFILES: [MixerProfile; 5] = [
    MixerProfile {
        oscillator_a: OtaHalfProfile::new(0.965, 1.055),
        oscillator_b: OtaHalfProfile::new(0.982, 1.027),
    },
    MixerProfile {
        oscillator_a: OtaHalfProfile::new(1.018, 0.973),
        oscillator_b: OtaHalfProfile::new(1.006, 0.991),
    },
    MixerProfile {
        oscillator_a: OtaHalfProfile::new(0.992, 1.009),
        oscillator_b: OtaHalfProfile::new(1.011, 0.982),
    },
    MixerProfile {
        oscillator_a: OtaHalfProfile::new(1.036, 0.945),
        oscillator_b: OtaHalfProfile::new(1.021, 0.967),
    },
    MixerProfile {
        oscillator_a: OtaHalfProfile::new(0.978, 1.033),
        oscillator_b: OtaHalfProfile::new(0.995, 1.018),
    },
];

// The service procedure separately trims each final voice level. Therefore
// these profiles intentionally retain unity small-signal gain and vary only
// the strong-signal knee left after diode linearization.
const FINAL_VCA_PROFILES: [OtaHalfProfile; 5] = [
    OtaHalfProfile::new(1.0, 1.040),
    OtaHalfProfile::new(1.0, 0.956),
    OtaHalfProfile::new(1.0, 1.018),
    OtaHalfProfile::new(1.0, 0.978),
    OtaHalfProfile::new(1.0, 1.009),
];

// Both halves of U422 on each voice card: direct filter-envelope amount and
// the inverted filter-envelope contribution to Poly Mod. The service trims
// cancel offsets, not the small remaining transconductance spread.
const ENVELOPE_AMOUNT_PROFILES: [EnvelopeAmountProfile; 5] = [
    EnvelopeAmountProfile {
        direct_filter: OtaHalfProfile::new(0.984, 1.018),
        poly_mod: OtaHalfProfile::new(0.997, 0.991),
    },
    EnvelopeAmountProfile {
        direct_filter: OtaHalfProfile::new(1.012, 0.973),
        poly_mod: OtaHalfProfile::new(0.989, 1.027),
    },
    EnvelopeAmountProfile {
        direct_filter: OtaHalfProfile::new(0.995, 1.009),
        poly_mod: OtaHalfProfile::new(1.015, 0.982),
    },
    EnvelopeAmountProfile {
        direct_filter: OtaHalfProfile::new(1.021, 0.956),
        poly_mod: OtaHalfProfile::new(1.008, 0.967),
    },
    EnvelopeAmountProfile {
        direct_filter: OtaHalfProfile::new(0.991, 1.033),
        poly_mod: OtaHalfProfile::new(1.018, 1.009),
    },
];

// One unlinearized oscillator-B Poly Mod amount OTA per voice card.
const POLY_MOD_OSCILLATOR_B_PROFILES: [OtaHalfProfile; 5] = [
    OtaHalfProfile::new(0.976, 1.040),
    OtaHalfProfile::new(1.019, 0.970),
    OtaHalfProfile::new(0.991, 1.010),
    OtaHalfProfile::new(1.028, 0.950),
    OtaHalfProfile::new(1.004, 1.020),
];

// U378 is the common dual OTA whose two halves move in opposite directions
// under the Wheel Mod source-mix CV. Its balance trimmers remove zero-input
// offset while retaining the two real transfer paths.
const WHEEL_MOD_LFO_PROFILE: OtaHalfProfile = OtaHalfProfile::new(0.993, 1.018);
const WHEEL_MOD_NOISE_PROFILE: OtaHalfProfile = OtaHalfProfile::new(1.007, 0.982);

const COMMON_NOISE_PROFILE: OtaHalfProfile = OtaHalfProfile::new(0.987, 1.036);
const MASTER_VCA_PROFILE: OtaHalfProfile = OtaHalfProfile::new(1.0, 0.991);

impl OtaHalfProfile {
    const fn new(transconductance_ratio: f32, input_drive_ratio: f32) -> Self {
        Self {
            transconductance_ratio,
            input_drive_ratio,
        }
    }
}

/// One half of the dual, unlinearized oscillator-level OTA on a voice card.
pub fn oscillator_mixer(
    input: f32,
    control: f32,
    voice_index: usize,
    channel: MixerChannel,
) -> f32 {
    let profile = MIXER_PROFILES[voice_index % MIXER_PROFILES.len()];
    let half = match channel {
        MixerChannel::OscillatorA => profile.oscillator_a,
        MixerChannel::OscillatorB => profile.oscillator_b,
    };
    ota_transfer(input, control, UNLINEARIZED_INPUT_DRIVE, half)
}

/// One oscillator mixer half including the finite CA3280 input loading shared
/// by every simultaneously selected waveform resistor.
///
/// `source_conductance` is relative to one 150 kohm path: saw and triangle are
/// 1.0 each, while the populated 200 kohm pulse path is 0.75. The normalization
/// deliberately leaves one selected saw unchanged, avoiding a second unknown
/// circuit-volts-to-host calibration.
pub fn oscillator_mixer_loaded(
    input: f32,
    source_conductance: f32,
    control: f32,
    voice_index: usize,
    channel: MixerChannel,
) -> f32 {
    if !input.is_finite() || !source_conductance.is_finite() || source_conductance <= 0.0 {
        return 0.0;
    }
    let reference_loaded_conductance = MIXER_INPUT_CONDUCTANCE_RATIO + 1.0;
    let selected_loaded_conductance = MIXER_INPUT_CONDUCTANCE_RATIO + source_conductance;
    oscillator_mixer(
        input * reference_loaded_conductance / selected_loaded_conductance,
        control,
        voice_index,
        channel,
    )
}

/// The single common noise-level OTA before noise reaches all five filters.
pub fn common_noise(input: f32, control: f32) -> f32 {
    ota_transfer(
        input,
        control,
        UNLINEARIZED_INPUT_DRIVE,
        COMMON_NOISE_PROFILE,
    )
}

/// The common dual-OTA crossfade feeding the physical modulation wheel.
pub fn wheel_mod_source(lfo: f32, noise: f32, source_mix: f32) -> f32 {
    if !source_mix.is_finite() {
        return 0.0;
    }
    let noise_control = source_mix.clamp(0.0, 1.0);
    let lfo_control = 1.0 - noise_control;
    ota_transfer(
        lfo,
        lfo_control,
        UNLINEARIZED_INPUT_DRIVE,
        WHEEL_MOD_LFO_PROFILE,
    ) + ota_transfer(
        noise,
        noise_control,
        UNLINEARIZED_INPUT_DRIVE,
        WHEEL_MOD_NOISE_PROFILE,
    )
}

/// The direct filter-envelope amount half of U422 on one voice card.
pub fn filter_envelope_amount(input: f32, control: f32, voice_index: usize) -> f32 {
    ota_transfer(
        input,
        control,
        LINEARIZED_INPUT_DRIVE,
        ENVELOPE_AMOUNT_PROFILES[voice_index % ENVELOPE_AMOUNT_PROFILES.len()].direct_filter,
    )
}

/// The second, inverted-at-the-summing-node U422 envelope amount path.
pub fn poly_mod_filter_envelope(input: f32, control: f32, voice_index: usize) -> f32 {
    ota_transfer(
        input,
        control,
        LINEARIZED_INPUT_DRIVE,
        ENVELOPE_AMOUNT_PROFILES[voice_index % ENVELOPE_AMOUNT_PROFILES.len()].poly_mod,
    )
}

/// The unlinearized oscillator-B waveform amount OTA in one Poly Mod path.
pub fn poly_mod_oscillator_b(input: f32, control: f32, voice_index: usize) -> f32 {
    ota_transfer(
        input,
        control,
        UNLINEARIZED_INPUT_DRIVE,
        POLY_MOD_OSCILLATOR_B_PROFILES[voice_index % POLY_MOD_OSCILLATOR_B_PROFILES.len()],
    )
}

/// The diode-linearized and service-calibrated final VCA on one voice card.
pub fn final_voice(input: f32, control: f32, voice_index: usize) -> f32 {
    ota_transfer(
        input,
        control,
        LINEARIZED_INPUT_DRIVE,
        FINAL_VCA_PROFILES[voice_index % FINAL_VCA_PROFILES.len()],
    )
}

/// The diode-linearized common VCA driven by the physical volume control.
pub fn master_output(input: f32, control: f32) -> f32 {
    ota_transfer(input, control, LINEARIZED_INPUT_DRIVE, MASTER_VCA_PROFILE)
}

fn ota_transfer(input: f32, control: f32, nominal_drive: f32, profile: OtaHalfProfile) -> f32 {
    if control <= 0.0 || !control.is_finite() || !input.is_finite() {
        return 0.0;
    }
    let drive = nominal_drive * profile.input_drive_ratio;
    let current = libm::tanhf(input * drive) / drive;
    current * control.clamp(0.0, 1.0) * profile.transconductance_ratio
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_bias_current_closes_every_physical_vca() {
        for input in [-8.0, -1.0, 0.0, 1.0, 8.0] {
            for voice in 0..5 {
                assert_eq!(
                    oscillator_mixer(input, 0.0, voice, MixerChannel::OscillatorA),
                    0.0
                );
                assert_eq!(
                    oscillator_mixer(input, 0.0, voice, MixerChannel::OscillatorB),
                    0.0
                );
                assert_eq!(final_voice(input, 0.0, voice), 0.0);
                assert_eq!(filter_envelope_amount(input, 0.0, voice), 0.0);
                assert_eq!(poly_mod_filter_envelope(input, 0.0, voice), 0.0);
                assert_eq!(poly_mod_oscillator_b(input, 0.0, voice), 0.0);
            }
            assert_eq!(common_noise(input, 0.0), 0.0);
            assert_eq!(master_output(input, 0.0), 0.0);
        }
    }

    #[test]
    fn control_current_changes_gain_monotonically() {
        let low = oscillator_mixer(0.5, 0.25, 2, MixerChannel::OscillatorA).abs();
        let middle = oscillator_mixer(0.5, 0.5, 2, MixerChannel::OscillatorA).abs();
        let high = oscillator_mixer(0.5, 1.0, 2, MixerChannel::OscillatorA).abs();
        assert!(low < middle && middle < high);
    }

    #[test]
    fn active_linearizing_diodes_extend_the_input_range() {
        let input = 3.0;
        let unlinearized = oscillator_mixer(input, 1.0, 2, MixerChannel::OscillatorA).abs();
        let linearized = final_voice(input, 1.0, 2).abs();
        let mixer_small_signal =
            oscillator_mixer(0.001, 1.0, 2, MixerChannel::OscillatorA).abs() / 0.001;
        let final_small_signal = final_voice(0.001, 1.0, 2).abs() / 0.001;
        let mixer_retained = unlinearized / (input * mixer_small_signal);
        let final_retained = linearized / (input * final_small_signal);
        assert!(final_retained > 0.99);
        assert!(mixer_retained < 0.65);
    }

    #[test]
    fn device_population_stays_inside_published_output_bounds() {
        fn assert_inside_bounds(profile: OtaHalfProfile) {
            assert!(
                (DATASHEET_MINIMUM_PEAK_CURRENT_RATIO..=DATASHEET_MAXIMUM_PEAK_CURRENT_RATIO)
                    .contains(&profile.transconductance_ratio)
            );
            assert!((0.90..=1.10).contains(&profile.input_drive_ratio));
        }

        for profile in MIXER_PROFILES {
            for half in [profile.oscillator_a, profile.oscillator_b] {
                assert_inside_bounds(half);
            }
        }
        for profile in FINAL_VCA_PROFILES {
            assert_eq!(profile.transconductance_ratio, 1.0);
            assert_inside_bounds(profile);
        }
        for profile in ENVELOPE_AMOUNT_PROFILES {
            assert_inside_bounds(profile.direct_filter);
            assert_inside_bounds(profile.poly_mod);
        }
        for profile in POLY_MOD_OSCILLATOR_B_PROFILES {
            assert_inside_bounds(profile);
        }
        assert_inside_bounds(WHEEL_MOD_LFO_PROFILE);
        assert_inside_bounds(WHEEL_MOD_NOISE_PROFILE);
        assert_inside_bounds(COMMON_NOISE_PROFILE);
        assert_inside_bounds(MASTER_VCA_PROFILE);
    }

    #[test]
    fn paired_mixer_halves_remain_close_but_not_identical() {
        for profile in MIXER_PROFILES {
            assert!(
                (profile.oscillator_a.transconductance_ratio
                    - profile.oscillator_b.transconductance_ratio)
                    .abs()
                    < 0.03
            );
            assert_ne!(
                profile.oscillator_a.transconductance_ratio,
                profile.oscillator_b.transconductance_ratio
            );
        }
    }

    #[test]
    fn mixer_loading_preserves_the_single_150k_path_anchor() {
        for voice in 0..5 {
            for channel in [MixerChannel::OscillatorA, MixerChannel::OscillatorB] {
                assert_eq!(
                    oscillator_mixer_loaded(0.75, 1.0, 0.6, voice, channel),
                    oscillator_mixer(0.75, 0.6, voice, channel)
                );
            }
        }
    }

    #[test]
    fn parallel_waveform_paths_load_the_finite_mixer_input() {
        let one_path = oscillator_mixer_loaded(0.5, 1.0, 1.0, 2, MixerChannel::OscillatorA);
        let two_equal_paths = oscillator_mixer_loaded(1.0, 2.0, 1.0, 2, MixerChannel::OscillatorA);
        let unloaded_linear_sum = one_path * 2.0;
        assert!(two_equal_paths > one_path);
        assert!(two_equal_paths < unloaded_linear_sum);

        let expected_input_ratio =
            2.0 * (MIXER_INPUT_CONDUCTANCE_RATIO + 1.0) / (MIXER_INPUT_CONDUCTANCE_RATIO + 2.0);
        let small_one = oscillator_mixer_loaded(0.001, 1.0, 1.0, 2, MixerChannel::OscillatorA);
        let small_two = oscillator_mixer_loaded(0.002, 2.0, 1.0, 2, MixerChannel::OscillatorA);
        assert!((small_two / small_one - expected_input_ratio).abs() < 1.0e-5);
    }

    #[test]
    fn pulse_path_uses_its_populated_200k_conductance() {
        let loaded = oscillator_mixer_loaded(0.75, 0.75, 1.0, 2, MixerChannel::OscillatorA);
        let reference = oscillator_mixer(0.75, 1.0, 2, MixerChannel::OscillatorA);
        assert!(loaded > reference);
        assert!(loaded < reference * 1.12);
    }

    #[test]
    fn absent_or_invalid_mixer_sources_are_silent() {
        for source in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                oscillator_mixer_loaded(1.0, source, 1.0, 0, MixerChannel::OscillatorA),
                0.0
            );
        }
    }

    #[test]
    fn paired_envelope_amount_halves_remain_close_but_not_identical() {
        for profile in ENVELOPE_AMOUNT_PROFILES {
            assert!(
                (profile.direct_filter.transconductance_ratio
                    - profile.poly_mod.transconductance_ratio)
                    .abs()
                    < 0.03
            );
            assert_ne!(
                profile.direct_filter.transconductance_ratio,
                profile.poly_mod.transconductance_ratio
            );
        }
    }

    #[test]
    fn wheel_mod_dual_ota_is_complementary_and_balanced() {
        let lfo_only = wheel_mod_source(0.75, -8.0, 0.0);
        assert_eq!(lfo_only, wheel_mod_source(0.75, 8.0, 0.0));
        let noise_only = wheel_mod_source(-8.0, 0.75, 1.0);
        assert_eq!(noise_only, wheel_mod_source(8.0, 0.75, 1.0));
        let middle = wheel_mod_source(0.75, 0.75, 0.5);
        assert!(middle > lfo_only.min(noise_only));
        assert!(middle < lfo_only.max(noise_only) * 1.02);
        for mix in [0.0, 0.25, 0.5, 0.75, 1.0] {
            assert_eq!(wheel_mod_source(0.0, 0.0, mix), 0.0);
        }
    }

    #[test]
    fn poly_mod_amount_vcas_are_monotonic_and_mode_correct() {
        for voice in 0..5 {
            let low = poly_mod_oscillator_b(0.7, 0.25, voice).abs();
            let high = poly_mod_oscillator_b(0.7, 0.75, voice).abs();
            assert!(low < high);

            let linearized = poly_mod_filter_envelope(3.0, 1.0, voice);
            let unlinearized = poly_mod_oscillator_b(3.0, 1.0, voice);
            let linearized_small = poly_mod_filter_envelope(0.001, 1.0, voice) / 0.001;
            let unlinearized_small = poly_mod_oscillator_b(0.001, 1.0, voice) / 0.001;
            assert!(linearized / (3.0 * linearized_small) > 0.99);
            assert!(unlinearized / (3.0 * unlinearized_small) < 0.65);
        }
    }

    #[test]
    fn service_calibration_equalizes_final_vca_small_signal_gain() {
        let reference = final_voice(0.001, 1.0, 0);
        for voice in 1..5 {
            assert!((final_voice(0.001, 1.0, voice) - reference).abs() < 1.0e-8);
        }
    }

    #[test]
    fn transfers_are_odd_symmetric_finite_and_profiled() {
        let mut profile_outputs = [0.0; 5];
        for (voice, profile_output) in profile_outputs.iter_mut().enumerate() {
            for index in 0..10_000 {
                let input = index as f32 * 0.002;
                let positive = oscillator_mixer(input, 1.0, voice, MixerChannel::OscillatorA);
                let negative = oscillator_mixer(-input, 1.0, voice, MixerChannel::OscillatorA);
                assert!(positive.is_finite());
                assert!((positive + negative).abs() < 1.0e-6);
            }
            *profile_output = oscillator_mixer(2.0, 1.0, voice, MixerChannel::OscillatorA);
        }
        assert!(profile_outputs.windows(2).any(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn non_finite_controls_and_inputs_are_silenced() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(master_output(invalid, 1.0), 0.0);
            assert_eq!(master_output(1.0, invalid), 0.0);
        }
    }
}
