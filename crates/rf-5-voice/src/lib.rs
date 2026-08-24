#![no_std]

//! Milestone 0 per-voice baseline. The oscillator, envelope and filter are
//! deliberately simple and are tracked as replacement gaps in the fidelity
//! matrix. They exist to exercise allocation, automation and packaging.

use rf_5_contract::{Parameter, Settings};

const SEMITONE_RATIO: f32 = 1.059_463_1;
const VOICE_SPREAD: [f32; 5] = [-0.0030, -0.0014, 0.0, 0.0016, 0.0031];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Copy, Debug)]
pub struct Voice {
    note: u8,
    channel: u8,
    velocity: f32,
    active: bool,
    phase_a: f32,
    phase_b: f32,
    increment_a: f32,
    increment_b: f32,
    envelope: f32,
    release_step: f32,
    stage: EnvelopeStage,
    filter: f32,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            note: 0,
            channel: 0,
            velocity: 0.0,
            active: false,
            phase_a: 0.0,
            phase_b: 0.0,
            increment_a: 0.0,
            increment_b: 0.0,
            envelope: 0.0,
            release_step: 0.0,
            stage: EnvelopeStage::Idle,
            filter: 0.0,
        }
    }
}

impl Voice {
    pub fn is_active(self) -> bool {
        self.active
    }

    pub fn matches(self, channel: u8, note: u8) -> bool {
        self.active && self.channel == channel && self.note == note
    }

    pub fn start(
        &mut self,
        channel: u8,
        note: u8,
        velocity: u8,
        voice_index: usize,
        sample_rate: f32,
        settings: Settings,
    ) {
        let base = note_frequency(note);
        let spread =
            VOICE_SPREAD[voice_index % VOICE_SPREAD.len()] * settings.get(Parameter::VintageSpread);
        let oscillator_b_detune = (settings.get(Parameter::OscillatorBDetune) - 0.5) * 0.04;
        self.note = note;
        self.channel = channel;
        self.velocity = f32::from(velocity) / 127.0;
        self.active = true;
        self.phase_a = 0.0;
        self.phase_b = 0.0;
        self.increment_a = base * (1.0 + spread) / sample_rate.max(1.0);
        self.increment_b = base * (1.0 - spread + oscillator_b_detune) / sample_rate.max(1.0);
        self.envelope = 0.0;
        self.release_step = 0.0;
        self.stage = EnvelopeStage::Attack;
        self.filter = 0.0;
    }

    pub fn release(&mut self, sample_rate: f32, settings: Settings) {
        if !self.active || self.stage == EnvelopeStage::Release {
            return;
        }
        let seconds = envelope_seconds(settings.get(Parameter::AmpRelease), 0.015, 8.0);
        self.release_step = self.envelope / (seconds * sample_rate.max(1.0)).max(1.0);
        self.stage = EnvelopeStage::Release;
    }

    pub fn next(&mut self, sample_rate: f32, settings: Settings) -> f32 {
        if !self.active {
            return 0.0;
        }

        self.advance_envelope(sample_rate, settings);
        self.phase_a = wrap_phase(self.phase_a + self.increment_a);
        self.phase_b = wrap_phase(self.phase_b + self.increment_b);
        let saw_a = self.phase_a * 2.0 - 1.0;
        let saw_b = self.phase_b * 2.0 - 1.0;
        let mix = settings.get(Parameter::OscillatorMix);
        let raw = saw_a * (1.0 - mix) + saw_b * mix;

        let cutoff = settings.get(Parameter::FilterCutoff);
        let sample_rate_scale = 48_000.0 / sample_rate.max(1.0);
        let coefficient = ((0.004 + cutoff * cutoff * 0.42) * sample_rate_scale).clamp(0.001, 0.95);
        let feedback = settings.get(Parameter::FilterResonance) * 0.82;
        self.filter += coefficient * ((raw - self.filter * feedback) - self.filter);
        self.filter * self.envelope * self.velocity
    }

    fn advance_envelope(&mut self, sample_rate: f32, settings: Settings) {
        match self.stage {
            EnvelopeStage::Idle => {}
            EnvelopeStage::Attack => {
                let seconds = envelope_seconds(settings.get(Parameter::AmpAttack), 0.001, 5.0);
                self.envelope += 1.0 / (seconds * sample_rate.max(1.0)).max(1.0);
                if self.envelope >= 1.0 {
                    self.envelope = 1.0;
                    self.stage = EnvelopeStage::Decay;
                }
            }
            EnvelopeStage::Decay => {
                let sustain = settings.get(Parameter::AmpSustain);
                let seconds = envelope_seconds(settings.get(Parameter::AmpDecay), 0.004, 8.0);
                self.envelope -= (1.0 - sustain) / (seconds * sample_rate.max(1.0)).max(1.0);
                if self.envelope <= sustain {
                    self.envelope = sustain;
                    self.stage = EnvelopeStage::Sustain;
                }
            }
            EnvelopeStage::Sustain => {
                self.envelope = settings.get(Parameter::AmpSustain);
            }
            EnvelopeStage::Release => {
                self.envelope -= self.release_step;
                if self.envelope <= 0.0 {
                    self.envelope = 0.0;
                    self.active = false;
                    self.stage = EnvelopeStage::Idle;
                }
            }
        }
    }
}

fn envelope_seconds(value: f32, minimum: f32, span: f32) -> f32 {
    minimum + value * value * span
}

fn wrap_phase(phase: f32) -> f32 {
    if phase >= 1.0 { phase - 1.0 } else { phase }
}

pub fn note_frequency(note: u8) -> f32 {
    let mut frequency = 440.0;
    let mut distance = i32::from(note) - 69;
    while distance > 0 {
        frequency *= SEMITONE_RATIO;
        distance -= 1;
    }
    while distance < 0 {
        frequency /= SEMITONE_RATIO;
        distance += 1;
    }
    frequency
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_temperament_reference_notes_are_stable() {
        assert!((note_frequency(69) - 440.0).abs() < 0.01);
        assert!((note_frequency(57) - 220.0).abs() < 0.2);
        assert!((note_frequency(81) - 880.0).abs() < 0.5);
    }

    #[test]
    fn note_starts_sounds_and_releases() {
        let settings = Settings::default();
        let mut voice = Voice::default();
        voice.start(0, 69, 127, 0, 48_000.0, settings);
        assert!((0..4096).any(|_| voice.next(48_000.0, settings).abs() > 0.001));
        voice.release(48_000.0, settings);
        for _ in 0..500_000 {
            let _ = voice.next(48_000.0, settings);
            if !voice.is_active() {
                break;
            }
        }
        assert!(!voice.is_active());
    }
}
