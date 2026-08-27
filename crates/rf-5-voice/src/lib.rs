#![no_std]

//! Per-voice synthesis path with dual CEM3340-class VCO candidates, two
//! CEM3310 true-RC envelopes and an oversampled four-pole CEM3320 candidate.

#[cfg(any(
    all(feature = "host-rate", feature = "two-times"),
    all(feature = "host-rate", feature = "hybrid-four-two"),
    all(feature = "two-times", feature = "hybrid-four-two"),
    all(
        feature = "held-filter-two-times",
        any(
            feature = "host-rate",
            feature = "two-times",
            feature = "hybrid-four-two"
        )
    )
))]
compile_error!("host-rate, two-times and hybrid-four-two are mutually exclusive profiles");

use rf_5_contract::{Parameter, Settings, hardware::quantize_analog_pot};

pub mod autotune;
mod decimator;
pub mod drift;
pub mod envelope;
pub mod filter;
pub mod poly_mod;
mod pulse_width;
#[cfg(feature = "fast-math")]
mod realtime_math;
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
#[cfg(not(any(
    feature = "host-rate",
    feature = "two-times",
    feature = "hybrid-four-two",
    feature = "held-filter-two-times"
)))]
const OSCILLATOR_OVERSAMPLING: usize = 4;
#[cfg(feature = "host-rate")]
const OSCILLATOR_OVERSAMPLING: usize = 1;
#[cfg(all(not(feature = "host-rate"), feature = "two-times"))]
const OSCILLATOR_OVERSAMPLING: usize = 2;
#[cfg(feature = "hybrid-four-two")]
const OSCILLATOR_OVERSAMPLING: usize = 4;
#[cfg(feature = "held-filter-two-times")]
const OSCILLATOR_OVERSAMPLING: usize = 4;

#[cfg(any(
    feature = "two-times",
    feature = "hybrid-four-two",
    feature = "held-filter-two-times"
))]
const FILTER_OVERSAMPLING: usize = 2;
#[cfg(feature = "host-rate")]
const FILTER_OVERSAMPLING: usize = 1;
#[cfg(not(any(
    feature = "host-rate",
    feature = "two-times",
    feature = "hybrid-four-two",
    feature = "held-filter-two-times"
)))]
const FILTER_OVERSAMPLING: usize = 4;
const FILTER_MINIMUM_LOG2_HZ: f32 = 4.031_359_7;
const FILTER_PANEL_OCTAVES: f32 = 10.0;
const FILTER_KEYBOARD_BASE_NOTE: f32 = 36.0;
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
struct ControlCurrentCache {
    control_bits: u32,
    current_amps: f32,
}

impl ControlCurrentCache {
    fn get(&mut self, control: f32, prepare: fn(f32) -> f32) -> f32 {
        let control_bits = control.to_bits();
        if self.control_bits != control_bits {
            self.control_bits = control_bits;
            self.current_amps = prepare(control);
        }
        self.current_amps
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct OscillatorFrequencyCache {
    note: u8,
    coarse_bits: u32,
    fine_bits: u32,
    mode: u8,
    frequency_hz: f32,
    valid: bool,
}

impl OscillatorFrequencyCache {
    fn oscillator_a(&mut self, note: u8, coarse: f32) -> f32 {
        let coarse = quantize_analog_pot(coarse);
        if !self.valid || self.note != note || self.coarse_bits != coarse.to_bits() {
            self.note = note;
            self.coarse_bits = coarse.to_bits();
            self.frequency_hz = tuning::oscillator_a_frequency(note, coarse);
            self.valid = true;
        }
        self.frequency_hz
    }

    fn oscillator_b(
        &mut self,
        note: u8,
        coarse: f32,
        fine: f32,
        keyboard_enabled: bool,
        low_frequency: bool,
    ) -> f32 {
        let coarse = quantize_analog_pot(coarse);
        let fine = quantize_analog_pot(fine);
        let mode = u8::from(keyboard_enabled) | (u8::from(low_frequency) << 1);
        if !self.valid
            || self.note != note
            || self.coarse_bits != coarse.to_bits()
            || self.fine_bits != fine.to_bits()
            || self.mode != mode
        {
            self.note = note;
            self.coarse_bits = coarse.to_bits();
            self.fine_bits = fine.to_bits();
            self.mode = mode;
            self.frequency_hz =
                tuning::oscillator_b_frequency(note, coarse, fine, keyboard_enabled, low_frequency);
            self.valid = true;
        }
        self.frequency_hz
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct SemitoneRatioCache {
    semitone_bits: u32,
    ratio: f32,
    valid: bool,
}

impl SemitoneRatioCache {
    fn get(&mut self, semitones: f32) -> f32 {
        if !self.valid || self.semitone_bits != semitones.to_bits() {
            self.semitone_bits = semitones.to_bits();
            self.ratio = semitone_ratio(semitones);
            self.valid = true;
        }
        self.ratio
    }
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
    #[cfg(any(feature = "two-times", feature = "hybrid-four-two"))]
    decimator: decimator::Decimator2x,
    #[cfg(not(any(
        feature = "host-rate",
        feature = "two-times",
        feature = "hybrid-four-two",
        feature = "held-filter-two-times"
    )))]
    decimator: decimator::Decimator4x,
    #[cfg(feature = "held-filter-two-times")]
    decimator: decimator::Decimator4x,
    #[cfg(feature = "hybrid-four-two")]
    mixer_decimator: decimator::WideTransitionDecimator2x,
    #[cfg(feature = "held-filter-two-times")]
    filter_phase: u8,
    #[cfg(feature = "held-filter-two-times")]
    filter_mixer_accumulator: f32,
    #[cfg(feature = "held-filter-two-times")]
    filter_cutoff_accumulator: f32,
    #[cfg(feature = "held-filter-two-times")]
    held_voice_sample: f32,
    oscillator_a_level_current: ControlCurrentCache,
    oscillator_b_level_current: ControlCurrentCache,
    poly_mod_filter_envelope_current: ControlCurrentCache,
    poly_mod_oscillator_b_current: ControlCurrentCache,
    filter_envelope_current: ControlCurrentCache,
    oscillator_a_frequency: OscillatorFrequencyCache,
    oscillator_b_frequency: OscillatorFrequencyCache,
    oscillator_a_modulation_ratio: SemitoneRatioCache,
    oscillator_b_modulation_ratio: SemitoneRatioCache,
}

impl Voice {
    pub fn note(self) -> u8 {
        self.note
    }

    pub fn is_active(self) -> bool {
        self.active
    }

    pub fn is_initialized(self) -> bool {
        self.oscillators_initialized
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
        settings: &Settings,
        modulation: VoiceModulation,
    ) -> f32 {
        // A never-allocated voice has no hardware state to preserve yet. Its
        // phase/profile seeds are installed by `start`, so running the full
        // oversampled signal path here would only create state that `start`
        // immediately discards.
        if !self.oscillators_initialized {
            return 0.0;
        }

        let allocated = self.active;
        let filter_envelope = self.filter_envelope.next(
            sample_rate,
            quantize_analog_pot(settings.get(Parameter::FilterAttack)),
            quantize_analog_pot(settings.get(Parameter::FilterDecay)),
            quantize_analog_pot(settings.get(Parameter::FilterSustain)),
            quantize_analog_pot(settings.get(Parameter::FilterRelease)),
        );
        let amplifier_envelope = self.amplifier_envelope.next(
            sample_rate,
            quantize_analog_pot(settings.get(Parameter::AmpAttack)),
            quantize_analog_pot(settings.get(Parameter::AmpDecay)),
            quantize_analog_pot(settings.get(Parameter::AmpSustain)),
            quantize_analog_pot(settings.get(Parameter::AmpRelease)),
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
            allocated,
        );
        if allocated { voice_output } else { 0.0 }
    }

    fn next_signal_path(
        &mut self,
        sample_rate: f32,
        settings: &Settings,
        modulation: VoiceModulation,
        filter_envelope: f32,
        amplifier_envelope: f32,
        render_audio: bool,
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
        let level_a_control_current = self
            .oscillator_a_level_current
            .get(level_a, vca::oscillator_mixer_control_current_amps);
        let level_b_control_current = self
            .oscillator_b_level_current
            .get(level_b, vca::oscillator_mixer_control_current_amps);
        let sync = parameter_enabled(settings, Parameter::OscillatorSync);
        let frequency_a = self
            .oscillator_a_frequency
            .oscillator_a(self.note, settings.get(Parameter::OscillatorAFrequency));
        let frequency_b = self.oscillator_b_frequency.oscillator_b(
            self.note,
            settings.get(Parameter::OscillatorBFrequency),
            settings.get(Parameter::OscillatorBDetune),
            parameter_enabled(settings, Parameter::OscillatorBKeyboard),
            parameter_enabled(settings, Parameter::OscillatorBLowFrequency),
        ) * self
            .oscillator_b_modulation_ratio
            .get(modulation.oscillator_b_semitones);
        let oscillator_rate = sample_rate.max(1.0) * OSCILLATOR_OVERSAMPLING as f32;
        let filter_rate = sample_rate.max(1.0) * FILTER_OVERSAMPLING as f32;
        let mut output = None;
        let poly_filter_envelope_amount =
            quantize_analog_pot(settings.get(Parameter::PolyModFilterEnvelopeAmount));
        let poly_filter_envelope_control_current = self.poly_mod_filter_envelope_current.get(
            poly_filter_envelope_amount,
            vca::poly_mod_filter_envelope_control_current_amps,
        );
        let poly_filter_envelope_current = -vca::poly_mod_filter_envelope_with_control_current_amps(
            filter_envelope,
            poly_filter_envelope_control_current,
            self.voice_index,
        );
        let poly_oscillator_b_amount =
            quantize_analog_pot(settings.get(Parameter::PolyModOscillatorBAmount));
        let poly_oscillator_b_control_current = self.poly_mod_oscillator_b_current.get(
            poly_oscillator_b_amount,
            vca::poly_mod_oscillator_b_control_current_amps,
        );
        let poly_frequency_a = parameter_enabled(settings, Parameter::PolyModOscillatorAFrequency);
        let poly_pulse_width_a =
            parameter_enabled(settings, Parameter::PolyModOscillatorAPulseWidth);
        let poly_filter = parameter_enabled(settings, Parameter::PolyModFilter);
        let poly_mod_routed = poly_frequency_a || poly_pulse_width_a || poly_filter;
        let filter_resonance = quantize_analog_pot(settings.get(Parameter::FilterResonance));
        let filter_envelope_amount =
            quantize_analog_pot(settings.get(Parameter::FilterEnvelopeAmount));
        let filter_envelope_control_current = self.filter_envelope_current.get(
            filter_envelope_amount,
            vca::filter_envelope_control_current_amps,
        );
        let direct_filter_envelope = vca::filter_envelope_cutoff_octaves_with_control_current(
            filter_envelope,
            filter_envelope_control_current,
            self.voice_index,
        );
        let filter_cutoff = quantize_analog_pot(settings.get(Parameter::FilterCutoff));
        let filter_keyboard = parameter_enabled(settings, Parameter::FilterKeyboard);
        let common_filter_octaves = direct_filter_envelope + modulation.filter_octaves;
        let amplifier_vca_control = vca::amplifier_envelope_control(amplifier_envelope);
        let common_frequency_a = frequency_a
            * self
                .oscillator_a_modulation_ratio
                .get(modulation.oscillator_a_semitones);
        let common_filter_cutoff_log2_hz = filter_cutoff_log2_hz(
            filter_cutoff,
            self.note,
            filter_keyboard,
            common_filter_octaves,
        );
        #[cfg(feature = "held-filter-two-times")]
        let noise_rate_filter_modulation = parameter_enabled(settings, Parameter::WheelModFilter)
            && settings.get(Parameter::WheelModSourceMix) > 0.5
            && modulation.filter_octaves != 0.0;

        for _ in 0..OSCILLATOR_OVERSAMPLING {
            let sample_b =
                self.oscillator_b
                    .next(frequency_b, oscillator_rate, pulse_width_b, waves_b);
            let poly_destinations = if poly_mod_routed
                && (poly_filter_envelope_current != 0.0 || poly_oscillator_b_control_current > 0.0)
            {
                let poly_oscillator_b_current =
                    vca::poly_mod_oscillator_b_with_control_current_amps(
                        sample_b.poly_mod_source_volts,
                        sample_b.poly_mod_source_conductance,
                        poly_oscillator_b_control_current,
                        self.voice_index,
                    );
                let poly_bus_volts = vca::poly_mod_bus_voltage(
                    poly_filter_envelope_current,
                    poly_oscillator_b_current,
                );
                poly_mod::destinations(poly_bus_volts)
            } else {
                poly_mod::PolyModDestinations::default()
            };
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
            let sync_event = sync.then_some(sample_b.hard_sync_event).flatten();
            let oscillator_a_frequency = if poly_frequency_a {
                frequency_a * semitone_ratio(modulation.oscillator_a_semitones + poly_pitch)
            } else {
                common_frequency_a
            };
            let sample_a = self.oscillator_a.next_with_sync(
                oscillator_a_frequency,
                oscillator_rate,
                pulse_width::add_modulation(pulse_width_a, poly_pulse_width),
                waves_a,
                sync_event,
            );
            // With the amplifier envelope at rest, the original voice still
            // has free-running VCOs. Preserve those phases, PWM history,
            // hard-sync events and Poly-Mod feedback exactly, but do not run
            // the inaudible mixer, filter solver, decimator and final VCA.
            if !render_audio {
                continue;
            }
            let poly_filter_octaves = if poly_filter {
                poly_destinations.filter_octaves
            } else {
                0.0
            };
            let cutoff_log2_hz = if poly_filter {
                filter_cutoff_log2_hz(
                    filter_cutoff,
                    self.note,
                    filter_keyboard,
                    common_filter_octaves + poly_filter_octaves,
                )
            } else {
                common_filter_cutoff_log2_hz
            };
            let mixer = vca::oscillator_mixer_loaded_with_control_current(
                sample_a.mixer_positive_source_volts,
                sample_a.mixer_positive_source_conductance,
                sample_a.mixer_negative_source_volts,
                sample_a.mixer_negative_source_conductance,
                level_a_control_current,
                self.voice_index,
                vca::MixerChannel::OscillatorA,
            ) + vca::oscillator_mixer_loaded_with_control_current(
                sample_b.mixer_positive_source_volts,
                sample_b.mixer_positive_source_conductance,
                sample_b.mixer_negative_source_volts,
                sample_b.mixer_negative_source_conductance,
                level_b_control_current,
                self.voice_index,
                vca::MixerChannel::OscillatorB,
            ) + modulation.noise;
            #[cfg(feature = "hybrid-four-two")]
            let Some(mixer) = self.mixer_decimator.push(mixer) else {
                continue;
            };
            #[cfg(feature = "held-filter-two-times")]
            {
                if noise_rate_filter_modulation {
                    self.filter_phase = 0;
                    self.filter_mixer_accumulator = 0.0;
                    self.filter_cutoff_accumulator = 0.0;
                    let voice_sample = self.process_filter_sample(
                        mixer,
                        cutoff_log2_hz,
                        filter_resonance,
                        oscillator_rate,
                        settings.get(Parameter::VintageSpread),
                        amplifier_vca_control,
                    );
                    self.held_voice_sample = voice_sample;
                    output = self.decimator.push(voice_sample);
                    continue;
                }
                self.filter_mixer_accumulator += mixer;
                self.filter_cutoff_accumulator += cutoff_log2_hz;
                self.filter_phase += 1;
                if self.filter_phase != 2 {
                    continue;
                }
                let averaged_mixer = self.filter_mixer_accumulator * 0.5;
                let averaged_cutoff = self.filter_cutoff_accumulator * 0.5;
                self.filter_phase = 0;
                self.filter_mixer_accumulator = 0.0;
                self.filter_cutoff_accumulator = 0.0;
                let next_voice_sample = self.process_filter_sample(
                    averaged_mixer,
                    averaged_cutoff,
                    filter_resonance,
                    filter_rate,
                    settings.get(Parameter::VintageSpread),
                    amplifier_vca_control,
                );
                let interpolated = (self.held_voice_sample + next_voice_sample) * 0.5;
                let _ = self.decimator.push(interpolated);
                output = self.decimator.push(next_voice_sample);
                self.held_voice_sample = next_voice_sample;
                continue;
            }
            #[cfg(not(feature = "held-filter-two-times"))]
            let voice_sample = self.process_filter_sample(
                mixer,
                cutoff_log2_hz,
                filter_resonance,
                filter_rate,
                settings.get(Parameter::VintageSpread),
                amplifier_vca_control,
            );
            #[cfg(feature = "host-rate")]
            {
                output = Some(voice_sample);
            }
            #[cfg(all(not(feature = "host-rate"), not(feature = "held-filter-two-times")))]
            {
                output = self.decimator.push(voice_sample);
            }
        }

        output.unwrap_or(0.0)
    }

    #[inline(always)]
    fn process_filter_sample(
        &mut self,
        mixer: f32,
        cutoff_log2_hz: f32,
        resonance: f32,
        sample_rate: f32,
        character: f32,
        amplifier_control: f32,
    ) -> f32 {
        let filtered = self.filter.next_with_character_log2(
            mixer,
            cutoff_log2_hz,
            resonance,
            sample_rate,
            character,
        );
        vca::final_voice(filtered, amplifier_control, self.voice_index)
    }
}

fn parameter_enabled(settings: &Settings, parameter: Parameter) -> bool {
    settings.get(parameter) >= 0.5
}

fn semitone_ratio(semitones: f32) -> f32 {
    // Most voices spend most samples without wheel, LFO or Poly-Mod pitch.
    // Preserve the exact mathematical identity and avoid entering libm twice
    // per voice/sample for the overwhelmingly common zero-CV path.
    if semitones == 0.0 {
        1.0
    } else {
        #[cfg(feature = "fast-math")]
        {
            realtime_math::exp2(semitones / 12.0)
        }
        #[cfg(not(feature = "fast-math"))]
        {
            libm::exp2f(semitones / 12.0)
        }
    }
}

#[cfg(test)]
fn filter_cutoff_hz(panel: f32, note: u8, keyboard_tracking: bool, modulation_octaves: f32) -> f32 {
    libm::exp2f(filter_cutoff_log2_hz(
        panel,
        note,
        keyboard_tracking,
        modulation_octaves,
    ))
}

fn filter_cutoff_log2_hz(
    panel: f32,
    note: u8,
    keyboard_tracking: bool,
    modulation_octaves: f32,
) -> f32 {
    let keyboard_octaves = if keyboard_tracking {
        (f32::from(note) - FILTER_KEYBOARD_BASE_NOTE) / 12.0
    } else {
        0.0
    };
    FILTER_MINIMUM_LOG2_HZ
        + panel.clamp(0.0, 1.0) * FILTER_PANEL_OCTAVES
        + keyboard_octaves
        + modulation_octaves
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
                .next(48_000.0, &settings, VoiceModulation::default())
                .abs()
                > 0.001
        }));
        voice.release();
        for _ in 0..500_000 {
            let _ = voice.next(48_000.0, &settings, VoiceModulation::default());
            if !voice.is_active() {
                break;
            }
        }
        assert!(!voice.is_active());
    }

    #[test]
    fn v81_fixed_release_control_is_fast_but_not_the_minimum() {
        use rf_5_contract::hardware::RELEASE_DISABLED_EQUIVALENT_NORMALIZED;

        let mut enabled = Settings::default();
        assert!(enabled.set(Parameter::AmpRelease as u32, 1.0));
        assert!(enabled.set(Parameter::FilterRelease as u32, 1.0));
        let mut disabled = enabled;
        assert!(disabled.set(
            Parameter::AmpRelease as u32,
            f64::from(RELEASE_DISABLED_EQUIVALENT_NORMALIZED)
        ));
        assert!(disabled.set(
            Parameter::FilterRelease as u32,
            f64::from(RELEASE_DISABLED_EQUIVALENT_NORMALIZED)
        ));
        let mut long = Voice::default();
        let mut short = Voice::default();
        long.start(0, 60, 100, 0);
        short.start(0, 60, 100, 0);
        for _ in 0..4_096 {
            let _ = long.next(48_000.0, &enabled, VoiceModulation::default());
            let _ = short.next(48_000.0, &disabled, VoiceModulation::default());
        }
        long.release();
        short.release();
        let mut disabled_release_frames = 0;
        while short.is_active() && disabled_release_frames < 48_000 {
            let _ = short.next(48_000.0, &disabled, VoiceModulation::default());
            disabled_release_frames += 1;
        }
        for _ in 0..48_000 {
            let _ = long.next(48_000.0, &enabled, VoiceModulation::default());
        }
        assert!(long.is_active());
        assert!(!short.is_active());
        assert!((2_000..=5_000).contains(&disabled_release_frames));
    }

    #[test]
    fn note_retrigger_does_not_reset_free_running_oscillators() {
        let settings = Settings::default();
        let mut voice = Voice::default();
        voice.start(0, 60, 100, 2);
        for _ in 0..137 {
            let _ = voice.next(48_000.0, &settings, VoiceModulation::default());
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
                quiet_velocity.next(48_000.0, &settings, VoiceModulation::default()),
                full_velocity.next(48_000.0, &settings, VoiceModulation::default())
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
                voice.next(48_000.0, &settings, VoiceModulation::default()),
                0.0
            );
        }
        assert_ne!(voice.oscillator_a.phase(), phase_before);
    }

    #[test]
    fn never_allocated_voice_stays_dormant_until_start() {
        let settings = Settings::default();
        let mut dormant = Voice::default();
        for _ in 0..64 {
            assert_eq!(
                dormant.next(48_000.0, &settings, VoiceModulation::default()),
                0.0
            );
        }
        assert!(!dormant.oscillators_initialized);

        dormant.start(0, 60, 100, 1);
        assert_eq!(dormant.oscillator_a.phase(), INITIAL_PHASE_A[1]);
        assert_eq!(dormant.oscillator_b.phase(), INITIAL_PHASE_B[1]);
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
            difference += (free_voice.next(48_000.0, &free_settings, VoiceModulation::default())
                - sync_voice.next(48_000.0, &sync_settings, VoiceModulation::default()))
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
        let _ = dry.next_signal_path(
            48_000.0,
            &dry_settings,
            VoiceModulation::default(),
            1.0,
            1.0,
            true,
        );
        let _ = modulated.next_signal_path(
            48_000.0,
            &modulated_settings,
            VoiceModulation::default(),
            1.0,
            1.0,
            true,
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
            difference += (dry.next(48_000.0, &dry_settings, VoiceModulation::default())
                - modulated.next(48_000.0, &modulated_settings, VoiceModulation::default()))
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
                (frequency_voice.next(48_000.0, &frequency_settings, VoiceModulation::default())
                    - filter_voice.next(48_000.0, &filter_settings, VoiceModulation::default()))
                .abs();
        }
        assert!(difference > 1.0);
    }
}
