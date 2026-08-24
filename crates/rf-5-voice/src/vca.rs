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

/// The single common noise-level OTA before noise reaches all five filters.
pub fn common_noise(input: f32, control: f32) -> f32 {
    ota_transfer(
        input,
        control,
        UNLINEARIZED_INPUT_DRIVE,
        COMMON_NOISE_PROFILE,
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
        for profile in MIXER_PROFILES {
            for half in [profile.oscillator_a, profile.oscillator_b] {
                assert!(
                    (DATASHEET_MINIMUM_PEAK_CURRENT_RATIO..=DATASHEET_MAXIMUM_PEAK_CURRENT_RATIO)
                        .contains(&half.transconductance_ratio)
                );
                assert!((0.90..=1.10).contains(&half.input_drive_ratio));
            }
        }
        for profile in FINAL_VCA_PROFILES {
            assert_eq!(profile.transconductance_ratio, 1.0);
            assert!((0.90..=1.10).contains(&profile.input_drive_ratio));
        }
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
