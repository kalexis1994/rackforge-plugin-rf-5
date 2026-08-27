//! Ten-device CEM3310 envelope population.
//!
//! The source-backed parts are the true-RC attack/decay/release shape,
//! exponential time control, linear sustain control and populated timing
//! network. The service dial-six observation supplies one isolated absolute
//! control-span anchor. Published electrical limits define one filter and one
//! amplifier profile for every voice card.

const IDLE_THRESHOLD: f32 = 1.0e-5;
const NOMINAL_CONTROL_SENSITIVITY_MV_PER_DECADE: f32 = 60.0;
const NOMINAL_PEAK_VOLTS: f32 = 5.0;
const NOMINAL_ATTACK_ASYMPTOTE_VOLTS: f32 = 6.5;
const TIMING_RESISTOR_OHMS: f32 = 24_300.0;
const TIMING_CAPACITOR_FARADS: f32 = 0.039e-6;
const NOMINAL_BUFFER_OUTPUT_RESISTANCE_OHMS: f32 = 200.0;
const SERVICE_DIAL_SIX: f32 = 0.6;
const SERVICE_ATTACK_SECONDS: f32 = 1.0;

#[derive(Clone, Copy, Debug)]
struct EnvelopeProfile {
    attack_asymptote_volts: f32,
    peak_volts: f32,
    control_sensitivity_mv_per_decade: f32,
    component_rc_ratio: f32,
    attack_current_ratio: f32,
    discharge_current_ratio: f32,
    buffer_output_resistance_ohms: f32,
    sustain_error_volts: f32,
}

// Two devices per voice: amplifier first, filter second. The electrical
// endpoints come from the CEM3310 data sheet. RC ratios stay inside the stated
// +/-15% practical unit-to-unit tracking envelope and intentionally include
// the 24.3 kohm 1% / 0.039 uF 5% timing-component population on SD431. Charge
// and discharge ratios stay inside their separate published current bounds.
const ENVELOPE_PROFILES: [EnvelopeProfile; 10] = [
    EnvelopeProfile {
        attack_asymptote_volts: 6.20,
        peak_volts: 4.80,
        control_sensitivity_mv_per_decade: 58.8,
        component_rc_ratio: 0.90,
        attack_current_ratio: 0.92,
        discharge_current_ratio: 1.08,
        buffer_output_resistance_ohms: 175.0,
        sustain_error_volts: -0.003,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.80,
        peak_volts: 5.20,
        control_sensitivity_mv_per_decade: 61.1,
        component_rc_ratio: 1.10,
        attack_current_ratio: 1.08,
        discharge_current_ratio: 0.94,
        buffer_output_resistance_ohms: 225.0,
        sustain_error_volts: 0.017,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.40,
        peak_volts: 4.90,
        control_sensitivity_mv_per_decade: 59.4,
        component_rc_ratio: 0.96,
        attack_current_ratio: 0.97,
        discharge_current_ratio: 1.05,
        buffer_output_resistance_ohms: 190.0,
        sustain_error_volts: 0.005,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.60,
        peak_volts: 5.10,
        control_sensitivity_mv_per_decade: 60.7,
        component_rc_ratio: 1.06,
        attack_current_ratio: 1.04,
        discharge_current_ratio: 0.96,
        buffer_output_resistance_ohms: 215.0,
        sustain_error_volts: 0.014,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.50,
        peak_volts: 5.00,
        control_sensitivity_mv_per_decade: 60.0,
        component_rc_ratio: 1.00,
        attack_current_ratio: 1.00,
        discharge_current_ratio: 1.00,
        buffer_output_resistance_ohms: NOMINAL_BUFFER_OUTPUT_RESISTANCE_OHMS,
        sustain_error_volts: 0.0,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.10,
        peak_volts: 4.70,
        control_sensitivity_mv_per_decade: 58.5,
        component_rc_ratio: 0.87,
        attack_current_ratio: 0.88,
        discharge_current_ratio: 1.12,
        buffer_output_resistance_ohms: 155.0,
        sustain_error_volts: -0.003,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.90,
        peak_volts: 5.30,
        control_sensitivity_mv_per_decade: 61.5,
        component_rc_ratio: 1.13,
        attack_current_ratio: 1.14,
        discharge_current_ratio: 0.90,
        buffer_output_resistance_ohms: 245.0,
        sustain_error_volts: 0.023,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.30,
        peak_volts: 4.85,
        control_sensitivity_mv_per_decade: 59.0,
        component_rc_ratio: 0.93,
        attack_current_ratio: 0.95,
        discharge_current_ratio: 1.06,
        buffer_output_resistance_ohms: 180.0,
        sustain_error_volts: 0.004,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.70,
        peak_volts: 5.15,
        control_sensitivity_mv_per_decade: 61.0,
        component_rc_ratio: 1.08,
        attack_current_ratio: 1.10,
        discharge_current_ratio: 0.93,
        buffer_output_resistance_ohms: 235.0,
        sustain_error_volts: 0.019,
    },
    EnvelopeProfile {
        attack_asymptote_volts: 6.45,
        peak_volts: 5.05,
        control_sensitivity_mv_per_decade: 59.8,
        component_rc_ratio: 1.02,
        attack_current_ratio: 0.99,
        discharge_current_ratio: 1.03,
        buffer_output_resistance_ohms: 205.0,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CurrentDirection {
    Charge,
    Discharge,
}

#[derive(Clone, Copy, Debug)]
struct EnvelopeCoefficientCache {
    sample_rate: f32,
    control: f32,
    direction: CurrentDirection,
    approach_coefficient: f32,
    resistance_multiplier: f32,
}

impl Default for EnvelopeCoefficientCache {
    fn default() -> Self {
        Self {
            sample_rate: 0.0,
            control: 0.0,
            direction: CurrentDirection::Charge,
            approach_coefficient: 0.0,
            resistance_multiplier: 0.0,
        }
    }
}

impl EnvelopeCoefficientCache {
    fn coefficients(
        &mut self,
        sample_rate: f32,
        control: f32,
        profile: EnvelopeProfile,
        direction: CurrentDirection,
    ) -> (f32, f32) {
        let sample_rate = sample_rate.max(1.0);
        let control = control.clamp(0.0, 1.0);
        if self.sample_rate.to_bits() != sample_rate.to_bits()
            || self.control.to_bits() != control.to_bits()
            || self.direction != direction
        {
            let time_constant = profiled_time_constant_seconds(control, profile, direction);
            self.approach_coefficient = libm::expf(-1.0 / (time_constant * sample_rate));
            let current_ratio = match direction {
                CurrentDirection::Charge => profile.attack_current_ratio,
                CurrentDirection::Discharge => profile.discharge_current_ratio,
            };
            self.resistance_multiplier = libm::powf(
                10.0,
                control * control_span_millivolts() / profile.control_sensitivity_mv_per_decade,
            ) / current_ratio;
            self.sample_rate = sample_rate;
            self.control = control;
            self.direction = direction;
        }
        (self.approach_coefficient, self.resistance_multiplier)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AdsrEnvelope {
    value: f32,
    stage: Stage,
    profile_index: usize,
    coefficient_cache: EnvelopeCoefficientCache,
}

impl Default for AdsrEnvelope {
    fn default() -> Self {
        Self {
            value: 0.0,
            stage: Stage::Idle,
            profile_index: 4,
            coefficient_cache: EnvelopeCoefficientCache::default(),
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
                let (coefficient, _) = self.coefficient_cache.coefficients(
                    sample_rate,
                    attack,
                    profile,
                    CurrentDirection::Charge,
                );
                self.value = approach_with_coefficient(
                    self.value,
                    profile.attack_asymptote_volts / NOMINAL_PEAK_VOLTS,
                    coefficient,
                );
                if self.value >= peak {
                    self.value = peak;
                    self.stage = Stage::Decay;
                }
            }
            Stage::Decay => {
                let (coefficient, _) = self.coefficient_cache.coefficients(
                    sample_rate,
                    decay,
                    profile,
                    CurrentDirection::Discharge,
                );
                self.value = approach_with_coefficient(self.value, sustain_target, coefficient);
                if (self.value - sustain_target).abs() <= IDLE_THRESHOLD {
                    self.value = sustain_target;
                    self.stage = Stage::Sustain;
                }
            }
            Stage::Sustain => self.value = sustain_target,
            Stage::Release => {
                let (coefficient, _) = self.coefficient_cache.coefficients(
                    sample_rate,
                    release,
                    profile,
                    CurrentDirection::Discharge,
                );
                self.value = approach_with_coefficient(self.value, 0.0, coefficient);
                if self.value <= IDLE_THRESHOLD {
                    self.value = 0.0;
                    self.stage = Stage::Idle;
                }
            }
        }
        self.value
            + buffer_drive_offset_cached(
                &mut self.coefficient_cache,
                sample_rate,
                self.value,
                self.stage,
                attack,
                decay,
                sustain_target,
                release,
                profile,
            ) / NOMINAL_PEAK_VOLTS
    }

    pub fn is_idle(self) -> bool {
        self.stage == Stage::Idle
    }

    pub fn value(self) -> f32 {
        self.value
    }
}

#[cfg(test)]
fn buffer_drive_offset(
    capacitor_value: f32,
    stage: Stage,
    attack: f32,
    decay: f32,
    sustain_target: f32,
    release: f32,
    profile: EnvelopeProfile,
) -> f32 {
    let mut cache = EnvelopeCoefficientCache::default();
    buffer_drive_offset_cached(
        &mut cache,
        1.0,
        capacitor_value,
        stage,
        attack,
        decay,
        sustain_target,
        release,
        profile,
    )
}

#[allow(clippy::too_many_arguments)]
fn buffer_drive_offset_cached(
    coefficient_cache: &mut EnvelopeCoefficientCache,
    sample_rate: f32,
    capacitor_value: f32,
    stage: Stage,
    attack: f32,
    decay: f32,
    sustain_target: f32,
    release: f32,
    profile: EnvelopeProfile,
) -> f32 {
    // The CEM3310's internal buffer must source or sink the timing current
    // through Rx. Its finite output resistance therefore moves ENV OUT away
    // from the continuously stored capacitor voltage whenever a phase is
    // active. Reversing current at a phase boundary creates the small steps
    // shown in the original data-sheet waveforms; it does not reset Cx.
    let (target, control, direction) = match stage {
        Stage::Attack => (
            profile.attack_asymptote_volts / NOMINAL_PEAK_VOLTS,
            attack,
            CurrentDirection::Charge,
        ),
        Stage::Decay => (sustain_target, decay, CurrentDirection::Discharge),
        Stage::Release => (0.0, release, CurrentDirection::Discharge),
        Stage::Idle | Stage::Sustain => return 0.0,
    };
    let (_, resistance_multiplier) =
        coefficient_cache.coefficients(sample_rate, control, profile, direction);
    let drive_current_amps = (target - capacitor_value) * NOMINAL_PEAK_VOLTS
        / (TIMING_RESISTOR_OHMS * resistance_multiplier);
    drive_current_amps * profile.buffer_output_resistance_ohms
}

fn approach_with_coefficient(value: f32, target: f32, coefficient: f32) -> f32 {
    target + (value - target) * coefficient
}

pub fn time_constant_seconds(value: f32) -> f32 {
    populated_rc_seconds()
        * libm::powf(
            10.0,
            value.clamp(0.0, 1.0) * control_span_millivolts()
                / NOMINAL_CONTROL_SENSITIVITY_MV_PER_DECADE,
        )
}

fn profiled_time_constant_seconds(
    value: f32,
    profile: EnvelopeProfile,
    direction: CurrentDirection,
) -> f32 {
    let current_ratio = match direction {
        CurrentDirection::Charge => profile.attack_current_ratio,
        CurrentDirection::Discharge => profile.discharge_current_ratio,
    };
    populated_rc_seconds() * profile.component_rc_ratio / current_ratio
        * libm::powf(
            10.0,
            value.clamp(0.0, 1.0) * control_span_millivolts()
                / profile.control_sensitivity_mv_per_decade,
        )
}

fn populated_rc_seconds() -> f32 {
    TIMING_RESISTOR_OHMS * TIMING_CAPACITOR_FARADS
}

fn nominal_attack_threshold_time_constants() -> f32 {
    -libm::logf(1.0 - NOMINAL_PEAK_VOLTS / NOMINAL_ATTACK_ASYMPTOTE_VOLTS)
}

fn control_span_millivolts() -> f32 {
    let dial_six_time_constant = SERVICE_ATTACK_SECONDS / nominal_attack_threshold_time_constants();
    let dial_six_decades = libm::log10f(dial_six_time_constant / populated_rc_seconds());
    dial_six_decades / SERVICE_DIAL_SIX * NOMINAL_CONTROL_SENSITIVITY_MV_PER_DECADE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coefficient_cache_is_sample_exact_across_stage_changes() {
        let mut cached = AdsrEnvelope::with_profile(7);
        cached.trigger();
        for _ in 0..256 {
            let _ = cached.next(48_000.0, 0.18, 0.31, 0.42, 0.27);
        }
        let mut rebuilt = cached;
        for sample in 0..4_096 {
            if sample == 2_048 {
                cached.release();
                rebuilt.release();
            }
            rebuilt.coefficient_cache = EnvelopeCoefficientCache::default();
            let cached_output = cached.next(48_000.0, 0.18, 0.31, 0.42, 0.27);
            let rebuilt_output = rebuilt.next(48_000.0, 0.18, 0.31, 0.42, 0.27);
            assert_eq!(cached_output.to_bits(), rebuilt_output.to_bits());
            assert_eq!(cached.value.to_bits(), rebuilt.value.to_bits());
            assert_eq!(cached.stage, rebuilt.stage);
        }
    }

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
    fn populated_components_set_the_fastest_time_constant() {
        assert!((populated_rc_seconds() - 0.000_947_7).abs() < 1.0e-9);
        assert_eq!(time_constant_seconds(0.0), populated_rc_seconds());
    }

    #[test]
    fn service_dial_six_anchors_one_second_nominal_attack() {
        let attack_seconds =
            time_constant_seconds(SERVICE_DIAL_SIX) * nominal_attack_threshold_time_constants();
        assert!((attack_seconds - SERVICE_ATTACK_SECONDS).abs() < 1.0e-5);
    }

    #[test]
    fn control_span_respects_the_published_time_range() {
        let ratio = time_constant_seconds(1.0) / time_constant_seconds(0.0);
        assert!((50_000.0..=250_000.0).contains(&ratio));
        assert!((285.0..286.0).contains(&control_span_millivolts()));
        assert!(time_constant_seconds(1.0) > 50.0);
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
            assert!((0.85..=1.15).contains(&profile.component_rc_ratio));
            assert!((0.75..=1.30).contains(&profile.attack_current_ratio));
            assert!((0.83..=1.20).contains(&profile.discharge_current_ratio));
            assert!((100.0..=350.0).contains(&profile.buffer_output_resistance_ohms));
            assert!((-0.003..=0.023).contains(&profile.sustain_error_volts));
            let ratio = profile.attack_asymptote_volts / profile.peak_volts;
            assert!((1.26..=1.34).contains(&ratio));
        }
    }

    #[test]
    fn device_time_curves_remain_ordered_and_bounded() {
        for profile in ENVELOPE_PROFILES {
            for direction in [CurrentDirection::Charge, CurrentDirection::Discharge] {
                let fast = profiled_time_constant_seconds(0.0, profile, direction);
                let middle = profiled_time_constant_seconds(0.6, profile, direction);
                let slow = profiled_time_constant_seconds(1.0, profile, direction);
                assert!(fast < middle && middle < slow);
                assert!((0.0007..=0.0013).contains(&fast));
                assert!((0.5..=0.85).contains(&middle));
                assert!((40.0..=75.0).contains(&slow));
            }
        }
    }

    #[test]
    fn paired_physical_devices_do_not_collapse_to_identical_curves() {
        for voice in 0..5 {
            let amplifier = ENVELOPE_PROFILES[voice * 2];
            let filter = ENVELOPE_PROFILES[voice * 2 + 1];
            assert_ne!(
                profiled_time_constant_seconds(0.6, amplifier, CurrentDirection::Charge),
                profiled_time_constant_seconds(0.6, filter, CurrentDirection::Charge)
            );
        }
    }

    #[test]
    fn charge_and_discharge_currents_remain_distinct() {
        for profile in ENVELOPE_PROFILES {
            let attack = profiled_time_constant_seconds(0.6, profile, CurrentDirection::Charge);
            let decay = profiled_time_constant_seconds(0.6, profile, CurrentDirection::Discharge);
            if profile.attack_current_ratio != profile.discharge_current_ratio {
                assert_ne!(attack, decay);
            }
        }
    }

    #[test]
    fn nominal_buffer_produces_the_published_attack_step() {
        let profile = ENVELOPE_PROFILES[4];
        let offset = buffer_drive_offset(0.0, Stage::Attack, 0.0, 0.0, 0.0, 0.0, profile);
        let expected = NOMINAL_BUFFER_OUTPUT_RESISTANCE_OHMS / TIMING_RESISTOR_OHMS
            * NOMINAL_ATTACK_ASYMPTOTE_VOLTS;
        assert!((offset - expected).abs() < 1.0e-6);
        assert!((0.050..=0.055).contains(&offset));
    }

    #[test]
    fn buffer_step_reverses_with_the_phase_current() {
        let profile = ENVELOPE_PROFILES[4];
        let attack = buffer_drive_offset(0.4, Stage::Attack, 0.0, 0.0, 0.2, 0.0, profile);
        let decay = buffer_drive_offset(0.4, Stage::Decay, 0.0, 0.0, 0.2, 0.0, profile);
        let release = buffer_drive_offset(0.4, Stage::Release, 0.0, 0.0, 0.2, 0.0, profile);
        assert!(attack > 0.0);
        assert!(decay < 0.0);
        assert!(release < decay);
    }

    #[test]
    fn phase_steps_do_not_discontinuously_move_the_timing_capacitor() {
        let mut envelope = AdsrEnvelope {
            value: 0.72,
            stage: Stage::Decay,
            ..AdsrEnvelope::default()
        };
        let capacitor_before = envelope.value();
        envelope.release();
        assert_eq!(envelope.value(), capacitor_before);
        let output = envelope.next(48_000.0, 0.0, 0.0, 0.3, 0.0);
        assert!(envelope.value() < capacitor_before);
        assert!(output < envelope.value());
    }
}
