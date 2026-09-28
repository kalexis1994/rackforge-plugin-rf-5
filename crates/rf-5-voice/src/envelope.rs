//! Ten-device CEM3310 envelope population.
//!
//! The source-backed parts are the true-RC attack/decay/release shape,
//! exponential time control, linear sustain control, the populated timing
//! network and the complete SD430 control path: V8.1's `0x7a - pot` time
//! complement, the 7-bit DAC step and the shared 24.3k/13k/806-ohm level
//! shifter in front of every time-control pin. Published electrical limits
//! define one filter and one amplifier profile for every voice card.

use rf_5_contract::hardware::{
    ENVELOPE_TIME_CONTROL_COMPLEMENT_CODE, analog_pot_code, general_control_volts,
    general_control_volts_per_code,
};

const IDLE_THRESHOLD: f32 = 1.0e-5;
#[cfg(test)]
const NOMINAL_CONTROL_SENSITIVITY_MV_PER_DECADE: f32 = 60.0;
const NOMINAL_PEAK_VOLTS: f32 = 5.0;
#[cfg(test)]
const NOMINAL_ATTACK_ASYMPTOTE_VOLTS: f32 = 6.5;
const TIMING_RESISTOR_OHMS: f32 = 24_300.0;
const TIMING_CAPACITOR_FARADS: f32 = 0.039e-6;
const NOMINAL_BUFFER_OUTPUT_RESISTANCE_OHMS: f32 = 200.0;
// SD430 R415/R413/R414 (amplifier) and R410/R412/R411 (filter) feed each
// held time CV into one node shared by all five voice cards; R407-class 13k
// resistors pull that node toward the -5 V rail and R402-class 806-ohm
// resistors return it to ground. The node drives the CEM3310 Va/Vd/Vr pins.
const TIME_CONTROL_INPUT_OHMS: f32 = 24_300.0;
const TIME_CONTROL_BIAS_OHMS: f32 = 13_000.0;
const TIME_CONTROL_SHUNT_OHMS: f32 = 806.0;
const TIME_CONTROL_BIAS_RAIL_VOLTS: f32 = -5.0;
// R405/R406 (amplifier) and R426/R433 (filter) halve the held sustain CV.
const SUSTAIN_CONTROL_DIVIDER: f32 = 4_750.0 / (4_750.0 + 4_750.0);
// The data sheet's 60 mV/decade is kT/q at 25 C and carries its published
// +3300 ppm/C temperature coefficient. Inside the closed instrument the die
// runs warm; 45 C places the nominal full-scale attack at the owner's-manual
// "approximately 30 seconds" while keeping the service manual's longer-than-
// 20-second release and factory 1-4's few-millisecond onset.
const DATASHEET_REFERENCE_KELVIN: f32 = 298.15;
const OPERATING_DIE_KELVIN: f32 = 318.15;

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
            self.resistance_multiplier = control_multiplier(control, profile, direction);
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
        // The sustain pin is followed linearly. A held level above the peak
        // threshold is reached after the peak, so it is not clipped to it.
        let sustain_target = (sustain_control_normalized(sustain)
            + profile.sustain_error_volts / NOMINAL_PEAK_VOLTS)
            .clamp(0.0, profile.attack_asymptote_volts / NOMINAL_PEAK_VOLTS);
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
                let coefficient = if sustain_target > self.value {
                    // Data sheet: a sustain voltage above the peak threshold
                    // is approached at the fastest attack rate.
                    libm::expf(
                        -1.0 / (populated_rc_seconds() * profile.component_rc_ratio
                            / profile.attack_current_ratio
                            * sample_rate.max(1.0)),
                    )
                } else {
                    self.coefficient_cache
                        .coefficients(sample_rate, decay, profile, CurrentDirection::Discharge)
                        .0
                };
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

#[cfg(test)]
fn time_constant_seconds(value: f32) -> f32 {
    populated_rc_seconds() * nominal_control_multiplier(value)
}

fn profiled_time_constant_seconds(
    value: f32,
    profile: EnvelopeProfile,
    direction: CurrentDirection,
) -> f32 {
    populated_rc_seconds()
        * profile.component_rc_ratio
        * control_multiplier(value, profile, direction)
}

/// Ratio of the CEM3310 timing current at `Vc = 0` to the current at the
/// held control, i.e. the data sheet's `exp(-Vc/VT)` time multiplier divided
/// by the device's charge or discharge current ratio.
fn control_multiplier(value: f32, profile: EnvelopeProfile, direction: CurrentDirection) -> f32 {
    let current_ratio = match direction {
        CurrentDirection::Charge => profile.attack_current_ratio,
        CurrentDirection::Discharge => profile.discharge_current_ratio,
    };
    time_exponential(
        time_control_pin_volts(value),
        profile.control_sensitivity_mv_per_decade,
    ) / current_ratio
}

#[cfg(test)]
fn nominal_control_multiplier(value: f32) -> f32 {
    time_exponential(
        time_control_pin_volts(value),
        NOMINAL_CONTROL_SENSITIVITY_MV_PER_DECADE,
    )
}

fn time_exponential(pin_volts: f32, reference_mv_per_decade: f32) -> f32 {
    let operating_mv_per_decade =
        reference_mv_per_decade * OPERATING_DIE_KELVIN / DATASHEET_REFERENCE_KELVIN;
    libm::powf(10.0, -pin_volts * 1_000.0 / operating_mv_per_decade)
}

fn populated_rc_seconds() -> f32 {
    TIMING_RESISTOR_OHMS * TIMING_CAPACITOR_FARADS
}

/// Held S/H voltage for an envelope time pot. V8.1 writes `0x7a - pot`, so a
/// short time is a high voltage; the eight-bit subtraction cannot go below
/// zero on the populated panel range.
fn time_control_dac_volts(value: f32) -> f32 {
    let written = ENVELOPE_TIME_CONTROL_COMPLEMENT_CODE.saturating_sub(analog_pot_code(value));
    f32::from(written) * general_control_volts_per_code()
}

/// Thevenin solution of the shared SD430 node: held CV through 24.3k, 13k to
/// -5 V and 806 ohm to ground. Roughly -283 mV at the longest time and
/// +25 mV at the shortest, i.e. about five decades of CEM3310 time.
fn time_control_pin_volts(value: f32) -> f32 {
    let input_volts = time_control_dac_volts(value);
    let conductance = 1.0 / TIME_CONTROL_INPUT_OHMS
        + 1.0 / TIME_CONTROL_BIAS_OHMS
        + 1.0 / TIME_CONTROL_SHUNT_OHMS;
    (input_volts / TIME_CONTROL_INPUT_OHMS + TIME_CONTROL_BIAS_RAIL_VOLTS / TIME_CONTROL_BIAS_OHMS)
        / conductance
}

/// CEM3310 sustain-pin voltage relative to the nominal 5 V peak. The sustain
/// pot is written uncomplemented and halved by the 4.75k/4.75k divider, so
/// the V8.1 10 V software ceiling (code 120) is exactly the 5 V peak.
fn sustain_control_normalized(value: f32) -> f32 {
    general_control_volts(value) * SUSTAIN_CONTROL_DIVIDER / NOMINAL_PEAK_VOLTS
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
        let sustain = sustain_control_normalized(0.4);
        let mut peak = 0.0_f32;
        for _ in 0..500_000 {
            peak = peak.max(envelope.next(48_000.0, 0.01, 0.05, 0.4, 0.01));
            if (envelope.value - sustain).abs() < 1.0e-5 {
                break;
            }
        }
        assert!(peak > 0.99);
        assert!((envelope.value - sustain).abs() < 1.0e-5);
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

    fn code(code: u8) -> f32 {
        f32::from(code) / 127.0
    }

    fn attack_to_peak_seconds(control: f32) -> f32 {
        time_constant_seconds(control)
            * -libm::logf(1.0 - NOMINAL_PEAK_VOLTS / NOMINAL_ATTACK_ASYMPTOTE_VOLTS)
    }

    #[test]
    fn populated_components_set_the_rc_time_constant() {
        assert!((populated_rc_seconds() - 0.000_947_7).abs() < 1.0e-9);
    }

    #[test]
    fn sd430_network_sets_the_time_control_pin_span() {
        // Firmware code 0x7a (pot 0) through 24.3k against 13k/-5 V and 806R.
        let fastest = time_control_pin_volts(0.0);
        let panel_ceiling = time_control_pin_volts(code(120));
        let slowest = time_control_pin_volts(1.0);
        assert!((0.024..0.026).contains(&fastest), "{fastest}");
        assert!(
            (-0.2795..-0.2780).contains(&panel_ceiling),
            "{panel_ceiling}"
        );
        assert!((-0.2835..-0.2826).contains(&slowest), "{slowest}");
        // The node is zero volts, i.e. exactly Rx*Cx, near pot code 10.
        assert!(time_control_pin_volts(code(9)) > 0.0);
        assert!(time_control_pin_volts(code(11)) < 0.0);
    }

    #[test]
    fn time_is_exactly_exponential_in_the_stored_code() {
        // One DAC step moves the pin by the same voltage everywhere, so each
        // code multiplies the time constant by the same factor.
        let step = time_constant_seconds(code(1)) / time_constant_seconds(code(0));
        for stored in 1..=121 {
            let ratio =
                time_constant_seconds(code(stored + 1)) / time_constant_seconds(code(stored));
            assert!((ratio / step - 1.0).abs() < 1.0e-3, "{stored}");
        }
        assert!((1.09..1.10).contains(&step));
        // V8.1's eight-bit complement cannot go below zero.
        assert_eq!(time_constant_seconds(code(122)), time_constant_seconds(1.0));
    }

    #[test]
    fn owner_and_service_endpoints_follow_from_the_circuit() {
        // Owner's manual: approximately 1 ms to 30 s over the panel range.
        let fastest_attack = attack_to_peak_seconds(0.0);
        let slowest_attack = attack_to_peak_seconds(code(120));
        assert!(
            (0.0004..=0.0012).contains(&fastest_attack),
            "{fastest_attack}"
        );
        assert!((28.0..=33.0).contains(&slowest_attack), "{slowest_attack}");
        // Service test 4-6: amplifier release at 10 is longer than 20 s.
        let slowest_release = time_constant_seconds(code(120)) * core::f32::consts::LN_10;
        assert!(slowest_release > 20.0, "{slowest_release}");
    }

    #[test]
    fn factory_e_piano_onset_and_release_off_stay_fast() {
        // Program 1-4's Attack code 30 reaches half of the final-VCA current
        // (about 2.8 V above Q410's ~0.56 V knee) in the 3-5 ms measured on
        // the Rev 3 recording.
        let half_current_volts = 0.56 + (NOMINAL_PEAK_VOLTS - 0.56) / 2.0;
        let half_level_seconds = time_constant_seconds(code(30))
            * libm::logf(
                NOMINAL_ATTACK_ASYMPTOTE_VOLTS
                    / (NOMINAL_ATTACK_ASYMPTOTE_VOLTS - half_current_volts),
            );
        assert!(
            (0.003..=0.005).contains(&half_level_seconds),
            "{half_level_seconds}"
        );
        // The fixed 0x64 RELEASE-off write is short but not the minimum.
        let release_off = time_constant_seconds(code(22));
        assert!((0.002..=0.004).contains(&release_off), "{release_off}");
        assert!(release_off > time_constant_seconds(0.0));
    }

    #[test]
    fn sustain_code_120_is_the_full_peak() {
        assert!((sustain_control_normalized(code(120)) - 1.0).abs() < 2.0e-3);
        assert!((sustain_control_normalized(code(60)) - 0.5).abs() < 2.0e-3);
        assert_eq!(sustain_control_normalized(0.0), 0.0);
        assert!(sustain_control_normalized(1.0) > 1.0);
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
                assert!((0.000_2..=0.000_6).contains(&fast), "{fast}");
                assert!((0.2..=0.7).contains(&middle), "{middle}");
                assert!((12.0..=45.0).contains(&slow), "{slow}");
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
        // The data sheet's (Ro/Rx)*Vz step applies at Vc = 0, which the SD430
        // node reaches near stored Attack code 10.
        let zero_volt_control = 10.0 / 127.0;
        let offset = buffer_drive_offset(
            0.0,
            Stage::Attack,
            zero_volt_control,
            0.0,
            0.0,
            0.0,
            profile,
        );
        let expected = NOMINAL_BUFFER_OUTPUT_RESISTANCE_OHMS / TIMING_RESISTOR_OHMS
            * NOMINAL_ATTACK_ASYMPTOTE_VOLTS;
        assert!((offset - expected).abs() < 1.0e-3);
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

    #[test]
    fn slower_release_can_extend_a_zero_sustain_percussive_tail() {
        // Original program 2-4 stores amplifier D=75, S=0 and R=89. An
        // early key-up therefore changes the CEM3310 from the faster decay
        // control to the slower release control; it must not be mistaken for
        // a rising envelope when the detuned oscillators beat in the tail.
        const DECAY: f32 = 75.0 / 127.0;
        const RELEASE: f32 = 89.0 / 127.0;
        let mut held = AdsrEnvelope::with_profile(0);
        held.trigger();
        for _ in 0..2_400 {
            let _ = held.next(48_000.0, 7.0 / 127.0, DECAY, 0.0, RELEASE);
        }
        let mut released = held;
        released.release();
        let mut previous_release_value = released.value();
        for _ in 0..48_000 {
            let _ = held.next(48_000.0, 7.0 / 127.0, DECAY, 0.0, RELEASE);
            let _ = released.next(48_000.0, 7.0 / 127.0, DECAY, 0.0, RELEASE);
            assert!(released.value() <= previous_release_value);
            previous_release_value = released.value();
        }
        assert!(released.value() > held.value());
    }

    #[test]
    fn firmware_release_off_code_remains_a_short_click_free_tail() {
        const RELEASE_OFF_CODE: f32 = 22.0 / 127.0;
        let mut envelope = AdsrEnvelope::default();
        envelope.trigger();
        for _ in 0..48_000 {
            let _ = envelope.next(48_000.0, 0.0, 0.0, 1.0, RELEASE_OFF_CODE);
        }
        envelope.release();
        let first_release_sample = envelope.next(48_000.0, 0.0, 0.0, 1.0, RELEASE_OFF_CODE);
        assert!(first_release_sample > 0.0);
        for _ in 0..48_000 {
            let _ = envelope.next(48_000.0, 0.0, 0.0, 1.0, RELEASE_OFF_CODE);
            if envelope.is_idle() {
                break;
            }
        }
        assert!(envelope.is_idle());
    }
}
