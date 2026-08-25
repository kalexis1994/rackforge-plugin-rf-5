#![no_std]

//! Stable normalized vocabulary shared by the RF-5 engine and host adapter.
//! The small Milestone 0 contract intentionally exposes only controls that the
//! audible baseline implements. Circuit blocks add parameters after their
//! mappings pass the fidelity gates.

pub mod hardware;

pub const PATCH_PARAMETER_COUNT: usize = 48;
pub const SCALE_NOTE_COUNT: usize = 12;
pub const PARAMETER_COUNT: usize = PATCH_PARAMETER_COUNT + SCALE_NOTE_COUNT + 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Parameter {
    MasterVolume = 0,
    OscillatorALevel = 1,
    OscillatorBDetune = 2,
    FilterCutoff = 3,
    FilterResonance = 4,
    AmpAttack = 5,
    AmpDecay = 6,
    AmpSustain = 7,
    AmpRelease = 8,
    VintageSpread = 9,
    OscillatorBLevel = 10,
    OscillatorASaw = 11,
    OscillatorAPulse = 12,
    OscillatorAPulseWidth = 13,
    OscillatorBSaw = 14,
    OscillatorBTriangle = 15,
    OscillatorBPulse = 16,
    OscillatorBPulseWidth = 17,
    OscillatorSync = 18,
    OscillatorAFrequency = 19,
    OscillatorBFrequency = 20,
    OscillatorBLowFrequency = 21,
    OscillatorBKeyboard = 22,
    LfoFrequency = 23,
    LfoSaw = 24,
    LfoTriangle = 25,
    LfoSquare = 26,
    WheelModOscillatorAFrequency = 27,
    WheelModOscillatorBFrequency = 28,
    WheelModOscillatorAPulseWidth = 29,
    WheelModOscillatorBPulseWidth = 30,
    WheelModFilter = 31,
    NoiseLevel = 32,
    WheelModSourceMix = 33,
    FilterAttack = 34,
    FilterDecay = 35,
    FilterSustain = 36,
    FilterRelease = 37,
    FilterEnvelopeAmount = 38,
    PolyModFilterEnvelopeAmount = 39,
    PolyModOscillatorBAmount = 40,
    PolyModOscillatorAFrequency = 41,
    PolyModOscillatorAPulseWidth = 42,
    PolyModFilter = 43,
    FilterKeyboard = 44,
    Glide = 45,
    Unison = 46,
    ReleaseSwitch = 47,
    ScaleC = 48,
    ScaleCSharp = 49,
    ScaleD = 50,
    ScaleDSharp = 51,
    ScaleE = 52,
    ScaleF = 53,
    ScaleFSharp = 54,
    ScaleG = 55,
    ScaleGSharp = 56,
    ScaleA = 57,
    ScaleASharp = 58,
    ScaleB = 59,
    MasterTune = 60,
    A440 = 61,
    Tune = 62,
}

pub const SCALE_PARAMETERS: [Parameter; SCALE_NOTE_COUNT] = [
    Parameter::ScaleC,
    Parameter::ScaleCSharp,
    Parameter::ScaleD,
    Parameter::ScaleDSharp,
    Parameter::ScaleE,
    Parameter::ScaleF,
    Parameter::ScaleFSharp,
    Parameter::ScaleG,
    Parameter::ScaleGSharp,
    Parameter::ScaleA,
    Parameter::ScaleASharp,
    Parameter::ScaleB,
];

impl TryFrom<u32> for Parameter {
    type Error = ();

    fn try_from(index: u32) -> Result<Self, Self::Error> {
        match index {
            0 => Ok(Self::MasterVolume),
            1 => Ok(Self::OscillatorALevel),
            2 => Ok(Self::OscillatorBDetune),
            3 => Ok(Self::FilterCutoff),
            4 => Ok(Self::FilterResonance),
            5 => Ok(Self::AmpAttack),
            6 => Ok(Self::AmpDecay),
            7 => Ok(Self::AmpSustain),
            8 => Ok(Self::AmpRelease),
            9 => Ok(Self::VintageSpread),
            10 => Ok(Self::OscillatorBLevel),
            11 => Ok(Self::OscillatorASaw),
            12 => Ok(Self::OscillatorAPulse),
            13 => Ok(Self::OscillatorAPulseWidth),
            14 => Ok(Self::OscillatorBSaw),
            15 => Ok(Self::OscillatorBTriangle),
            16 => Ok(Self::OscillatorBPulse),
            17 => Ok(Self::OscillatorBPulseWidth),
            18 => Ok(Self::OscillatorSync),
            19 => Ok(Self::OscillatorAFrequency),
            20 => Ok(Self::OscillatorBFrequency),
            21 => Ok(Self::OscillatorBLowFrequency),
            22 => Ok(Self::OscillatorBKeyboard),
            23 => Ok(Self::LfoFrequency),
            24 => Ok(Self::LfoSaw),
            25 => Ok(Self::LfoTriangle),
            26 => Ok(Self::LfoSquare),
            27 => Ok(Self::WheelModOscillatorAFrequency),
            28 => Ok(Self::WheelModOscillatorBFrequency),
            29 => Ok(Self::WheelModOscillatorAPulseWidth),
            30 => Ok(Self::WheelModOscillatorBPulseWidth),
            31 => Ok(Self::WheelModFilter),
            32 => Ok(Self::NoiseLevel),
            33 => Ok(Self::WheelModSourceMix),
            34 => Ok(Self::FilterAttack),
            35 => Ok(Self::FilterDecay),
            36 => Ok(Self::FilterSustain),
            37 => Ok(Self::FilterRelease),
            38 => Ok(Self::FilterEnvelopeAmount),
            39 => Ok(Self::PolyModFilterEnvelopeAmount),
            40 => Ok(Self::PolyModOscillatorBAmount),
            41 => Ok(Self::PolyModOscillatorAFrequency),
            42 => Ok(Self::PolyModOscillatorAPulseWidth),
            43 => Ok(Self::PolyModFilter),
            44 => Ok(Self::FilterKeyboard),
            45 => Ok(Self::Glide),
            46 => Ok(Self::Unison),
            47 => Ok(Self::ReleaseSwitch),
            48 => Ok(Self::ScaleC),
            49 => Ok(Self::ScaleCSharp),
            50 => Ok(Self::ScaleD),
            51 => Ok(Self::ScaleDSharp),
            52 => Ok(Self::ScaleE),
            53 => Ok(Self::ScaleF),
            54 => Ok(Self::ScaleFSharp),
            55 => Ok(Self::ScaleG),
            56 => Ok(Self::ScaleGSharp),
            57 => Ok(Self::ScaleA),
            58 => Ok(Self::ScaleASharp),
            59 => Ok(Self::ScaleB),
            60 => Ok(Self::MasterTune),
            61 => Ok(Self::A440),
            62 => Ok(Self::Tune),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    values: [f32; PARAMETER_COUNT],
}

impl Default for Settings {
    fn default() -> Self {
        let concert_frequency = hardware::OSCILLATOR_FREQUENCY_CONCERT_NORMALIZED;
        Self {
            values: [
                0.72,
                0.72,
                5.0 / 127.0,
                0.72,
                0.08,
                0.01,
                0.2,
                0.82,
                0.28,
                0.18,
                0.64,
                1.0,
                0.0,
                0.5,
                1.0,
                0.0,
                0.0,
                0.5,
                0.0,
                concert_frequency,
                concert_frequency,
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
                hardware::SCALE_EQUAL_TEMPERAMENT_NORMALIZED,
                hardware::SCALE_EQUAL_TEMPERAMENT_NORMALIZED,
                hardware::SCALE_EQUAL_TEMPERAMENT_NORMALIZED,
                hardware::SCALE_EQUAL_TEMPERAMENT_NORMALIZED,
                hardware::SCALE_EQUAL_TEMPERAMENT_NORMALIZED,
                hardware::SCALE_EQUAL_TEMPERAMENT_NORMALIZED,
                hardware::SCALE_EQUAL_TEMPERAMENT_NORMALIZED,
                hardware::SCALE_EQUAL_TEMPERAMENT_NORMALIZED,
                hardware::SCALE_EQUAL_TEMPERAMENT_NORMALIZED,
                hardware::SCALE_EQUAL_TEMPERAMENT_NORMALIZED,
                hardware::SCALE_EQUAL_TEMPERAMENT_NORMALIZED,
                hardware::SCALE_EQUAL_TEMPERAMENT_NORMALIZED,
                0.5,
                0.0,
                0.0,
            ],
        }
    }
}

impl Settings {
    pub fn get(self, parameter: Parameter) -> f32 {
        self.values[parameter as usize]
    }

    pub fn get_index(self, index: u32) -> Option<f32> {
        let parameter = Parameter::try_from(index).ok()?;
        Some(self.get(parameter))
    }

    pub fn set(&mut self, index: u32, value: f64) -> bool {
        let Ok(parameter) = Parameter::try_from(index) else {
            return false;
        };
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return false;
        }
        self.values[parameter as usize] = value as f32;
        true
    }

    pub fn as_array(self) -> [f32; PARAMETER_COUNT] {
        self.values
    }

    pub fn from_array(values: [f32; PARAMETER_COUNT]) -> Option<Self> {
        values
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
            .then_some(Self { values })
    }

    pub fn apply_patch_array(&mut self, values: [f32; PATCH_PARAMETER_COUNT]) -> bool {
        if !values
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        {
            return false;
        }
        self.values[..PATCH_PARAMETER_COUNT].copy_from_slice(&values);
        true
    }

    pub fn scale_values(self) -> [f32; SCALE_NOTE_COUNT] {
        core::array::from_fn(|index| self.get(SCALE_PARAMETERS[index]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_rejects_unknown_and_out_of_range_values() {
        let mut settings = Settings::default();
        assert!(!settings.set(99, 0.5));
        assert!(!settings.set(Parameter::FilterCutoff as u32, -0.1));
        assert!(!settings.set(Parameter::FilterCutoff as u32, f64::NAN));
        assert!(settings.set(Parameter::FilterCutoff as u32, 0.25));
        assert_eq!(settings.get(Parameter::FilterCutoff), 0.25);
    }

    #[test]
    fn state_array_round_trips() {
        let settings = Settings::default();
        assert_eq!(Settings::from_array(settings.as_array()), Some(settings));
    }

    #[test]
    fn patch_updates_preserve_the_active_scale_program() {
        let mut settings = Settings::default();
        assert!(settings.set(Parameter::ScaleE as u32, 0.25));
        let scale = settings.scale_values();
        assert!(settings.apply_patch_array([0.5; PATCH_PARAMETER_COUNT]));
        assert_eq!(settings.scale_values(), scale);
        assert_eq!(settings.get(Parameter::FilterCutoff), 0.5);
    }
}
