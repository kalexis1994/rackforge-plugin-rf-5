#![no_std]

mod allocation;
mod control;
mod cv;
mod lfo;
mod noise;
mod output;
mod programs;
mod wheel_mod;

use allocation::PolyAllocator;
use lfo::{Lfo, LfoWaveSelection};
use noise::PinkNoise;
use rf_5_contract::{PARAMETER_COUNT, Parameter, Settings, hardware::quantize_analog_pot};
use rf_5_voice::{
    Voice, VoiceModulation,
    autotune::{AutoTune, Oscillator},
    drift::VcoDriftBank,
    tuning, vca,
};

pub const VOICE_COUNT: usize = 5;
pub const STATE_BYTES: usize = PARAMETER_COUNT * 4;
const PITCH_WHEEL_RANGE_SEMITONES: f32 = 7.0;
const MAXIMUM_GLIDE_RATE_SEMITONES_PER_SECOND: f32 = 2_400.0;
const MINIMUM_GLIDE_RATE_SEMITONES_PER_SECOND: f32 = 12.0;

#[derive(Clone, Copy, Debug, Default)]
struct HeldNote {
    channel: u8,
    note: u8,
    velocity: u8,
}

#[derive(Clone, Copy, Debug)]
struct HeldNoteStack {
    entries: [HeldNote; 128],
    len: usize,
}

impl Default for HeldNoteStack {
    fn default() -> Self {
        Self {
            entries: [HeldNote::default(); 128],
            len: 0,
        }
    }
}

impl HeldNoteStack {
    fn push(&mut self, channel: u8, note: u8, velocity: u8) {
        self.remove(channel, note);
        if self.len < self.entries.len() {
            self.entries[self.len] = HeldNote {
                channel,
                note,
                velocity,
            };
            self.len += 1;
        }
    }

    fn remove(&mut self, channel: u8, note: u8) -> bool {
        let Some(index) = self.entries[..self.len]
            .iter()
            .position(|held| held.channel == channel && held.note == note)
        else {
            return false;
        };
        self.entries.copy_within(index + 1..self.len, index);
        self.len -= 1;
        true
    }

    fn lowest(self) -> Option<HeldNote> {
        self.entries[..self.len]
            .iter()
            .copied()
            .min_by_key(|held| held.note)
    }

    fn contains(self, channel: u8, note: u8) -> bool {
        self.entries[..self.len]
            .iter()
            .any(|held| held.channel == channel && held.note == note)
    }

    fn clear(&mut self) {
        self.len = 0;
    }
}

pub struct Engine {
    settings: Settings,
    voices: [Voice; VOICE_COUNT],
    sample_rate: f32,
    poly_allocator: PolyAllocator,
    lfo: Lfo,
    noise: PinkNoise,
    mod_wheel: f32,
    audition_mod_wheel: Option<f32>,
    pitch_wheel: f32,
    sustain_pedal: bool,
    held_notes: HeldNoteStack,
    glide_current_note: f32,
    glide_target_note: f32,
    glide_initialized: bool,
    controls: control::ControlScheduler,
    autotune: AutoTune,
    vco_drift: VcoDriftBank,
    cv: cv::CvDistributor,
}

impl Default for Engine {
    fn default() -> Self {
        let settings = Settings::default();
        let autotune = AutoTune::default();
        let mut cv = cv::CvDistributor::default();
        cv.prepare(cv::CvTargets::from_state(
            settings,
            [0; VOICE_COUNT],
            autotune,
        ));
        Self {
            settings,
            voices: [Voice::default(); VOICE_COUNT],
            sample_rate: 48_000.0,
            poly_allocator: PolyAllocator::default(),
            lfo: Lfo::default(),
            noise: PinkNoise::default(),
            mod_wheel: 0.0,
            audition_mod_wheel: None,
            pitch_wheel: 0.0,
            sustain_pedal: false,
            held_notes: HeldNoteStack::default(),
            glide_current_note: 0.0,
            glide_target_note: 0.0,
            glide_initialized: false,
            controls: control::ControlScheduler::default(),
            autotune,
            vco_drift: VcoDriftBank::default(),
            cv,
        }
    }
}

impl Engine {
    pub fn prepare(&mut self, sample_rate: f64) -> bool {
        if !sample_rate.is_finite() || sample_rate <= 0.0 {
            return false;
        }
        self.sample_rate = sample_rate as f32;
        self.controls.prepare(self.settings, self.sample_rate);
        self.autotune = AutoTune::calibrated();
        self.vco_drift = VcoDriftBank::default();
        self.vco_drift.retune();
        self.reset_voices();
        self.cv.prepare(self.cv_targets(self.settings));
        self.lfo.reset();
        self.noise.reset();
        self.mod_wheel = 0.0;
        self.pitch_wheel = 0.0;
        self.sustain_pedal = false;
        self.held_notes.clear();
        self.glide_initialized = false;
        true
    }

    pub fn reset_voices(&mut self) {
        self.voices = [Voice::default(); VOICE_COUNT];
        self.poly_allocator.reset();
    }

    /// Re-runs the ten-channel oscillator calibration and captures the present
    /// thermal condition. Like the hardware Tune control, this changes machine
    /// state but never patch or serialized host state.
    pub fn tune_oscillators(&mut self) {
        self.autotune = AutoTune::calibrated();
        self.vco_drift.retune();
        self.refresh_all_voice_cvs();
    }

    pub fn settings(&self) -> Settings {
        self.settings
    }

    pub fn set_parameter(&mut self, index: u32, value: f64) -> bool {
        let was_unison = self.unison_enabled();
        if !self.settings.set(index, value) {
            return false;
        }
        if !matches!(
            Parameter::try_from(index),
            Ok(Parameter::MasterVolume | Parameter::VintageSpread)
        ) {
            self.controls.notify_change(self.sample_rate);
        }
        if index == Parameter::Unison as u32 && was_unison != self.unison_enabled() {
            self.rebuild_allocation_for_mode();
        }
        true
    }

    pub fn parameter(&self, index: u32) -> Option<f64> {
        self.settings.get_index(index).map(f64::from)
    }

    pub fn note_on(&mut self, channel: u8, note: u8, velocity: u8) {
        if velocity == 0 {
            self.note_off(channel, note);
            return;
        }
        let had_held_keys = self.held_notes.len != 0;
        self.held_notes.push(channel, note, velocity);
        if self.unison_enabled() {
            let lowest = self
                .held_notes
                .lowest()
                .expect("the incoming note is now held");
            if had_held_keys {
                if self.glide_target_note as u8 != lowest.note {
                    self.retune_unison(lowest.channel, lowest.note);
                }
            } else {
                self.start_unison(lowest.channel, lowest.note, lowest.velocity);
            }
            return;
        }
        let voice_index = self.poly_allocator.assign(channel, note);
        self.voices[voice_index].start(channel, note, velocity, voice_index);
        self.refresh_voice_cv(voice_index);
    }

    pub fn note_off(&mut self, channel: u8, note: u8) {
        if !self.held_notes.remove(channel, note) {
            return;
        }
        if self.sustain_pedal {
            return;
        }
        if self.unison_enabled() {
            if self.glide_target_note as u8 != note {
                return;
            }
            if let Some(lowest) = self.held_notes.lowest() {
                self.retune_unison(lowest.channel, lowest.note);
            } else {
                self.release_all_voices();
            }
            return;
        }
        for voice in &mut self.voices {
            if voice.matches(channel, note) {
                voice.release();
            }
        }
    }

    pub fn all_notes_off(&mut self) {
        self.held_notes.clear();
        self.sustain_pedal = false;
        self.release_all_voices();
    }

    fn release_all_voices(&mut self) {
        for voice in &mut self.voices {
            if voice.is_active() {
                voice.release();
            }
        }
    }

    fn release_sustained_notes(&mut self) {
        if self.unison_enabled() {
            if let Some(lowest) = self.held_notes.lowest() {
                if self.glide_target_note as u8 != lowest.note {
                    self.retune_unison(lowest.channel, lowest.note);
                }
            } else {
                self.release_all_voices();
            }
            return;
        }
        for voice in &mut self.voices {
            if let Some((channel, note)) = voice.identity()
                && !self.held_notes.contains(channel, note)
            {
                voice.release();
            }
        }
    }

    pub fn handle_midi(&mut self, data: [u8; 3]) {
        let channel = data[0] & 0x0f;
        match data[0] & 0xf0 {
            0x90 => self.note_on(channel, data[1] & 0x7f, data[2] & 0x7f),
            0x80 => self.note_off(channel, data[1] & 0x7f),
            0xb0 if data[1] == 120 || data[1] == 123 => self.all_notes_off(),
            0xb0 if data[1] == 1 => {
                self.mod_wheel = f32::from(data[2] & 0x7f) / 127.0;
                self.audition_mod_wheel = None;
            }
            0xb0 if data[1] == 64 => {
                let was_down = self.sustain_pedal;
                self.sustain_pedal = data[2] >= 64;
                if was_down && !self.sustain_pedal {
                    self.release_sustained_notes();
                }
            }
            0xe0 => {
                let value = u16::from(data[1] & 0x7f) | (u16::from(data[2] & 0x7f) << 7);
                self.pitch_wheel = pitch_wheel_normalized(value);
            }
            _ => {}
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let control_tick = self.controls.next(self.settings, self.sample_rate);
        let targets = self.cv_targets(control_tick.settings);
        self.cv.age(self.sample_rate);
        if let Some(destination) = control_tick.cv_strobe {
            self.cv.strobe(destination, targets);
        }
        let applied_settings = self.cv.apply_common(control_tick.settings);
        self.vco_drift.advance(self.sample_rate);
        let drift_character = applied_settings.get(Parameter::VintageSpread);
        let glide_offset = self.advance_glide(applied_settings);
        let performance_pitch = self.pitch_wheel * PITCH_WHEEL_RANGE_SEMITONES + glide_offset;
        let lfo_sample = self.lfo.next(
            self.sample_rate,
            applied_settings.get(Parameter::LfoFrequency),
            LfoWaveSelection {
                saw: parameter_enabled(applied_settings, Parameter::LfoSaw),
                triangle: parameter_enabled(applied_settings, Parameter::LfoTriangle),
                square: parameter_enabled(applied_settings, Parameter::LfoSquare),
            },
        );
        let noise_sample = self.noise.next(self.sample_rate);
        let source_mix = quantize_analog_pot(applied_settings.get(Parameter::WheelModSourceMix));
        let wheel_source = vca::wheel_mod_source(lfo_sample, noise_sample, source_mix);
        let effective_mod_wheel = self.audition_mod_wheel.unwrap_or(self.mod_wheel);
        let wheel_destinations = wheel_mod::destinations(wheel_source, effective_mod_wheel);
        let modulation = VoiceModulation {
            oscillator_a_semitones: performance_pitch
                + destination_value(
                    applied_settings,
                    Parameter::WheelModOscillatorAFrequency,
                    wheel_destinations.oscillator_semitones,
                ),
            oscillator_b_semitones: performance_pitch
                + destination_value(
                    applied_settings,
                    Parameter::WheelModOscillatorBFrequency,
                    wheel_destinations.oscillator_semitones,
                ),
            oscillator_a_pulse_width: destination_value(
                applied_settings,
                Parameter::WheelModOscillatorAPulseWidth,
                wheel_destinations.pulse_width,
            ),
            oscillator_b_pulse_width: destination_value(
                applied_settings,
                Parameter::WheelModOscillatorBPulseWidth,
                wheel_destinations.pulse_width,
            ),
            filter_octaves: destination_value(
                applied_settings,
                Parameter::WheelModFilter,
                wheel_destinations.filter_octaves,
            ),
            noise: vca::common_noise(
                noise_sample,
                quantize_analog_pot(applied_settings.get(Parameter::NoiseLevel)),
            ),
        };
        let mut sample = 0.0;
        for (voice_index, voice) in self.voices.iter_mut().enumerate() {
            let note = voice.note();
            let tuning_a = tuning::oscillator_a_tuning_semitones(
                note,
                applied_settings.get(Parameter::OscillatorAFrequency),
            );
            let tuning_b = tuning::oscillator_b_tuning_semitones(
                note,
                applied_settings.get(Parameter::OscillatorBFrequency),
                applied_settings.get(Parameter::OscillatorBDetune),
                parameter_enabled(applied_settings, Parameter::OscillatorBKeyboard),
                parameter_enabled(applied_settings, Parameter::OscillatorBLowFrequency),
            );
            let mut calibrated_modulation = modulation;
            calibrated_modulation.oscillator_a_semitones += self
                .cv
                .oscillator_semitones(voice_index, false)
                - tuning_a
                + self
                    .vco_drift
                    .correction_semitones(voice_index, Oscillator::A, drift_character);
            calibrated_modulation.oscillator_b_semitones += self
                .cv
                .oscillator_semitones(voice_index, true)
                - tuning_b
                + self
                    .vco_drift
                    .correction_semitones(voice_index, Oscillator::B, drift_character);
            let filter_keyboard = cv::filter_keyboard_octaves(
                note,
                parameter_enabled(applied_settings, Parameter::FilterKeyboard),
            );
            calibrated_modulation.filter_octaves +=
                self.cv.filter_keyboard_octaves(voice_index) - filter_keyboard;
            sample += voice.next(self.sample_rate, applied_settings, calibrated_modulation);
        }
        output::render(sample, applied_settings.get(Parameter::MasterVolume))
    }

    pub fn load_program(&mut self, id: &str) -> bool {
        let Some(program) = programs::find(id) else {
            return false;
        };
        let master_volume = self.settings.get(Parameter::MasterVolume);
        self.settings =
            Settings::from_array(program.values).expect("factory program values are valid");
        self.audition_mod_wheel = program.audition_mod_wheel;
        let restored = self
            .settings
            .set(Parameter::MasterVolume as u32, f64::from(master_volume));
        debug_assert!(restored);
        self.controls.notify_change(self.sample_rate);
        self.rebuild_allocation_for_mode();
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
        self.audition_mod_wheel = None;
        self.controls.notify_change(self.sample_rate);
        self.rebuild_allocation_for_mode();
        true
    }

    fn unison_enabled(&self) -> bool {
        parameter_enabled(self.settings, Parameter::Unison)
    }

    fn start_unison(&mut self, channel: u8, note: u8, velocity: u8) {
        self.retarget_glide(note);
        for (voice_index, voice) in self.voices.iter_mut().enumerate() {
            voice.start(channel, note, velocity, voice_index);
        }
        self.refresh_all_voice_cvs();
    }

    fn retune_unison(&mut self, channel: u8, note: u8) {
        self.retarget_glide(note);
        for voice in &mut self.voices {
            voice.retune(channel, note);
        }
        self.refresh_all_voice_cvs();
    }

    fn retarget_glide(&mut self, note: u8) {
        let target = f32::from(note);
        if !self.glide_initialized {
            self.glide_current_note = target;
            self.glide_initialized = true;
        }
        self.glide_target_note = target;
    }

    fn advance_glide(&mut self, applied_settings: Settings) -> f32 {
        if !self.unison_enabled() || !self.glide_initialized {
            return 0.0;
        }
        let amount = quantize_analog_pot(applied_settings.get(Parameter::Glide));
        self.glide_current_note = advance_glide_note(
            self.glide_current_note,
            self.glide_target_note,
            amount,
            self.sample_rate,
        );
        self.glide_current_note - self.glide_target_note
    }

    fn rebuild_allocation_for_mode(&mut self) {
        self.voices = [Voice::default(); VOICE_COUNT];
        self.poly_allocator.reset();
        self.glide_initialized = false;
        if self.unison_enabled() {
            if let Some(lowest) = self.held_notes.lowest() {
                self.start_unison(lowest.channel, lowest.note, lowest.velocity);
            }
            return;
        }
        let start = self.held_notes.len.saturating_sub(VOICE_COUNT);
        for index in start..self.held_notes.len {
            let held = self.held_notes.entries[index];
            self.note_on_without_tracking(held.channel, held.note, held.velocity);
        }
    }

    fn note_on_without_tracking(&mut self, channel: u8, note: u8, velocity: u8) {
        let voice_index = self.poly_allocator.assign(channel, note);
        self.voices[voice_index].start(channel, note, velocity, voice_index);
        self.refresh_voice_cv(voice_index);
    }

    fn voice_notes(&self) -> [u8; VOICE_COUNT] {
        core::array::from_fn(|index| self.voices[index].note())
    }

    fn cv_targets(&self, settings: Settings) -> cv::CvTargets {
        cv::CvTargets::from_state(settings, self.voice_notes(), self.autotune)
    }

    fn refresh_voice_cv(&mut self, voice_index: usize) {
        let settings = self.controls.current(self.settings);
        let targets = self.cv_targets(settings);
        self.cv.refresh_voice(voice_index, targets);
    }

    fn refresh_all_voice_cvs(&mut self) {
        let settings = self.controls.current(self.settings);
        let targets = self.cv_targets(settings);
        for voice in 0..VOICE_COUNT {
            self.cv.refresh_voice(voice, targets);
        }
    }
}

fn parameter_enabled(settings: Settings, parameter: Parameter) -> bool {
    settings.get(parameter) >= 0.5
}

fn destination_value(settings: Settings, parameter: Parameter, value: f32) -> f32 {
    if parameter_enabled(settings, parameter) {
        value
    } else {
        0.0
    }
}

fn pitch_wheel_normalized(value: u16) -> f32 {
    let value = value.min(16_383);
    if value < 8_192 {
        (f32::from(value) - 8_192.0) / 8_192.0
    } else {
        (f32::from(value) - 8_192.0) / 8_191.0
    }
}

fn glide_rate_semitones_per_second(amount: f32) -> f32 {
    let amount = amount.clamp(0.0, 1.0);
    MAXIMUM_GLIDE_RATE_SEMITONES_PER_SECOND
        * libm::powf(
            MINIMUM_GLIDE_RATE_SEMITONES_PER_SECOND / MAXIMUM_GLIDE_RATE_SEMITONES_PER_SECOND,
            amount,
        )
}

fn advance_glide_note(current: f32, target: f32, amount: f32, sample_rate: f32) -> f32 {
    if amount <= 0.0 {
        return target;
    }
    let maximum_step = glide_rate_semitones_per_second(amount) / sample_rate.max(1.0);
    current + (target - current).clamp(-maximum_step, maximum_step)
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
    fn note_assignment_acquires_all_three_voice_sample_holds_before_audio() {
        let mut engine = Engine::default();
        assert!(engine.set_parameter(Parameter::FilterKeyboard as u32, 1.0));
        assert!(engine.prepare(48_000.0));
        engine.note_on(0, 60, 100);

        let settings = engine.controls.current(engine.settings);
        let ideal_a = tuning::oscillator_a_tuning_semitones(
            60,
            settings.get(Parameter::OscillatorAFrequency),
        );
        let ideal_b = tuning::oscillator_b_tuning_semitones(
            60,
            settings.get(Parameter::OscillatorBFrequency),
            settings.get(Parameter::OscillatorBDetune),
            parameter_enabled(settings, Parameter::OscillatorBKeyboard),
            parameter_enabled(settings, Parameter::OscillatorBLowFrequency),
        );
        assert!((engine.cv.oscillator_semitones(0, false) - ideal_a).abs() < 0.03);
        assert!((engine.cv.oscillator_semitones(0, true) - ideal_b).abs() < 0.03);
        assert_eq!(engine.cv.filter_keyboard_octaves(0), 2.0);
    }

    #[test]
    fn voice_reassignment_reacquires_pitch_without_waiting_for_full_control_cycle() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        engine.note_on(0, 36, 100);
        let low = engine.cv.oscillator_semitones(0, false);
        engine.reset_voices();
        engine.note_on(0, 84, 100);
        let high = engine.cv.oscillator_semitones(0, false);
        assert!(high - low > 47.9);
    }

    #[test]
    fn pitch_wheel_uses_the_full_fourteen_bit_midi_range() {
        assert_eq!(pitch_wheel_normalized(0), -1.0);
        assert_eq!(pitch_wheel_normalized(8_192), 0.0);
        assert_eq!(pitch_wheel_normalized(16_383), 1.0);

        let mut engine = Engine::default();
        engine.handle_midi([0xe0, 0, 127]);
        assert!((engine.pitch_wheel - (16_256.0 - 8_192.0) / 8_191.0).abs() < 1.0e-6);
    }

    #[test]
    fn maximum_glide_traverses_five_octaves_in_five_seconds() {
        let sample_rate = 48_000.0;
        let mut note = 0.0;
        for _ in 0..(sample_rate as usize * 5) {
            note = advance_glide_note(note, 60.0, 1.0, sample_rate);
        }
        assert!((note - 60.0).abs() < 0.001, "five-octave result: {note}");
        assert_eq!(advance_glide_note(24.0, 60.0, 0.0, sample_rate), 60.0);
    }

    #[test]
    fn unison_uses_low_note_priority_and_releases_after_the_last_key() {
        let mut engine = Engine::default();
        assert!(engine.set_parameter(Parameter::Unison as u32, 1.0));
        engine.note_on(0, 67, 100);
        assert!(engine.voices.iter().all(|voice| voice.matches(0, 67)));

        engine.note_on(0, 60, 100);
        assert!(engine.voices.iter().all(|voice| voice.matches(0, 60)));
        engine.note_on(0, 72, 100);
        assert!(engine.voices.iter().all(|voice| voice.matches(0, 60)));

        engine.note_off(0, 60);
        assert!(engine.voices.iter().all(|voice| voice.matches(0, 67)));
        engine.note_off(0, 67);
        assert!(engine.voices.iter().all(|voice| voice.matches(0, 72)));
        engine.note_off(0, 72);
        assert!(engine.voices.iter().all(|voice| voice.is_active()));
    }

    #[test]
    fn unison_legato_note_on_is_a_retune_without_envelope_retrigger() {
        let mut routed = Engine::default();
        let mut explicit = Engine::default();
        for engine in [&mut routed, &mut explicit] {
            assert!(engine.prepare(48_000.0));
            assert!(engine.set_parameter(Parameter::Unison as u32, 1.0));
            engine.note_on(0, 67, 100);
            for _ in 0..4_096 {
                let _ = engine.next_sample();
            }
        }

        routed.note_on(0, 60, 100);
        explicit.held_notes.push(0, 60, 100);
        explicit.retune_unison(0, 60);

        for _ in 0..2_048 {
            assert_eq!(routed.next_sample(), explicit.next_sample());
        }
    }

    #[test]
    fn glide_offset_exists_only_while_unison_is_enabled() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        assert!(engine.set_parameter(Parameter::Unison as u32, 1.0));
        assert!(engine.set_parameter(Parameter::Glide as u32, 1.0));
        engine.note_on(0, 60, 100);
        engine.note_on(0, 48, 100);
        let target_settings = engine.settings;
        let offset = engine.advance_glide(target_settings);
        assert!(offset > 11.9);

        assert!(engine.set_parameter(Parameter::Unison as u32, 0.0));
        let target_settings = engine.settings;
        assert_eq!(engine.advance_glide(target_settings), 0.0);
    }

    #[test]
    fn sustain_defers_release_until_the_pedal_rises() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        assert!(engine.set_parameter(Parameter::AmpRelease as u32, 0.0));
        engine.note_on(0, 60, 100);
        engine.handle_midi([0xb0, 64, 127]);
        engine.note_off(0, 60);
        for _ in 0..4_096 {
            let _ = engine.next_sample();
        }
        assert!(engine.voices.iter().any(|voice| voice.matches(0, 60)));

        engine.handle_midi([0xb0, 64, 0]);
        for _ in 0..32_768 {
            let _ = engine.next_sample();
        }
        assert!(engine.voices.iter().all(|voice| !voice.is_active()));
    }

    #[test]
    fn engine_uses_the_documented_polyphonic_assignment_queue() {
        let mut engine = Engine::default();
        for note in 60..65 {
            engine.note_on(0, note, 100);
        }
        for (voice, note) in engine.voices.iter().zip(60..65) {
            assert!(voice.matches(0, note));
        }

        engine.note_on(0, 65, 100);
        assert!(engine.voices[0].matches(0, 65));
        assert!(engine.held_notes.contains(0, 60));

        engine.note_on(0, 62, 100);
        assert!(engine.voices[2].matches(0, 62));
        engine.note_on(0, 66, 100);
        assert!(engine.voices[1].matches(0, 66));
    }

    #[test]
    fn state_and_programs_round_trip() {
        let mut engine = Engine::default();
        assert!(engine.load_program("baseline-pad"));
        let expected = engine.settings();
        let mut state = [0_u8; STATE_BYTES];
        assert_eq!(engine.save_state(&mut state), Some(STATE_BYTES));
        assert!(engine.load_program("baseline-lead"));
        assert!(engine.load_state(&state));
        assert_eq!(engine.settings(), expected);
    }

    #[test]
    fn audition_wheel_is_temporary_machine_state() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        assert!(engine.load_program("audition-wheel-vibrato"));
        assert_eq!(engine.audition_mod_wheel, Some(0.42));

        // Hosts may restore a preset before (re)starting the audio device.
        assert!(engine.prepare(96_000.0));
        assert_eq!(engine.audition_mod_wheel, Some(0.42));

        let mut state = [0_u8; STATE_BYTES];
        assert_eq!(engine.save_state(&mut state), Some(STATE_BYTES));
        assert!(engine.load_state(&state));
        assert_eq!(engine.audition_mod_wheel, None);

        assert!(engine.load_program("audition-wheel-filter"));
        assert!(engine.audition_mod_wheel.is_some());
        assert!(engine.load_program("baseline-warm"));
        assert_eq!(engine.audition_mod_wheel, None);
    }

    #[test]
    fn real_mod_wheel_immediately_replaces_audition_override() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        assert!(engine.load_program("audition-wheel-pwm"));
        assert!(engine.audition_mod_wheel.is_some());

        engine.handle_midi([0xb0, 1, 96]);

        assert_eq!(engine.audition_mod_wheel, None);
        assert_eq!(engine.mod_wheel, 96.0 / 127.0);
    }

    #[test]
    fn audition_programs_are_audible_finite_and_distinct() {
        let mut signatures = [0.0_f32; 3];
        for (signature, id) in signatures.iter_mut().zip([
            "audition-wheel-vibrato",
            "audition-wheel-pwm",
            "audition-wheel-filter",
        ]) {
            let mut engine = Engine::default();
            assert!(engine.prepare(48_000.0));
            assert!(engine.load_program(id));
            engine.note_on(0, 60, 127);
            let mut peak = 0.0_f32;
            for index in 0..48_000 {
                let sample = engine.next_sample();
                assert!(sample.is_finite(), "non-finite sample in {id}");
                peak = peak.max(sample.abs());
                if index > 4_800 {
                    *signature += sample.abs() * (1.0 + (index % 97) as f32 / 97.0);
                }
            }
            assert!(peak > 0.01, "silent audition program {id}");
            assert!(peak <= 1.0, "unbounded audition program {id}");
        }
        assert!((signatures[0] - signatures[1]).abs() > 1.0);
        assert!((signatures[1] - signatures[2]).abs() > 1.0);
        assert!((signatures[0] - signatures[2]).abs() > 1.0);
    }

    #[test]
    fn filter_audition_programs_are_audible_and_distinct() {
        let mut signatures = [0.0_f32; 2];
        for (signature, id) in signatures
            .iter_mut()
            .zip(["audition-filter-drive", "audition-filter-resonance"])
        {
            let mut engine = Engine::default();
            assert!(engine.prepare(48_000.0));
            assert!(engine.load_program(id));
            engine.note_on(0, 48, 127);
            engine.note_on(0, 55, 127);
            engine.note_on(0, 60, 127);
            let mut peak = 0.0_f32;
            for index in 0..48_000 {
                let sample = engine.next_sample();
                assert!(sample.is_finite(), "non-finite sample in {id}");
                peak = peak.max(sample.abs());
                if index > 4_800 {
                    *signature += sample.abs() * (1.0 + (index % 89) as f32 / 89.0);
                }
            }
            assert!(peak > 0.01, "silent filter audition program {id}");
            assert!(peak <= 1.0, "unbounded filter audition program {id}");
        }
        assert!((signatures[0] - signatures[1]).abs() > 1.0);
    }

    #[test]
    fn oscillator_tune_reconditions_machine_state_without_changing_the_patch() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        assert!(engine.load_program("baseline-pad"));
        let expected_settings = engine.settings();
        for _ in 0..480_000 {
            let _ = engine.next_sample();
        }
        assert!(engine.vco_drift.correction_ppm(0, Oscillator::A, 1.0).abs() > 0.001);

        engine.tune_oscillators();

        assert_eq!(engine.settings(), expected_settings);
        for voice in 0..VOICE_COUNT {
            for oscillator in [Oscillator::A, Oscillator::B] {
                assert!(
                    engine
                        .vco_drift
                        .correction_ppm(voice, oscillator, 1.0)
                        .abs()
                        <= f32::EPSILON
                );
            }
        }
    }

    #[test]
    fn loading_a_program_preserves_the_physical_master_volume() {
        let mut engine = Engine::default();
        assert!(engine.set_parameter(Parameter::MasterVolume as u32, 0.31));
        assert!(engine.load_program("baseline-pad"));
        assert_eq!(
            engine.parameter(Parameter::MasterVolume as u32),
            Some(0.31_f32 as f64)
        );
    }

    #[test]
    fn oscillator_candidate_is_finite_across_supported_sample_rates() {
        for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
            let mut engine = Engine::default();
            assert!(engine.prepare(sample_rate));
            assert!(engine.load_program("baseline-lead"));
            engine.note_on(0, 93, 127);
            let mut peak = 0.0_f32;
            for _ in 0..16_384 {
                let sample = engine.next_sample();
                assert!(sample.is_finite());
                peak = peak.max(sample.abs());
            }
            assert!(peak > 0.001, "silent render at {sample_rate} Hz");
            assert!(peak <= 1.0);
        }
    }

    #[test]
    fn single_voice_render_has_usable_level_and_headroom() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        assert!(engine.load_program("baseline-warm"));
        engine.note_on(0, 60, 127);
        let mut peak = 0.0_f32;
        let mut energy = 0.0_f32;
        for index in 0..48_000 {
            let sample = engine.next_sample();
            if index > 4_800 {
                peak = peak.max(sample.abs());
                energy += sample * sample;
            }
        }
        let rms = libm::sqrtf(energy / 43_199.0);
        assert!(peak > 0.03, "single-voice peak too low: {peak}");
        assert!(rms > 0.005, "single-voice RMS too low: {rms}");
        assert!(peak < 0.95, "single-voice headroom exhausted: {peak}");
    }

    #[test]
    fn shared_lfo_free_runs_and_note_events_do_not_reset_it() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        let initial_phase = engine.lfo.phase();
        for _ in 0..257 {
            assert_eq!(engine.next_sample(), 0.0);
        }
        let free_running_phase = engine.lfo.phase();
        assert_ne!(free_running_phase, initial_phase);
        engine.note_on(0, 60, 100);
        assert_eq!(engine.lfo.phase(), free_running_phase);
    }

    #[test]
    fn midi_mod_wheel_drives_enabled_lfo_destinations() {
        let mut dry = Engine::default();
        let mut modulated = Engine::default();
        assert!(dry.prepare(48_000.0));
        assert!(modulated.prepare(48_000.0));
        assert!(dry.load_program("baseline-lead"));
        assert!(modulated.load_program("baseline-lead"));
        dry.note_on(0, 69, 127);
        modulated.note_on(0, 69, 127);
        modulated.handle_midi([0xb0, 1, 127]);

        let mut difference = 0.0;
        for _ in 0..8_192 {
            difference += (dry.next_sample() - modulated.next_sample()).abs();
        }
        assert_eq!(modulated.mod_wheel, 1.0);
        assert!(difference > 1.0);
    }

    #[test]
    fn shared_noise_free_runs_and_note_events_do_not_reset_it() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        let initial_state = engine.noise.state();
        for _ in 0..257 {
            assert_eq!(engine.next_sample(), 0.0);
        }
        let free_running_state = engine.noise.state();
        assert_ne!(free_running_state, initial_state);
        engine.note_on(0, 60, 100);
        assert_eq!(engine.noise.state(), free_running_state);
    }

    #[test]
    fn common_noise_level_reaches_each_voice_filter_input() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        assert!(engine.set_parameter(Parameter::OscillatorALevel as u32, 0.0));
        assert!(engine.set_parameter(Parameter::OscillatorBLevel as u32, 0.0));
        assert!(engine.set_parameter(Parameter::NoiseLevel as u32, 1.0));
        assert!(engine.set_parameter(Parameter::FilterCutoff as u32, 1.0));
        engine.note_on(0, 60, 127);
        assert!((0..16_384).any(|_| engine.next_sample().abs() > 0.001));
    }

    #[test]
    fn wheel_source_mix_crossfades_from_lfo_to_noise() {
        let mut lfo_side = Engine::default();
        let mut noise_side = Engine::default();
        assert!(lfo_side.prepare(48_000.0));
        assert!(noise_side.prepare(48_000.0));
        for engine in [&mut lfo_side, &mut noise_side] {
            assert!(engine.set_parameter(Parameter::LfoSaw as u32, 0.0));
            assert!(engine.set_parameter(Parameter::LfoTriangle as u32, 0.0));
            assert!(engine.set_parameter(Parameter::LfoSquare as u32, 0.0));
            assert!(engine.set_parameter(Parameter::WheelModOscillatorAFrequency as u32, 1.0,));
            engine.handle_midi([0xb0, 1, 127]);
            engine.note_on(0, 69, 127);
        }
        assert!(noise_side.set_parameter(Parameter::WheelModSourceMix as u32, 1.0));

        let mut difference = 0.0;
        for _ in 0..16_384 {
            difference += (lfo_side.next_sample() - noise_side.next_sample()).abs();
        }
        assert!(difference > 1.0);
    }
}
