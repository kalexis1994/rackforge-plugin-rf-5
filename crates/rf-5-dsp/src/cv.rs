//! Sequential control-voltage distribution and physical sample/hold cells.

use rf_5_contract::{
    Parameter, Settings,
    hardware::{
        COMMON_AND_PATCH_SAMPLE_HOLD_COUNT, CONTROL_VOLTAGE_DESTINATION_COUNT,
        ControlVoltageDestination, VOICE_COUNT,
    },
};
use rf_5_voice::{
    autotune::{AutoTune, Oscillator},
    tuning,
};

pub(crate) const GLIDE_CV_SPAN_VOLTS: f32 = 5.0;
const DEFAULT_COMMON_CV_SPAN_VOLTS: f32 = 5.0;
const FILTER_CONTROL_CV_SPAN_VOLTS: f32 = 10.0;
const SEMITONES_PER_CONTROL_VOLT: f32 = 12.0;

#[derive(Clone, Copy, Debug)]
struct SampleHoldCell {
    sampled_volts: f32,
    accumulated_leakage_volts: f64,
    leakage_volts_per_second: f64,
}

impl SampleHoldCell {
    fn new(index: usize) -> Self {
        let magnitude = 0.003 + ((index * 17) % 13) as f32 * 0.0008;
        let direction = if index.is_multiple_of(4) { -1.0 } else { 1.0 };
        Self {
            sampled_volts: 0.0,
            accumulated_leakage_volts: 0.0,
            leakage_volts_per_second: f64::from(magnitude * direction),
        }
    }

    fn age(&mut self, sample_rate: f32) {
        self.accumulated_leakage_volts +=
            self.leakage_volts_per_second / f64::from(sample_rate.max(1.0));
    }

    fn sample(&mut self, volts: f32) {
        self.sampled_volts = if volts.is_finite() { volts } else { 0.0 };
        self.accumulated_leakage_volts = 0.0;
    }

    fn volts(self) -> f32 {
        self.sampled_volts + self.accumulated_leakage_volts as f32
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CvTargets {
    volts: [f32; CONTROL_VOLTAGE_DESTINATION_COUNT],
}

impl CvTargets {
    pub fn from_state(settings: Settings, notes: [u8; VOICE_COUNT], autotune: AutoTune) -> Self {
        let mut volts = [0.0; CONTROL_VOLTAGE_DESTINATION_COUNT];
        for (index, value) in volts
            .iter_mut()
            .enumerate()
            .take(COMMON_AND_PATCH_SAMPLE_HOLD_COUNT)
        {
            let destination = ControlVoltageDestination::try_from(index as u8)
                .expect("valid common CV destination");
            if let Some(parameter) = common_parameter(destination) {
                *value = settings.get(parameter) * common_cv_span_volts(destination);
            }
        }

        for (voice, note) in notes.into_iter().enumerate() {
            let tuning_a = tuning::oscillator_a_tuning_semitones(
                note,
                settings.get(Parameter::OscillatorAFrequency),
            );
            let tuning_b = tuning::oscillator_b_tuning_semitones(
                note,
                settings.get(Parameter::OscillatorBFrequency),
                settings.get(Parameter::OscillatorBDetune),
                parameter_enabled(settings, Parameter::OscillatorBKeyboard),
                parameter_enabled(settings, Parameter::OscillatorBLowFrequency),
            );
            let oscillator_a = ControlVoltageDestination::oscillator(voice, false)
                .expect("valid oscillator-A CV destination")
                as usize;
            let oscillator_b = ControlVoltageDestination::oscillator(voice, true)
                .expect("valid oscillator-B CV destination")
                as usize;
            volts[oscillator_a] = (tuning_a
                + autotune.residual_semitones(voice, Oscillator::A, tuning_a))
                / SEMITONES_PER_CONTROL_VOLT;
            volts[oscillator_b] = (tuning_b
                + autotune.residual_semitones(voice, Oscillator::B, tuning_b))
                / SEMITONES_PER_CONTROL_VOLT;

            let filter = ControlVoltageDestination::filter(voice)
                .expect("valid filter CV destination") as usize;
            volts[filter] = filter_keyboard_octaves(
                note,
                parameter_enabled(settings, Parameter::FilterKeyboard),
            );
        }
        Self { volts }
    }

    fn get(self, destination: usize) -> f32 {
        self.volts[destination]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CvDistributor {
    cells: [SampleHoldCell; CONTROL_VOLTAGE_DESTINATION_COUNT],
}

impl Default for CvDistributor {
    fn default() -> Self {
        Self {
            cells: core::array::from_fn(SampleHoldCell::new),
        }
    }
}

impl CvDistributor {
    pub fn prepare(&mut self, targets: CvTargets) {
        for (index, cell) in self.cells.iter_mut().enumerate() {
            cell.sample(targets.get(index));
        }
    }

    pub fn age(&mut self, sample_rate: f32) {
        for cell in &mut self.cells {
            cell.age(sample_rate);
        }
    }

    pub fn strobe(&mut self, destination: usize, targets: CvTargets) {
        if let Some(cell) = self.cells.get_mut(destination) {
            cell.sample(targets.get(destination));
        }
    }

    pub fn refresh_voice(&mut self, voice: usize, targets: CvTargets) {
        for destination in [
            ControlVoltageDestination::oscillator(voice, false),
            ControlVoltageDestination::oscillator(voice, true),
            ControlVoltageDestination::filter(voice),
        ]
        .into_iter()
        .flatten()
        {
            self.strobe(destination as usize, targets);
        }
    }

    pub fn apply_common(self, input: Settings) -> Settings {
        let mut output = input;
        for index in 0..COMMON_AND_PATCH_SAMPLE_HOLD_COUNT {
            let destination = ControlVoltageDestination::try_from(index as u8)
                .expect("valid common CV destination");
            if let Some(parameter) = common_parameter(destination) {
                let normalized =
                    (self.cells[index].volts() / common_cv_span_volts(destination)).clamp(0.0, 1.0);
                let copied = output.set(parameter as u32, f64::from(normalized));
                debug_assert!(copied);
            }
        }
        output
    }

    pub fn oscillator_semitones(self, voice: usize, oscillator_b: bool) -> f32 {
        ControlVoltageDestination::oscillator(voice, oscillator_b)
            .map(|destination| {
                self.cells[destination as usize].volts() * SEMITONES_PER_CONTROL_VOLT
            })
            .unwrap_or(0.0)
    }

    pub fn filter_keyboard_octaves(self, voice: usize) -> f32 {
        ControlVoltageDestination::filter(voice)
            .map(|destination| self.cells[destination as usize].volts())
            .unwrap_or(0.0)
    }
}

fn common_cv_span_volts(destination: ControlVoltageDestination) -> f32 {
    match destination {
        // Service trim 4-14 measures the Filter Cutoff S/H at approximately
        // 10 V for panel maximum. Filter Resonance shares the same common DAC
        // and reaches the populated 200 kohm current-input resistor on SD431.
        ControlVoltageDestination::FilterCutoff | ControlVoltageDestination::FilterResonance => {
            FILTER_CONTROL_CV_SPAN_VOLTS
        }
        _ => DEFAULT_COMMON_CV_SPAN_VOLTS,
    }
}

fn common_parameter(destination: ControlVoltageDestination) -> Option<Parameter> {
    match destination {
        ControlVoltageDestination::FilterAttack => Some(Parameter::FilterAttack),
        ControlVoltageDestination::FilterDecay => Some(Parameter::FilterDecay),
        ControlVoltageDestination::FilterSustain => Some(Parameter::FilterSustain),
        ControlVoltageDestination::FilterRelease => Some(Parameter::FilterRelease),
        ControlVoltageDestination::AmplifierAttack => Some(Parameter::AmpAttack),
        ControlVoltageDestination::AmplifierDecay => Some(Parameter::AmpDecay),
        ControlVoltageDestination::AmplifierSustain => Some(Parameter::AmpSustain),
        ControlVoltageDestination::AmplifierRelease => Some(Parameter::AmpRelease),
        ControlVoltageDestination::FilterCutoff => Some(Parameter::FilterCutoff),
        ControlVoltageDestination::FilterEnvelopeAmount => Some(Parameter::FilterEnvelopeAmount),
        ControlVoltageDestination::OscillatorBMix => Some(Parameter::OscillatorBLevel),
        ControlVoltageDestination::OscillatorBPulseWidth => Some(Parameter::OscillatorBPulseWidth),
        ControlVoltageDestination::OscillatorAMix => Some(Parameter::OscillatorALevel),
        ControlVoltageDestination::OscillatorAPulseWidth => Some(Parameter::OscillatorAPulseWidth),
        ControlVoltageDestination::NoiseMix => Some(Parameter::NoiseLevel),
        ControlVoltageDestination::FilterResonance => Some(Parameter::FilterResonance),
        ControlVoltageDestination::Glide => Some(Parameter::Glide),
        ControlVoltageDestination::LfoFrequency => Some(Parameter::LfoFrequency),
        ControlVoltageDestination::WheelModSourceMix => Some(Parameter::WheelModSourceMix),
        ControlVoltageDestination::PolyModOscillatorBAmount => {
            Some(Parameter::PolyModOscillatorBAmount)
        }
        ControlVoltageDestination::PolyModFilterEnvelopeAmount => {
            Some(Parameter::PolyModFilterEnvelopeAmount)
        }
        ControlVoltageDestination::Unison => Some(Parameter::Unison),
        _ => None,
    }
}

fn parameter_enabled(settings: Settings, parameter: Parameter) -> bool {
    settings.get(parameter) >= 0.5
}

pub(crate) fn filter_keyboard_octaves(note: u8, keyboard_enabled: bool) -> f32 {
    if keyboard_enabled {
        (f32::from(note) - 36.0) / 12.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rf_5_contract::hardware::SAMPLE_HOLD_SERVICE_DROOP_LIMIT_VOLTS_PER_7_MS;

    #[test]
    fn prepared_common_cells_round_trip_all_mapped_controls() {
        let mut settings = Settings::default();
        assert!(settings.set(Parameter::FilterCutoff as u32, 0.73));
        assert!(settings.set(Parameter::AmpRelease as u32, 0.41));
        assert!(settings.set(Parameter::PolyModOscillatorBAmount as u32, 0.62));
        let targets = CvTargets::from_state(settings, [60; VOICE_COUNT], AutoTune::calibrated());
        let mut distributor = CvDistributor::default();
        distributor.prepare(targets);
        let applied = distributor.apply_common(settings);
        for parameter in [
            Parameter::FilterCutoff,
            Parameter::AmpRelease,
            Parameter::PolyModOscillatorBAmount,
        ] {
            assert!((applied.get(parameter) - settings.get(parameter)).abs() < 1.0e-6);
        }
    }

    #[test]
    fn service_calibrated_filter_controls_use_the_ten_volt_domain() {
        let mut settings = Settings::default();
        assert!(settings.set(Parameter::FilterCutoff as u32, 1.0));
        assert!(settings.set(Parameter::FilterResonance as u32, 1.0));
        assert!(settings.set(Parameter::Glide as u32, 1.0));
        let targets = CvTargets::from_state(settings, [60; VOICE_COUNT], AutoTune::calibrated());

        assert_eq!(
            targets.get(ControlVoltageDestination::FilterCutoff as usize),
            FILTER_CONTROL_CV_SPAN_VOLTS
        );
        assert_eq!(
            targets.get(ControlVoltageDestination::FilterResonance as usize),
            FILTER_CONTROL_CV_SPAN_VOLTS
        );
        assert_eq!(
            targets.get(ControlVoltageDestination::Glide as usize),
            GLIDE_CV_SPAN_VOLTS
        );
    }

    #[test]
    fn every_cell_stays_below_the_service_droop_limit() {
        let mut distributor = CvDistributor::default();
        let before = distributor.cells;
        for _ in 0..336 {
            distributor.age(48_000.0);
        }
        for (aged, initial) in distributor.cells.iter().zip(before) {
            assert!(
                (aged.volts() - initial.volts()).abs()
                    <= SAMPLE_HOLD_SERVICE_DROOP_LIMIT_VOLTS_PER_7_MS
            );
        }
    }

    #[test]
    fn strobe_reacquires_the_target_exactly() {
        let settings = Settings::default();
        let targets = CvTargets::from_state(settings, [60; VOICE_COUNT], AutoTune::calibrated());
        let mut distributor = CvDistributor::default();
        distributor.prepare(targets);
        for _ in 0..528 {
            distributor.age(48_000.0);
        }
        let destination = ControlVoltageDestination::Oscillator3B as usize;
        assert_ne!(
            distributor.cells[destination].volts(),
            targets.get(destination)
        );
        distributor.strobe(destination, targets);
        assert_eq!(
            distributor.cells[destination].volts(),
            targets.get(destination)
        );
    }

    #[test]
    fn ten_oscillator_and_five_filter_cells_are_independent() {
        let mut settings = Settings::default();
        assert!(settings.set(Parameter::FilterKeyboard as u32, 1.0));
        let targets = CvTargets::from_state(settings, [36, 48, 60, 72, 84], AutoTune::calibrated());
        let mut distributor = CvDistributor::default();
        distributor.prepare(targets);
        for voice in 0..VOICE_COUNT - 1 {
            assert_ne!(
                distributor.oscillator_semitones(voice, false),
                distributor.oscillator_semitones(voice + 1, false)
            );
            assert_ne!(
                distributor.filter_keyboard_octaves(voice),
                distributor.filter_keyboard_octaves(voice + 1)
            );
        }
    }
}
