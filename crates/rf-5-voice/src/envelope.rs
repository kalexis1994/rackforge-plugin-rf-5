//! Ten-device CEM3310 envelope population.
//!
//! The source-backed parts are the true-RC attack/decay/release shape,
//! exponential time control and linear sustain control. The panel-to-time
//! calibration remains bounded because no measurements from the target unit
//! are available. Published electrical limits now define one filter and one
//! amplifier profile for every voice card.

const MINIMUM_TIME_CONSTANT_SECONDS: f32 = 0.002;
const MAXIMUM_TIME_CONSTANT_SECONDS: f32 = 20.0;
const IDLE_THRESHOLD: f32 = 1.0e-5;
const NOMINAL_CONTROL_SENSITIVITY_MV_PER_DECADE: f32 = 60.0;
const NOMINAL_PEAK_VOLTS: f32 = 5.0;

#[derive(Clone, Copy, Debug)]
struct EnvelopeProfile {
    attack_asymptote_volts: f32,
    peak_volts: f32,
    control_sensitivity_mv_per_decade: f32,
    rc_time_ratio: f32,
    sustain_error_volts: f32,
}

// Two devices per voice: amplifier first, filter second. The electrical
// endpoints come from the CEM3310 data sheet. RC ratios stay inside the stated
// +/-15% practical unit-to-unit tracking envelope and intentionally include
// the external 0.02 uF mylar capacitor population on SD431.
const ENVELOPE_PROFILES: [EnvelopeProfile; 10] = [
    EnvelopeProfile {
        attack_asymptote_volts: 6.20,
        peak_volts: 4.80,
        control_sensitivity_mv_per_decade: 58.8,
        rc_time_ratio: 0.90,
        sustain_error_volts: -0.003,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.80,
        peak_volts: 5.20,
        control_sensitivity_mv_per_decade: 61.1,
        rc_time_ratio: 1.10,
        sustain_error_volts: 0.017,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.40,
        peak_volts: 4.90,
        control_sensitivity_mv_per_decade: 59.4,
        rc_time_ratio: 0.96,
        sustain_error_volts: 0.005,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.60,
        peak_volts: 5.10,
        control_sensitivity_mv_per_decade: 60.7,
        rc_time_ratio: 1.06,
        sustain_error_volts: 0.014,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.50,
        peak_volts: 5.00,
        control_sensitivity_mv_per_decade: 60.0,
        rc_time_ratio: 1.00,
        sustain_error_volts: 0.0,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.10,
        peak_volts: 4.70,
        control_sensitivity_mv_per_decade: 58.5,
        rc_time_ratio: 0.87,
        sustain_error_volts: -0.003,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.90,
        peak_volts: 5.30,
        control_sensitivity_mv_per_decade: 61.5,
        rc_time_ratio: 1.13,
        sustain_error_volts: 0.023,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.30,
        peak_volts: 4.85,
        control_sensitivity_mv_per_decade: 59.0,
        rc_time_ratio: 0.93,
        sustain_error_volts: 0.004,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.70,
        peak_volts: 5.15,
        control_sensitivity_mv_per_decade: 61.0,
        rc_time_ratio: 1.08,
        sustain_error_volts: 0.019,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.45,
        peak_volts: 5.05,
        control_sensitivity_mv_per_decade: 59.8,
        rc_time_ratio: 1.02,
        sustain_error_volts: 0.008,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Copy, Debug)]
pub struct AdsrEnvelope {
    value: f32,
    stage: Stage,
    profile_index: usize,
}

impl Default for AdsrEnvelope {
    fn default() -> Self {
        Self {
            value: 0.0,
            stage: Stage::Idle,
            profile_index: 4,
        }
    }
}

impl AdsrEnvelope {
    pub fn with_profile(profile_index: usize) -> Self {
        Self {
            profile_index: profile_index % ENVELOPE_PROFILES.len(),
            ..Self::default()
        }
    }

    pub fn trigger(&mut self) {
        // The CEM3310 charges one external capacitor. A trigger changes the
        // phase but does not digitally clear the capacitor voltage.
        self.stage = Stage::Attack;
    }

    pub fn release(&mut self) {
        if matches!(self.stage, Stage::Idle | Stage::Release) {
            return;
        }
        self.stage = Stage::Release;
    }

    pub fn next(
        &mut self,
        sample_rate: f32,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    ) -> f32 {
        let profile = ENVELOPE_PROFILES[self.profile_index];
        let peak = profile.peak_volts / NOMINAL_PEAK_VOLTS;
        let sustain_target = (sustain.clamp(0.0, 1.0) * peak
            + profile.sustain_error_volts / NOMINAL_PEAK_VOLTS)
            .clamp(0.0, peak);
        match self.stage {
            Stage::Idle => {}
            Stage::Attack => {
                self.value = approach(
                    self.value,
                    profile.attack_asymptote_volts / NOMINAL_PEAK_VOLTS,
                    profiled_time_constant_seconds(attack, profile),
                    sample_rate,
                );
                if self.value >= peak {
                    self.value = peak;
                    self.stage = Stage::Decay;
                }
            }
            Stage::Decay => {
                self.value = approach(
                    self.value,
                    sustain_target,
                    profiled_time_constant_seconds(decay, profile),
                    sample_rate,
                );
                if (self.value - sustain_target).abs() <= IDLE_THRESHOLD {
                    self.value = sustain_target;
                    self.stage = Stage::Sustain;
                }
            }
            Stage::Sustain => self.value = sustain_target,
            Stage::Release => {
                self.value = approach(
                    self.value,
                    0.0,
                    profiled_time_constant_seconds(release, profile),
                    sample_rate,
                );
                if self.value <= IDLE_THRESHOLD {
                    self.value = 0.0;
                    self.stage = Stage::Idle;
                }
            }
        }
        self.value
    }

    pub fn is_idle(self) -> bool {
        self.stage == Stage::Idle
    }

    pub fn value(self) -> f32 {
        self.value
    }
}

fn approach(value: f32, target: f32, time_constant: f32, sample_rate: f32) -> f32 {
    let coefficient = libm::expf(-1.0 / (time_constant * sample_rate.max(1.0)));
    target + (value - target) * coefficient
}

pub fn time_constant_seconds(value: f32) -> f32 {
    let decades = libm::log10f(MAXIMUM_TIME_CONSTANT_SECONDS / MINIMUM_TIME_CONSTANT_SECONDS);
    MINIMUM_TIME_CONSTANT_SECONDS * libm::powf(10.0, value.clamp(0.0, 1.0) * decades)
}

fn profiled_time_constant_seconds(value: f32, profile: EnvelopeProfile) -> f32 {
    let decades = libm::log10f(MAXIMUM_TIME_CONSTANT_SECONDS / MINIMUM_TIME_CONSTANT_SECONDS);
    let scale =
        NOMINAL_CONTROL_SENSITIVITY_MV_PER_DECADE / profile.control_sensitivity_mv_per_decade;
    MINIMUM_TIME_CONSTANT_SECONDS
        * profile.rc_time_ratio
        * libm::powf(10.0, value.clamp(0.0, 1.0) * decades * scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger_attack_decay_sustain_and_release_are_complete() {
        let mut envelope = AdsrEnvelope::default();
        envelope.trigger();
        let mut peak = 0.0_f32;
        for _ in 0..500_000 {
            peak = peak.max(envelope.next(48_000.0, 0.01, 0.05, 0.4, 0.01));
            if (envelope.value - 0.4).abs() < 1.0e-5 {
                break;
            }
        }
        assert!(peak > 0.99);
        assert!((envelope.value - 0.4).abs() < 1.0e-5);
        envelope.release();
        for _ in 0..500_000 {
            let _ = envelope.next(48_000.0, 0.01, 0.05, 0.4, 0.01);
            if envelope.is_idle() {
                break;
            }
        }
        assert!(envelope.is_idle());
        assert_eq!(envelope.value, 0.0);
    }

    #[test]
    fn two_envelopes_advance_independently() {
        let mut fast = AdsrEnvelope::default();
        let mut slow = AdsrEnvelope::default();
        fast.trigger();
        slow.trigger();
        for _ in 0..1_000 {
            let _ = fast.next(48_000.0, 0.0, 0.0, 1.0, 0.0);
            let _ = slow.next(48_000.0, 0.5, 0.0, 1.0, 0.0);
        }
        assert!(fast.value > slow.value);
    }

    #[test]
    fn panel_time_mapping_spans_the_datasheet_range() {
        assert!((time_constant_seconds(0.0) - 0.002).abs() < 1.0e-6);
        assert!((time_constant_seconds(1.0) - 20.0).abs() < 1.0e-3);
        assert!((0.4..1.0).contains(&time_constant_seconds(0.6)));
    }

    #[test]
    fn attack_is_a_true_rc_curve() {
        let mut envelope = AdsrEnvelope::default();
        envelope.trigger();
        let first = envelope.next(1_000.0, 0.25, 0.0, 0.0, 0.0);
        let second = envelope.next(1_000.0, 0.25, 0.0, 0.0, 0.0);
        assert!(first > 0.0);
        assert!(second - first < first);
    }

    #[test]
    fn retrigger_preserves_the_external_capacitor_voltage() {
        let mut envelope = AdsrEnvelope::default();
        envelope.trigger();
        for _ in 0..100 {
            let _ = envelope.next(48_000.0, 0.2, 0.2, 0.3, 0.2);
        }
        let before = envelope.value();
        envelope.release();
        for _ in 0..100 {
            let _ = envelope.next(48_000.0, 0.2, 0.2, 0.3, 0.2);
        }
        let released = envelope.value();
        envelope.trigger();
        assert_eq!(envelope.value(), released);
        assert!(envelope.value() < before);
        assert!(envelope.value() > 0.0);
    }

    #[test]
    fn ten_device_population_stays_inside_published_limits() {
        for profile in ENVELOPE_PROFILES {
            assert!((6.1..=6.9).contains(&profile.attack_asymptote_volts));
            assert!((4.7..=5.3).contains(&profile.peak_volts));
            assert!((58.5..=61.5).contains(&profile.control_sensitivity_mv_per_decade));
            assert!((0.85..=1.15).contains(&profile.rc_time_ratio));
            assert!((-0.003..=0.023).contains(&profile.sustain_error_volts));
            let ratio = profile.attack_asymptote_volts / profile.peak_volts;
            assert!((1.26..=1.34).contains(&ratio));
        }
    }

    #[test]
    fn device_time_curves_remain_ordered_and_bounded() {
        for profile in ENVELOPE_PROFILES {
            let fast = profiled_time_constant_seconds(0.0, profile);
            let middle = profiled_time_constant_seconds(0.6, profile);
            let slow = profiled_time_constant_seconds(1.0, profile);
            assert!(fast < middle && middle < slow);
            assert!((0.0017..=0.0023).contains(&fast));
            assert!((0.35..=0.75).contains(&middle));
            assert!((14.0..=30.0).contains(&slow));
        }
    }

    #[test]
    fn paired_physical_devices_do_not_collapse_to_identical_curves() {
        for voice in 0..5 {
            let amplifier = ENVELOPE_PROFILES[voice * 2];
            let filter = ENVELOPE_PROFILES[voice * 2 + 1];
            assert_ne!(
                profiled_time_constant_seconds(0.6, amplifier),
                profiled_time_constant_seconds(0.6, filter)
            );
        }
    }
}
