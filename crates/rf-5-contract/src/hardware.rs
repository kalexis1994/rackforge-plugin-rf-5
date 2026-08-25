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
pub const PANEL_POT_FULL_SCALE_VOLTS: f32 = 5.0;
pub const ADC_WINDOW_HYSTERESIS_MILLIVOLTS: f32 = 34.0;
pub const PANEL_POT_CONFIRMING_STEPS: u8 = 2;

pub const OSCILLATOR_FREQUENCY_CONCERT_POT_CODE: u8 = 48;
pub const OSCILLATOR_FREQUENCY_CONCERT_NORMALIZED: f32 =
    OSCILLATOR_FREQUENCY_CONCERT_POT_CODE as f32 / (ANALOG_POT_STEPS - 1) as f32;
pub const OSCILLATOR_FREQUENCY_NORMAL_MAX_SEMITONES: u8 = 48;
pub const OSCILLATOR_FREQUENCY_LOW_MAX_SEMITONES: u8 = 108;
pub const OSCILLATOR_FREQUENCY_LOW_OFFSET_SEMITONES: f32 = -90.0;
pub const SCALE_EQUAL_TEMPERAMENT_POT_CODE: u8 = 64;
pub const SCALE_EQUAL_TEMPERAMENT_NORMALIZED: f32 =
    SCALE_EQUAL_TEMPERAMENT_POT_CODE as f32 / (ANALOG_POT_STEPS - 1) as f32;

pub const CONTROL_DAC_PHYSICAL_BITS: u8 = 16;
pub const CONTROL_DAC_WRITABLE_BITS: u8 = 14;
pub const GENERAL_CONTROL_VOLTAGE_BITS: u8 = 7;
pub const OSCILLATOR_CONTROL_VOLTAGE_BITS: u8 = 14;
pub const DAC_FULL_SCALE_VOLTS: f32 = 10.67;
pub const SOFTWARE_CONTROL_VOLTAGE_LIMIT_VOLTS: f32 = 10.0;

pub const TUNE_CPU_CLOCK_HZ: u32 = 2_500_000;
pub const TUNE_OSCILLATOR_COUNT: usize = AUDIO_OSCILLATOR_COUNT;
pub const TUNE_OCTAVE_BIAS_COUNT: usize = 10;
pub const TUNE_DIRECT_MEASUREMENT_FIRST_OCTAVE: usize = 3;
pub const TUNE_DIRECT_MEASUREMENT_LAST_OCTAVE: usize = 9;
pub const TUNE_BIAS_BYTES: usize =
    TUNE_OSCILLATOR_COUNT * TUNE_OCTAVE_BIAS_COUNT * size_of::<i16>();
pub const IDEAL_SEMITONE_CONTROL_VOLTS: f32 = 0.083;

pub const CONTROL_VOLTAGE_DESTINATION_COUNT: usize = 38;
/// V8.1 always addresses five complete banks of eight S/H positions. The
/// final address on PCB3 and PCB4 is physically unconnected, leaving 38 real
/// destinations inside 40 firmware strobe slots.
pub const CONTROL_VOLTAGE_STROBE_SLOT_COUNT: usize = 40;
pub const UNUSED_CONTROL_VOLTAGE_STROBE_SLOT_COUNT: usize =
    CONTROL_VOLTAGE_STROBE_SLOT_COUNT - CONTROL_VOLTAGE_DESTINATION_COUNT;
pub const COMMON_AND_PATCH_SAMPLE_HOLD_COUNT: usize = 23;
pub const INDIVIDUAL_OSCILLATOR_AND_FILTER_SAMPLE_HOLD_COUNT: usize = 15;
pub const CONTROL_LOOP_IDLE_MICROSECONDS: u32 = 6_000;
pub const CONTROL_LOOP_CHANGED_MICROSECONDS: u32 = 11_000;
pub const SAMPLE_HOLD_SERVICE_DROOP_LIMIT_VOLTS_PER_7_MS: f32 = 0.0005;
/// Populated DAC-buffer output resistor R354 on SD332.
pub const SAMPLE_HOLD_DAC_OUTPUT_RESISTANCE_OHMS: f32 = 5_000.0;
/// Populated hold capacitor in each SD333/SD430 CV sample/hold cell.
pub const SAMPLE_HOLD_CAPACITANCE_FARADS: f32 = 0.01e-6;
/// V8.1 instructions executed while the selected 4051 path remains enabled.
pub const SAMPLE_HOLD_STROBE_T_STATES: u32 = 64;

/// Logical sample/hold destinations grouped as shown on SD333 and SD430.
/// The numeric order is RF-5's stable control-plane vocabulary. Physical V8.1
/// strobe addresses are represented separately because each board also has an
/// unconnected eighth position.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ControlVoltageDestination {
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
    Unison = 21,
    SequencerOutput = 22,
    Oscillator1A = 23,
    Oscillator1B = 24,
    Oscillator2A = 25,
    Oscillator2B = 26,
    Oscillator3A = 27,
    Oscillator3B = 28,
    Oscillator4A = 29,
    Oscillator4B = 30,
    Oscillator5A = 31,
    Oscillator5B = 32,
    Filter1 = 33,
    Filter2 = 34,
    Filter3 = 35,
    Filter4 = 36,
    Filter5 = 37,
}

/// Exact V8.1 S/H address sequence recovered from the five eight-address
/// firmware passes at 0x0583-0x05c4 and corroborated by SD333/SD430.
///
/// Slots 0-23 address the three common-CV 4051s on PCB3. Slot 23 is U355 X7,
/// which is marked NC. Slots 24-39 address the two individual-CV 4051s on
/// PCB4. Slot 39 is U405 X7, also marked NC.
pub const CONTROL_VOLTAGE_STROBE_ORDER: [Option<ControlVoltageDestination>;
    CONTROL_VOLTAGE_STROBE_SLOT_COUNT] = [
    Some(ControlVoltageDestination::FilterAttack),
    Some(ControlVoltageDestination::FilterDecay),
    Some(ControlVoltageDestination::FilterSustain),
    Some(ControlVoltageDestination::FilterRelease),
    Some(ControlVoltageDestination::AmplifierAttack),
    Some(ControlVoltageDestination::AmplifierDecay),
    Some(ControlVoltageDestination::AmplifierSustain),
    Some(ControlVoltageDestination::AmplifierRelease),
    Some(ControlVoltageDestination::FilterCutoff),
    Some(ControlVoltageDestination::FilterEnvelopeAmount),
    Some(ControlVoltageDestination::OscillatorBMix),
    Some(ControlVoltageDestination::OscillatorBPulseWidth),
    Some(ControlVoltageDestination::OscillatorAMix),
    Some(ControlVoltageDestination::OscillatorAPulseWidth),
    Some(ControlVoltageDestination::NoiseMix),
    Some(ControlVoltageDestination::FilterResonance),
    Some(ControlVoltageDestination::Glide),
    Some(ControlVoltageDestination::LfoFrequency),
    Some(ControlVoltageDestination::WheelModSourceMix),
    Some(ControlVoltageDestination::PolyModOscillatorBAmount),
    Some(ControlVoltageDestination::PolyModFilterEnvelopeAmount),
    Some(ControlVoltageDestination::Unison),
    Some(ControlVoltageDestination::SequencerOutput),
    None,
    Some(ControlVoltageDestination::Oscillator1A),
    Some(ControlVoltageDestination::Oscillator1B),
    Some(ControlVoltageDestination::Oscillator2A),
    Some(ControlVoltageDestination::Oscillator2B),
    Some(ControlVoltageDestination::Oscillator3A),
    Some(ControlVoltageDestination::Oscillator3B),
    Some(ControlVoltageDestination::Oscillator4A),
    Some(ControlVoltageDestination::Oscillator4B),
    Some(ControlVoltageDestination::Oscillator5A),
    Some(ControlVoltageDestination::Oscillator5B),
    Some(ControlVoltageDestination::Filter1),
    Some(ControlVoltageDestination::Filter2),
    Some(ControlVoltageDestination::Filter3),
    Some(ControlVoltageDestination::Filter4),
    Some(ControlVoltageDestination::Filter5),
    None,
];

impl TryFrom<u8> for ControlVoltageDestination {
    type Error = ();

    fn try_from(index: u8) -> Result<Self, Self::Error> {
        const DESTINATIONS: [ControlVoltageDestination; CONTROL_VOLTAGE_DESTINATION_COUNT] = [
            ControlVoltageDestination::FilterAttack,
            ControlVoltageDestination::FilterDecay,
            ControlVoltageDestination::FilterSustain,
            ControlVoltageDestination::FilterRelease,
            ControlVoltageDestination::AmplifierAttack,
            ControlVoltageDestination::AmplifierDecay,
            ControlVoltageDestination::AmplifierSustain,
            ControlVoltageDestination::AmplifierRelease,
            ControlVoltageDestination::FilterCutoff,
            ControlVoltageDestination::FilterEnvelopeAmount,
            ControlVoltageDestination::OscillatorBMix,
            ControlVoltageDestination::OscillatorBPulseWidth,
            ControlVoltageDestination::OscillatorAMix,
            ControlVoltageDestination::OscillatorAPulseWidth,
            ControlVoltageDestination::NoiseMix,
            ControlVoltageDestination::FilterResonance,
            ControlVoltageDestination::Glide,
            ControlVoltageDestination::LfoFrequency,
            ControlVoltageDestination::WheelModSourceMix,
            ControlVoltageDestination::PolyModOscillatorBAmount,
            ControlVoltageDestination::PolyModFilterEnvelopeAmount,
            ControlVoltageDestination::Unison,
            ControlVoltageDestination::SequencerOutput,
            ControlVoltageDestination::Oscillator1A,
            ControlVoltageDestination::Oscillator1B,
            ControlVoltageDestination::Oscillator2A,
            ControlVoltageDestination::Oscillator2B,
            ControlVoltageDestination::Oscillator3A,
            ControlVoltageDestination::Oscillator3B,
            ControlVoltageDestination::Oscillator4A,
            ControlVoltageDestination::Oscillator4B,
            ControlVoltageDestination::Oscillator5A,
            ControlVoltageDestination::Oscillator5B,
            ControlVoltageDestination::Filter1,
            ControlVoltageDestination::Filter2,
            ControlVoltageDestination::Filter3,
            ControlVoltageDestination::Filter4,
            ControlVoltageDestination::Filter5,
        ];
        DESTINATIONS.get(index as usize).copied().ok_or(())
    }
}

impl ControlVoltageDestination {
    pub const fn oscillator(voice_index: usize, oscillator_b: bool) -> Option<Self> {
        if voice_index >= VOICE_COUNT {
            return None;
        }
        let index = Self::Oscillator1A as u8 + voice_index as u8 * 2 + oscillator_b as u8;
        Self::from_valid_index(index)
    }

    pub const fn filter(voice_index: usize) -> Option<Self> {
        if voice_index >= VOICE_COUNT {
            return None;
        }
        Self::from_valid_index(Self::Filter1 as u8 + voice_index as u8)
    }

    const fn from_valid_index(index: u8) -> Option<Self> {
        // Kept explicit so these helpers remain const without unsafe casts.
        match index {
            23 => Some(Self::Oscillator1A),
            24 => Some(Self::Oscillator1B),
            25 => Some(Self::Oscillator2A),
            26 => Some(Self::Oscillator2B),
            27 => Some(Self::Oscillator3A),
            28 => Some(Self::Oscillator3B),
            29 => Some(Self::Oscillator4A),
            30 => Some(Self::Oscillator4B),
            31 => Some(Self::Oscillator5A),
            32 => Some(Self::Oscillator5B),
            33 => Some(Self::Filter1),
            34 => Some(Self::Filter2),
            35 => Some(Self::Filter3),
            36 => Some(Self::Filter4),
            37 => Some(Self::Filter5),
            _ => None,
        }
    }
}

pub const PROGRAM_COUNT: usize = 40;
pub const PROGRAM_BYTES: usize = 24;
pub const PROGRAM_SWITCH_COUNT: usize = 22;

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

/// Convert a normalized host value to the seven-bit code compared by V8.1.
pub fn analog_pot_code(value: f32) -> u8 {
    (quantize_analog_pot(value) * (ANALOG_POT_STEPS - 1) as f32) as u8
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

    /// Scale Mode reuses these twelve physical pots as C-through-B offsets.
    pub const fn scale_parameter(self) -> Option<Parameter> {
        match self {
            Self::LfoFrequency => Some(Parameter::ScaleC),
            Self::OscillatorBFrequency => Some(Parameter::ScaleCSharp),
            Self::OscillatorBFine => Some(Parameter::ScaleD),
            Self::OscillatorBPulseWidth => Some(Parameter::ScaleDSharp),
            Self::FilterAttack => Some(Parameter::ScaleE),
            Self::FilterDecay => Some(Parameter::ScaleF),
            Self::FilterSustain => Some(Parameter::ScaleFSharp),
            Self::FilterRelease => Some(Parameter::ScaleG),
            Self::AmplifierAttack => Some(Parameter::ScaleGSharp),
            Self::AmplifierDecay => Some(Parameter::ScaleA),
            Self::AmplifierSustain => Some(Parameter::ScaleASharp),
            Self::AmplifierRelease => Some(Parameter::ScaleB),
            _ => None,
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

/// V8.1 program-switch order inside the high bits of the 24 program bytes.
///
/// The first two groups also contain the software-only Unison and Release
/// flags and the separately clocked keyboard switches. The final two storage
/// bits are physically unused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ProgramSwitch {
    OscillatorAPulse = 0,
    OscillatorASaw = 1,
    OscillatorSync = 2,
    OscillatorBSaw = 3,
    OscillatorBTriangle = 4,
    OscillatorBPulse = 5,
    OscillatorBKeyboard = 6,
    Unison = 7,
    PolyModOscillatorAFrequency = 8,
    PolyModOscillatorAPulseWidth = 9,
    PolyModFilter = 10,
    LfoSaw = 11,
    LfoTriangle = 12,
    LfoSquare = 13,
    FilterKeyboard = 14,
    Release = 15,
    WheelModOscillatorAFrequency = 16,
    WheelModOscillatorBFrequency = 17,
    WheelModOscillatorAPulseWidth = 18,
    WheelModOscillatorBPulseWidth = 19,
    WheelModFilter = 20,
    OscillatorBLowFrequency = 21,
}

impl ProgramSwitch {
    pub const fn from_storage_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::OscillatorAPulse),
            1 => Some(Self::OscillatorASaw),
            2 => Some(Self::OscillatorSync),
            3 => Some(Self::OscillatorBSaw),
            4 => Some(Self::OscillatorBTriangle),
            5 => Some(Self::OscillatorBPulse),
            6 => Some(Self::OscillatorBKeyboard),
            7 => Some(Self::Unison),
            8 => Some(Self::PolyModOscillatorAFrequency),
            9 => Some(Self::PolyModOscillatorAPulseWidth),
            10 => Some(Self::PolyModFilter),
            11 => Some(Self::LfoSaw),
            12 => Some(Self::LfoTriangle),
            13 => Some(Self::LfoSquare),
            14 => Some(Self::FilterKeyboard),
            15 => Some(Self::Release),
            16 => Some(Self::WheelModOscillatorAFrequency),
            17 => Some(Self::WheelModOscillatorBFrequency),
            18 => Some(Self::WheelModOscillatorAPulseWidth),
            19 => Some(Self::WheelModOscillatorBPulseWidth),
            20 => Some(Self::WheelModFilter),
            21 => Some(Self::OscillatorBLowFrequency),
            _ => None,
        }
    }

    pub const fn parameter(self) -> Parameter {
        match self {
            Self::OscillatorAPulse => Parameter::OscillatorAPulse,
            Self::OscillatorASaw => Parameter::OscillatorASaw,
            Self::OscillatorSync => Parameter::OscillatorSync,
            Self::OscillatorBSaw => Parameter::OscillatorBSaw,
            Self::OscillatorBTriangle => Parameter::OscillatorBTriangle,
            Self::OscillatorBPulse => Parameter::OscillatorBPulse,
            Self::OscillatorBKeyboard => Parameter::OscillatorBKeyboard,
            Self::Unison => Parameter::Unison,
            Self::PolyModOscillatorAFrequency => Parameter::PolyModOscillatorAFrequency,
            Self::PolyModOscillatorAPulseWidth => Parameter::PolyModOscillatorAPulseWidth,
            Self::PolyModFilter => Parameter::PolyModFilter,
            Self::LfoSaw => Parameter::LfoSaw,
            Self::LfoTriangle => Parameter::LfoTriangle,
            Self::LfoSquare => Parameter::LfoSquare,
            Self::FilterKeyboard => Parameter::FilterKeyboard,
            Self::Release => Parameter::ReleaseSwitch,
            Self::WheelModOscillatorAFrequency => Parameter::WheelModOscillatorAFrequency,
            Self::WheelModOscillatorBFrequency => Parameter::WheelModOscillatorBFrequency,
            Self::WheelModOscillatorAPulseWidth => Parameter::WheelModOscillatorAPulseWidth,
            Self::WheelModOscillatorBPulseWidth => Parameter::WheelModOscillatorBPulseWidth,
            Self::WheelModFilter => Parameter::WheelModFilter,
            Self::OscillatorBLowFrequency => Parameter::OscillatorBLowFrequency,
        }
    }
}

/// Encode only the controls that the original 24-byte program memory stores.
pub fn encode_program(settings: crate::Settings) -> [ProgramByte; PROGRAM_BYTES] {
    core::array::from_fn(|index| {
        let pot = AnalogPot::try_from(index as u8).expect("program pot index");
        let pot_value = analog_pot_code(settings.get(pot.parameter()));
        let switch_on = ProgramSwitch::from_storage_index(index as u8)
            .is_some_and(|switch| settings.get(switch.parameter()) >= 0.5);
        ProgramByte::from_parts(pot_value, switch_on).expect("seven-bit program pot")
    })
}

/// Decode a 24-byte program over existing machine state.
///
/// Master volume, MASTER TUNE, RF-5's machine-character control and Scale Mode
/// remain untouched because none of them belongs to a Rev 3 patch record.
pub fn decode_program(
    raw: [ProgramByte; PROGRAM_BYTES],
    mut settings: crate::Settings,
) -> crate::Settings {
    for (index, byte) in raw.into_iter().enumerate() {
        let pot = AnalogPot::try_from(index as u8).expect("program pot index");
        let normalized = f64::from(byte.pot_value()) / f64::from(ANALOG_POT_STEPS - 1);
        let updated = settings.set(pot.parameter() as u32, normalized);
        debug_assert!(updated);
        if let Some(switch) = ProgramSwitch::from_storage_index(index as u8) {
            let updated = settings.set(switch.parameter() as u32, f64::from(byte.switch_on()));
            debug_assert!(updated);
        }
    }
    settings
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
    fn scale_mode_reuses_exactly_twelve_scanned_pots() {
        let mut seen = [false; crate::SCALE_NOTE_COUNT];
        for index in 0..ANALOG_POT_COUNT as u8 {
            if let Some(parameter) = AnalogPot::try_from(index).unwrap().scale_parameter() {
                let note = parameter as usize - Parameter::ScaleC as usize;
                assert!(!seen[note]);
                seen[note] = true;
            }
        }
        assert!(seen.into_iter().all(|mapped| mapped));
    }

    #[test]
    fn sample_hold_partition_matches_cv_destination_count() {
        assert_eq!(
            COMMON_AND_PATCH_SAMPLE_HOLD_COUNT + INDIVIDUAL_OSCILLATOR_AND_FILTER_SAMPLE_HOLD_COUNT,
            CONTROL_VOLTAGE_DESTINATION_COUNT
        );
    }

    #[test]
    fn control_voltage_destination_map_is_complete() {
        for index in 0..CONTROL_VOLTAGE_DESTINATION_COUNT as u8 {
            assert_eq!(
                ControlVoltageDestination::try_from(index).unwrap() as u8,
                index
            );
        }
        assert!(
            ControlVoltageDestination::try_from(CONTROL_VOLTAGE_DESTINATION_COUNT as u8).is_err()
        );
        for voice in 0..VOICE_COUNT {
            assert_eq!(
                ControlVoltageDestination::oscillator(voice, false).unwrap() as usize,
                COMMON_AND_PATCH_SAMPLE_HOLD_COUNT + voice * 2
            );
            assert_eq!(
                ControlVoltageDestination::filter(voice).unwrap() as usize,
                COMMON_AND_PATCH_SAMPLE_HOLD_COUNT + AUDIO_OSCILLATOR_COUNT + voice
            );
        }
    }

    #[test]
    fn firmware_strobe_order_contains_every_destination_once() {
        let mut seen = [false; CONTROL_VOLTAGE_DESTINATION_COUNT];
        for destination in CONTROL_VOLTAGE_STROBE_ORDER.into_iter().flatten() {
            let index = destination as usize;
            assert!(!seen[index]);
            seen[index] = true;
        }
        assert!(seen.into_iter().all(|strobed| strobed));
        assert_eq!(
            CONTROL_VOLTAGE_STROBE_ORDER
                .into_iter()
                .filter(Option::is_none)
                .count(),
            UNUSED_CONTROL_VOLTAGE_STROBE_SLOT_COUNT
        );
    }

    #[test]
    fn unconnected_strobe_slots_end_each_physical_board_group() {
        assert_eq!(CONTROL_VOLTAGE_STROBE_ORDER[23], None);
        assert_eq!(CONTROL_VOLTAGE_STROBE_ORDER[39], None);
        assert_eq!(
            CONTROL_VOLTAGE_STROBE_ORDER[24],
            Some(ControlVoltageDestination::Oscillator1A)
        );
    }

    #[test]
    fn automatic_tune_table_matches_scratchpad_layout() {
        assert_eq!(TUNE_OSCILLATOR_COUNT, 10);
        assert_eq!(TUNE_OCTAVE_BIAS_COUNT, 10);
        assert_eq!(TUNE_BIAS_BYTES, 200);
        assert_eq!(TUNE_DIRECT_MEASUREMENT_FIRST_OCTAVE, 3);
        assert_eq!(TUNE_DIRECT_MEASUREMENT_LAST_OCTAVE, 9);
    }

    #[test]
    fn oscillator_frequency_codes_match_the_admitted_rom_ranges() {
        assert_eq!(OSCILLATOR_FREQUENCY_CONCERT_POT_CODE >> 1, 24);
        assert_eq!(OSCILLATOR_FREQUENCY_NORMAL_MAX_SEMITONES, 48);
        assert_eq!(OSCILLATOR_FREQUENCY_LOW_MAX_SEMITONES, 108);
        assert_eq!(OSCILLATOR_FREQUENCY_LOW_OFFSET_SEMITONES, -90.0);
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
    fn program_switch_map_matches_the_three_v81_latch_bytes() {
        assert_eq!(ProgramSwitch::OscillatorAPulse as u8, 0);
        assert_eq!(ProgramSwitch::Unison as u8, 7);
        assert_eq!(ProgramSwitch::PolyModOscillatorAFrequency as u8, 8);
        assert_eq!(ProgramSwitch::Release as u8, 15);
        assert_eq!(ProgramSwitch::WheelModOscillatorAFrequency as u8, 16);
        assert_eq!(ProgramSwitch::OscillatorBLowFrequency as u8, 21);
        assert!(ProgramSwitch::from_storage_index(22).is_none());
        assert!(ProgramSwitch::from_storage_index(23).is_none());
        assert_eq!(PROGRAM_SWITCH_COUNT, 22);
    }

    #[test]
    fn program_codec_round_trips_every_stored_control_only() {
        let mut source = crate::Settings::default();
        for index in 0..ANALOG_POT_COUNT as u8 {
            let pot = AnalogPot::try_from(index).unwrap();
            assert!(source.set(pot.parameter() as u32, f64::from(index) / 23.0));
        }
        for index in 0..PROGRAM_SWITCH_COUNT as u8 {
            let switch = ProgramSwitch::from_storage_index(index).unwrap();
            assert!(source.set(switch.parameter() as u32, f64::from(index % 2)));
        }

        let mut destination = crate::Settings::default();
        assert!(destination.set(Parameter::MasterVolume as u32, 0.23));
        assert!(destination.set(Parameter::VintageSpread as u32, 0.67));
        assert!(destination.set(Parameter::ScaleE as u32, 0.31));
        let decoded = decode_program(encode_program(source), destination);

        for index in 0..ANALOG_POT_COUNT as u8 {
            let parameter = AnalogPot::try_from(index).unwrap().parameter();
            assert_eq!(
                decoded.get(parameter),
                quantize_analog_pot(source.get(parameter))
            );
        }
        for index in 0..PROGRAM_SWITCH_COUNT as u8 {
            let parameter = ProgramSwitch::from_storage_index(index)
                .unwrap()
                .parameter();
            assert_eq!(decoded.get(parameter), source.get(parameter));
        }
        assert_eq!(decoded.get(Parameter::MasterVolume), 0.23);
        assert_eq!(decoded.get(Parameter::VintageSpread), 0.67);
        assert_eq!(decoded.get(Parameter::ScaleE), 0.31);
        assert!(!encode_program(source)[22].switch_on());
        assert!(!encode_program(source)[23].switch_on());
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

    #[test]
    fn one_pot_code_exceeds_the_documented_comparator_window() {
        let millivolts_per_code =
            PANEL_POT_FULL_SCALE_VOLTS * 1_000.0 / (ANALOG_POT_STEPS - 1) as f32;
        assert!(millivolts_per_code > ADC_WINDOW_HYSTERESIS_MILLIVOLTS);
        assert_eq!(PANEL_POT_CONFIRMING_STEPS, 2);
        for code in 0..ANALOG_POT_STEPS as u8 {
            assert_eq!(
                analog_pot_code(code as f32 / (ANALOG_POT_STEPS - 1) as f32),
                code
            );
        }
    }
}
