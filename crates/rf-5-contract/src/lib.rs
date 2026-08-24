#![no_std]

//! Stable normalized vocabulary shared by the RF-5 engine and host adapter.
//! The small Milestone 0 contract intentionally exposes only controls that the
//! audible baseline implements. Circuit blocks add parameters after their
//! mappings pass the fidelity gates.

pub const PARAMETER_COUNT: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Parameter {
    MasterVolume = 0,
    OscillatorMix = 1,
    OscillatorBDetune = 2,
    FilterCutoff = 3,
    FilterResonance = 4,
    AmpAttack = 5,
    AmpDecay = 6,
    AmpSustain = 7,
    AmpRelease = 8,
    VintageSpread = 9,
}

impl TryFrom<u32> for Parameter {
    type Error = ();

    fn try_from(index: u32) -> Result<Self, Self::Error> {
        match index {
            0 => Ok(Self::MasterVolume),
            1 => Ok(Self::OscillatorMix),
            2 => Ok(Self::OscillatorBDetune),
            3 => Ok(Self::FilterCutoff),
            4 => Ok(Self::FilterResonance),
            5 => Ok(Self::AmpAttack),
            6 => Ok(Self::AmpDecay),
            7 => Ok(Self::AmpSustain),
            8 => Ok(Self::AmpRelease),
            9 => Ok(Self::VintageSpread),
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
        Self {
            values: [0.72, 0.5, 0.54, 0.72, 0.08, 0.01, 0.2, 0.82, 0.28, 0.18],
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
}
