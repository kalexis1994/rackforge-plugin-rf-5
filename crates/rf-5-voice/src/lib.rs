#![no_std]

//! Per-voice synthesis path with dual CEM3340-class VCO candidates, two
//! CEM3310 true-RC envelopes and an oversampled four-pole CEM3320 candidate.

use rf_5_contract::{Parameter, Settings, hardware::quantize_analog_pot};

pub mod autotune;
mod decimator;
pub mod drift;
pub mod envelope;
pub mod filter;
pub mod poly_mod;
mod pulse_width;
pub mod scale;
#[cfg(test)]
mod spectral_tests;
pub mod tuning;
pub mod vca;
pub mod vco;

use envelope::AdsrEnvelope;
use filter::Cem3320Filter;
pub use tuning::note_frequency;
use vco::{Vco, WaveSelection};

const INITIAL_PHASE_A: [f32; 5] = [0.07, 0.31, 0.58, 0.83, 0.19];
const INITIAL_PHASE_B: [f32; 5] = [0.67, 0.11, 0.42, 0.74, 0.93];
const OSCILLATOR_OVERSAMPLING: usize = 4;
const FILTER_MINIMUM_HZ: f32 = 16.351_599;
const FILTER_PANEL_OCTAVES: f32 = 10.0;
const FILTER_KEYBOARD_BASE_NOTE: f32 = 36.0;
const RELEASE_SWITCH_OFF_CONTROL: f32 = 0.0;
#[cfg(test)]
const FILTER_SERVICE_CV_PANEL_POSITION: f32 = 0.2;
#[cfg(test)]
const FILTER_SERVICE_REFERENCE_NOTE: u8 = 69;
#[cfg(test)]
const FILTER_SERVICE_REFERENCE_HZ: f32 = 440.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct VoiceModulation {
    pub oscillator_a_semitones: f32,
    pub oscillator_b_semitones: f32,
    pub oscillator_a_pulse_width: f32,
    pub oscillator_b_pulse_width: f32,
    pub filter_octaves: f32,
    pub noise: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Voice {
    note: u8,
    channel: u8,
    active: bool,
    oscillator_a: Vco,
    oscillator_b: Vco,
    oscillators_initialized: bool,
    voice_index: usize,
    amplifier_envelope: AdsrEnvelope,
    filter_envelope: AdsrEnvelope,
    filter: Cem3320Filter,
    decimator: decimator::Decimator4x,
}

impl Voice {
    pub fn note(self) -> u8 {
        self.note
    }

    pub fn is_active(self) -> bool {
        self.active
    }

    pub fn matches(self, channel: u8, note: u8) -> bool {
        self.active && self.channel == channel && self.note == note
    }

    pub fn identity(self) -> Option<(u8, u8)> {
        self.active.then_some((self.channel, self.note))
    }

    pub fn start(&mut self, channel: u8, note: u8, _velocity: u8, voice_index: usize) {
        self.note = note;
        self.channel = channel;
        self.voice_index = voice_index % INITIAL_PHASE_A.len();
        self.active = true;
        if !self.oscillators_initialized {
            let index = self.voice_index;
            self.oscillator_a = Vco::with_phase_and_profile(INITIAL_PHASE_A[index], index * 2);
            self.oscillator_b = Vco::with_phase_and_profile(INITIAL_PHASE_B[index], index * 2 + 1);
            self.filter = Cem3320Filter::with_profile(index);
            self.amplifier_envelope = AdsrEnvelope::with_profile(index * 2);
            self.filter_envelope = AdsrEnvelope::with_profile(index * 2 + 1);
            self.oscillators_initialized = true;
        }
        self.amplifier_envelope.trigger();
        self.filter_envelope.trigger();
    }

    pub fn retune(&mut self, channel: u8, note: u8) {
        self.note = note;
        self.channel = channel;
    }

    pub fn release(&mut self) {
        if !self.active {
            return;
        }
        self.amplifier_envelope.release();
        self.filter_envelope.release();
    }

    pub fn next(
        &mut self,
        sample_rate: f32,
        settings: Settings,
        modulation: VoiceModulation,
    ) -> f32 {
        let allocated = self.active;
        let release_enabled = parameter_enabled(settings, Parameter::ReleaseSwitch);
        let filter_release = if release_enabled {
            settings.get(Parameter::FilterRelease)
        } else {
            RELEASE_SWITCH_OFF_CONTROL
        };
        let amplifier_release = if release_enabled {
            settings.get(Parameter::AmpRelease)
        } else {
            RELEASE_SWITCH_OFF_CONTROL
        };
        let filter_envelope = self.filter_envelope.next(
            sample_rate,
            quantize_analog_pot(settings.get(Parameter::FilterAttack)),
            quantize_analog_pot(settings.get(Parameter::FilterDecay)),
            quantize_analog_pot(settings.get(Parameter::FilterSustain)),
            quantize_analog_pot(filter_release),
        );
        let amplifier_envelope = self.amplifier_envelope.next(
            sample_rate,
            quantize_analog_pot(settings.get(Parameter::AmpAttack)),
            quantize_analog_pot(settings.get(Parameter::AmpDecay)),
            quantize_analog_pot(settings.get(Parameter::AmpSustain)),
            quantize_analog_pot(amplifier_release),
        );
        if self.amplifier_envelope.is_idle() {
            self.active = false;
        }
        let voice_output = self.next_signal_path(
            sample_rate,
            settings,
            modulation,
            filter_envelope,
            amplifier_envelope,
        );
        if allocated { voice_output } else { 0.0 }
    }

    fn next_signal_path(
        &mut self,
        sample_rate: f32,
        settings: Settings,
        modulation: VoiceModulation,
        filter_envelope: f32,
        amplifier_envelope: f32,
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
        let pulse_width_a = pulse_width::add_modulation(
            pulse_width::panel_duty_cycle(settings.get(Parameter::OscillatorAPulseWidth)),
            modulation.oscillator_a_pulse_width,
        );
        let pulse_width_b = pulse_width::add_modulation(
            pulse_width::panel_duty_cycle(settings.get(Parameter::OscillatorBPulseWidth)),
            modulation.oscillator_b_pulse_width,
        );
        let level_a = quantize_analog_pot(settings.get(Parameter::OscillatorALevel));
        let level_b = quantize_analog_pot(settings.get(Parameter::OscillatorBLevel));
        let sync = parameter_enabled(settings, Parameter::OscillatorSync);
        let frequency_a = tuning::oscillator_a_frequency(
            self.note,
            settings.get(Parameter::OscillatorAFrequency),
        );
        let frequency_b = tuning::oscillator_b_frequency(
            self.note,
            settings.get(Parameter::OscillatorBFrequency),
            settings.get(Parameter::OscillatorBDetune),
            parameter_enabled(settings, Parameter::OscillatorBKeyboard),
            parameter_enabled(settings, Parameter::OscillatorBLowFrequency),
        ) * semitone_ratio(modulation.oscillator_b_semitones);
        let internal_rate = sample_rate.max(1.0) * OSCILLATOR_OVERSAMPLING as f32;
        let mut output = None;
        let poly_filter_envelope_current = -vca::poly_mod_filter_envelope_current_amps(
            filter_envelope,
            quantize_analog_pot(settings.get(Parameter::PolyModFilterEnvelopeAmount)),
            self.voice_index,
        );
        let poly_oscillator_b_amount =
            quantize_analog_pot(settings.get(Parameter::PolyModOscillatorBAmount));
        let poly_frequency_a = parameter_enabled(settings, Parameter::PolyModOscillatorAFrequency);
        let poly_pulse_width_a =
            parameter_enabled(settings, Parameter::PolyModOscillatorAPulseWidth);
        let poly_filter = parameter_enabled(settings, Parameter::PolyModFilter);
        let filter_resonance = quantize_analog_pot(settings.get(Parameter::FilterResonance));
        let direct_filter_envelope = vca::filter_envelope_cutoff_octaves(
            filter_envelope,
            quantize_analog_pot(settings.get(Parameter::FilterEnvelopeAmount)),
            self.voice_index,
        );
        let filter_cutoff = quantize_analog_pot(settings.get(Parameter::FilterCutoff));
        let filter_keyboard = parameter_enabled(settings, Parameter::FilterKeyboard);
        let common_filter_octaves = direct_filter_envelope + modulation.filter_octaves;
        let amplifier_vca_control = vca::amplifier_envelope_control(amplifier_envelope);

        for _ in 0..OSCILLATOR_OVERSAMPLING {
            let sample_b =
                self.oscillator_b
                    .next(frequency_b, internal_rate, pulse_width_b, waves_b);
            let poly_oscillator_b_current = vca::poly_mod_oscillator_b_current_amps(
                sample_b.poly_mod_source_volts,
                sample_b.poly_mod_source_conductance,
                poly_oscillator_b_amount,
                self.voice_index,
            );
            let poly_bus_volts =
                vca::poly_mod_bus_voltage(poly_filter_envelope_current, poly_oscillator_b_current);
            let poly_destinations = poly_mod::destinations(poly_bus_volts);
            let poly_pitch = if poly_frequency_a {
                poly_destinations.oscillator_a_semitones
            } else {
                0.0
            };
            let poly_pulse_width = if poly_pulse_width_a {
                poly_destinations.oscillator_a_pulse_width
            } else {
                0.0
            };
            let sync_events = if sync {
                sample_b.sync_events
            } else {
                [vco::HardSyncEvent::default(); 2]
            };
            let sample_a = self.oscillator_a.next_with_sync(
                frequency_a * semitone_ratio(modulation.oscillator_a_semitones + poly_pitch),
                internal_rate,
                pulse_width::add_modulation(pulse_width_a, poly_pulse_width),
                waves_a,
                sync_events,
            );
            let poly_filter_octaves = if poly_filter {
                poly_destinations.filter_octaves
            } else {
                0.0
            };
            let cutoff_hz = filter_cutoff_hz(
                filter_cutoff,
                self.note,
                filter_keyboard,
                common_filter_octaves + poly_filter_octaves,
            );
            let mixer = vca::oscillator_mixer_loaded(
                sample_a.mixer_positive_source_volts,
                sample_a.mixer_positive_source_conductance,
                sample_a.mixer_negative_source_volts,
                sample_a.mixer_negative_source_conductance,
                level_a,
                self.voice_index,
                vca::MixerChannel::OscillatorA,
            ) + vca::oscillator_mixer_loaded(
                sample_b.mixer_positive_source_volts,
                sample_b.mixer_positive_source_conductance,
                sample_b.mixer_negative_source_volts,
                sample_b.mixer_negative_source_conductance,
                level_b,
                self.voice_index,
                vca::MixerChannel::OscillatorB,
            ) + modulation.noise;
            let filtered = self.filter.next_with_character(
                mixer,
                cutoff_hz,
                filter_resonance,
                internal_rate,
                settings.get(Parameter::VintageSpread),
            );
            output = self.decimator.push(vca::final_voice(
                filtered,
                amplifier_vca_control,
                self.voice_index,
            ));
        }

        output.unwrap_or(0.0)
    }
}

fn parameter_enabled(settings: Settings, parameter: Parameter) -> bool {
    settings.get(parameter) >= 0.5
}

fn semitone_ratio(semitones: f32) -> f32 {
    libm::powf(2.0, semitones / 12.0)
}

fn filter_cutoff_hz(panel: f32, note: u8, keyboard_tracking: bool, modulation_octaves: f32) -> f32 {
    let keyboard_octaves = if keyboard_tracking {
        (f32::from(note) - FILTER_KEYBOARD_BASE_NOTE) / 12.0
    } else {
        0.0
    };
    FILTER_MINIMUM_HZ
        * libm::powf(
            2.0,
            panel.clamp(0.0, 1.0) * FILTER_PANEL_OCTAVES + keyboard_octaves + modulation_octaves,
        )
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
    fn filter_keyboard_switch_tracks_one_volt_per_octave() {
        let lower = filter_cutoff_hz(0.4, 36, true, 0.0);
        let upper = filter_cutoff_hz(0.4, 48, true, 0.0);
        assert!((upper / lower - 2.0).abs() < 1.0e-5);

        let lower_off = filter_cutoff_hz(0.4, 36, false, 0.0);
        let upper_off = filter_cutoff_hz(0.4, 48, false, 0.0);
        assert_eq!(lower_off, upper_off);
    }

    #[test]
    fn filter_panel_spans_ten_octaves() {
        let bottom = filter_cutoff_hz(0.0, 36, false, 0.0);
        let top = filter_cutoff_hz(1.0, 36, false, 0.0);
        assert!((top / bottom - 1_024.0).abs() < 0.01);
    }

    #[test]
    fn filter_service_voltage_anchor_produces_440_and_880_hz() {
        let a3 = filter_cutoff_hz(
            FILTER_SERVICE_CV_PANEL_POSITION,
            FILTER_SERVICE_REFERENCE_NOTE,
            true,
            0.0,
        );
        let a4 = filter_cutoff_hz(
            FILTER_SERVICE_CV_PANEL_POSITION,
            FILTER_SERVICE_REFERENCE_NOTE + 12,
            true,
            0.0,
        );
        assert!((a3 - FILTER_SERVICE_REFERENCE_HZ).abs() < 0.001);
        assert!((a4 - FILTER_SERVICE_REFERENCE_HZ * 2.0).abs() < 0.001);
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
        voice.release();
        for _ in 0..500_000 {
            let _ = voice.next(48_000.0, settings, VoiceModulation::default());
            if !voice.is_active() {
                break;
            }
        }
        assert!(!voice.is_active());
    }

    #[test]
    fn release_switch_off_uses_the_global_minimum_time() {
        let mut enabled = Settings::default();
        assert!(enabled.set(Parameter::AmpRelease as u32, 1.0));
        assert!(enabled.set(Parameter::FilterRelease as u32, 1.0));
        let mut disabled = enabled;
        assert!(disabled.set(Parameter::ReleaseSwitch as u32, 0.0));
        let mut long = Voice::default();
        let mut short = Voice::default();
        long.start(0, 60, 100, 0);
        short.start(0, 60, 100, 0);
        for _ in 0..4_096 {
            let _ = long.next(48_000.0, enabled, VoiceModulation::default());
            let _ = short.next(48_000.0, disabled, VoiceModulation::default());
        }
        long.release();
        short.release();
        for _ in 0..48_000 {
            let _ = long.next(48_000.0, enabled, VoiceModulation::default());
            let _ = short.next(48_000.0, disabled, VoiceModulation::default());
        }
        assert!(long.is_active());
        assert!(!short.is_active());
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
    fn original_keyboard_dynamics_do_not_scale_voice_level() {
        let settings = Settings::default();
        let mut quiet_velocity = Voice::default();
        let mut full_velocity = Voice::default();
        quiet_velocity.start(0, 60, 1, 0);
        full_velocity.start(0, 60, 127, 0);
        for _ in 0..4_096 {
            assert_eq!(
                quiet_velocity.next(48_000.0, settings, VoiceModulation::default()),
                full_velocity.next(48_000.0, settings, VoiceModulation::default())
            );
        }
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

    #[test]
    fn filter_envelope_poly_mod_descends_oscillator_a_frequency() {
        let dry_settings = Settings::default();
        let mut modulated_settings = dry_settings;
        assert!(modulated_settings.set(Parameter::PolyModFilterEnvelopeAmount as u32, 1.0));
        assert!(modulated_settings.set(Parameter::PolyModOscillatorAFrequency as u32, 1.0));
        let mut dry = Voice::default();
        let mut modulated = Voice::default();
        dry.start(0, 36, 127, 0);
        modulated.start(0, 36, 127, 0);
        let _ = dry.next_signal_path(48_000.0, dry_settings, VoiceModulation::default(), 1.0, 1.0);
        let _ = modulated.next_signal_path(
            48_000.0,
            modulated_settings,
            VoiceModulation::default(),
            1.0,
            1.0,
        );
        assert!(modulated.oscillator_a.phase() < dry.oscillator_a.phase());
    }

    #[test]
    fn oscillator_b_poly_mod_is_independent_of_b_mixer_level() {
        let mut dry_settings = Settings::default();
        assert!(dry_settings.set(Parameter::OscillatorBLevel as u32, 0.0));
        assert!(dry_settings.set(Parameter::OscillatorBSaw as u32, 0.0));
        assert!(dry_settings.set(Parameter::OscillatorBTriangle as u32, 1.0));
        let mut modulated_settings = dry_settings;
        assert!(modulated_settings.set(Parameter::PolyModOscillatorBAmount as u32, 0.7));
        assert!(modulated_settings.set(Parameter::PolyModOscillatorAFrequency as u32, 1.0));
        let mut dry = Voice::default();
        let mut modulated = Voice::default();
        dry.start(0, 57, 127, 0);
        modulated.start(0, 57, 127, 0);
        let mut difference = 0.0;
        for _ in 0..8_192 {
            difference += (dry.next(48_000.0, dry_settings, VoiceModulation::default())
                - modulated.next(48_000.0, modulated_settings, VoiceModulation::default()))
            .abs();
        }
        assert!(difference > 1.0);
    }

    #[test]
    fn poly_mod_destinations_are_independent_switches() {
        let mut frequency_settings = Settings::default();
        assert!(frequency_settings.set(Parameter::PolyModOscillatorBAmount as u32, 0.5));
        assert!(frequency_settings.set(Parameter::PolyModOscillatorAFrequency as u32, 1.0));
        let mut filter_settings = frequency_settings;
        assert!(filter_settings.set(Parameter::PolyModOscillatorAFrequency as u32, 0.0));
        assert!(filter_settings.set(Parameter::PolyModFilter as u32, 1.0));
        let mut frequency_voice = Voice::default();
        let mut filter_voice = Voice::default();
        frequency_voice.start(0, 60, 127, 0);
        filter_voice.start(0, 60, 127, 0);
        let mut difference = 0.0;
        for _ in 0..8_192 {
            difference +=
                (frequency_voice.next(48_000.0, frequency_settings, VoiceModulation::default())
                    - filter_voice.next(48_000.0, filter_settings, VoiceModulation::default()))
                .abs();
        }
        assert!(difference > 1.0);
    }
}
