//! Per-voice CEM3310 envelope candidate.
//!
//! The source-backed parts are the true-RC attack/decay/release shape,
//! exponential time control and linear sustain control. The panel-to-time
//! calibration remains bounded because no measurements from the target unit
//! are available.

const ATTACK_ASYMPTOTE: f32 = 1.3;
const MINIMUM_TIME_CONSTANT_SECONDS: f32 = 0.002;
const MAXIMUM_TIME_CONSTANT_SECONDS: f32 = 20.0;
const IDLE_THRESHOLD: f32 = 1.0e-5;

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
}

impl Default for AdsrEnvelope {
    fn default() -> Self {
        Self {
            value: 0.0,
            stage: Stage::Idle,
        }
    }
}

impl AdsrEnvelope {
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
        match self.stage {
            Stage::Idle => {}
            Stage::Attack => {
                self.value = approach(
                    self.value,
                    ATTACK_ASYMPTOTE,
                    time_constant_seconds(attack),
                    sample_rate,
                );
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.stage = Stage::Decay;
                }
            }
            Stage::Decay => {
                let sustain = sustain.clamp(0.0, 1.0);
                self.value = approach(
                    self.value,
                    sustain,
                    time_constant_seconds(decay),
                    sample_rate,
                );
                if (self.value - sustain).abs() <= IDLE_THRESHOLD {
                    self.value = sustain;
                    self.stage = Stage::Sustain;
                }
            }
            Stage::Sustain => self.value = sustain.clamp(0.0, 1.0),
            Stage::Release => {
                self.value = approach(self.value, 0.0, time_constant_seconds(release), sample_rate);
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
}
