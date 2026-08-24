#![no_std]

//! Per-voice synthesis path. The VCO A/B core is source-backed and
//! band-limited; envelope, filter and gain staging remain replacement gaps.

use rf_5_contract::{Parameter, Settings, hardware::quantize_analog_pot};

pub mod tuning;
pub mod vco;

pub use tuning::note_frequency;
use vco::{Vco, WaveSelection};

const VOICE_SPREAD: [f32; 5] = [-0.0030, -0.0014, 0.0, 0.0016, 0.0031];
const INITIAL_PHASE_A: [f32; 5] = [0.07, 0.31, 0.58, 0.83, 0.19];
const INITIAL_PHASE_B: [f32; 5] = [0.67, 0.11, 0.42, 0.74, 0.93];
const OSCILLATOR_OVERSAMPLING: usize = 4;

#[derive(Clone, Copy, Debug, Default)]
pub struct VoiceModulation {
    pub oscillator_a_semitones: f32,
    pub oscillator_b_semitones: f32,
    pub oscillator_a_pulse_width: f32,
    pub oscillator_b_pulse_width: f32,
    pub filter_cutoff: f32,
    pub noise: f32,
}

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
    oscillator_a: Vco,
    oscillator_b: Vco,
    oscillators_initialized: bool,
    voice_index: usize,
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
            oscillator_a: Vco::default(),
            oscillator_b: Vco::default(),
            oscillators_initialized: false,
            voice_index: 0,
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

    pub fn start(&mut self, channel: u8, note: u8, velocity: u8, voice_index: usize) {
        self.note = note;
        self.channel = channel;
        self.voice_index = voice_index % VOICE_SPREAD.len();
        self.velocity = f32::from(velocity) / 127.0;
        self.active = true;
        if !self.oscillators_initialized {
            let index = self.voice_index;
            self.oscillator_a = Vco::with_phase(INITIAL_PHASE_A[index]);
            self.oscillator_b = Vco::with_phase(INITIAL_PHASE_B[index]);
            self.oscillators_initialized = true;
        }
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

    pub fn next(
        &mut self,
        sample_rate: f32,
        settings: Settings,
        modulation: VoiceModulation,
    ) -> f32 {
        let raw = self.next_oscillators(sample_rate, settings, modulation) + modulation.noise;
        if !self.active {
            return 0.0;
        }

        self.advance_envelope(sample_rate, settings);

        let cutoff =
            (settings.get(Parameter::FilterCutoff) + modulation.filter_cutoff).clamp(0.0, 1.0);
        let sample_rate_scale = 48_000.0 / sample_rate.max(1.0);
        let coefficient = ((0.004 + cutoff * cutoff * 0.42) * sample_rate_scale).clamp(0.001, 0.95);
        let feedback = settings.get(Parameter::FilterResonance) * 0.82;
        self.filter += coefficient * ((raw - self.filter * feedback) - self.filter);
        self.filter * self.envelope * self.velocity
    }

    fn next_oscillators(
        &mut self,
        sample_rate: f32,
        settings: Settings,
        modulation: VoiceModulation,
    ) -> f32 {
        let waves_a = WaveSelection {
            saw: parameter_enabled(settings, Parameter::OscillatorASaw),
            triangle: false,
            pulse: parameter_enabled(settings, Parameter::OscillatorAPulse),
        };
        let waves_b = WaveSelection {
            saw: parameter_enabled(settings, Parameter::OscillatorBSaw),
            triangle: parameter_enabled(settings, Parameter::OscillatorBTriangle),
            pulse: parameter_enabled(settings, Parameter::OscillatorBPulse),
        };
        let pulse_width_a = (quantize_analog_pot(settings.get(Parameter::OscillatorAPulseWidth))
            + modulation.oscillator_a_pulse_width)
            .clamp(0.02, 0.98);
        let pulse_width_b = (quantize_analog_pot(settings.get(Parameter::OscillatorBPulseWidth))
            + modulation.oscillator_b_pulse_width)
            .clamp(0.02, 0.98);
        let level_a = quantize_analog_pot(settings.get(Parameter::OscillatorALevel));
        let level_b = quantize_analog_pot(settings.get(Parameter::OscillatorBLevel));
        let sync = parameter_enabled(settings, Parameter::OscillatorSync);
        let spread = VOICE_SPREAD[self.voice_index] * settings.get(Parameter::VintageSpread);
        let frequency_a = tuning::oscillator_a_frequency(
            self.note,
            settings.get(Parameter::OscillatorAFrequency),
        ) * semitone_ratio(modulation.oscillator_a_semitones)
            * (1.0 + spread);
        let frequency_b = tuning::oscillator_b_frequency(
            self.note,
            settings.get(Parameter::OscillatorBFrequency),
            settings.get(Parameter::OscillatorBDetune),
            parameter_enabled(settings, Parameter::OscillatorBKeyboard),
            parameter_enabled(settings, Parameter::OscillatorBLowFrequency),
        ) * semitone_ratio(modulation.oscillator_b_semitones)
            * (1.0 - spread);
        let internal_rate = sample_rate.max(1.0) * OSCILLATOR_OVERSAMPLING as f32;
        let mut output = 0.0;

        for _ in 0..OSCILLATOR_OVERSAMPLING {
            let sample_a =
                self.oscillator_a
                    .next(frequency_a, internal_rate, pulse_width_a, waves_a);
            let sample_b =
                self.oscillator_b
                    .next(frequency_b, internal_rate, pulse_width_b, waves_b);
            output += sample_a.value * level_a + sample_b.value * level_b;
            if sync && sample_b.wrapped {
                self.oscillator_a.hard_sync();
            }
        }

        output / OSCILLATOR_OVERSAMPLING as f32
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

fn parameter_enabled(settings: Settings, parameter: Parameter) -> bool {
    settings.get(parameter) >= 0.5
}

fn semitone_ratio(semitones: f32) -> f32 {
    libm::powf(2.0, semitones / 12.0)
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
        voice.start(0, 69, 127, 0);
        assert!((0..4096).any(|_| {
            voice
                .next(48_000.0, settings, VoiceModulation::default())
                .abs()
                > 0.001
        }));
        voice.release(48_000.0, settings);
        for _ in 0..500_000 {
            let _ = voice.next(48_000.0, settings, VoiceModulation::default());
            if !voice.is_active() {
                break;
            }
        }
        assert!(!voice.is_active());
    }

    #[test]
    fn note_retrigger_does_not_reset_free_running_oscillators() {
        let settings = Settings::default();
        let mut voice = Voice::default();
        voice.start(0, 60, 100, 2);
        for _ in 0..137 {
            let _ = voice.next(48_000.0, settings, VoiceModulation::default());
        }
        let phase_a = voice.oscillator_a.phase();
        let phase_b = voice.oscillator_b.phase();
        voice.start(0, 67, 100, 2);
        assert_eq!(voice.oscillator_a.phase(), phase_a);
        assert_eq!(voice.oscillator_b.phase(), phase_b);
    }

    #[test]
    fn inactive_voice_oscillators_continue_running() {
        let settings = Settings::default();
        let mut voice = Voice::default();
        voice.start(0, 60, 100, 1);
        voice.active = false;
        let phase_before = voice.oscillator_a.phase();
        for _ in 0..64 {
            assert_eq!(
                voice.next(48_000.0, settings, VoiceModulation::default()),
                0.0
            );
        }
        assert_ne!(voice.oscillator_a.phase(), phase_before);
    }

    #[test]
    fn hard_sync_changes_the_render_when_b_is_detuned() {
        let mut free_settings = Settings::default();
        assert!(free_settings.set(Parameter::OscillatorBDetune as u32, 0.9));
        let mut sync_settings = free_settings;
        assert!(sync_settings.set(Parameter::OscillatorSync as u32, 1.0));
        let mut free_voice = Voice::default();
        let mut sync_voice = Voice::default();
        free_voice.start(0, 57, 127, 0);
        sync_voice.start(0, 57, 127, 0);
        let mut difference = 0.0;
        for _ in 0..8_192 {
            difference += (free_voice.next(48_000.0, free_settings, VoiceModulation::default())
                - sync_voice.next(48_000.0, sync_settings, VoiceModulation::default()))
            .abs();
        }
        assert!(difference > 1.0);
    }
}
