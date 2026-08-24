#![no_std]

use rf_5_contract::{PARAMETER_COUNT, Settings};
use rf_5_voice::Voice;

pub const VOICE_COUNT: usize = 5;
pub const STATE_BYTES: usize = PARAMETER_COUNT * 4;

pub struct Engine {
    settings: Settings,
    voices: [Voice; VOICE_COUNT],
    sample_rate: f32,
    next_voice: usize,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            voices: [Voice::default(); VOICE_COUNT],
            sample_rate: 48_000.0,
            next_voice: 0,
        }
    }
}

impl Engine {
    pub fn prepare(&mut self, sample_rate: f64) -> bool {
        if !sample_rate.is_finite() || sample_rate <= 0.0 {
            return false;
        }
        self.sample_rate = sample_rate as f32;
        self.reset_voices();
        true
    }

    pub fn reset_voices(&mut self) {
        self.voices = [Voice::default(); VOICE_COUNT];
        self.next_voice = 0;
    }

    pub fn settings(&self) -> Settings {
        self.settings
    }

    pub fn set_parameter(&mut self, index: u32, value: f64) -> bool {
        self.settings.set(index, value)
    }

    pub fn parameter(&self, index: u32) -> Option<f64> {
        self.settings.get_index(index).map(f64::from)
    }

    pub fn note_on(&mut self, channel: u8, note: u8, velocity: u8) {
        if velocity == 0 {
            self.note_off(channel, note);
            return;
        }
        let voice_index = self
            .voices
            .iter()
            .position(|voice| !voice.is_active())
            .unwrap_or_else(|| {
                let index = self.next_voice;
                self.next_voice = (self.next_voice + 1) % VOICE_COUNT;
                index
            });
        self.voices[voice_index].start(
            channel,
            note,
            velocity,
            voice_index,
            self.sample_rate,
            self.settings,
        );
    }

    pub fn note_off(&mut self, channel: u8, note: u8) {
        for voice in &mut self.voices {
            if voice.matches(channel, note) {
                voice.release(self.sample_rate, self.settings);
            }
        }
    }

    pub fn all_notes_off(&mut self) {
        for voice in &mut self.voices {
            if voice.is_active() {
                voice.release(self.sample_rate, self.settings);
            }
        }
    }

    pub fn handle_midi(&mut self, data: [u8; 3]) {
        let channel = data[0] & 0x0f;
        match data[0] & 0xf0 {
            0x90 => self.note_on(channel, data[1] & 0x7f, data[2] & 0x7f),
            0x80 => self.note_off(channel, data[1] & 0x7f),
            0xb0 if data[1] == 120 || data[1] == 123 => self.all_notes_off(),
            _ => {}
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let mut sample = 0.0;
        for voice in &mut self.voices {
            sample += voice.next(self.sample_rate, self.settings);
        }
        let level = self.settings.get(rf_5_contract::Parameter::MasterVolume);
        (sample * level * 0.16).clamp(-1.0, 1.0)
    }

    pub fn load_baseline_program(&mut self, id: &str) -> bool {
        let values = match id {
            "baseline-init" => [0.72, 0.5, 0.54, 0.72, 0.08, 0.01, 0.20, 0.82, 0.28, 0.18],
            "baseline-warm" => [0.76, 0.43, 0.58, 0.46, 0.12, 0.01, 0.28, 0.72, 0.34, 0.36],
            "baseline-pad" => [0.68, 0.50, 0.62, 0.38, 0.04, 0.54, 0.52, 0.78, 0.70, 0.42],
            "baseline-lead" => [0.64, 0.36, 0.72, 0.82, 0.18, 0.01, 0.12, 0.90, 0.24, 0.26],
            _ => return false,
        };
        self.settings = Settings::from_array(values).expect("baseline program values are valid");
        true
    }

    pub fn save_state(&self, destination: &mut [u8]) -> Option<usize> {
        let target = destination.get_mut(..STATE_BYTES)?;
        let (chunks, remainder) = target.as_chunks_mut::<4>();
        debug_assert!(remainder.is_empty());
        for (chunk, value) in chunks.iter_mut().zip(self.settings.as_array()) {
            chunk.copy_from_slice(&value.to_le_bytes());
        }
        Some(STATE_BYTES)
    }

    pub fn load_state(&mut self, state: &[u8]) -> bool {
        if state.len() != STATE_BYTES {
            return false;
        }
        let mut values = [0.0_f32; PARAMETER_COUNT];
        let (chunks, remainder) = state.as_chunks::<4>();
        if !remainder.is_empty() {
            return false;
        }
        for (value, chunk) in values.iter_mut().zip(chunks) {
            *value = f32::from_le_bytes(*chunk);
        }
        let Some(settings) = Settings::from_array(values) else {
            return false;
        };
        self.settings = settings;
        true
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    #[test]
    fn engine_is_silent_until_midi_arrives() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        assert!((0..128).all(|_| engine.next_sample() == 0.0));
        engine.note_on(0, 60, 100);
        assert!((0..4096).any(|_| engine.next_sample().abs() > 0.001));
    }

    #[test]
    fn a_sixth_note_steals_without_exceeding_five_voices() {
        let mut engine = Engine::default();
        for note in 60..66 {
            engine.note_on(0, note, 100);
        }
        assert_eq!(
            engine
                .voices
                .iter()
                .filter(|voice| voice.is_active())
                .count(),
            5
        );
    }

    #[test]
    fn state_and_programs_round_trip() {
        let mut engine = Engine::default();
        assert!(engine.load_baseline_program("baseline-pad"));
        let expected = engine.settings();
        let mut state = [0_u8; STATE_BYTES];
        assert_eq!(engine.save_state(&mut state), Some(STATE_BYTES));
        assert!(engine.load_baseline_program("baseline-lead"));
        assert!(engine.load_state(&state));
        assert_eq!(engine.settings(), expected);
    }
}
