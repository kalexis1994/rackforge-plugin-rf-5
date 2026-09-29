#![no_std]

mod a440;
mod allocation;
mod control;
mod cv;
mod glide;
mod lfo;
mod master_tune;
mod noise;
mod original_programs_data;
mod output;
mod pitch_wheel;
mod programs;
mod tune_cycle;
mod wheel_mod;

use allocation::PolyAllocator;
use lfo::{Lfo, LfoWaveSelection};
use noise::{PinkNoise, WhiteNoise};
use rf_5_contract::{
    PARAMETER_COUNT, Parameter, Settings,
    hardware::{
        ControlVoltageDestination, PROGRAM_BYTES, ProgramByte, decode_program, encode_program,
        quantize_analog_pot,
    },
};

/// Bytes in one program of the Rev 3's memory: 24, one per pot position (7
/// bits) with a switch in the eighth.
pub const PROGRAM_MEMORY_BYTES: usize = PROGRAM_BYTES;
use rf_5_voice::{
    Voice, VoiceModulation, VoiceSettings,
    autotune::{AutoTune, Oscillator},
    drift::VcoDriftBank,
    scale::ScaleProgram,
    tuning, vca,
};

pub const VOICE_COUNT: usize = 5;
pub const STATE_BYTES: usize = PARAMETER_COUNT * 4;
const PRE_SCALE_PATCH_PARAMETER_COUNT: usize = 47;
const PRE_SCALE_STATE_BYTES: usize = PRE_SCALE_PATCH_PARAMETER_COUNT * 4;
const PRE_RELEASE_PARAMETER_COUNT: usize = 59;
const PRE_RELEASE_STATE_BYTES: usize = PRE_RELEASE_PARAMETER_COUNT * 4;
const PRE_MASTER_TUNE_PARAMETER_COUNT: usize = 60;
const PRE_MASTER_TUNE_STATE_BYTES: usize = PRE_MASTER_TUNE_PARAMETER_COUNT * 4;
const PRE_MACHINE_OPERATIONS_PARAMETER_COUNT: usize = 61;
const PRE_MACHINE_OPERATIONS_STATE_BYTES: usize = PRE_MACHINE_OPERATIONS_PARAMETER_COUNT * 4;
const PRE_TAIL_MUTE_PARAMETER_COUNT: usize = 63;
const PRE_TAIL_MUTE_STATE_BYTES: usize = PRE_TAIL_MUTE_PARAMETER_COUNT * 4;
// MIDI CC1 has only 128 positions, whereas the original wheel is a continuous
// passive potentiometer. A short reconstruction filter removes controller
// steps without adding perceptible lag to a physical wheel gesture.
const MOD_WHEEL_DEZIPPER_TIME_SECONDS: f32 = 0.003;
const MAX_PENDING_VOICE_COMMANDS: usize = VOICE_COUNT * 4;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum VoiceCommandKind {
    #[default]
    Reset = 0,
    Start = 1,
    Retune = 2,
    Release = 3,
    Damp = 4,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct VoiceCommand {
    pub unit: u8,
    pub kind: VoiceCommandKind,
    pub channel: u8,
    pub note: u8,
    pub velocity: u8,
    pub reserved: [u8; 3],
    pub epoch: u32,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct VoiceCalibration {
    pub oscillator_a_semitones: f32,
    pub oscillator_b_semitones: f32,
    pub filter_octaves: f32,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct CommonVoiceFrame {
    pub settings: VoiceSettings,
    pub modulation: VoiceModulation,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PreparedSample {
    pub common: CommonVoiceFrame,
    pub calibration: [VoiceCalibration; VOICE_COUNT],
    pub a440: f32,
    pub master_volume: f32,
    pub tuning: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ParallelVoiceUnit {
    voice: Voice,
    epoch: u32,
}

impl ParallelVoiceUnit {
    pub fn synchronize_epoch(&mut self, epoch: u32, unit: usize) {
        if self.epoch != epoch {
            self.voice = Voice::initialized(unit);
            self.epoch = epoch;
        }
    }

    pub fn apply_command(&mut self, command: VoiceCommand) {
        if command.kind == VoiceCommandKind::Reset {
            self.voice = Voice::initialized(command.unit as usize);
            self.epoch = command.epoch;
            return;
        }
        self.synchronize_epoch(command.epoch, command.unit as usize);
        match command.kind {
            VoiceCommandKind::Reset => {}
            VoiceCommandKind::Start => self.voice.start(
                command.channel,
                command.note,
                command.velocity,
                command.unit as usize,
            ),
            VoiceCommandKind::Retune => self.voice.retune(command.channel, command.note),
            VoiceCommandKind::Release => self.voice.release(),
            VoiceCommandKind::Damp => self.voice.damp_release(),
        }
    }

    pub fn next(
        &mut self,
        sample_rate: f32,
        common: CommonVoiceFrame,
        calibration: VoiceCalibration,
    ) -> f32 {
        let mut modulation = common.modulation;
        modulation.oscillator_a_semitones += calibration.oscillator_a_semitones;
        modulation.oscillator_b_semitones += calibration.oscillator_b_semitones;
        modulation.filter_octaves += calibration.filter_octaves;
        self.voice
            .next_prepared(sample_rate, &common.settings, modulation)
    }
}

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

#[derive(Default)]
struct ControlCurrentCache {
    control: Option<f32>,
    current_amps: f32,
}

impl ControlCurrentCache {
    fn get(&mut self, control: f32, prepare: fn(f32) -> f32) -> f32 {
        if self.control.map(f32::to_bits) != Some(control.to_bits()) {
            self.control = Some(control);
            self.current_amps = prepare(control);
        }
        self.current_amps
    }
}

#[derive(Default)]
struct WheelControlCurrentCache {
    source_mix: Option<f32>,
    currents_amps: [f32; 2],
}

impl WheelControlCurrentCache {
    fn get(&mut self, source_mix: f32) -> [f32; 2] {
        if self.source_mix.map(f32::to_bits) != Some(source_mix.to_bits()) {
            self.source_mix = Some(source_mix);
            self.currents_amps = vca::wheel_mod_control_currents_amps(source_mix);
        }
        self.currents_amps
    }
}

pub struct Engine {
    settings: Settings,
    voices: [Voice; VOICE_COUNT],
    sample_rate: f32,
    poly_allocator: PolyAllocator,
    lfo: Lfo,
    wheel_noise: PinkNoise,
    audio_noise: WhiteNoise,
    mod_wheel: f32,
    smoothed_mod_wheel: f32,
    mod_wheel_smoothing_coefficient: f32,
    audition_mod_wheel: Option<f32>,
    pitch_wheel: f32,
    summer_lags: SummerLags,
    sustain_pedal: bool,
    held_notes: HeldNoteStack,
    // C376 voltage in keyboard semitones. A slow glide closes its last
    // cents in steps far below f32 resolution near note 60.
    glide_current_note: f64,
    glide_target_note: f32,
    glide_initialized: bool,
    glide_waiting_for_unison_cv: bool,
    controls: control::ControlScheduler,
    autotune: AutoTune,
    vco_drift: VcoDriftBank,
    cv: cv::CvDistributor,
    cv_notes: [u8; VOICE_COUNT],
    reference_tone: a440::ReferenceTone,
    tune_cycle: tune_cycle::TuneCycle,
    output: output::OutputStage,
    wheel_control_currents: WheelControlCurrentCache,
    noise_control_current: ControlCurrentCache,
    glide_rate: ControlCurrentCache,
    capture_voice_commands: bool,
    pending_voice_commands: [VoiceCommand; MAX_PENDING_VOICE_COMMANDS],
    pending_voice_command_count: usize,
    voice_epoch: u32,
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
            scale_program(settings),
        ));
        Self {
            settings,
            voices: core::array::from_fn(Voice::initialized),
            sample_rate: 48_000.0,
            poly_allocator: PolyAllocator::default(),
            lfo: Lfo::default(),
            wheel_noise: PinkNoise::default(),
            audio_noise: WhiteNoise::default(),
            mod_wheel: 0.0,
            smoothed_mod_wheel: 0.0,
            mod_wheel_smoothing_coefficient: 1.0,
            audition_mod_wheel: None,
            pitch_wheel: 0.0,
            summer_lags: SummerLags::default(),
            sustain_pedal: false,
            held_notes: HeldNoteStack::default(),
            glide_current_note: 0.0,
            glide_target_note: 0.0,
            glide_initialized: false,
            glide_waiting_for_unison_cv: false,
            controls: control::ControlScheduler::default(),
            autotune,
            vco_drift: VcoDriftBank::default(),
            cv,
            cv_notes: [0; VOICE_COUNT],
            reference_tone: a440::ReferenceTone::default(),
            tune_cycle: tune_cycle::TuneCycle::default(),
            output: output::OutputStage::default(),
            wheel_control_currents: WheelControlCurrentCache::default(),
            noise_control_current: ControlCurrentCache::default(),
            glide_rate: ControlCurrentCache::default(),
            capture_voice_commands: false,
            pending_voice_commands: [VoiceCommand::default(); MAX_PENDING_VOICE_COMMANDS],
            pending_voice_command_count: 0,
            voice_epoch: 0,
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
        self.cv_notes = self.voice_notes();
        self.cv.prepare(self.cv_targets(self.settings));
        self.lfo.reset();
        self.wheel_noise.reset();
        self.audio_noise.reset();
        self.reference_tone.reset();
        self.tune_cycle = tune_cycle::TuneCycle::default();
        self.output.reset();
        self.mod_wheel = 0.0;
        self.smoothed_mod_wheel = 0.0;
        self.mod_wheel_smoothing_coefficient =
            1.0 - libm::expf(-1.0 / (self.sample_rate * MOD_WHEEL_DEZIPPER_TIME_SECONDS));
        self.pitch_wheel = 0.0;
        self.summer_lags = SummerLags::prepared(self.sample_rate);
        self.sustain_pedal = false;
        self.held_notes.clear();
        self.glide_initialized = false;
        self.glide_waiting_for_unison_cv = false;
        true
    }

    pub fn reset_voices(&mut self) {
        self.reset_voice_cards();
        self.poly_allocator.reset();
    }

    /// Enables the bounded command journal used by the parallel adapter.
    /// Sequential/offline callers leave it disabled and pay no queueing cost.
    pub fn capture_voice_commands(&mut self, enabled: bool) {
        self.capture_voice_commands = enabled;
        self.pending_voice_command_count = 0;
    }

    pub fn voice_epoch(&self) -> u32 {
        self.voice_epoch
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn drain_voice_commands(&mut self, destination: &mut [VoiceCommand]) -> usize {
        let count = self.pending_voice_command_count.min(destination.len());
        destination[..count].copy_from_slice(&self.pending_voice_commands[..count]);
        self.pending_voice_command_count = 0;
        count
    }

    fn push_voice_command(&mut self, command: VoiceCommand) {
        if !self.capture_voice_commands {
            return;
        }
        debug_assert!(self.pending_voice_command_count < self.pending_voice_commands.len());
        if let Some(slot) = self
            .pending_voice_commands
            .get_mut(self.pending_voice_command_count)
        {
            *slot = command;
            self.pending_voice_command_count += 1;
        }
    }

    fn reset_voice_cards(&mut self) {
        self.voices = core::array::from_fn(Voice::initialized);
        self.voice_epoch = self.voice_epoch.wrapping_add(1).max(1);
        for unit in 0..VOICE_COUNT {
            self.push_voice_command(VoiceCommand {
                unit: unit as u8,
                kind: VoiceCommandKind::Reset,
                epoch: self.voice_epoch,
                ..VoiceCommand::default()
            });
        }
    }

    fn start_voice(&mut self, unit: usize, channel: u8, note: u8, velocity: u8) {
        self.admit_voice_pitch(unit, note);
        self.voices[unit].start(channel, note, velocity, unit);
        self.push_voice_command(VoiceCommand {
            unit: unit as u8,
            kind: VoiceCommandKind::Start,
            channel,
            note,
            velocity,
            epoch: self.voice_epoch,
            ..VoiceCommand::default()
        });
    }

    fn retune_voice(&mut self, unit: usize, channel: u8, note: u8) {
        self.admit_voice_pitch(unit, note);
        self.voices[unit].retune(channel, note);
        self.push_voice_command(VoiceCommand {
            unit: unit as u8,
            kind: VoiceCommandKind::Retune,
            channel,
            note,
            epoch: self.voice_epoch,
            ..VoiceCommand::default()
        });
    }

    /// Admit a host note at the hardware boundary after keyboard scanning and
    /// the first complete per-voice pitch pass. The firmware really does
    /// assert the gate before its following DAC sweep, but exposing that
    /// private interval directly to sample-accurate MIDI made the newly opened
    /// VCA reproduce the previous voice assignment for several milliseconds.
    /// The physical keyboard cannot be observed at that intermediate point.
    fn admit_voice_pitch(&mut self, unit: usize, note: u8) {
        self.cv_notes[unit] = note;
        let settings = self.controls.current(self.settings);
        let targets = self.cv_targets(settings);
        self.cv.refresh_voice(unit, targets);
    }

    fn release_voice(&mut self, unit: usize) {
        // Every path that puts a voice into its release comes through here
        // -- a key up, the sustain pedal rising, all-notes-off -- so this is
        // the one place the assigner has to be told, and the one that was
        // missing. See `allocation`.
        self.poly_allocator.mark_released(unit);
        self.voices[unit].release();
        self.push_voice_command(VoiceCommand {
            unit: unit as u8,
            kind: VoiceCommandKind::Release,
            epoch: self.voice_epoch,
            ..VoiceCommand::default()
        });
    }

    pub fn voice_initialized(&self, unit: usize) -> bool {
        self.voices
            .get(unit)
            .is_some_and(|voice| voice.is_initialized())
    }

    /// Re-runs the ten-channel oscillator calibration and captures the present
    /// thermal condition. Like the hardware Tune control, this changes machine
    /// state but never patch or serialized host state.
    pub fn tune_oscillators(&mut self) {
        self.autotune = AutoTune::calibrated();
        self.vco_drift.retune();
        self.cv_notes = self.voice_notes();
        self.refresh_all_voice_cvs();
    }

    /// Starts the front-panel TUNE operation. A second press is ignored while
    /// the CPU is already occupied by its ten-VCO measurement pass.
    pub fn request_tune(&mut self) -> bool {
        let character = self.settings.get(Parameter::VintageSpread);
        self.tune_cycle.start(
            self.sample_rate,
            self.vco_drift.normalized_tune_error(character),
        )
    }

    pub fn is_tuning(&self) -> bool {
        self.tune_cycle.is_active()
    }

    pub fn tune_duration_seconds(&self) -> f32 {
        self.tune_cycle.duration_seconds(self.sample_rate)
    }

    pub fn settings(&self) -> Settings {
        self.settings
    }

    pub fn set_parameter(&mut self, index: u32, value: f64) -> bool {
        if index == Parameter::Tune as u32 {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return false;
            }
            if value >= 0.5 && !self.is_tuning() {
                return self.request_tune();
            }
            return true;
        }
        let was_unison = self.unison_enabled();
        if !self.settings.set(index, value) {
            return false;
        }
        if !matches!(
            Parameter::try_from(index),
            Ok(Parameter::MasterVolume
                | Parameter::MasterTune
                | Parameter::VintageSpread
                | Parameter::A440)
        ) {
            self.controls.notify_change(self.sample_rate);
        }
        if index == Parameter::Unison as u32 && was_unison != self.unison_enabled() {
            self.rebuild_allocation_for_mode();
        }
        true
    }

    pub fn parameter(&self, index: u32) -> Option<f64> {
        if index == Parameter::Tune as u32 {
            return Some(f64::from(self.is_tuning()));
        }
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
        self.start_voice(voice_index, channel, note, velocity);
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
        for voice_index in 0..VOICE_COUNT {
            if self.voices[voice_index].matches(channel, note) {
                self.release_voice(voice_index);
            }
        }
    }

    pub fn all_notes_off(&mut self) {
        self.held_notes.clear();
        self.sustain_pedal = false;
        self.release_all_voices();
    }

    fn release_all_voices(&mut self) {
        for voice_index in 0..VOICE_COUNT {
            if self.voices[voice_index].is_active() {
                self.release_voice(voice_index);
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
        for voice_index in 0..VOICE_COUNT {
            if let Some((channel, note)) = self.voices[voice_index].identity()
                && !self.held_notes.contains(channel, note)
            {
                self.release_voice(voice_index);
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
                self.pitch_wheel = pitch_wheel::normalized_output(value);
            }
            _ => {}
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let prepared = self.prepare_next_sample();
        let mut voice_sum = 0.0;
        for voice_index in 0..VOICE_COUNT {
            voice_sum += self.render_prepared_voice(voice_index, prepared);
        }
        self.finish_prepared_sample(prepared, voice_sum)
    }

    /// Advances every global/control-plane circuit for one host sample and
    /// returns the immutable values needed by the five physical voice cards.
    /// Voice-card state and the common output stage are deliberately untouched.
    pub fn prepare_next_sample(&mut self) -> PreparedSample {
        let tuning = self.is_tuning();
        let control_tick = self.controls.next(self.settings, self.sample_rate);
        if control_tick.cycle_started {
            self.cv_notes = self.voice_notes();
        }
        self.cv.age(self.sample_rate);
        if let Some(destination) = control_tick.cv_strobe {
            // The original DAC only drives a destination while its mux slot is
            // strobed. Building all 38 target voltages on intervening audio
            // samples cannot affect the held cells and needlessly repeats the
            // complete ten-VCO tuning calculation.
            let destination_kind = ControlVoltageDestination::try_from(destination as u8)
                .expect("valid scheduled CV destination");
            let target_volts = cv::destination_voltage(
                control_tick.settings,
                self.cv_notes,
                self.autotune,
                scale_program(control_tick.settings),
                destination_kind,
            );
            self.cv.strobe_voltage(destination, target_volts);
            if destination == ControlVoltageDestination::UnisonKeyboard as usize {
                self.glide_waiting_for_unison_cv = false;
            }
        }
        let applied_settings = self.cv.apply_common(control_tick.settings);
        self.vco_drift.advance(self.sample_rate);
        let drift_character = applied_settings.get(Parameter::VintageSpread);
        let glide_offset = if tuning {
            0.0
        } else {
            self.advance_glide(applied_settings)
        };
        let tune_summer_pitch = self.summer_lags.tune.next(if tuning {
            0.0
        } else {
            master_tune::offset_semitones(applied_settings.get(Parameter::MasterTune))
                + self.pitch_wheel * pitch_wheel::RANGE_SEMITONES
        });
        let lfo_sample = self.lfo.next(
            self.sample_rate,
            applied_settings.get(Parameter::LfoFrequency),
            LfoWaveSelection {
                saw: parameter_enabled(applied_settings, Parameter::LfoSaw),
                triangle: parameter_enabled(applied_settings, Parameter::LfoTriangle),
                square: parameter_enabled(applied_settings, Parameter::LfoSquare),
            },
        );
        let wheel_noise_sample = self.wheel_noise.next(self.sample_rate);
        let audio_noise_sample = self.audio_noise.next(self.sample_rate);
        let effective_mod_wheel = if let Some(audition_amount) = self.audition_mod_wheel {
            audition_amount
        } else {
            self.advance_mod_wheel()
        };
        // The LFO and noise source continue free-running with the wheel down,
        // but the passive wheel grounds every destination. Avoid solving the
        // inaudible dual-OTA source mixer until the wheel can pass a voltage.
        let wheel_destinations = if effective_mod_wheel > 0.0 {
            let source_mix =
                quantize_analog_pot(applied_settings.get(Parameter::WheelModSourceMix));
            let [wheel_lfo_current, wheel_noise_current] =
                self.wheel_control_currents.get(source_mix);
            let wheel_source = vca::wheel_mod_source_with_control_currents(
                lfo_sample,
                wheel_noise_sample,
                wheel_lfo_current,
                wheel_noise_current,
            );
            wheel_mod::destinations(wheel_source, effective_mod_wheel)
        } else {
            wheel_mod::WheelModDestinations::default()
        };
        // SD334 R378/C368 give the MASTER TUNE/PITCH summer U367 a 10 ms
        // lag, and R363/C364, R360/C362 and R357/C363 give the A SUM, B SUM
        // and FILT SUM output stages 1 ms each. The PW summers have none.
        let oscillator_a_sum = self.summer_lags.oscillator_a.next(
            tune_summer_pitch
                + glide_offset
                + destination_value(
                    applied_settings,
                    Parameter::WheelModOscillatorAFrequency,
                    wheel_destinations.oscillator_semitones,
                ),
        );
        let oscillator_b_sum = self.summer_lags.oscillator_b.next(
            tune_summer_pitch
                + glide_offset
                + destination_value(
                    applied_settings,
                    Parameter::WheelModOscillatorBFrequency,
                    wheel_destinations.oscillator_semitones,
                ),
        );
        // FILT MSUM's first LM348 stage adds FILT CUTOFF, the Unison keyboard
        // (through FILT KBD) and the Wheel Mod filter path (R399, x7.5) before
        // its output can swing past the op-amp's +/-13 V. Only the Wheel Mod
        // share is carried here, so the rail limits what remains of it.
        let wheel_filter_volts = destination_value(
            applied_settings,
            Parameter::WheelModFilter,
            wheel_destinations.filter_octaves,
        );
        let unison_keyboard_volts = if self.unison_enabled()
            && parameter_enabled(applied_settings, Parameter::FilterKeyboard)
        {
            glide_offset / 12.0
        } else {
            0.0
        };
        let cutoff_and_keyboard_volts = rf_5_contract::hardware::normalized_control_volts(
            applied_settings.get(Parameter::FilterCutoff),
        ) + unison_keyboard_volts;
        let filter_summer_input = (cutoff_and_keyboard_volts + wheel_filter_volts)
            .clamp(-FILTER_SUMMER_SWING_VOLTS, FILTER_SUMMER_SWING_VOLTS)
            - cutoff_and_keyboard_volts;
        let filter_sum = self.summer_lags.filter.next(filter_summer_input);
        let modulation = VoiceModulation {
            oscillator_a_semitones: oscillator_a_sum,
            oscillator_b_semitones: oscillator_b_sum,
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
            filter_octaves: filter_sum,
            noise: {
                let noise_level = quantize_analog_pot(applied_settings.get(Parameter::NoiseLevel));
                let noise_control_current = self
                    .noise_control_current
                    .get(noise_level, vca::common_noise_control_current_amps);
                vca::common_noise_with_control_current(audio_noise_sample, noise_control_current)
            },
        };
        let mut calibration = [VoiceCalibration::default(); VOICE_COUNT];
        for (voice_index, voice) in self.voices.iter().enumerate() {
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
            calibration[voice_index].oscillator_a_semitones = self
                .cv
                .oscillator_semitones(voice_index, false)
                - tuning_a
                + self
                    .vco_drift
                    .correction_semitones(voice_index, Oscillator::A, drift_character);
            calibration[voice_index].oscillator_b_semitones = self
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
            calibration[voice_index].filter_octaves =
                self.cv.filter_keyboard_octaves(voice_index) - filter_keyboard;
        }
        let a440 = self.reference_tone.next(
            !tuning && parameter_enabled(applied_settings, Parameter::A440),
            self.sample_rate,
        );
        if self.tune_cycle.advance() {
            self.tune_oscillators();
        }
        PreparedSample {
            common: CommonVoiceFrame {
                settings: VoiceSettings::from_settings(&applied_settings),
                modulation,
            },
            calibration,
            a440,
            master_volume: applied_settings.get(Parameter::MasterVolume),
            tuning,
        }
    }

    pub fn render_prepared_voice(&mut self, voice_index: usize, prepared: PreparedSample) -> f32 {
        let Some(voice) = self.voices.get_mut(voice_index) else {
            return 0.0;
        };
        if !voice.is_initialized() {
            return 0.0;
        }
        let calibration = prepared.calibration[voice_index];
        let mut modulation = prepared.common.modulation;
        modulation.oscillator_a_semitones += calibration.oscillator_a_semitones;
        modulation.oscillator_b_semitones += calibration.oscillator_b_semitones;
        modulation.filter_octaves += calibration.filter_octaves;
        voice.next_prepared(self.sample_rate, &prepared.common.settings, modulation)
    }

    pub fn finish_prepared_sample(&mut self, prepared: PreparedSample, voice_sum: f32) -> f32 {
        let (voice_sum, reference_volts) = if prepared.tuning {
            (0.0, 0.0)
        } else {
            (voice_sum, prepared.a440)
        };
        self.output.next(
            voice_sum,
            reference_volts,
            prepared.master_volume,
            self.sample_rate,
        )
    }

    fn advance_mod_wheel(&mut self) -> f32 {
        let difference = self.mod_wheel - self.smoothed_mod_wheel;
        if difference.abs() <= 1.0e-6 {
            self.smoothed_mod_wheel = self.mod_wheel;
        } else {
            self.smoothed_mod_wheel += difference * self.mod_wheel_smoothing_coefficient;
        }
        self.smoothed_mod_wheel
    }

    pub fn load_program(&mut self, id: &str) -> bool {
        let Some(program) = programs::find(id) else {
            return false;
        };
        self.apply_program(program)
    }

    /// The program memory the panel would store now, as the Rev 3's RECORD
    /// does: every pot at its 128 positions, every switch as a bit, in the
    /// 24 bytes of one program.
    pub fn program_memory(&self) -> [u8; PROGRAM_MEMORY_BYTES] {
        encode_program(self.settings).map(ProgramByte::raw)
    }

    /// Recalls a program from its memory bytes, as a program button does.
    pub fn load_program_memory(&mut self, memory: [u8; PROGRAM_MEMORY_BYTES]) -> bool {
        self.apply_program(programs::Program::original(
            memory.map(ProgramByte::from_raw),
        ))
    }

    /// A factory program's memory, for a copy of it.
    pub fn factory_program_memory(id: &str) -> Option<[u8; PROGRAM_MEMORY_BYTES]> {
        let program = programs::find(id)?;
        let raw = match program.raw_v81 {
            Some(raw) => raw,
            None => {
                let mut settings = Settings::default();
                if !settings.apply_patch_array(program.values) {
                    return None;
                }
                encode_program(settings)
            }
        };
        Some(raw.map(ProgramByte::raw))
    }

    #[cfg(any(test, feature = "diagnostic-programs"))]
    pub fn load_diagnostic_program(&mut self, id: &str) -> bool {
        let Some(program) = programs::find_diagnostic(id) else {
            return false;
        };
        self.apply_program(program)
    }

    fn apply_program(&mut self, program: programs::Program) -> bool {
        let was_unison = self.unison_enabled();
        self.settings = if let Some(raw) = program.raw_v81 {
            decode_program(raw, self.settings)
        } else {
            let mut source = Settings::default();
            if !source.apply_patch_array(program.values) {
                return false;
            }
            decode_program(encode_program(source), self.settings)
        };
        self.audition_mod_wheel = program.audition_mod_wheel;
        self.controls.notify_recall(self.settings, self.sample_rate);
        // Program recall on the original changes panel/CV state without
        // power-cycling the five voice cards. Preserve their free-running
        // oscillators and filter capacitor memories; only a Poly/Unison mode
        // transition requires rebuilding gate and pitch assignment.
        if was_unison != self.unison_enabled() {
            self.rebuild_allocation_for_mode();
        }
        if self.settings.get(Parameter::ProgramChangeMutesTails) >= 0.5 {
            self.damp_releasing_voices();
        }
        true
    }

    /// Only a voice already in its release obeys; held notes take the new
    /// program exactly as on the original.
    fn damp_releasing_voices(&mut self) {
        for unit in 0..VOICE_COUNT {
            self.voices[unit].damp_release();
            self.push_voice_command(VoiceCommand {
                unit: unit as u8,
                kind: VoiceCommandKind::Damp,
                epoch: self.voice_epoch,
                ..VoiceCommand::default()
            });
        }
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
        if state.len() == PRE_SCALE_STATE_BYTES {
            let mut values = [0.0_f32; PRE_SCALE_PATCH_PARAMETER_COUNT];
            let (chunks, remainder) = state.as_chunks::<4>();
            if !remainder.is_empty() {
                return false;
            }
            for (value, chunk) in values.iter_mut().zip(chunks) {
                *value = f32::from_le_bytes(*chunk);
            }
            let mut settings = Settings::default();
            for (index, value) in values.into_iter().enumerate() {
                if !settings.set(index as u32, f64::from(value)) {
                    return false;
                }
            }
            self.install_loaded_settings(settings);
            return true;
        }
        if state.len() == PRE_RELEASE_STATE_BYTES {
            let mut old = [0.0_f32; PRE_RELEASE_PARAMETER_COUNT];
            let (chunks, remainder) = state.as_chunks::<4>();
            if !remainder.is_empty() {
                return false;
            }
            for (value, chunk) in old.iter_mut().zip(chunks) {
                *value = f32::from_le_bytes(*chunk);
            }
            let mut values = Settings::default().as_array();
            values[..PRE_SCALE_PATCH_PARAMETER_COUNT]
                .copy_from_slice(&old[..PRE_SCALE_PATCH_PARAMETER_COUNT]);
            values[Parameter::ScaleC as usize..Parameter::MasterTune as usize]
                .copy_from_slice(&old[PRE_SCALE_PATCH_PARAMETER_COUNT..]);
            let Some(settings) = Settings::from_array(values) else {
                return false;
            };
            self.install_loaded_settings(settings);
            return true;
        }
        if state.len() == PRE_MASTER_TUNE_STATE_BYTES {
            let mut old = [0.0_f32; PRE_MASTER_TUNE_PARAMETER_COUNT];
            let (chunks, remainder) = state.as_chunks::<4>();
            if !remainder.is_empty() {
                return false;
            }
            for (value, chunk) in old.iter_mut().zip(chunks) {
                *value = f32::from_le_bytes(*chunk);
            }
            let mut values = Settings::default().as_array();
            values[..PRE_MASTER_TUNE_PARAMETER_COUNT].copy_from_slice(&old);
            let Some(settings) = Settings::from_array(values) else {
                return false;
            };
            self.install_loaded_settings(settings);
            return true;
        }
        if state.len() == PRE_MACHINE_OPERATIONS_STATE_BYTES {
            let mut old = [0.0_f32; PRE_MACHINE_OPERATIONS_PARAMETER_COUNT];
            let (chunks, remainder) = state.as_chunks::<4>();
            if !remainder.is_empty() {
                return false;
            }
            for (value, chunk) in old.iter_mut().zip(chunks) {
                *value = f32::from_le_bytes(*chunk);
            }
            let mut values = Settings::default().as_array();
            values[..PRE_MACHINE_OPERATIONS_PARAMETER_COUNT].copy_from_slice(&old);
            let Some(settings) = Settings::from_array(values) else {
                return false;
            };
            self.install_loaded_settings(settings);
            return true;
        }
        if state.len() == PRE_TAIL_MUTE_STATE_BYTES {
            let mut old = [0.0_f32; PRE_TAIL_MUTE_PARAMETER_COUNT];
            let (chunks, remainder) = state.as_chunks::<4>();
            if !remainder.is_empty() {
                return false;
            }
            for (value, chunk) in old.iter_mut().zip(chunks) {
                *value = f32::from_le_bytes(*chunk);
            }
            let mut values = Settings::default().as_array();
            values[..PRE_TAIL_MUTE_PARAMETER_COUNT].copy_from_slice(&old);
            let Some(settings) = Settings::from_array(values) else {
                return false;
            };
            self.install_loaded_settings(settings);
            return true;
        }
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
        self.install_loaded_settings(settings);
        true
    }

    fn install_loaded_settings(&mut self, mut settings: Settings) {
        let was_unison = self.unison_enabled();
        let cleared = settings.set(Parameter::Tune as u32, 0.0);
        debug_assert!(cleared);
        self.settings = settings;
        self.tune_cycle = tune_cycle::TuneCycle::default();
        self.audition_mod_wheel = None;
        self.controls.notify_recall(self.settings, self.sample_rate);
        if was_unison != self.unison_enabled() {
            self.rebuild_allocation_for_mode();
        }
    }

    fn unison_enabled(&self) -> bool {
        parameter_enabled(self.settings, Parameter::Unison)
    }

    fn start_unison(&mut self, channel: u8, note: u8, velocity: u8) {
        self.retarget_glide(note);
        self.glide_waiting_for_unison_cv = true;
        for voice_index in 0..VOICE_COUNT {
            self.start_voice(voice_index, channel, note, velocity);
        }
    }

    fn retune_unison(&mut self, channel: u8, note: u8) {
        self.retarget_glide(note);
        for voice_index in 0..VOICE_COUNT {
            self.retune_voice(voice_index, channel, note);
        }
    }

    fn retarget_glide(&mut self, note: u8) {
        let target = f32::from(note);
        if !self.glide_initialized {
            self.glide_current_note = f64::from(target);
            self.glide_initialized = true;
        }
        self.glide_target_note = target;
    }

    fn advance_glide(&mut self, applied_settings: Settings) -> f32 {
        if !self.unison_enabled() || !self.glide_initialized {
            return 0.0;
        }
        if self.glide_waiting_for_unison_cv {
            return (self.glide_current_note - f64::from(tuning::LOWEST_KEY_MIDI_NOTE)) as f32;
        }
        let amount = quantize_analog_pot(applied_settings.get(Parameter::Glide));
        let circuit_target =
            f32::from(tuning::LOWEST_KEY_MIDI_NOTE) + self.cv.unison_keyboard_semitones();
        let rate = self
            .glide_rate
            .get(amount, glide::rate_semitones_per_second);
        self.glide_current_note = glide::advance_note(
            self.glide_current_note,
            f64::from(circuit_target),
            rate,
            self.sample_rate,
        );
        (self.glide_current_note - f64::from(tuning::LOWEST_KEY_MIDI_NOTE)) as f32
    }

    fn rebuild_allocation_for_mode(&mut self) {
        self.reset_voices();
        self.glide_initialized = false;
        self.glide_waiting_for_unison_cv = false;
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
        self.start_voice(voice_index, channel, note, velocity);
    }

    fn voice_notes(&self) -> [u8; VOICE_COUNT] {
        core::array::from_fn(|index| self.voices[index].note())
    }

    fn cv_targets(&self, settings: Settings) -> cv::CvTargets {
        cv::CvTargets::from_state(
            settings,
            self.cv_notes,
            self.autotune,
            scale_program(settings),
        )
    }

    fn refresh_all_voice_cvs(&mut self) {
        let settings = self.controls.current(self.settings);
        let targets = self.cv_targets(settings);
        for voice in 0..VOICE_COUNT {
            self.cv.refresh_voice(voice, targets);
        }
    }
}

fn scale_program(settings: Settings) -> ScaleProgram {
    ScaleProgram::from_normalized(settings.scale_values())
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

/// LM348 typical output swing on +/-15 V into the summer's 100k loads.
const FILTER_SUMMER_SWING_VOLTS: f32 = 13.0;

/// One-pole lags formed by the feedback capacitors of SD334's inverting
/// summers: the output settles toward the summed input with time constant R*C.
#[derive(Clone, Copy, Debug, Default)]
struct SummerLag {
    state: f32,
    retained: f32,
}

impl SummerLag {
    fn with_time_constant(sample_rate: f32, seconds: f32) -> Self {
        Self {
            state: 0.0,
            retained: libm::expf(-1.0 / (sample_rate.max(1.0) * seconds)),
        }
    }

    fn next(&mut self, input: f32) -> f32 {
        self.state = input + (self.state - input) * self.retained;
        self.state
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct SummerLags {
    tune: SummerLag,
    oscillator_a: SummerLag,
    oscillator_b: SummerLag,
    filter: SummerLag,
}

impl SummerLags {
    fn prepared(sample_rate: f32) -> Self {
        // R378 100k / C368 0.1 uF and 100k / 0.01 uF on each output stage.
        const TUNE_SUMMER_SECONDS: f32 = 100_000.0 * 0.1e-6;
        const OUTPUT_SUMMER_SECONDS: f32 = 100_000.0 * 0.01e-6;
        Self {
            tune: SummerLag::with_time_constant(sample_rate, TUNE_SUMMER_SECONDS),
            oscillator_a: SummerLag::with_time_constant(sample_rate, OUTPUT_SUMMER_SECONDS),
            oscillator_b: SummerLag::with_time_constant(sample_rate, OUTPUT_SUMMER_SECONDS),
            filter: SummerLag::with_time_constant(sample_rate, OUTPUT_SUMMER_SECONDS),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::vec::Vec;

    fn windowed_frequency_magnitude(samples: &[f32], sample_rate: f32, frequency: f32) -> f32 {
        let last = (samples.len() - 1) as f32;
        let mut real = 0.0_f32;
        let mut imaginary = 0.0_f32;
        for (index, sample) in samples.iter().copied().enumerate() {
            let phase = core::f32::consts::TAU * index as f32 / last;
            let window = 0.5 - 0.5 * libm::cosf(phase);
            let angle = core::f32::consts::TAU * frequency * index as f32 / sample_rate;
            real += sample * window * libm::cosf(angle);
            imaginary -= sample * window * libm::sinf(angle);
        }
        libm::sqrtf(real * real + imaginary * imaginary)
    }

    #[test]
    fn engine_is_silent_until_midi_arrives() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        assert!((0..128).all(|_| engine.next_sample() == 0.0));
        engine.note_on(0, 60, 100);
        assert!((0..4096).any(|_| engine.next_sample().abs() > 0.001));
    }

    #[test]
    fn high_sync_i_attack_has_no_doubled_numerical_click() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        assert!(engine.load_program("original-17-sync-i"));
        for _ in 0..96_000 {
            let _ = engine.next_sample();
        }

        // A doubled numerical click is an onset event: it makes the first
        // milliseconds rougher than the synced waveform itself. Compare the
        // attack against the same note's own steady second difference.
        engine.note_on(0, 96, 127);
        let mut previous = 0.0_f32;
        let mut previous_delta = 0.0_f32;
        let mut onset_maximum = 0.0_f32;
        let mut steady_maximum = 0.0_f32;
        for index in 0..24_000 {
            let sample = engine.next_sample();
            let delta = sample - previous;
            let second_difference = (delta - previous_delta).abs();
            if index < 480 {
                onset_maximum = onset_maximum.max(second_difference);
            } else if index >= 4_800 {
                steady_maximum = steady_maximum.max(second_difference);
            }
            previous = sample;
            previous_delta = delta;
        }

        assert!(
            onset_maximum < steady_maximum * 1.1,
            "Sync I high-note attack produced a doubled click: onset {onset_maximum}, steady {steady_maximum}"
        );
    }

    // Open discrepancy, kept visible rather than refitted. The Rev 3
    // recording of 1-4 places the first two octave bands within about 0.1 dB.
    // With the V8.1 ROM-confirmed filter anchor, U451's doubled triangle and
    // the CEM3340 pulse-width law, RF-5 reaches only about -19 dB even with
    // the filter fully open, so the gap lies in the synced oscillator A /
    // Poly Mod / mixer generation, not in the filter. The former pass relied
    // on the old 1-99% PW law and half-level triangle.
    #[test]
    #[ignore = "open 1-4 oscillator-section discrepancy; see ORIGINAL_FACTORY_PROGRAMS.md"]
    fn factory_e_piano_attack_exposes_its_documented_octave_overtone() {
        const SAMPLE_RATE: f32 = 48_000.0;
        let mut engine = Engine::default();
        assert!(engine.prepare(SAMPLE_RATE as f64));
        assert!(engine.load_program("original-14-percussive-e-piano"));
        for _ in 0..96_000 {
            let _ = engine.next_sample();
        }

        engine.note_on(0, 50, 127);
        let samples: Vec<f32> = (0..7_680).map(|_| engine.next_sample()).collect();
        let attack = &samples[1_440..7_680];
        let fundamental = windowed_frequency_magnitude(attack, SAMPLE_RATE, 146.832_38);
        let octave = windowed_frequency_magnitude(attack, SAMPLE_RATE, 293.664_76);
        let ratio = octave / fundamental;

        // The Rev 3 recording places both bands within about 0.1 dB. 1-4's
        // PW A code 59 lands near 51% on a typical CEM3340, where the pulse's
        // own second harmonic nearly cancels; the data sheet's 4.6-5.4 V PWM
        // span moves that device point by several percent and the band ratio
        // by more than 10 dB. Require the sync/Poly Mod octave to be clearly
        // present rather than a single device's exact duty.
        assert!(
            (0.2..=1.50).contains(&ratio),
            "factory 1-4 octave/fundamental attack ratio {ratio}"
        );
    }

    #[test]
    fn host_note_is_admitted_after_its_first_pitch_sweep() {
        let mut engine = Engine::default();
        assert!(engine.set_parameter(Parameter::FilterKeyboard as u32, 1.0));
        assert!(engine.prepare(48_000.0));
        engine.note_on(0, 60, 100);

        assert!(engine.voices[0].matches(0, 60));
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
        assert!((engine.cv.filter_keyboard_octaves(0) - 2.0).abs() < 1.0e-5);
    }

    #[test]
    fn host_note_admission_is_immediate_at_every_audio_rate() {
        for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
            let mut engine = Engine::default();
            assert!(engine.prepare(sample_rate));
            let held = engine.cv.oscillator_semitones(0, false);
            engine.note_on(0, 60, 100);
            assert!((engine.cv.oscillator_semitones(0, false) - held).abs() > 1.0);
        }
    }

    #[test]
    fn voice_reassignment_never_exposes_the_previous_pitch() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        engine.note_on(0, 36, 100);
        for _ in 0..288 {
            let _ = engine.next_sample();
        }
        let low = engine.cv.oscillator_semitones(0, false);
        engine.reset_voices();
        engine.note_on(0, 84, 100);
        let high = engine.cv.oscillator_semitones(0, false);
        assert!(high - low > 47.9);
    }

    #[test]
    fn unison_routes_keyboard_pitch_through_its_common_sample_hold() {
        let mut engine = Engine::default();
        assert!(engine.set_parameter(Parameter::FilterKeyboard as u32, 1.0));
        assert!(engine.set_parameter(Parameter::Unison as u32, 1.0));
        assert!(engine.prepare(48_000.0));
        engine.note_on(0, 60, 100);
        assert!(engine.glide_waiting_for_unison_cv);
        for _ in 0..288 {
            let _ = engine.next_sample();
        }

        let settings = engine.controls.current(engine.settings);
        let full_pitch = tuning::oscillator_a_tuning_semitones(
            60,
            settings.get(Parameter::OscillatorAFrequency),
        );
        assert!(!engine.glide_waiting_for_unison_cv);
        let unison_keyboard = engine.cv.unison_keyboard_semitones();
        assert!(
            (unison_keyboard - 24.0).abs() < 0.001,
            "unison keyboard semitones: {unison_keyboard}"
        );
        assert!((engine.cv.oscillator_semitones(0, false) - (full_pitch - 24.0)).abs() < 0.03);
        assert!(engine.cv.filter_keyboard_octaves(0).abs() < 1.0e-5);
    }

    #[test]
    fn pitch_wheel_uses_the_full_fourteen_bit_midi_range() {
        assert_eq!(pitch_wheel::normalized_output(0), -1.0);
        assert_eq!(pitch_wheel::normalized_output(8_192), 0.0);
        assert_eq!(pitch_wheel::normalized_output(16_383), 1.0);

        let mut engine = Engine::default();
        engine.handle_midi([0xe0, 0, 127]);
        assert_eq!(engine.pitch_wheel, pitch_wheel::normalized_output(16_256));
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

    /// What makes one program cost twice what another does.
    ///
    /// On the appliance a voice of "1-4 Percussive e Piano" costs 2367 us a
    /// block and one of "2-8 Bass in Fifths" costs 1333. Same engine, same
    /// five voices: the difference is entirely in the patch, and until it is
    /// named it cannot be attacked.
    ///
    /// ```text
    /// cargo test --release -p rf-5-dsp -- --ignored --nocapture print_program_difference
    /// ```
    #[test]
    #[ignore = "a measurement, not a check"]
    fn print_program_difference() {
        let read = |id: &str| {
            let mut engine = Engine::default();
            assert!(engine.prepare(48_000.0));
            assert!(engine.load_program(id), "no se pudo cargar {id}");
            engine.settings
        };
        let costly = read("original-14-percussive-e-piano");
        let cheap = read("original-28-bass-in-fifths");
        std::println!(
            "{:<40} {:>10} {:>10}",
            "parametro",
            "1-4 caro",
            "2-8 barato"
        );
        for index in 0..PARAMETER_COUNT as u32 {
            let Ok(parameter) = Parameter::try_from(index) else {
                continue;
            };
            let (a, b) = (costly.get(parameter), cheap.get(parameter));
            if (a - b).abs() > 1e-6 {
                std::println!("{:<40} {a:>10.3} {b:>10.3}", std::format!("{parameter:?}"));
            }
        }
    }

    #[test]
    fn state_and_programs_round_trip() {
        let mut engine = Engine::default();
        assert!(engine.set_parameter(Parameter::ScaleE as u32, 46.0 / 127.0));
        assert!(engine.load_diagnostic_program("baseline-pad"));
        let expected = engine.settings();
        let mut state = [0_u8; STATE_BYTES];
        assert_eq!(engine.save_state(&mut state), Some(STATE_BYTES));
        assert!(engine.load_diagnostic_program("baseline-lead"));
        assert!(engine.load_state(&state));
        assert_eq!(engine.settings(), expected);
    }

    #[test]
    fn patch_programs_do_not_replace_the_active_scale_program() {
        let mut engine = Engine::default();
        assert!(engine.set_parameter(Parameter::ScaleCSharp as u32, 79.0 / 127.0));
        assert!(engine.set_parameter(Parameter::ScaleE as u32, 46.0 / 127.0));
        let expected = engine.settings.scale_values();
        assert!(engine.load_diagnostic_program("baseline-pad"));
        assert!(engine.load_diagnostic_program("baseline-lead"));
        assert_eq!(engine.settings.scale_values(), expected);
    }

    #[test]
    fn legacy_patch_only_state_loads_with_equal_temperament() {
        let mut legacy = [0_u8; PRE_SCALE_STATE_BYTES];
        let values = Settings::default().as_array();
        let (chunks, remainder) = legacy.as_chunks_mut::<4>();
        assert!(remainder.is_empty());
        for (chunk, value) in chunks
            .iter_mut()
            .zip(values[..PRE_SCALE_PATCH_PARAMETER_COUNT].iter())
        {
            chunk.copy_from_slice(&value.to_le_bytes());
        }
        let mut engine = Engine::default();
        assert!(engine.set_parameter(Parameter::ScaleE as u32, 0.0));
        assert!(engine.load_state(&legacy));
        assert_eq!(
            engine.settings.scale_values(),
            [rf_5_contract::hardware::SCALE_EQUAL_TEMPERAMENT_NORMALIZED;
                rf_5_contract::SCALE_NOTE_COUNT]
        );
        assert_eq!(engine.settings.get(Parameter::ReleaseSwitch), 1.0);
    }

    #[test]
    fn pre_release_state_migrates_scale_values_and_enables_release() {
        let mut old_values = [0.0_f32; PRE_RELEASE_PARAMETER_COUNT];
        let current = Settings::default().as_array();
        old_values[..PRE_SCALE_PATCH_PARAMETER_COUNT]
            .copy_from_slice(&current[..PRE_SCALE_PATCH_PARAMETER_COUNT]);
        old_values[PRE_SCALE_PATCH_PARAMETER_COUNT..]
            .copy_from_slice(&current[Parameter::ScaleC as usize..Parameter::MasterTune as usize]);
        old_values[Parameter::ScaleE as usize - 1] = 0.31;
        let mut state = [0_u8; PRE_RELEASE_STATE_BYTES];
        for (chunk, value) in state.as_chunks_mut::<4>().0.iter_mut().zip(old_values) {
            chunk.copy_from_slice(&value.to_le_bytes());
        }

        let mut engine = Engine::default();
        assert!(engine.set_parameter(Parameter::ReleaseSwitch as u32, 0.0));
        assert!(engine.load_state(&state));
        assert_eq!(engine.settings.get(Parameter::ReleaseSwitch), 1.0);
        assert_eq!(engine.settings.get(Parameter::ScaleE), 0.31);
    }

    #[test]
    fn pre_master_tune_state_migrates_to_the_centre_detent() {
        let mut old_values = Settings::default().as_array();
        old_values[Parameter::ScaleE as usize] = 0.31;
        let mut state = [0_u8; PRE_MASTER_TUNE_STATE_BYTES];
        for (chunk, value) in state
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(old_values[..PRE_MASTER_TUNE_PARAMETER_COUNT].iter())
        {
            chunk.copy_from_slice(&value.to_le_bytes());
        }

        let mut engine = Engine::default();
        assert!(engine.set_parameter(Parameter::MasterTune as u32, 1.0));
        assert!(engine.load_state(&state));
        assert_eq!(engine.settings.get(Parameter::ScaleE), 0.31);
        assert_eq!(engine.settings.get(Parameter::MasterTune), 0.5);
    }

    #[test]
    fn pre_machine_operation_state_defaults_a440_off_and_tune_ready() {
        let mut old_values = Settings::default().as_array();
        old_values[Parameter::MasterTune as usize] = 0.31;
        let mut state = [0_u8; PRE_MACHINE_OPERATIONS_STATE_BYTES];
        for (chunk, value) in state
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(old_values[..PRE_MACHINE_OPERATIONS_PARAMETER_COUNT].iter())
        {
            chunk.copy_from_slice(&value.to_le_bytes());
        }

        let mut engine = Engine::default();
        assert!(engine.set_parameter(Parameter::A440 as u32, 1.0));
        assert!(engine.request_tune());
        assert!(engine.load_state(&state));
        assert_eq!(engine.settings.get(Parameter::MasterTune), 0.31);
        assert_eq!(engine.settings.get(Parameter::A440), 0.0);
        assert_eq!(engine.parameter(Parameter::Tune as u32), Some(0.0));
    }

    #[test]
    fn pre_tail_mute_state_enables_the_tail_cut() {
        let mut old_values = Settings::default().as_array();
        old_values[Parameter::VintageSpread as usize] = 0.4;
        let mut state = [0_u8; PRE_TAIL_MUTE_STATE_BYTES];
        for (chunk, value) in state
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(old_values[..PRE_TAIL_MUTE_PARAMETER_COUNT].iter())
        {
            chunk.copy_from_slice(&value.to_le_bytes());
        }

        let mut engine = Engine::default();
        assert!(engine.set_parameter(Parameter::ProgramChangeMutesTails as u32, 0.0));
        assert!(engine.load_state(&state));
        assert_eq!(engine.settings.get(Parameter::VintageSpread), 0.4);
        assert_eq!(engine.settings.get(Parameter::ProgramChangeMutesTails), 1.0);
    }

    /// Plays a note into its release and holds another, recalls a program,
    /// and reports whether each voice is still sounding 60 ms later.
    fn voices_after_program_change(cut_tails: bool) -> (bool, bool) {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        assert!(engine.load_program("original-38-echo-repeat"));
        assert!(engine.set_parameter(
            Parameter::ProgramChangeMutesTails as u32,
            if cut_tails { 1.0 } else { 0.0 }
        ));
        engine.note_on(0, 60, 100);
        for _ in 0..24_000 {
            let _ = engine.next_sample();
        }
        engine.note_off(0, 60);
        engine.note_on(0, 67, 100);
        for _ in 0..4_800 {
            let _ = engine.next_sample();
        }
        let released = (0..VOICE_COUNT)
            .find(|&unit| engine.voices[unit].note() == 60)
            .unwrap();
        let held = (0..VOICE_COUNT)
            .find(|&unit| engine.voices[unit].matches(0, 67))
            .unwrap();
        assert!(
            engine.voices[released].is_active(),
            "the tail should still ring"
        );

        assert!(engine.load_program("original-11-brass"));
        for _ in 0..2_880 {
            let _ = engine.next_sample();
        }
        (
            engine.voices[released].is_active(),
            engine.voices[held].is_active(),
        )
    }

    #[test]
    fn program_change_cuts_only_releasing_tails() {
        let (tail, held) = voices_after_program_change(true);
        assert!(!tail, "the releasing voice kept ringing on the new program");
        assert!(
            held,
            "a held note must take the new program, as on the original"
        );
    }

    #[test]
    fn original_program_change_lets_tails_finish_on_the_new_program() {
        let (tail, held) = voices_after_program_change(false);
        assert!(tail && held);
    }

    #[test]
    fn audition_wheel_is_temporary_machine_state() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        assert!(engine.load_diagnostic_program("audition-wheel-vibrato"));
        assert_eq!(engine.audition_mod_wheel, Some(0.42));

        // Hosts may restore a preset before (re)starting the audio device.
        assert!(engine.prepare(96_000.0));
        assert_eq!(engine.audition_mod_wheel, Some(0.42));

        let mut state = [0_u8; STATE_BYTES];
        assert_eq!(engine.save_state(&mut state), Some(STATE_BYTES));
        assert!(engine.load_state(&state));
        assert_eq!(engine.audition_mod_wheel, None);

        assert!(engine.load_diagnostic_program("audition-wheel-filter"));
        assert!(engine.audition_mod_wheel.is_some());
        assert!(engine.load_diagnostic_program("baseline-warm"));
        assert_eq!(engine.audition_mod_wheel, None);
    }

    #[test]
    fn real_mod_wheel_immediately_replaces_audition_override() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        assert!(engine.load_diagnostic_program("audition-wheel-pwm"));
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
            assert!(engine.load_diagnostic_program(id));
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
    fn original_programs_are_finite_and_audible() {
        for program in original_programs_data::ORIGINAL_PROGRAMS {
            let mut engine = Engine::default();
            assert!(engine.prepare(48_000.0));
            assert!(engine.load_program(program.id));
            engine.note_on(0, 48, 127);

            let mut peak = 0.0_f32;
            for _ in 0..96_000 {
                let sample = engine.next_sample();
                assert!(
                    sample.is_finite(),
                    "non-finite original program {}",
                    program.id
                );
                peak = peak.max(sample.abs());
            }
            assert!(peak > 1.0e-4, "silent original program {}", program.id);
        }
    }

    #[test]
    fn original_harpsichord_retains_high_register_level() {
        fn strike_rms(note: u8) -> f32 {
            const SAMPLE_RATE: usize = 48_000;
            let mut engine = Engine::default();
            assert!(engine.prepare(SAMPLE_RATE as f64));
            assert!(engine.load_program("original-16-harpsichord"));
            for _ in 0..SAMPLE_RATE * 2 {
                let _ = engine.next_sample();
            }
            engine.note_on(0, note, 127);
            let mut energy = 0.0;
            let mut count = 0;
            for frame in 0..SAMPLE_RATE * 3 / 10 {
                let sample = engine.next_sample();
                if frame >= SAMPLE_RATE / 200 {
                    energy += sample * sample;
                    count += 1;
                }
            }
            libm::sqrtf(energy / count as f32)
        }

        let low = strike_rms(36);
        let high = strike_rms(96);
        assert!(
            high / low >= 0.45,
            "harpsichord high-register level collapsed: low={low}, high={high}"
        );
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
            assert!(engine.load_diagnostic_program(id));
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
            assert!(
                peak <= 1.0,
                "unbounded filter audition program {id}: peak={peak}",
            );
        }
        assert!((signatures[0] - signatures[1]).abs() > 1.0);
    }

    #[test]
    fn oscillator_tune_reconditions_machine_state_without_changing_the_patch() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        assert!(engine.load_diagnostic_program("baseline-pad"));
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
    fn momentary_tune_reports_busy_then_completes_without_entering_state() {
        let mut engine = Engine::default();
        assert!(engine.prepare(100.0));
        let expected = engine.settings();
        assert!(engine.set_parameter(Parameter::Tune as u32, 1.0));
        assert!(engine.is_tuning());
        assert_eq!(engine.tune_duration_seconds(), 2.0);
        assert_eq!(engine.parameter(Parameter::Tune as u32), Some(1.0));
        assert_eq!(engine.settings.get(Parameter::Tune), 0.0);
        for _ in 0..199 {
            assert_eq!(engine.next_sample(), 0.0);
            assert!(engine.is_tuning());
        }
        assert_eq!(engine.next_sample(), 0.0);
        assert!(!engine.is_tuning());
        assert_eq!(engine.parameter(Parameter::Tune as u32), Some(0.0));
        assert_eq!(engine.settings(), expected);

        let mut state = [0_u8; STATE_BYTES];
        assert_eq!(engine.save_state(&mut state), Some(STATE_BYTES));
        let tune_offset = Parameter::Tune as usize * 4;
        assert_eq!(&state[tune_offset..tune_offset + 4], &0.0_f32.to_le_bytes());
    }

    #[test]
    fn filter_summer_rail_limits_the_wheel_upswing_at_high_cutoff() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        for (parameter, value) in [
            (Parameter::FilterCutoff, 1.0),
            (Parameter::WheelModFilter, 1.0),
            (Parameter::WheelModSourceMix, 0.0),
            (Parameter::LfoSquare, 1.0),
            (Parameter::LfoFrequency, 0.5),
            (Parameter::Unison, 1.0),
            (Parameter::FilterKeyboard, 1.0),
        ] {
            assert!(engine.set_parameter(parameter as u32, value));
        }
        engine.handle_midi([0xb0, 1, 127]);
        // Eighteen keys up, the unison keyboard CV leaves U367 less headroom
        // than a full wheel's upswing of about two volts.
        engine.note_on(0, tuning::LOWEST_KEY_MIDI_NOTE + 18, 100);
        let cutoff_volts = rf_5_contract::hardware::normalized_control_volts(1.0) + 1.5;
        let mut highest = f32::NEG_INFINITY;
        let mut lowest = f32::INFINITY;
        // The scanned FILT CUTOFF pot walks to its code first.
        for index in 0..192_000 {
            let octaves = engine
                .prepare_next_sample()
                .common
                .modulation
                .filter_octaves;
            if index >= 96_000 {
                highest = highest.max(octaves);
                lowest = lowest.min(octaves);
            }
        }
        // U367 cannot pass the full x7.5 Wheel Mod swing above FILT CUTOFF
        // and the unison keyboard.
        assert!(
            highest <= FILTER_SUMMER_SWING_VOLTS - cutoff_volts + 0.05,
            "{highest}"
        );
        assert!(
            highest > FILTER_SUMMER_SWING_VOLTS - cutoff_volts - 0.05 && lowest < -1.0,
            "{lowest} {highest}"
        );
    }

    #[test]
    fn a440_is_audible_without_midi_and_obeys_master_volume() {
        let mut open = Engine::default();
        let mut closed = Engine::default();
        for engine in [&mut open, &mut closed] {
            assert!(engine.prepare(48_000.0));
            assert!(engine.set_parameter(Parameter::A440 as u32, 1.0));
        }
        assert!(closed.set_parameter(Parameter::MasterVolume as u32, 0.0));

        let mut open_energy = 0.0;
        for _ in 0..8_192 {
            let sample = open.next_sample();
            assert!(sample.is_finite());
            open_energy += sample * sample;
            assert_eq!(closed.next_sample(), 0.0);
        }
        assert!(open_energy > 1.0);
    }

    #[test]
    fn loading_a_program_preserves_the_physical_master_volume() {
        let mut engine = Engine::default();
        assert!(engine.set_parameter(Parameter::MasterVolume as u32, 0.31));
        assert!(engine.set_parameter(Parameter::MasterTune as u32, 0.73));
        assert!(engine.set_parameter(Parameter::A440 as u32, 1.0));
        assert!(engine.load_diagnostic_program("baseline-pad"));
        assert_eq!(
            engine.parameter(Parameter::MasterVolume as u32),
            Some(0.31_f32 as f64)
        );
        assert_eq!(
            engine.parameter(Parameter::MasterTune as u32),
            Some(0.73_f32 as f64)
        );
        assert_eq!(engine.parameter(Parameter::A440 as u32), Some(1.0));
    }

    #[test]
    fn loading_a_program_preserves_non_program_machine_character() {
        let mut engine = Engine::default();
        assert!(engine.set_parameter(Parameter::VintageSpread as u32, 0.63));
        assert!(engine.load_diagnostic_program("baseline-pad"));
        assert_eq!(engine.settings.get(Parameter::VintageSpread), 0.63);
    }

    #[test]
    fn oscillator_candidate_is_finite_across_supported_sample_rates() {
        for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
            let mut engine = Engine::default();
            assert!(engine.prepare(sample_rate));
            assert!(engine.load_diagnostic_program("baseline-lead"));
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
        assert!(engine.load_diagnostic_program("baseline-warm"));
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
        assert!(dry.load_diagnostic_program("baseline-lead"));
        assert!(modulated.load_diagnostic_program("baseline-lead"));
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
    fn midi_mod_wheel_reconstructs_continuous_travel_without_controller_steps() {
        for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
            let mut engine = Engine::default();
            assert!(engine.prepare(sample_rate));
            engine.handle_midi([0xb0, 1, 127]);

            let first = engine.advance_mod_wheel();
            assert!(
                first > 0.0 && first < 0.01,
                "first={first} at {sample_rate}"
            );
            let samples = (sample_rate as f32 * MOD_WHEEL_DEZIPPER_TIME_SECONDS) as usize;
            for _ in 1..samples {
                let _ = engine.advance_mod_wheel();
            }
            assert!(
                (engine.smoothed_mod_wheel - 0.632_120_55).abs() < 0.002,
                "smoothed={} at {sample_rate}",
                engine.smoothed_mod_wheel
            );
        }
    }

    #[test]
    fn both_noise_sources_free_run_and_note_events_do_not_reset_them() {
        let mut engine = Engine::default();
        assert!(engine.prepare(48_000.0));
        let initial_wheel_state = engine.wheel_noise.state();
        let initial_audio_state = engine.audio_noise.state();
        for _ in 0..257 {
            assert_eq!(engine.next_sample(), 0.0);
        }
        let free_running_wheel_state = engine.wheel_noise.state();
        let free_running_audio_state = engine.audio_noise.state();
        assert_ne!(free_running_wheel_state, initial_wheel_state);
        assert_ne!(free_running_audio_state, initial_audio_state);
        assert_ne!(free_running_wheel_state, free_running_audio_state);
        engine.note_on(0, 60, 100);
        assert_eq!(engine.wheel_noise.state(), free_running_wheel_state);
        assert_eq!(engine.audio_noise.state(), free_running_audio_state);
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
