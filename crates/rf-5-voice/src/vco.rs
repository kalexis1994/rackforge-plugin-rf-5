//! Band-limited numerical model of one CEM3340-class oscillator core.
//!
//! The chip topology and available outputs are source-backed. PolyBLEP edge
//! correction and four-times internal oversampling are RF-5's numerical
//! strategy, not claims about circuitry inside the physical IC.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WaveSelection {
    pub saw: bool,
    pub triangle: bool,
    pub pulse: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OscillatorSample {
    /// Conductance-weighted physical source voltage delivered to the positive
    /// oscillator-mixer input. SD431 routes saw here.
    pub mixer_positive_source_volts: f32,
    /// Selected positive-input conductance relative to one 150 kohm path.
    pub mixer_positive_source_conductance: f32,
    /// Conductance-weighted physical source voltage delivered to the negative
    /// oscillator-mixer input. SD431 routes pulse and oscillator-B triangle
    /// here, preserving their phase relationship to saw.
    pub mixer_negative_source_volts: f32,
    /// Selected negative-input conductance relative to one 150 kohm path.
    pub mixer_negative_source_conductance: f32,
    /// Conductance-weighted physical source voltage delivered to
    /// oscillator-B Poly Mod. U451 level-shifts only the triangle path.
    pub poly_mod_source_volts: f32,
    /// Sum of the selected Poly Mod source conductances relative to one
    /// 150 kohm path. All three sources meet one U428 input.
    pub poly_mod_source_conductance: f32,
    pub wrapped: bool,
    /// Bipolar pulses and their fractional positions inside this internal
    /// sample, produced by capacitively coupling the pulse output into another
    /// CEM3340 hard-sync input.
    pub sync_events: [HardSyncEvent; 2],
}

impl OscillatorSample {
    /// Unloaded differential source voltage, useful for oscillator-only
    /// spectral probes. The signal path itself loads both inputs separately.
    pub fn mixer_differential_source_volts(self) -> f32 {
        self.mixer_positive_source_volts - self.mixer_negative_source_volts
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum HardSyncPulse {
    #[default]
    None,
    Positive,
    Negative,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HardSyncEvent {
    pub pulse: HardSyncPulse,
    /// Position inside the current internal sample, from zero to one.
    pub offset: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Vco {
    phase: f32,
    profile_index: usize,
}

impl Default for Vco {
    fn default() -> Self {
        Self {
            phase: 0.0,
            profile_index: 0,
        }
    }
}

const OUTPUT_PROFILE_COUNT: usize = 10;
const SAW_UPPER_VOLTS: [f32; OUTPUT_PROFILE_COUNT] = [
    9.40, 9.55, 9.72, 9.88, 10.0, 10.12, 10.28, 10.42, 10.55, 10.60,
];
const SAW_LOWER_VOLTS: [f32; OUTPUT_PROFILE_COUNT] = [
    -0.025, 0.012, -0.018, 0.006, 0.0, 0.021, -0.009, 0.016, -0.004, 0.025,
];
const TRIANGLE_UPPER_VOLTS: [f32; OUTPUT_PROFILE_COUNT] =
    [4.85, 4.91, 4.96, 5.02, 5.08, 5.13, 4.88, 4.94, 5.05, 5.15];
const TRIANGLE_LOWER_VOLTS: [f32; OUTPUT_PROFILE_COUNT] = [
    -0.015, 0.007, -0.011, 0.004, 0.0, 0.013, -0.006, 0.009, -0.003, 0.015,
];
const TRIANGLE_SYMMETRY: [f32; OUTPUT_PROFILE_COUNT] = [
    0.450, 0.472, 0.489, 0.507, 0.529, 0.550, 0.462, 0.481, 0.518, 0.541,
];
const TRIANGLE_OUTPUT_IMPEDANCE_OHMS: [f32; OUTPUT_PROFILE_COUNT] = [
    65.0, 78.0, 91.0, 100.0, 112.0, 125.0, 138.0, 150.0, 84.0, 106.0,
];

// The voice board uses 150 kohm inputs for saw/triangle and 200 kohm for
// pulse. With +15/-5 V supplies, the data-sheet pulse formulas and the 4016
// negative clamp give approximately -0.6 V and +14.1 V after selection.
// Values below retain circuit volts and are expressed relative to one 150
// kohm input conductance. Loading by the CA3280 input itself is applied at the
// VCA boundary, where all simultaneously selected paths are known.
const SAW_TRIANGLE_MIXER_CONDUCTANCE: f32 = 1.0;
const TRIANGLE_MIXER_LOAD_RESISTANCE_OHMS: f32 = 150_000.0;
const PULSE_MIXER_CONDUCTANCE: f32 = 150_000.0 / 200_000.0;
const PULSE_LOWER_VOLTS: f32 = -0.6;
const PULSE_UPPER_VOLTS: f32 = 14.1;
// SD431 derives 2.27 V TRI REF and U451 subtracts it from OSC B's raw
// positive-going triangle before the Poly Mod amount OTA.
const TRIANGLE_POLY_MOD_REFERENCE_VOLTS: f32 = 2.27;
// The correction spans two host samples at the four-times internal rate. A
// wider polynomial transition is necessary when the 1%/99% hardware pulse
// endpoints put both discontinuities inside one short reconstruction window.
const POLY_BLEP_WIDTH: f32 = 8.0;

impl Vco {
    pub fn with_phase(phase: f32) -> Self {
        Self::with_phase_and_profile(phase, 0)
    }

    pub fn with_phase_and_profile(phase: f32, profile_index: usize) -> Self {
        let phase = if phase.is_finite() { phase % 1.0 } else { 0.0 };
        Self {
            phase: if phase < 0.0 { phase + 1.0 } else { phase },
            profile_index: profile_index % OUTPUT_PROFILE_COUNT,
        }
    }

    /// Apply one polarity from the physical CEM3340 hard-sync input.
    ///
    /// Positive pulses reverse only a rising triangle and negative pulses
    /// reverse only a falling triangle. Reflecting phase onto the opposite
    /// branch preserves triangle voltage while allowing saw and pulse to make
    /// the discontinuities shown in the data sheet.
    pub fn hard_sync_pulse(&mut self, pulse: HardSyncPulse) -> bool {
        let symmetry = TRIANGLE_SYMMETRY[self.profile_index];
        match pulse {
            HardSyncPulse::Positive if self.phase < symmetry => {
                self.phase = 1.0 - self.phase * (1.0 - symmetry) / symmetry;
                self.phase = self.phase.min(1.0 - f32::EPSILON);
                true
            }
            HardSyncPulse::Negative if self.phase > symmetry => {
                self.phase = symmetry * (1.0 - self.phase) / (1.0 - symmetry);
                true
            }
            HardSyncPulse::None | HardSyncPulse::Positive | HardSyncPulse::Negative => false,
        }
    }

    pub fn phase(self) -> f32 {
        self.phase
    }

    pub fn next(
        &mut self,
        frequency: f32,
        sample_rate: f32,
        pulse_width: f32,
        waves: WaveSelection,
    ) -> OscillatorSample {
        self.next_with_sync(
            frequency,
            sample_rate,
            pulse_width,
            waves,
            [HardSyncEvent::default(); 2],
        )
    }

    pub fn next_with_sync(
        &mut self,
        frequency: f32,
        sample_rate: f32,
        pulse_width: f32,
        waves: WaveSelection,
        external_sync: [HardSyncEvent; 2],
    ) -> OscillatorSample {
        let profile = self.profile_index;
        let frequency = triangle_loaded_frequency(frequency.max(0.0), profile, waves.triangle);
        let increment = (frequency / sample_rate.max(1.0)).clamp(0.0, 0.49);
        let pulse_width = pulse_width.clamp(0.0, 1.0);
        let phase = self.phase;
        let mut mixer_positive_source_volts = 0.0;
        let mut mixer_positive_source_conductance = 0.0;
        let mut mixer_negative_source_volts = 0.0;
        let mut mixer_negative_source_conductance = 0.0;
        let mut poly_mod_source_volts = 0.0;
        let mut poly_mod_source_conductance = 0.0;

        if waves.saw {
            let centered = band_limited_saw(phase, increment);
            let half_range = (SAW_UPPER_VOLTS[profile] - SAW_LOWER_VOLTS[profile]) * 0.5;
            let midpoint = (SAW_UPPER_VOLTS[profile] + SAW_LOWER_VOLTS[profile]) * 0.5;
            let source_volts = centered * half_range + midpoint;
            mixer_positive_source_volts += source_volts;
            poly_mod_source_volts += source_volts;
            mixer_positive_source_conductance += SAW_TRIANGLE_MIXER_CONDUCTANCE;
            poly_mod_source_conductance += SAW_TRIANGLE_MIXER_CONDUCTANCE;
        }
        if waves.triangle {
            let centered = triangle(phase, TRIANGLE_SYMMETRY[profile]);
            let half_range = (TRIANGLE_UPPER_VOLTS[profile] - TRIANGLE_LOWER_VOLTS[profile]) * 0.5;
            let midpoint = (TRIANGLE_UPPER_VOLTS[profile] + TRIANGLE_LOWER_VOLTS[profile]) * 0.5;
            let raw_source_volts = centered * half_range + midpoint;
            mixer_negative_source_volts += raw_source_volts;
            poly_mod_source_volts += raw_source_volts - TRIANGLE_POLY_MOD_REFERENCE_VOLTS;
            mixer_negative_source_conductance += SAW_TRIANGLE_MIXER_CONDUCTANCE;
            poly_mod_source_conductance += SAW_TRIANGLE_MIXER_CONDUCTANCE;
        }
        if waves.pulse {
            let centered = band_limited_pulse(phase, increment, pulse_width);
            let half_range = (PULSE_UPPER_VOLTS - PULSE_LOWER_VOLTS) * 0.5;
            let midpoint = (PULSE_UPPER_VOLTS + PULSE_LOWER_VOLTS) * 0.5;
            let equivalent_source_volts =
                (centered * half_range + midpoint) * PULSE_MIXER_CONDUCTANCE;
            mixer_negative_source_volts += equivalent_source_volts;
            poly_mod_source_volts += equivalent_source_volts;
            mixer_negative_source_conductance += PULSE_MIXER_CONDUCTANCE;
            poly_mod_source_conductance += PULSE_MIXER_CONDUCTANCE;
        }

        let sync_events = pulse_edges(phase, increment, pulse_width);
        let wrapped = self.advance_with_sync(increment, external_sync);

        OscillatorSample {
            mixer_positive_source_volts,
            mixer_positive_source_conductance,
            mixer_negative_source_volts,
            mixer_negative_source_conductance,
            poly_mod_source_volts,
            poly_mod_source_conductance,
            wrapped,
            sync_events,
        }
    }

    fn advance_with_sync(&mut self, increment: f32, sync_events: [HardSyncEvent; 2]) -> bool {
        let mut elapsed = 0.0;
        let mut wrapped = false;
        for event in sync_events {
            if event.pulse == HardSyncPulse::None {
                continue;
            }
            let offset = if event.offset.is_finite() {
                event.offset.clamp(elapsed, 1.0)
            } else {
                elapsed
            };
            wrapped |= self.advance_phase(increment * (offset - elapsed));
            self.hard_sync_pulse(event.pulse);
            elapsed = offset;
        }
        wrapped | self.advance_phase(increment * (1.0 - elapsed))
    }

    fn advance_phase(&mut self, increment: f32) -> bool {
        let advanced = self.phase + increment;
        if advanced >= 1.0 {
            self.phase = advanced - 1.0;
            true
        } else {
            self.phase = advanced;
            false
        }
    }
}

fn triangle_loaded_frequency(frequency: f32, profile: usize, triangle_selected: bool) -> f32 {
    if !triangle_selected {
        return frequency;
    }

    frequency * triangle_load_frequency_ratio(profile)
}

pub(crate) fn triangle_load_frequency_ratio(profile: usize) -> f32 {
    // The triangle buffer also drives the internal comparator, so its finite
    // output impedance lets an external load pull oscillator frequency. The
    // CEM3340 sheet gives the first-order reduction directly as Rout/Rload.
    let pull = TRIANGLE_OUTPUT_IMPEDANCE_OHMS[profile % OUTPUT_PROFILE_COUNT]
        / TRIANGLE_MIXER_LOAD_RESISTANCE_OHMS;
    1.0 - pull
}

fn pulse_edges(phase: f32, increment: f32, pulse_width: f32) -> [HardSyncEvent; 2] {
    if pulse_width <= 0.0 || pulse_width >= 1.0 {
        return [HardSyncEvent::default(); 2];
    }
    let advanced = phase + increment;
    let mut events = [HardSyncEvent::default(); 2];
    let mut count = 0;
    let mut push = |pulse, distance: f32| {
        if count < events.len() && increment > 0.0 {
            events[count] = HardSyncEvent {
                pulse,
                offset: (distance / increment).clamp(0.0, 1.0),
            };
            count += 1;
        }
    };

    if phase < pulse_width && advanced >= pulse_width {
        push(HardSyncPulse::Negative, pulse_width - phase);
    }
    if advanced >= 1.0 {
        push(HardSyncPulse::Positive, 1.0 - phase);
        let wrapped_phase = advanced - 1.0;
        if wrapped_phase >= pulse_width {
            push(HardSyncPulse::Negative, 1.0 - phase + pulse_width);
        }
    }

    events
}

fn band_limited_saw(phase: f32, increment: f32) -> f32 {
    let naive = phase * 2.0 - 1.0;
    naive - poly_blep(phase, blep_width(increment))
}

fn band_limited_pulse(phase: f32, increment: f32, pulse_width: f32) -> f32 {
    if pulse_width <= 0.0 {
        return -1.0;
    }
    if pulse_width >= 1.0 {
        return 1.0;
    }
    let naive = if phase < pulse_width { 1.0 } else { -1.0 };
    let falling_phase = if phase >= pulse_width {
        phase - pulse_width
    } else {
        phase + (1.0 - pulse_width)
    };
    let correction_width = blep_width(increment);
    naive + poly_blep(phase, correction_width) - poly_blep(falling_phase, correction_width)
}

fn blep_width(increment: f32) -> f32 {
    (increment * POLY_BLEP_WIDTH).min(0.5)
}

fn triangle(phase: f32, symmetry: f32) -> f32 {
    let symmetry = symmetry.clamp(0.01, 0.99);
    if phase < symmetry {
        -1.0 + 2.0 * phase / symmetry
    } else {
        1.0 - 2.0 * (phase - symmetry) / (1.0 - symmetry)
    }
}

fn poly_blep(phase: f32, increment: f32) -> f32 {
    if increment <= 0.0 {
        return 0.0;
    }
    if phase < increment {
        let x = phase / increment;
        return x + x - x * x - 1.0;
    }
    if phase > 1.0 - increment {
        let x = (phase - 1.0) / increment;
        return x * x + x + x + 1.0;
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAW: WaveSelection = WaveSelection {
        saw: true,
        triangle: false,
        pulse: false,
    };
    const PULSE: WaveSelection = WaveSelection {
        saw: false,
        triangle: false,
        pulse: true,
    };

    #[test]
    fn phase_wrap_is_reported_and_bounded() {
        let mut oscillator = Vco::with_phase(0.99);
        let sample = oscillator.next(1_000.0, 48_000.0, 0.5, SAW);
        assert!(sample.wrapped);
        assert!((0.0..1.0).contains(&oscillator.phase()));
    }

    #[test]
    fn pulse_width_changes_duty_cycle() {
        let mut narrow = Vco::default();
        let mut wide = Vco::default();
        let mut narrow_positive = 0;
        let mut wide_positive = 0;
        for _ in 0..1_000 {
            narrow_positive += (narrow
                .next(100.0, 10_000.0, 0.25, PULSE)
                .mixer_negative_source_volts
                > 0.0) as usize;
            wide_positive += (wide
                .next(100.0, 10_000.0, 0.75, PULSE)
                .mixer_negative_source_volts
                > 0.0) as usize;
        }
        assert!(narrow_positive < wide_positive);
    }

    #[test]
    fn pulse_dc_endpoints_are_stable_and_emit_no_sync_edges() {
        for (width, expected) in [
            (0.0, PULSE_LOWER_VOLTS * PULSE_MIXER_CONDUCTANCE),
            (1.0, PULSE_UPPER_VOLTS * PULSE_MIXER_CONDUCTANCE),
        ] {
            let mut oscillator = Vco::default();
            for _ in 0..2_000 {
                let sample = oscillator.next(440.0, 48_000.0, width, PULSE);
                assert!((sample.mixer_negative_source_volts - expected).abs() < 1.0e-6);
                assert_eq!(sample.sync_events, [HardSyncEvent::default(); 2]);
            }
        }
    }

    #[test]
    fn one_and_ninety_nine_percent_remain_complementary_pulses() {
        let mut narrow = Vco::default();
        let mut wide = Vco::default();
        let mut narrow_sum = 0.0;
        let mut wide_sum = 0.0;
        let mut narrow_edges = 0;
        let mut wide_edges = 0;
        for _ in 0..10_000 {
            let narrow_sample = narrow.next(100.0, 10_000.0, 0.01, PULSE);
            let wide_sample = wide.next(100.0, 10_000.0, 0.99, PULSE);
            narrow_sum += narrow_sample.mixer_negative_source_volts;
            wide_sum += wide_sample.mixer_negative_source_volts;
            narrow_edges += narrow_sample
                .sync_events
                .iter()
                .filter(|event| event.pulse != HardSyncPulse::None)
                .count();
            wide_edges += wide_sample
                .sync_events
                .iter()
                .filter(|event| event.pulse != HardSyncPulse::None)
                .count();
        }
        let narrow_mean = narrow_sum / 10_000.0;
        let wide_mean = wide_sum / 10_000.0;
        let midpoint = (PULSE_UPPER_VOLTS + PULSE_LOWER_VOLTS) * 0.5 * PULSE_MIXER_CONDUCTANCE;
        let half_range = (PULSE_UPPER_VOLTS - PULSE_LOWER_VOLTS) * 0.5 * PULSE_MIXER_CONDUCTANCE;
        assert!((narrow_mean + wide_mean - 2.0 * midpoint).abs() < 2.0e-3);
        assert!(((narrow_mean - midpoint) / half_range + 0.98).abs() < 0.01);
        assert!(((wide_mean - midpoint) / half_range - 0.98).abs() < 0.01);
        assert!((narrow_edges as isize - 200).abs() <= 2);
        assert!((wide_edges as isize - 200).abs() <= 2);
    }

    #[test]
    fn bipolar_hard_sync_reflects_only_the_matching_triangle_branch() {
        let symmetry = TRIANGLE_SYMMETRY[0];
        let mut rising = Vco::with_phase(0.20);
        let before = triangle(rising.phase(), symmetry);
        assert!(rising.hard_sync_pulse(HardSyncPulse::Positive));
        assert!(rising.phase() > symmetry);
        assert!((triangle(rising.phase(), symmetry) - before).abs() < 1.0e-6);
        let reflected = rising.phase();
        assert!(!rising.hard_sync_pulse(HardSyncPulse::Positive));
        assert_eq!(rising.phase(), reflected);

        let mut falling = Vco::with_phase(0.80);
        let before = triangle(falling.phase(), symmetry);
        assert!(falling.hard_sync_pulse(HardSyncPulse::Negative));
        assert!(falling.phase() < symmetry);
        assert!((triangle(falling.phase(), symmetry) - before).abs() < 1.0e-6);
        let reflected = falling.phase();
        assert!(!falling.hard_sync_pulse(HardSyncPulse::Negative));
        assert_eq!(falling.phase(), reflected);
    }

    #[test]
    fn pulse_output_reports_both_capacitively_coupled_sync_polarities() {
        let mut oscillator = Vco::with_phase(0.45);
        let falling = oscillator.next(1_000.0, 10_000.0, 0.50, SAW);
        assert_eq!(falling.sync_events[0].pulse, HardSyncPulse::Negative);
        assert!((falling.sync_events[0].offset - 0.5).abs() < 1.0e-6);
        assert_eq!(falling.sync_events[1], HardSyncEvent::default());

        let mut oscillator = Vco::with_phase(0.95);
        let rising = oscillator.next(1_000.0, 10_000.0, 0.50, SAW);
        assert_eq!(rising.sync_events[0].pulse, HardSyncPulse::Positive);
        assert!((rising.sync_events[0].offset - 0.5).abs() < 1.0e-6);
        assert_eq!(rising.sync_events[1], HardSyncEvent::default());
    }

    #[test]
    fn sync_edges_exist_even_when_pulse_is_not_selected_for_audio() {
        let mut oscillator = Vco::with_phase(0.45);
        let sample = oscillator.next(1_000.0, 10_000.0, 0.50, WaveSelection::default());
        assert_eq!(sample.mixer_positive_source_volts, 0.0);
        assert_eq!(sample.mixer_negative_source_volts, 0.0);
        assert_eq!(sample.poly_mod_source_volts, 0.0);
        assert_eq!(sample.sync_events[0].pulse, HardSyncPulse::Negative);
    }

    #[test]
    fn two_edges_inside_one_sample_are_ordered_and_fractional() {
        let mut oscillator = Vco::with_phase(0.95);
        let sample = oscillator.next(4_000.0, 10_000.0, 0.10, WaveSelection::default());
        assert_eq!(sample.sync_events[0].pulse, HardSyncPulse::Positive);
        assert!((sample.sync_events[0].offset - 0.125).abs() < 1.0e-6);
        assert_eq!(sample.sync_events[1].pulse, HardSyncPulse::Negative);
        assert!((sample.sync_events[1].offset - 0.375).abs() < 1.0e-6);
    }

    #[test]
    fn external_sync_is_applied_at_its_sub_sample_position() {
        let mut oscillator = Vco::with_phase_and_profile(0.20, 0);
        let sample = oscillator.next_with_sync(
            1_000.0,
            10_000.0,
            0.5,
            SAW,
            [
                HardSyncEvent {
                    pulse: HardSyncPulse::Positive,
                    offset: 0.25,
                },
                HardSyncEvent::default(),
            ],
        );

        let phase_at_edge = 0.20 + 0.10 * 0.25;
        let symmetry = TRIANGLE_SYMMETRY[0];
        let reflected = 1.0 - phase_at_edge * (1.0 - symmetry) / symmetry;
        let expected_phase = reflected + 0.10 * 0.75;
        assert!((oscillator.phase() - expected_phase).abs() < 1.0e-6);

        let expected_audio =
            band_limited_saw(0.20, 0.10) * (SAW_UPPER_VOLTS[0] - SAW_LOWER_VOLTS[0]) * 0.5
                + (SAW_UPPER_VOLTS[0] + SAW_LOWER_VOLTS[0]) * 0.5;
        assert!((sample.mixer_positive_source_volts - expected_audio).abs() < 1.0e-6);
    }

    #[test]
    fn invalid_or_reversed_sync_offsets_cannot_break_phase_bounds() {
        let mut oscillator = Vco::with_phase(0.25);
        for _ in 0..1_000 {
            let sample = oscillator.next_with_sync(
                9_000.0,
                48_000.0,
                0.5,
                SAW,
                [
                    HardSyncEvent {
                        pulse: HardSyncPulse::Positive,
                        offset: f32::NAN,
                    },
                    HardSyncEvent {
                        pulse: HardSyncPulse::Negative,
                        offset: -10.0,
                    },
                ],
            );
            assert!(sample.mixer_differential_source_volts().is_finite());
            assert!((0.0..1.0).contains(&oscillator.phase()));
        }
    }

    #[test]
    fn all_wave_combinations_stay_finite() {
        let mut oscillator = Vco::default();
        let waves = WaveSelection {
            saw: true,
            triangle: true,
            pulse: true,
        };
        for _ in 0..10_000 {
            let sample = oscillator.next(12_000.0, 48_000.0, 0.37, waves);
            assert!(sample.mixer_positive_source_volts.is_finite());
            assert!(sample.mixer_negative_source_volts.is_finite());
            assert!(sample.poly_mod_source_volts.is_finite());
            assert!(sample.mixer_positive_source_volts.abs() <= 11.0);
            assert!(sample.mixer_negative_source_volts.abs() <= 16.0);
            assert!(sample.poly_mod_source_volts.abs() <= 25.0);
        }
    }

    #[test]
    fn all_output_profiles_stay_inside_data_sheet_limits() {
        for profile in 0..OUTPUT_PROFILE_COUNT {
            assert!((9.4..=10.6).contains(&SAW_UPPER_VOLTS[profile]));
            assert!((-0.025..=0.025).contains(&SAW_LOWER_VOLTS[profile]));
            assert!((4.85..=5.15).contains(&TRIANGLE_UPPER_VOLTS[profile]));
            assert!((-0.015..=0.015).contains(&TRIANGLE_LOWER_VOLTS[profile]));
            assert!((0.45..=0.55).contains(&TRIANGLE_SYMMETRY[profile]));
            assert!((65.0..=150.0).contains(&TRIANGLE_OUTPUT_IMPEDANCE_OHMS[profile]));
        }
    }

    #[test]
    fn triangle_load_reproduces_the_data_sheet_frequency_pull() {
        let frequency = 1_000.0;
        let profile = 7;
        let loaded = triangle_loaded_frequency(frequency, profile, true);
        let fractional_pull = 1.0 - loaded / frequency;

        assert!((fractional_pull - 150.0 / 150_000.0).abs() < 1.0e-7);
        let cents = 1_200.0 * libm::log2f(loaded / frequency);
        assert!((-1.74..-1.72).contains(&cents));
    }

    #[test]
    fn only_a_selected_triangle_loads_the_oscillator_core() {
        let mut saw = Vco::with_phase_and_profile(0.0, 7);
        let mut pulse = Vco::with_phase_and_profile(0.0, 7);
        let mut triangle = Vco::with_phase_and_profile(0.0, 7);
        let triangle_wave = WaveSelection {
            saw: false,
            triangle: true,
            pulse: false,
        };

        saw.next(1_000.0, 48_000.0, 0.5, SAW);
        pulse.next(1_000.0, 48_000.0, 0.5, PULSE);
        triangle.next(1_000.0, 48_000.0, 0.5, triangle_wave);

        assert_eq!(saw.phase(), pulse.phase());
        assert!(triangle.phase() < saw.phase());
        let expected = triangle_loaded_frequency(1_000.0, 7, true) / 48_000.0;
        assert!((triangle.phase() - expected).abs() < 1.0e-7);
    }

    #[test]
    fn triangle_audio_is_raw_while_poly_mod_is_level_shifted() {
        let profile = 4;
        let mut low = Vco::with_phase_and_profile(0.0, profile);
        let mut high = Vco::with_phase_and_profile(TRIANGLE_SYMMETRY[profile], profile);
        let waves = WaveSelection {
            saw: false,
            triangle: true,
            pulse: false,
        };
        let low = low.next(0.0, 48_000.0, 0.5, waves);
        let high = high.next(0.0, 48_000.0, 0.5, waves);
        let triangle_low = low.mixer_negative_source_volts;
        let triangle_high = high.mixer_negative_source_volts;
        let saw_peak_to_peak = SAW_UPPER_VOLTS[profile] - SAW_LOWER_VOLTS[profile];
        let triangle_peak_to_peak = triangle_high - triangle_low;
        assert!((triangle_low - TRIANGLE_LOWER_VOLTS[profile]).abs() < 1.0e-6);
        assert!((triangle_high - TRIANGLE_UPPER_VOLTS[profile]).abs() < 1.0e-6);
        assert!(
            (low.poly_mod_source_volts
                - (TRIANGLE_LOWER_VOLTS[profile] - TRIANGLE_POLY_MOD_REFERENCE_VOLTS))
                .abs()
                < 1.0e-6
        );
        assert!(
            (high.poly_mod_source_volts
                - (TRIANGLE_UPPER_VOLTS[profile] - TRIANGLE_POLY_MOD_REFERENCE_VOLTS))
                .abs()
                < 1.0e-6
        );
        assert!((triangle_peak_to_peak / saw_peak_to_peak - 0.508).abs() < 0.01);
    }

    #[test]
    fn poly_mod_preserves_documented_waveform_polarities() {
        let mut saw = Vco::with_phase_and_profile(0.25, 4);
        let mut triangle = Vco::with_phase_and_profile(0.25, 4);
        let mut pulse = Vco::with_phase_and_profile(0.25, 4);
        let saw_sample = saw.next(0.0, 48_000.0, 0.5, SAW);
        let triangle_sample = triangle.next(
            0.0,
            48_000.0,
            0.5,
            WaveSelection {
                saw: false,
                triangle: true,
                pulse: false,
            },
        );
        let pulse_sample = pulse.next(0.0, 48_000.0, 0.5, PULSE);
        assert!(saw_sample.poly_mod_source_volts >= 0.0);
        assert!(pulse_sample.poly_mod_source_volts >= -0.5);
        assert!(
            (triangle_sample.mixer_negative_source_volts
                - triangle_sample.poly_mod_source_volts
                - TRIANGLE_POLY_MOD_REFERENCE_VOLTS)
                .abs()
                < 1.0e-6
        );
    }

    #[test]
    fn voice_board_resistors_make_pulse_slightly_hotter_than_saw() {
        let saw_peak_to_peak = SAW_UPPER_VOLTS[4] - SAW_LOWER_VOLTS[4];
        let pulse_peak_to_peak = (PULSE_UPPER_VOLTS - PULSE_LOWER_VOLTS) * PULSE_MIXER_CONDUCTANCE;
        assert!(pulse_peak_to_peak > saw_peak_to_peak);
        assert!(pulse_peak_to_peak / saw_peak_to_peak < 1.11);
    }

    #[test]
    fn selected_waveforms_report_their_populated_mixer_conductances() {
        let mut oscillator = Vco::default();
        let saw = oscillator.next(0.0, 48_000.0, 0.5, SAW);
        let pulse = oscillator.next(0.0, 48_000.0, 0.5, PULSE);
        let all = oscillator.next(
            0.0,
            48_000.0,
            0.5,
            WaveSelection {
                saw: true,
                triangle: true,
                pulse: true,
            },
        );
        let disconnected = oscillator.next(0.0, 48_000.0, 0.5, WaveSelection::default());

        assert_eq!(saw.mixer_positive_source_conductance, 1.0);
        assert_eq!(saw.mixer_negative_source_conductance, 0.0);
        assert_eq!(pulse.mixer_positive_source_conductance, 0.0);
        assert_eq!(pulse.mixer_negative_source_conductance, 0.75);
        assert_eq!(all.mixer_positive_source_conductance, 1.0);
        assert_eq!(all.mixer_negative_source_conductance, 1.75);
        assert_eq!(all.poly_mod_source_conductance, 2.75);
        assert_eq!(disconnected.mixer_positive_source_conductance, 0.0);
        assert_eq!(disconnected.mixer_negative_source_conductance, 0.0);
        assert_eq!(disconnected.poly_mod_source_conductance, 0.0);
    }

    #[test]
    fn widened_polyblep_saw_is_continuous_finite_and_bounded() {
        for increment in [0.001, 0.01, 0.10, 0.49] {
            let edge_epsilon = blep_width(increment) * 1.0e-5;
            let below_wrap = band_limited_saw(1.0 - edge_epsilon, increment);
            let at_wrap = band_limited_saw(0.0, increment);
            assert!((below_wrap - at_wrap).abs() < 1.0e-4);
            for index in 0..10_000 {
                let sample = band_limited_saw(index as f32 / 10_000.0, increment);
                assert!(sample.is_finite());
                assert!(sample.abs() <= 1.0 + f32::EPSILON);
            }
        }
    }
}
