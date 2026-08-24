//! Source-backed facts about the frozen RF-5 reference hardware.
//!
//! These constants describe observable control-system structure. They are not
//! DSP approximations and must only change when the source ledger records new
//! evidence.

use crate::Parameter;

pub const VOICE_COUNT: usize = 5;
pub const OSCILLATORS_PER_VOICE: usize = 2;
pub const AUDIO_OSCILLATOR_COUNT: usize = VOICE_COUNT * OSCILLATORS_PER_VOICE;
pub const LFO_COUNT: usize = 1;

pub const ANALOG_POT_COUNT: usize = 24;
pub const ANALOG_POT_RESOLUTION_BITS: u8 = 7;
pub const ANALOG_POT_STEPS: u16 = 1 << ANALOG_POT_RESOLUTION_BITS;

pub const CONTROL_DAC_PHYSICAL_BITS: u8 = 16;
pub const CONTROL_DAC_WRITABLE_BITS: u8 = 14;
pub const GENERAL_CONTROL_VOLTAGE_BITS: u8 = 7;
pub const OSCILLATOR_CONTROL_VOLTAGE_BITS: u8 = 14;
pub const DAC_FULL_SCALE_VOLTS: f32 = 10.67;
pub const SOFTWARE_CONTROL_VOLTAGE_LIMIT_VOLTS: f32 = 10.0;

pub const CONTROL_VOLTAGE_DESTINATION_COUNT: usize = 38;
pub const COMMON_AND_PATCH_SAMPLE_HOLD_COUNT: usize = 23;
pub const INDIVIDUAL_OSCILLATOR_AND_FILTER_SAMPLE_HOLD_COUNT: usize = 15;
pub const CONTROL_LOOP_IDLE_MICROSECONDS: u32 = 6_000;
pub const CONTROL_LOOP_CHANGED_MICROSECONDS: u32 = 11_000;
pub const SAMPLE_HOLD_SERVICE_DROOP_LIMIT_VOLTS_PER_7_MS: f32 = 0.0005;

pub const PROGRAM_COUNT: usize = 40;
pub const PROGRAM_BYTES: usize = 24;

/// Quantize a normalized host value to the 128 positions stored by a panel pot.
pub fn quantize_analog_pot(value: f32) -> f32 {
    let value = if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let step = (value * (ANALOG_POT_STEPS - 1) as f32 + 0.5) as u16;
    step as f32 / (ANALOG_POT_STEPS - 1) as f32
}

/// Hardware multiplexer order used when the CPU scans the 24 analog controls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum AnalogPot {
    FilterAttack = 0,
    FilterDecay = 1,
    FilterSustain = 2,
    FilterRelease = 3,
    AmplifierAttack = 4,
    AmplifierDecay = 5,
    AmplifierSustain = 6,
    AmplifierRelease = 7,
    FilterCutoff = 8,
    FilterEnvelopeAmount = 9,
    OscillatorBMix = 10,
    OscillatorBPulseWidth = 11,
    OscillatorAMix = 12,
    OscillatorAPulseWidth = 13,
    NoiseMix = 14,
    FilterResonance = 15,
    Glide = 16,
    LfoFrequency = 17,
    WheelModSourceMix = 18,
    PolyModOscillatorBAmount = 19,
    PolyModFilterEnvelopeAmount = 20,
    OscillatorAFrequency = 21,
    OscillatorBFrequency = 22,
    OscillatorBFine = 23,
}

impl TryFrom<u8> for AnalogPot {
    type Error = ();

    fn try_from(index: u8) -> Result<Self, Self::Error> {
        match index {
            0 => Ok(Self::FilterAttack),
            1 => Ok(Self::FilterDecay),
            2 => Ok(Self::FilterSustain),
            3 => Ok(Self::FilterRelease),
            4 => Ok(Self::AmplifierAttack),
            5 => Ok(Self::AmplifierDecay),
            6 => Ok(Self::AmplifierSustain),
            7 => Ok(Self::AmplifierRelease),
            8 => Ok(Self::FilterCutoff),
            9 => Ok(Self::FilterEnvelopeAmount),
            10 => Ok(Self::OscillatorBMix),
            11 => Ok(Self::OscillatorBPulseWidth),
            12 => Ok(Self::OscillatorAMix),
            13 => Ok(Self::OscillatorAPulseWidth),
            14 => Ok(Self::NoiseMix),
            15 => Ok(Self::FilterResonance),
            16 => Ok(Self::Glide),
            17 => Ok(Self::LfoFrequency),
            18 => Ok(Self::WheelModSourceMix),
            19 => Ok(Self::PolyModOscillatorBAmount),
            20 => Ok(Self::PolyModFilterEnvelopeAmount),
            21 => Ok(Self::OscillatorAFrequency),
            22 => Ok(Self::OscillatorBFrequency),
            23 => Ok(Self::OscillatorBFine),
            _ => Err(()),
        }
    }
}

impl AnalogPot {
    pub const fn parameter(self) -> Parameter {
        match self {
            Self::FilterAttack => Parameter::FilterAttack,
            Self::FilterDecay => Parameter::FilterDecay,
            Self::FilterSustain => Parameter::FilterSustain,
            Self::FilterRelease => Parameter::FilterRelease,
            Self::AmplifierAttack => Parameter::AmpAttack,
            Self::AmplifierDecay => Parameter::AmpDecay,
            Self::AmplifierSustain => Parameter::AmpSustain,
            Self::AmplifierRelease => Parameter::AmpRelease,
            Self::FilterCutoff => Parameter::FilterCutoff,
            Self::FilterEnvelopeAmount => Parameter::FilterEnvelopeAmount,
            Self::OscillatorBMix => Parameter::OscillatorBLevel,
            Self::OscillatorBPulseWidth => Parameter::OscillatorBPulseWidth,
            Self::OscillatorAMix => Parameter::OscillatorALevel,
            Self::OscillatorAPulseWidth => Parameter::OscillatorAPulseWidth,
            Self::NoiseMix => Parameter::NoiseLevel,
            Self::FilterResonance => Parameter::FilterResonance,
            Self::Glide => Parameter::Glide,
            Self::LfoFrequency => Parameter::LfoFrequency,
            Self::WheelModSourceMix => Parameter::WheelModSourceMix,
            Self::PolyModOscillatorBAmount => Parameter::PolyModOscillatorBAmount,
            Self::PolyModFilterEnvelopeAmount => Parameter::PolyModFilterEnvelopeAmount,
            Self::OscillatorAFrequency => Parameter::OscillatorAFrequency,
            Self::OscillatorBFrequency => Parameter::OscillatorBFrequency,
            Self::OscillatorBFine => Parameter::OscillatorBDetune,
        }
    }
}

/// One raw program-memory byte: a 7-bit pot value plus one switch bit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgramByte(u8);

impl ProgramByte {
    pub const fn from_raw(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn from_parts(pot_value: u8, switch_on: bool) -> Option<Self> {
        if pot_value >= ANALOG_POT_STEPS as u8 {
            return None;
        }
        Some(Self(pot_value | ((switch_on as u8) << 7)))
    }

    pub const fn raw(self) -> u8 {
        self.0
    }

    pub const fn pot_value(self) -> u8 {
        self.0 & 0x7f
    }

    pub const fn switch_on(self) -> bool {
        self.0 & 0x80 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analog_pot_scan_map_is_contiguous_and_complete() {
        for index in 0..ANALOG_POT_COUNT as u8 {
            let pot = AnalogPot::try_from(index).expect("documented scan index");
            assert_eq!(pot as u8, index);
        }
        assert!(AnalogPot::try_from(ANALOG_POT_COUNT as u8).is_err());
    }

    #[test]
    fn every_scanned_pot_maps_to_a_distinct_public_parameter() {
        let mut seen = [false; crate::PARAMETER_COUNT];
        for index in 0..ANALOG_POT_COUNT as u8 {
            let parameter = AnalogPot::try_from(index).unwrap().parameter() as usize;
            assert!(!seen[parameter]);
            seen[parameter] = true;
        }
        assert_eq!(
            seen.iter().filter(|mapped| **mapped).count(),
            ANALOG_POT_COUNT
        );
    }

    #[test]
    fn sample_hold_partition_matches_cv_destination_count() {
        assert_eq!(
            COMMON_AND_PATCH_SAMPLE_HOLD_COUNT + INDIVIDUAL_OSCILLATOR_AND_FILTER_SAMPLE_HOLD_COUNT,
            CONTROL_VOLTAGE_DESTINATION_COUNT
        );
    }

    #[test]
    fn all_raw_program_bytes_round_trip() {
        for raw in u8::MIN..=u8::MAX {
            let byte = ProgramByte::from_raw(raw);
            let rebuilt = ProgramByte::from_parts(byte.pot_value(), byte.switch_on())
                .expect("seven-bit pot value");
            assert_eq!(rebuilt.raw(), raw);
        }
    }

    #[test]
    fn analog_pot_quantizer_has_128_reachable_positions() {
        assert_eq!(quantize_analog_pot(-1.0), 0.0);
        assert_eq!(quantize_analog_pot(2.0), 1.0);
        assert_eq!(quantize_analog_pot(f32::NAN), 0.0);
        for step in 0..ANALOG_POT_STEPS {
            let normalized = step as f32 / (ANALOG_POT_STEPS - 1) as f32;
            let quantized = quantize_analog_pot(normalized);
            assert!((quantized - normalized).abs() < 1.0e-6);
        }
    }
}
