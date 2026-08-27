//! Sequential control-voltage distribution and physical sample/hold cells.

use rf_5_contract::{
    Parameter, Settings,
    hardware::{
        COMMON_AND_PATCH_SAMPLE_HOLD_COUNT, CONTROL_VOLTAGE_DESTINATION_COUNT,
        ControlVoltageDestination, RELEASE_DISABLED_EQUIVALENT_NORMALIZED,
        SAMPLE_HOLD_CAPACITANCE_FARADS, SAMPLE_HOLD_STROBE_T_STATES,
        SAMPLE_HOLD_SWITCH_ON_RESISTANCE_UPPER_BOUND_OHMS, TUNE_CPU_CLOCK_HZ, VOICE_COUNT,
    },
};
use rf_5_voice::{
    autotune::{AutoTune, Oscillator},
    scale::ScaleProgram,
    tuning,
};

pub(crate) const GLIDE_CV_SPAN_VOLTS: f32 = 5.0;
const DEFAULT_COMMON_CV_SPAN_VOLTS: f32 = 5.0;
const FILTER_CONTROL_CV_SPAN_VOLTS: f32 = 10.0;
const PATCH_AMOUNT_CV_SPAN_VOLTS: f32 = 10.0;
const AUDIO_LEVEL_CV_SPAN_VOLTS: f32 = 10.0;
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

    #[cfg(test)]
    fn age(&mut self, sample_rate: f32) {
        self.age_by(self.leakage_per_sample(sample_rate));
    }

    fn leakage_per_sample(self, sample_rate: f32) -> f64 {
        self.leakage_volts_per_second / f64::from(sample_rate.max(1.0))
    }

    fn age_by(&mut self, leakage_volts: f64) {
        self.accumulated_leakage_volts += leakage_volts;
    }

    fn force(&mut self, volts: f32) {
        self.sampled_volts = if volts.is_finite() { volts } else { 0.0 };
        self.accumulated_leakage_volts = 0.0;
    }

    fn acquire(&mut self, volts: f32) {
        let target = if volts.is_finite() { volts } else { 0.0 };
        let held = self.volts();
        self.sampled_volts = held + (target - held) * sample_hold_acquisition_fraction();
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
    pub fn from_state(
        settings: Settings,
        notes: [u8; VOICE_COUNT],
        autotune: AutoTune,
        scale: ScaleProgram,
    ) -> Self {
        let volts = core::array::from_fn(|index| {
            let destination =
                ControlVoltageDestination::try_from(index as u8).expect("valid CV destination");
            destination_voltage(settings, notes, autotune, scale, destination)
        });
        Self { volts }
    }

    fn get(self, destination: usize) -> f32 {
        self.volts[destination]
    }
}

pub(crate) fn destination_voltage(
    settings: Settings,
    notes: [u8; VOICE_COUNT],
    autotune: AutoTune,
    scale: ScaleProgram,
    destination: ControlVoltageDestination,
) -> f32 {
    let index = destination as usize;
    if index < COMMON_AND_PATCH_SAMPLE_HOLD_COUNT {
        if let Some(parameter) = common_parameter(destination) {
            let normalized = if matches!(
                destination,
                ControlVoltageDestination::FilterRelease
                    | ControlVoltageDestination::AmplifierRelease
            ) && !parameter_enabled(settings, Parameter::ReleaseSwitch)
            {
                RELEASE_DISABLED_EQUIVALENT_NORMALIZED
            } else {
                settings.get(parameter)
            };
            return normalized * common_cv_span_volts(destination);
        }
        return if destination == ControlVoltageDestination::UnisonKeyboard
            && parameter_enabled(settings, Parameter::Unison)
        {
            f32::from(notes[0].saturating_sub(tuning::LOWEST_KEY_MIDI_NOTE))
                / SEMITONES_PER_CONTROL_VOLT
        } else {
            0.0
        };
    }

    if index < ControlVoltageDestination::Filter1 as usize {
        let oscillator_index = index - ControlVoltageDestination::Oscillator1A as usize;
        let voice = oscillator_index / 2;
        let oscillator_b = !oscillator_index.is_multiple_of(2);
        let note = notes[voice];
        let scale_offset = scale.offset_semitones(note);
        let unison = parameter_enabled(settings, Parameter::Unison);
        let keyboard_semitones = f32::from(note.saturating_sub(tuning::LOWEST_KEY_MIDI_NOTE));
        if oscillator_b {
            let keyboard_enabled = parameter_enabled(settings, Parameter::OscillatorBKeyboard);
            let pitch = tuning::oscillator_b_pitch(
                note,
                settings.get(Parameter::OscillatorBFrequency),
                settings.get(Parameter::OscillatorBDetune),
                keyboard_enabled,
                parameter_enabled(settings, Parameter::OscillatorBLowFrequency),
            );
            return (pitch.output_semitones()
                + autotune.residual_semitones(
                    voice,
                    Oscillator::B,
                    pitch.tune_dac_semitones(),
                    pitch.tune_table_semitone(),
                )
                + scale_offset
                - if unison && keyboard_enabled {
                    keyboard_semitones
                } else {
                    0.0
                })
                / SEMITONES_PER_CONTROL_VOLT;
        }
        let pitch = tuning::oscillator_a_pitch(note, settings.get(Parameter::OscillatorAFrequency));
        return (pitch.output_semitones()
            + autotune.residual_semitones(
                voice,
                Oscillator::A,
                pitch.tune_dac_semitones(),
                pitch.tune_table_semitone(),
            )
            + scale_offset
            - if unison { keyboard_semitones } else { 0.0 })
            / SEMITONES_PER_CONTROL_VOLT;
    }

    let voice = index - ControlVoltageDestination::Filter1 as usize;
    if parameter_enabled(settings, Parameter::Unison) {
        0.0
    } else {
        filter_keyboard_octaves(
            notes[voice],
            parameter_enabled(settings, Parameter::FilterKeyboard),
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CvDistributor {
    cells: [SampleHoldCell; CONTROL_VOLTAGE_DESTINATION_COUNT],
    leakage_sample_rate: f32,
    leakage_per_sample: [f64; CONTROL_VOLTAGE_DESTINATION_COUNT],
}

impl Default for CvDistributor {
    fn default() -> Self {
        Self {
            cells: core::array::from_fn(SampleHoldCell::new),
            leakage_sample_rate: 0.0,
            leakage_per_sample: [0.0; CONTROL_VOLTAGE_DESTINATION_COUNT],
        }
    }
}

impl CvDistributor {
    pub fn prepare(&mut self, targets: CvTargets) {
        for (index, cell) in self.cells.iter_mut().enumerate() {
            cell.force(targets.get(index));
        }
    }

    pub fn age(&mut self, sample_rate: f32) {
        let sample_rate = sample_rate.max(1.0);
        if self.leakage_sample_rate.to_bits() != sample_rate.to_bits() {
            for (increment, cell) in self.leakage_per_sample.iter_mut().zip(self.cells) {
                *increment = cell.leakage_per_sample(sample_rate);
            }
            self.leakage_sample_rate = sample_rate;
        }
        for (cell, increment) in self.cells.iter_mut().zip(self.leakage_per_sample) {
            cell.age_by(increment);
        }
    }

    #[cfg(test)]
    fn strobe(&mut self, destination: usize, targets: CvTargets) {
        self.strobe_voltage(destination, targets.get(destination));
    }

    pub fn strobe_voltage(&mut self, destination: usize, target_volts: f32) {
        if let Some(cell) = self.cells.get_mut(destination) {
            cell.acquire(target_volts);
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
            let destination = destination as usize;
            self.cells[destination].force(targets.get(destination));
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

    pub fn unison_keyboard_semitones(self) -> f32 {
        self.cells[ControlVoltageDestination::UnisonKeyboard as usize].volts()
            * SEMITONES_PER_CONTROL_VOLT
    }
}

fn sample_hold_acquisition_fraction() -> f32 {
    let dwell_seconds = SAMPLE_HOLD_STROBE_T_STATES as f32 / TUNE_CPU_CLOCK_HZ as f32;
    let time_constant_seconds =
        SAMPLE_HOLD_SWITCH_ON_RESISTANCE_UPPER_BOUND_OHMS * SAMPLE_HOLD_CAPACITANCE_FARADS;
    1.0 - libm::expf(-dwell_seconds / time_constant_seconds)
}

fn common_cv_span_volts(destination: ControlVoltageDestination) -> f32 {
    match destination {
        // Service trim 4-14 measures the Filter Cutoff S/H at approximately
        // 10 V for panel maximum. Filter Resonance shares the same common DAC
        // and reaches the populated 200 kohm current-input resistor on SD431.
        ControlVoltageDestination::FilterCutoff | ControlVoltageDestination::FilterResonance => {
            FILTER_CONTROL_CV_SPAN_VOLTS
        }
        // SD332 reaches approximately 10.67 V and V8.1 normally caps patch
        // CVs at 10 V. SD333 sends these three held voltages through Q301,
        // Q303 and Q304 before their collector currents reach the voice cards.
        ControlVoltageDestination::FilterEnvelopeAmount
        | ControlVoltageDestination::PolyModOscillatorBAmount
        | ControlVoltageDestination::PolyModFilterEnvelopeAmount => PATCH_AMOUNT_CV_SPAN_VOLTS,
        // SD333 buffers these held voltages into grounded-base Q306/Q302/Q305.
        // Their populated 33k/33k/75k emitter resistors set the two oscillator
        // mixer currents and the common noise-VCA current respectively.
        ControlVoltageDestination::OscillatorAMix
        | ControlVoltageDestination::OscillatorBMix
        | ControlVoltageDestination::NoiseMix => AUDIO_LEVEL_CV_SPAN_VOLTS,
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
        let targets = CvTargets::from_state(
            settings,
            [60; VOICE_COUNT],
            AutoTune::calibrated(),
            ScaleProgram::default(),
        );
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
    fn release_switch_off_targets_both_v81_fixed_voltage_cells() {
        let mut settings = Settings::default();
        assert!(settings.set(Parameter::FilterRelease as u32, 1.0));
        assert!(settings.set(Parameter::AmpRelease as u32, 1.0));
        assert!(settings.set(Parameter::ReleaseSwitch as u32, 0.0));
        let targets = CvTargets::from_state(
            settings,
            [60; VOICE_COUNT],
            AutoTune::calibrated(),
            ScaleProgram::default(),
        );
        let expected = RELEASE_DISABLED_EQUIVALENT_NORMALIZED * DEFAULT_COMMON_CV_SPAN_VOLTS;
        assert_eq!(
            targets.get(ControlVoltageDestination::FilterRelease as usize),
            expected
        );
        assert_eq!(
            targets.get(ControlVoltageDestination::AmplifierRelease as usize),
            expected
        );
    }

    #[test]
    fn service_calibrated_filter_controls_use_the_ten_volt_domain() {
        let mut settings = Settings::default();
        assert!(settings.set(Parameter::FilterCutoff as u32, 1.0));
        assert!(settings.set(Parameter::FilterResonance as u32, 1.0));
        assert!(settings.set(Parameter::FilterEnvelopeAmount as u32, 1.0));
        assert!(settings.set(Parameter::PolyModOscillatorBAmount as u32, 1.0));
        assert!(settings.set(Parameter::PolyModFilterEnvelopeAmount as u32, 1.0));
        assert!(settings.set(Parameter::OscillatorALevel as u32, 1.0));
        assert!(settings.set(Parameter::OscillatorBLevel as u32, 1.0));
        assert!(settings.set(Parameter::NoiseLevel as u32, 1.0));
        assert!(settings.set(Parameter::Glide as u32, 1.0));
        let targets = CvTargets::from_state(
            settings,
            [60; VOICE_COUNT],
            AutoTune::calibrated(),
            ScaleProgram::default(),
        );

        assert_eq!(
            targets.get(ControlVoltageDestination::FilterCutoff as usize),
            FILTER_CONTROL_CV_SPAN_VOLTS
        );
        assert_eq!(
            targets.get(ControlVoltageDestination::FilterResonance as usize),
            FILTER_CONTROL_CV_SPAN_VOLTS
        );
        for destination in [
            ControlVoltageDestination::FilterEnvelopeAmount,
            ControlVoltageDestination::PolyModOscillatorBAmount,
            ControlVoltageDestination::PolyModFilterEnvelopeAmount,
        ] {
            assert_eq!(
                targets.get(destination as usize),
                PATCH_AMOUNT_CV_SPAN_VOLTS
            );
        }
        for destination in [
            ControlVoltageDestination::OscillatorAMix,
            ControlVoltageDestination::OscillatorBMix,
            ControlVoltageDestination::NoiseMix,
        ] {
            assert_eq!(targets.get(destination as usize), AUDIO_LEVEL_CV_SPAN_VOLTS);
        }
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
    fn scheduled_strobe_obeys_the_populated_rc_acquisition() {
        let mut initial_settings = Settings::default();
        assert!(initial_settings.set(Parameter::FilterCutoff as u32, 0.0));
        let initial_targets = CvTargets::from_state(
            initial_settings,
            [60; VOICE_COUNT],
            AutoTune::calibrated(),
            ScaleProgram::default(),
        );
        let mut target_settings = initial_settings;
        assert!(target_settings.set(Parameter::FilterCutoff as u32, 1.0));
        let targets = CvTargets::from_state(
            target_settings,
            [60; VOICE_COUNT],
            AutoTune::calibrated(),
            ScaleProgram::default(),
        );
        let mut distributor = CvDistributor::default();
        distributor.prepare(initial_targets);
        let destination = ControlVoltageDestination::FilterCutoff as usize;
        distributor.strobe(destination, targets);
        let expected = targets.get(destination) * sample_hold_acquisition_fraction();
        assert!(
            (distributor.cells[destination].volts() - expected).abs() < 1.0e-6,
            "{} != {expected}",
            distributor.cells[destination].volts()
        );
    }

    #[test]
    fn populated_strobe_window_has_the_expected_time_and_fraction() {
        let dwell_microseconds =
            SAMPLE_HOLD_STROBE_T_STATES as f32 / TUNE_CPU_CLOCK_HZ as f32 * 1.0e6;
        let time_constant_microseconds = SAMPLE_HOLD_SWITCH_ON_RESISTANCE_UPPER_BOUND_OHMS
            * SAMPLE_HOLD_CAPACITANCE_FARADS
            * 1.0e6;
        assert!((dwell_microseconds - 25.6).abs() < 1.0e-5);
        assert!((time_constant_microseconds - 1.75).abs() < 1.0e-5);
        assert!(sample_hold_acquisition_fraction() > 0.999_999);
    }

    #[test]
    fn repeated_strobes_converge_monotonically_to_the_target() {
        let mut cell = SampleHoldCell::new(0);
        let mut previous = cell.volts();
        for _ in 0..2 {
            cell.acquire(5.0);
            assert!(cell.volts() > previous);
            assert!(cell.volts() <= 5.0);
            previous = cell.volts();
        }
        assert!((5.0 - cell.volts()) < 1.0e-6);
    }

    #[test]
    fn unison_cell_holds_keyboard_cv_while_individual_cells_remove_it() {
        let mut poly_settings = Settings::default();
        assert!(poly_settings.set(Parameter::FilterKeyboard as u32, 1.0));
        let poly = CvTargets::from_state(
            poly_settings,
            [60; VOICE_COUNT],
            AutoTune::calibrated(),
            ScaleProgram::default(),
        );
        let mut unison_settings = poly_settings;
        assert!(unison_settings.set(Parameter::Unison as u32, 1.0));
        let unison = CvTargets::from_state(
            unison_settings,
            [60; VOICE_COUNT],
            AutoTune::calibrated(),
            ScaleProgram::default(),
        );

        let oscillator_a = ControlVoltageDestination::Oscillator1A as usize;
        let filter = ControlVoltageDestination::Filter1 as usize;
        assert!(
            (unison.get(ControlVoltageDestination::UnisonKeyboard as usize) - 2.0).abs() < 1.0e-6
        );
        assert!((poly.get(oscillator_a) - unison.get(oscillator_a) - 2.0).abs() < 1.0e-6);
        assert_eq!(unison.get(filter), 0.0);
        assert_eq!(poly.get(filter), 2.0);
    }

    #[test]
    fn acquisition_continues_from_the_leaked_voltage() {
        let mut cell = SampleHoldCell::new(1);
        cell.force(2.0);
        for _ in 0..336 {
            cell.age(48_000.0);
        }
        let held = cell.volts();
        cell.acquire(4.0);
        let expected = held + (4.0 - held) * sample_hold_acquisition_fraction();
        assert!((cell.volts() - expected).abs() < 1.0e-6);
        assert_eq!(cell.accumulated_leakage_volts, 0.0);
    }

    #[test]
    fn ten_oscillator_and_five_filter_cells_are_independent() {
        let mut settings = Settings::default();
        assert!(settings.set(Parameter::FilterKeyboard as u32, 1.0));
        let targets = CvTargets::from_state(
            settings,
            [36, 48, 60, 72, 84],
            AutoTune::calibrated(),
            ScaleProgram::default(),
        );
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

    #[test]
    fn scale_mode_offsets_both_vcos_but_not_filter_keyboard_cv() {
        let mut settings = Settings::default();
        assert!(settings.set(Parameter::FilterKeyboard as u32, 1.0));
        let equal = CvTargets::from_state(
            settings,
            [64; VOICE_COUNT],
            AutoTune::calibrated(),
            ScaleProgram::default(),
        );
        let mut codes = [64; 12];
        codes[4] = 46;
        let alternate = CvTargets::from_state(
            settings,
            [64; VOICE_COUNT],
            AutoTune::calibrated(),
            ScaleProgram::from_codes(codes).unwrap(),
        );
        let expected_volts = (-36.0 / 256.0) / SEMITONES_PER_CONTROL_VOLT;
        for oscillator_b in [false, true] {
            let destination =
                ControlVoltageDestination::oscillator(0, oscillator_b).unwrap() as usize;
            assert!(
                (alternate.get(destination) - equal.get(destination) - expected_volts).abs()
                    < 1.0e-6
            );
        }
        let filter = ControlVoltageDestination::filter(0).unwrap() as usize;
        assert_eq!(alternate.get(filter), equal.get(filter));
    }
}
