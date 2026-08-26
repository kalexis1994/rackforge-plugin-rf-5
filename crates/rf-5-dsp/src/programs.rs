//! Factory programs owned by the DSP engine.
//!
//! The audition programs deliberately set a temporary performance-wheel
//! position. This lets a listener verify Wheel Mod without a front panel or a
//! physical modulation wheel. The override is machine state, never patch
//! state, and the first incoming CC1 immediately replaces it.

#[cfg(test)]
use rf_5_contract::Settings;
use rf_5_contract::{
    PATCH_PARAMETER_COUNT, Parameter, hardware::OSCILLATOR_FREQUENCY_CONCERT_NORMALIZED,
};

const BASELINE_INIT: [f32; PATCH_PARAMETER_COUNT] = [
    0.72,
    0.72,
    5.0 / 127.0,
    0.72,
    0.08,
    0.01,
    0.20,
    0.82,
    0.28,
    0.18,
    0.64,
    1.0,
    0.0,
    0.50,
    1.0,
    0.0,
    0.0,
    0.50,
    0.0,
    OSCILLATOR_FREQUENCY_CONCERT_NORMALIZED,
    OSCILLATOR_FREQUENCY_CONCERT_NORMALIZED,
    0.0,
    1.0,
    0.35,
    0.0,
    1.0,
    0.0,
    1.0,
    1.0,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    0.01,
    0.20,
    0.20,
    0.28,
    0.35,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    1.0,
];

const BASELINE_WARM: [f32; PATCH_PARAMETER_COUNT] = [
    0.76,
    0.76,
    10.0 / 127.0,
    0.46,
    0.12,
    0.01,
    0.28,
    0.72,
    0.34,
    0.36,
    0.54,
    1.0,
    1.0,
    0.44,
    1.0,
    0.0,
    0.0,
    0.56,
    0.0,
    OSCILLATOR_FREQUENCY_CONCERT_NORMALIZED,
    OSCILLATOR_FREQUENCY_CONCERT_NORMALIZED,
    0.0,
    1.0,
    0.28,
    0.0,
    1.0,
    0.0,
    1.0,
    1.0,
    0.0,
    0.0,
    0.0,
    0.06,
    0.0,
    0.01,
    0.32,
    0.25,
    0.34,
    0.45,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    1.0,
];

const BASELINE_PAD: [f32; PATCH_PARAMETER_COUNT] = [
    0.68,
    0.62,
    15.0 / 127.0,
    0.38,
    0.04,
    0.54,
    0.52,
    0.78,
    0.70,
    0.42,
    0.62,
    0.0,
    1.0,
    0.37,
    0.0,
    1.0,
    1.0,
    0.63,
    0.0,
    OSCILLATOR_FREQUENCY_CONCERT_NORMALIZED,
    OSCILLATOR_FREQUENCY_CONCERT_NORMALIZED,
    0.0,
    1.0,
    0.18,
    0.0,
    1.0,
    0.0,
    0.0,
    0.0,
    1.0,
    1.0,
    1.0,
    0.12,
    0.15,
    0.48,
    0.55,
    0.65,
    0.70,
    0.50,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    1.0,
    0.0,
    0.0,
    1.0,
];

const BASELINE_LEAD: [f32; PATCH_PARAMETER_COUNT] = [
    0.64,
    0.72,
    21.0 / 127.0,
    0.82,
    0.18,
    0.01,
    0.12,
    0.90,
    0.24,
    0.26,
    0.52,
    1.0,
    0.0,
    0.50,
    1.0,
    0.0,
    0.0,
    0.50,
    1.0,
    68.0 / 127.0,
    OSCILLATOR_FREQUENCY_CONCERT_NORMALIZED,
    0.0,
    1.0,
    0.50,
    1.0,
    0.0,
    0.0,
    1.0,
    1.0,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    0.01,
    0.12,
    0.30,
    0.20,
    0.25,
    0.0,
    0.18,
    1.0,
    0.0,
    0.0,
    1.0,
    0.0,
    0.0,
    1.0,
];

#[derive(Clone, Copy, Debug)]
pub(crate) struct Program {
    pub values: [f32; PATCH_PARAMETER_COUNT],
    pub audition_mod_wheel: Option<f32>,
}

impl Program {
    const fn normal(values: [f32; PATCH_PARAMETER_COUNT]) -> Self {
        Self {
            values,
            audition_mod_wheel: None,
        }
    }

    fn audition(mut values: [f32; PATCH_PARAMETER_COUNT], route: AuditionRoute) -> Self {
        // Common LFO audition setup: triangle only, no noise in the Wheel Mod
        // source crossfade. Destination switches are set below per route.
        values[Parameter::LfoFrequency as usize] = match route {
            AuditionRoute::Vibrato => 0.42,
            AuditionRoute::PulseWidth => 0.34,
            AuditionRoute::Filter => 0.28,
        };
        values[Parameter::LfoSaw as usize] = 0.0;
        values[Parameter::LfoTriangle as usize] = 1.0;
        values[Parameter::LfoSquare as usize] = 0.0;
        values[Parameter::WheelModSourceMix as usize] = 0.0;
        for destination in [
            Parameter::WheelModOscillatorAFrequency,
            Parameter::WheelModOscillatorBFrequency,
            Parameter::WheelModOscillatorAPulseWidth,
            Parameter::WheelModOscillatorBPulseWidth,
            Parameter::WheelModFilter,
        ] {
            values[destination as usize] = 0.0;
        }

        match route {
            AuditionRoute::Vibrato => {
                values[Parameter::WheelModOscillatorAFrequency as usize] = 1.0;
                values[Parameter::WheelModOscillatorBFrequency as usize] = 1.0;
            }
            AuditionRoute::PulseWidth => {
                values[Parameter::OscillatorASaw as usize] = 0.0;
                values[Parameter::OscillatorAPulse as usize] = 1.0;
                values[Parameter::OscillatorBSaw as usize] = 0.0;
                values[Parameter::OscillatorBTriangle as usize] = 0.0;
                values[Parameter::OscillatorBPulse as usize] = 1.0;
                values[Parameter::OscillatorAPulseWidth as usize] = 0.42;
                values[Parameter::OscillatorBPulseWidth as usize] = 0.58;
                values[Parameter::WheelModOscillatorAPulseWidth as usize] = 1.0;
                values[Parameter::WheelModOscillatorBPulseWidth as usize] = 1.0;
            }
            AuditionRoute::Filter => {
                values[Parameter::FilterCutoff as usize] = 0.34;
                values[Parameter::FilterResonance as usize] = 0.30;
                values[Parameter::FilterEnvelopeAmount as usize] = 0.18;
                values[Parameter::WheelModFilter as usize] = 1.0;
            }
        }

        Self {
            values,
            audition_mod_wheel: Some(match route {
                AuditionRoute::Vibrato => 0.42,
                AuditionRoute::PulseWidth => 0.72,
                AuditionRoute::Filter => 0.58,
            }),
        }
    }

    fn filter_drive() -> Self {
        let mut values = BASELINE_WARM;
        values[Parameter::OscillatorALevel as usize] = 1.0;
        values[Parameter::OscillatorBLevel as usize] = 1.0;
        values[Parameter::OscillatorASaw as usize] = 1.0;
        values[Parameter::OscillatorAPulse as usize] = 1.0;
        values[Parameter::OscillatorBSaw as usize] = 1.0;
        values[Parameter::OscillatorBTriangle as usize] = 0.0;
        values[Parameter::OscillatorBPulse as usize] = 1.0;
        values[Parameter::FilterCutoff as usize] = 0.42;
        values[Parameter::FilterResonance as usize] = 0.24;
        values[Parameter::FilterEnvelopeAmount as usize] = 0.38;
        Self::normal(values)
    }

    fn filter_resonance() -> Self {
        let mut values = BASELINE_INIT;
        values[Parameter::OscillatorALevel as usize] = 0.42;
        values[Parameter::OscillatorBLevel as usize] = 0.0;
        values[Parameter::OscillatorASaw as usize] = 1.0;
        values[Parameter::OscillatorAPulse as usize] = 0.0;
        values[Parameter::FilterCutoff as usize] = 0.40;
        values[Parameter::FilterResonance as usize] = 0.88;
        values[Parameter::FilterEnvelopeAmount as usize] = 0.18;
        values[Parameter::FilterAttack as usize] = 0.01;
        values[Parameter::FilterDecay as usize] = 0.36;
        values[Parameter::FilterSustain as usize] = 0.20;
        values[Parameter::FilterRelease as usize] = 0.32;
        Self::normal(values)
    }

    fn envelope_punch() -> Self {
        let mut values = BASELINE_WARM;
        values[Parameter::FilterCutoff as usize] = 0.30;
        values[Parameter::FilterResonance as usize] = 0.20;
        values[Parameter::AmpAttack as usize] = 0.0;
        values[Parameter::AmpDecay as usize] = 0.18;
        values[Parameter::AmpSustain as usize] = 0.0;
        values[Parameter::AmpRelease as usize] = 0.12;
        values[Parameter::FilterAttack as usize] = 0.0;
        values[Parameter::FilterDecay as usize] = 0.16;
        values[Parameter::FilterSustain as usize] = 0.0;
        values[Parameter::FilterRelease as usize] = 0.14;
        values[Parameter::FilterEnvelopeAmount as usize] = 0.62;
        Self::normal(values)
    }

    fn envelope_slow() -> Self {
        let mut values = BASELINE_PAD;
        values[Parameter::AmpAttack as usize] = 0.62;
        values[Parameter::AmpDecay as usize] = 0.48;
        values[Parameter::AmpSustain as usize] = 0.68;
        values[Parameter::AmpRelease as usize] = 0.55;
        values[Parameter::FilterAttack as usize] = 0.58;
        values[Parameter::FilterDecay as usize] = 0.46;
        values[Parameter::FilterSustain as usize] = 0.52;
        values[Parameter::FilterRelease as usize] = 0.57;
        values[Parameter::FilterEnvelopeAmount as usize] = 0.44;
        Self::normal(values)
    }

    fn release_switch_off() -> Self {
        let mut values = BASELINE_PAD;
        // The long stored pots make the global override unmistakable: the
        // V8.1 RELEASE bit forces both generators to their minimum time.
        values[Parameter::AmpRelease as usize] = 1.0;
        values[Parameter::FilterRelease as usize] = 1.0;
        values[Parameter::ReleaseSwitch as usize] = 0.0;
        Self::normal(values)
    }

    fn ca3280_drive() -> Self {
        let mut values = BASELINE_WARM;
        values[Parameter::OscillatorALevel as usize] = 1.0;
        values[Parameter::OscillatorBLevel as usize] = 1.0;
        values[Parameter::OscillatorASaw as usize] = 1.0;
        values[Parameter::OscillatorAPulse as usize] = 1.0;
        values[Parameter::OscillatorBSaw as usize] = 1.0;
        values[Parameter::OscillatorBTriangle as usize] = 1.0;
        values[Parameter::OscillatorBPulse as usize] = 1.0;
        values[Parameter::NoiseLevel as usize] = 0.0;
        values[Parameter::FilterCutoff as usize] = 1.0;
        values[Parameter::FilterResonance as usize] = 0.0;
        values[Parameter::FilterEnvelopeAmount as usize] = 0.0;
        values[Parameter::AmpAttack as usize] = 0.0;
        values[Parameter::AmpDecay as usize] = 0.20;
        values[Parameter::AmpSustain as usize] = 1.0;
        values[Parameter::AmpRelease as usize] = 0.12;
        Self::normal(values)
    }

    fn common_noise_vca() -> Self {
        let mut values = BASELINE_INIT;
        values[Parameter::OscillatorALevel as usize] = 0.0;
        values[Parameter::OscillatorBLevel as usize] = 0.0;
        values[Parameter::NoiseLevel as usize] = 0.82;
        values[Parameter::FilterCutoff as usize] = 0.52;
        values[Parameter::FilterResonance as usize] = 0.24;
        values[Parameter::FilterEnvelopeAmount as usize] = 0.28;
        values[Parameter::AmpAttack as usize] = 0.0;
        values[Parameter::AmpDecay as usize] = 0.24;
        values[Parameter::AmpSustain as usize] = 0.78;
        values[Parameter::AmpRelease as usize] = 0.18;
        Self::normal(values)
    }

    fn poly_mod_oscillator_b() -> Self {
        let mut values = BASELINE_LEAD;
        values[Parameter::OscillatorBLevel as usize] = 0.0;
        values[Parameter::OscillatorBSaw as usize] = 0.0;
        values[Parameter::OscillatorBTriangle as usize] = 1.0;
        values[Parameter::OscillatorBPulse as usize] = 0.0;
        values[Parameter::PolyModFilterEnvelopeAmount as usize] = 0.0;
        values[Parameter::PolyModOscillatorBAmount as usize] = 0.68;
        values[Parameter::PolyModOscillatorAFrequency as usize] = 1.0;
        values[Parameter::PolyModOscillatorAPulseWidth as usize] = 0.0;
        values[Parameter::PolyModFilter as usize] = 0.0;
        values[Parameter::FilterCutoff as usize] = 0.58;
        values[Parameter::FilterResonance as usize] = 0.20;
        Self::normal(values)
    }

    fn poly_mod_filter_envelope() -> Self {
        let mut values = BASELINE_INIT;
        values[Parameter::OscillatorALevel as usize] = 0.54;
        values[Parameter::OscillatorBLevel as usize] = 0.0;
        values[Parameter::FilterCutoff as usize] = 0.48;
        values[Parameter::FilterResonance as usize] = 0.84;
        values[Parameter::FilterEnvelopeAmount as usize] = 0.0;
        values[Parameter::FilterAttack as usize] = 0.0;
        values[Parameter::FilterDecay as usize] = 0.38;
        values[Parameter::FilterSustain as usize] = 0.12;
        values[Parameter::FilterRelease as usize] = 0.28;
        values[Parameter::PolyModFilterEnvelopeAmount as usize] = 0.62;
        values[Parameter::PolyModOscillatorBAmount as usize] = 0.0;
        values[Parameter::PolyModOscillatorAFrequency as usize] = 0.0;
        values[Parameter::PolyModOscillatorAPulseWidth as usize] = 0.0;
        values[Parameter::PolyModFilter as usize] = 1.0;
        Self::normal(values)
    }

    fn wheel_noise_filter() -> Self {
        let mut program = Self::audition(BASELINE_WARM, AuditionRoute::Filter);
        program.values[Parameter::WheelModSourceMix as usize] = 1.0;
        program.values[Parameter::FilterCutoff as usize] = 0.44;
        program.values[Parameter::FilterResonance as usize] = 0.46;
        program.values[Parameter::FilterEnvelopeAmount as usize] = 0.10;
        program.audition_mod_wheel = Some(0.64);
        program
    }

    fn lfo_rate(control: f32) -> Self {
        let mut program = Self::audition(BASELINE_LEAD, AuditionRoute::Vibrato);
        program.values[Parameter::LfoFrequency as usize] = control;
        // A restrained temporary wheel depth keeps the fast-rate comparison
        // pitched and musical while still making both endpoints unmistakable.
        program.audition_mod_wheel = Some(0.28);
        program
    }

    fn oscillator_b_fine(fine: f32) -> Self {
        let mut values = BASELINE_INIT;
        values[Parameter::OscillatorALevel as usize] = 0.62;
        values[Parameter::OscillatorBLevel as usize] = 0.62;
        values[Parameter::OscillatorASaw as usize] = 1.0;
        values[Parameter::OscillatorAPulse as usize] = 0.0;
        values[Parameter::OscillatorBSaw as usize] = 1.0;
        values[Parameter::OscillatorBTriangle as usize] = 0.0;
        values[Parameter::OscillatorBPulse as usize] = 0.0;
        values[Parameter::OscillatorBDetune as usize] = fine;
        values[Parameter::FilterCutoff as usize] = 0.78;
        values[Parameter::FilterResonance as usize] = 0.08;
        values[Parameter::FilterEnvelopeAmount as usize] = 0.0;
        values[Parameter::AmpAttack as usize] = 0.0;
        values[Parameter::AmpDecay as usize] = 0.20;
        values[Parameter::AmpSustain as usize] = 0.90;
        values[Parameter::AmpRelease as usize] = 0.18;
        Self::normal(values)
    }

    fn oscillator_a_pulse_width(pulse_width: f32) -> Self {
        let mut values = BASELINE_INIT;
        values[Parameter::OscillatorALevel as usize] = 0.78;
        values[Parameter::OscillatorBLevel as usize] = 0.0;
        values[Parameter::OscillatorASaw as usize] = 0.0;
        values[Parameter::OscillatorAPulse as usize] = 1.0;
        values[Parameter::OscillatorAPulseWidth as usize] = pulse_width;
        values[Parameter::FilterCutoff as usize] = 0.92;
        values[Parameter::FilterResonance as usize] = 0.0;
        values[Parameter::FilterEnvelopeAmount as usize] = 0.0;
        values[Parameter::AmpAttack as usize] = 0.0;
        values[Parameter::AmpDecay as usize] = 0.2;
        values[Parameter::AmpSustain as usize] = 0.9;
        values[Parameter::AmpRelease as usize] = 0.18;
        Self::normal(values)
    }

    fn oscillator_b_triangle() -> Self {
        let mut values = BASELINE_INIT;
        values[Parameter::OscillatorALevel as usize] = 0.0;
        values[Parameter::OscillatorASaw as usize] = 0.0;
        values[Parameter::OscillatorAPulse as usize] = 0.0;
        values[Parameter::OscillatorBLevel as usize] = 0.78;
        values[Parameter::OscillatorBSaw as usize] = 0.0;
        values[Parameter::OscillatorBTriangle as usize] = 1.0;
        values[Parameter::OscillatorBPulse as usize] = 0.0;
        values[Parameter::OscillatorBKeyboard as usize] = 1.0;
        values[Parameter::OscillatorBLowFrequency as usize] = 0.0;
        values[Parameter::FilterCutoff as usize] = 0.92;
        values[Parameter::FilterResonance as usize] = 0.0;
        values[Parameter::FilterEnvelopeAmount as usize] = 0.0;
        values[Parameter::AmpAttack as usize] = 0.0;
        values[Parameter::AmpDecay as usize] = 0.20;
        values[Parameter::AmpSustain as usize] = 0.90;
        values[Parameter::AmpRelease as usize] = 0.18;
        Self::normal(values)
    }

    fn hard_sync() -> Self {
        let mut values = BASELINE_LEAD;
        values[Parameter::OscillatorAFrequency as usize] = OSCILLATOR_FREQUENCY_CONCERT_NORMALIZED;
        values[Parameter::OscillatorALevel as usize] = 0.82;
        values[Parameter::OscillatorASaw as usize] = 1.0;
        values[Parameter::OscillatorAPulse as usize] = 0.0;
        values[Parameter::OscillatorBLevel as usize] = 0.0;
        values[Parameter::OscillatorBFrequency as usize] = 74.0 / 127.0;
        // Preserve the former program's 22/127-semitone musical offset after
        // correcting OSC B FINE from a centred guess to the physical 0..+1 law.
        values[Parameter::OscillatorBDetune as usize] = 22.0 / 127.0;
        values[Parameter::OscillatorBKeyboard as usize] = 1.0;
        values[Parameter::OscillatorBLowFrequency as usize] = 0.0;
        // The hardware sync tap precedes B's waveform-selection switches, so
        // no oscillator-B waveform needs to enter the audio mixer.
        values[Parameter::OscillatorBSaw as usize] = 0.0;
        values[Parameter::OscillatorBTriangle as usize] = 0.0;
        values[Parameter::OscillatorBPulse as usize] = 0.0;
        values[Parameter::OscillatorSync as usize] = 1.0;
        values[Parameter::PolyModFilterEnvelopeAmount as usize] = 0.0;
        values[Parameter::PolyModOscillatorBAmount as usize] = 0.0;
        values[Parameter::FilterCutoff as usize] = 0.68;
        values[Parameter::FilterResonance as usize] = 0.12;
        values[Parameter::FilterEnvelopeAmount as usize] = 0.08;
        Self::normal(values)
    }

    fn unison_priority() -> Self {
        let mut values = BASELINE_LEAD;
        values[Parameter::Unison as usize] = 1.0;
        values[Parameter::Glide as usize] = 0.14;
        values[Parameter::AmpAttack as usize] = 0.01;
        values[Parameter::AmpDecay as usize] = 0.32;
        values[Parameter::AmpSustain as usize] = 0.68;
        values[Parameter::AmpRelease as usize] = 0.22;
        values[Parameter::FilterAttack as usize] = 0.01;
        values[Parameter::FilterDecay as usize] = 0.36;
        values[Parameter::FilterSustain as usize] = 0.42;
        values[Parameter::FilterRelease as usize] = 0.25;
        Self::normal(values)
    }

    fn unison_glide() -> Self {
        let mut program = Self::unison_priority();
        // Service test 4-4 calls panel position 6 a medium glide.
        program.values[Parameter::Glide as usize] = 0.6;
        program
    }
}

#[derive(Clone, Copy, Debug)]
enum AuditionRoute {
    Vibrato,
    PulseWidth,
    Filter,
}

pub(crate) fn find(id: &str) -> Option<Program> {
    Some(match id {
        "baseline-init" => Program::normal(BASELINE_INIT),
        "baseline-warm" => Program::normal(BASELINE_WARM),
        "baseline-pad" => Program::normal(BASELINE_PAD),
        "baseline-lead" => Program::normal(BASELINE_LEAD),
        "audition-wheel-vibrato" => Program::audition(BASELINE_LEAD, AuditionRoute::Vibrato),
        "audition-wheel-pwm" => Program::audition(BASELINE_INIT, AuditionRoute::PulseWidth),
        "audition-wheel-filter" => Program::audition(BASELINE_WARM, AuditionRoute::Filter),
        "audition-filter-drive" => Program::filter_drive(),
        "audition-filter-resonance" => Program::filter_resonance(),
        "audition-envelope-punch" => Program::envelope_punch(),
        "audition-envelope-slow" => Program::envelope_slow(),
        "audition-release-switch-off" => Program::release_switch_off(),
        "audition-ca3280-drive" => Program::ca3280_drive(),
        "audition-common-noise-vca" => Program::common_noise_vca(),
        "audition-poly-mod-oscillator-b" => Program::poly_mod_oscillator_b(),
        "audition-poly-mod-filter-envelope" => Program::poly_mod_filter_envelope(),
        "audition-wheel-noise-filter" => Program::wheel_noise_filter(),
        "audition-hard-sync" => Program::hard_sync(),
        "audition-unison-priority" => Program::unison_priority(),
        "audition-unison-glide" => Program::unison_glide(),
        "audition-lfo-slow" => Program::lfo_rate(0.10),
        "audition-lfo-fast" => Program::lfo_rate(0.82),
        "audition-oscillator-b-fine-zero" => Program::oscillator_b_fine(0.0),
        "audition-oscillator-b-fine-semitone" => Program::oscillator_b_fine(1.0),
        "audition-pulse-width-minimum" => Program::oscillator_a_pulse_width(0.0),
        "audition-pulse-width-square" => Program::oscillator_a_pulse_width(64.0 / 127.0),
        "audition-pulse-width-maximum" => Program::oscillator_a_pulse_width(1.0),
        "audition-oscillator-b-triangle" => Program::oscillator_b_triangle(),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_public_program_is_valid_contract_state() {
        for id in [
            "baseline-init",
            "baseline-warm",
            "baseline-pad",
            "baseline-lead",
            "audition-wheel-vibrato",
            "audition-wheel-pwm",
            "audition-wheel-filter",
            "audition-filter-drive",
            "audition-filter-resonance",
            "audition-envelope-punch",
            "audition-envelope-slow",
            "audition-release-switch-off",
            "audition-ca3280-drive",
            "audition-common-noise-vca",
            "audition-poly-mod-oscillator-b",
            "audition-poly-mod-filter-envelope",
            "audition-wheel-noise-filter",
            "audition-hard-sync",
            "audition-unison-priority",
            "audition-unison-glide",
            "audition-lfo-slow",
            "audition-lfo-fast",
            "audition-oscillator-b-fine-zero",
            "audition-oscillator-b-fine-semitone",
            "audition-pulse-width-minimum",
            "audition-pulse-width-square",
            "audition-pulse-width-maximum",
            "audition-oscillator-b-triangle",
        ] {
            let program = find(id).expect("catalog program exists");
            let mut settings = Settings::default();
            assert!(settings.apply_patch_array(program.values));
        }
    }

    #[test]
    fn oscillator_b_triangle_audition_is_electrically_isolated() {
        let program = find("audition-oscillator-b-triangle").unwrap();
        assert_eq!(program.values[Parameter::OscillatorALevel as usize], 0.0);
        assert!(program.values[Parameter::OscillatorBLevel as usize] > 0.0);
        assert_eq!(program.values[Parameter::OscillatorBSaw as usize], 0.0);
        assert_eq!(program.values[Parameter::OscillatorBTriangle as usize], 1.0);
        assert_eq!(program.values[Parameter::OscillatorBPulse as usize], 0.0);
        assert_eq!(
            program.values[Parameter::PolyModOscillatorBAmount as usize],
            0.0
        );
    }

    #[test]
    fn oscillator_b_fine_auditions_reach_both_physical_endpoints() {
        let flat = find("audition-oscillator-b-fine-zero").unwrap();
        let sharp = find("audition-oscillator-b-fine-semitone").unwrap();
        assert_eq!(flat.values[Parameter::OscillatorBDetune as usize], 0.0);
        assert_eq!(sharp.values[Parameter::OscillatorBDetune as usize], 1.0);
    }

    #[test]
    fn pulse_width_auditions_reach_both_panel_limits_and_nearest_square_code() {
        let minimum = find("audition-pulse-width-minimum").unwrap();
        let square = find("audition-pulse-width-square").unwrap();
        let maximum = find("audition-pulse-width-maximum").unwrap();
        let index = Parameter::OscillatorAPulseWidth as usize;
        assert_eq!(minimum.values[index], 0.0);
        assert_eq!(
            rf_5_contract::hardware::analog_pot_code(square.values[index]),
            64
        );
        assert_eq!(maximum.values[index], 1.0);
    }

    #[test]
    fn baseline_program_migration_preserves_their_musical_fine_offsets() {
        for (id, expected_code) in [
            ("baseline-init", 5),
            ("baseline-warm", 10),
            ("baseline-pad", 15),
            ("baseline-lead", 21),
        ] {
            let program = find(id).unwrap();
            assert_eq!(
                rf_5_contract::hardware::analog_pot_code(
                    program.values[Parameter::OscillatorBDetune as usize]
                ),
                expected_code
            );
        }

        let pad = find("baseline-pad").unwrap();
        assert_eq!(
            rf_5_contract::hardware::analog_pot_code(
                pad.values[Parameter::OscillatorALevel as usize]
            ),
            79
        );
    }

    #[test]
    fn unknown_program_is_rejected() {
        assert!(find("not-a-program").is_none());
    }
}
