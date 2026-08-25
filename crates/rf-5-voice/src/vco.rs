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
    /// AC representation delivered to the oscillator mixer.
    pub audio: f32,
    /// Board-level polarity delivered to oscillator-B Poly Mod.
    pub modulation: f32,
    pub wrapped: bool,
    /// Bipolar pulses and their fractional positions inside this internal
    /// sample, produced by capacitively coupling the pulse output into another
    /// CEM3340 hard-sync input.
    pub sync_events: [HardSyncEvent; 2],
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

// The voice board uses 150 kohm inputs for saw/triangle and 200 kohm for
// pulse. With +15/-5 V supplies, the data-sheet pulse formulas and the 4016
// negative clamp give approximately -0.6 V and +14.1 V after selection.
// Values below are expressed relative to a nominal 5 V saw half-excursion.
const PULSE_AC_GAIN: f32 = 1.1025;
const PULSE_MODULATION_OFFSET: f32 = 1.0125;
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
        let increment = (frequency.max(0.0) / sample_rate.max(1.0)).clamp(0.0, 0.49);
        let pulse_width = pulse_width.clamp(0.0, 1.0);
        let phase = self.phase;
        let profile = self.profile_index;
        let mut audio = 0.0;
        let mut modulation = 0.0;

        if waves.saw {
            let centered = band_limited_saw(phase, increment);
            let half_range = (SAW_UPPER_VOLTS[profile] - SAW_LOWER_VOLTS[profile]) / 10.0;
            let midpoint = (SAW_UPPER_VOLTS[profile] + SAW_LOWER_VOLTS[profile]) / 10.0;
            audio += centered * half_range;
            modulation += centered * half_range + midpoint;
        }
        if waves.triangle {
            let centered = triangle(phase, TRIANGLE_SYMMETRY[profile]);
            let half_range = (TRIANGLE_UPPER_VOLTS[profile] - TRIANGLE_LOWER_VOLTS[profile]) / 10.0;
            let shifted = centered * half_range;
            audio += shifted;
            modulation += shifted;
        }
        if waves.pulse {
            let centered = band_limited_pulse(phase, increment, pulse_width) * PULSE_AC_GAIN;
            audio += centered;
            modulation += centered + PULSE_MODULATION_OFFSET;
        }

        let sync_events = pulse_edges(phase, increment, pulse_width);
        let wrapped = self.advance_with_sync(increment, external_sync);

        OscillatorSample {
            audio,
            modulation,
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
            narrow_positive += (narrow.next(100.0, 10_000.0, 0.25, PULSE).audio > 0.0) as usize;
            wide_positive += (wide.next(100.0, 10_000.0, 0.75, PULSE).audio > 0.0) as usize;
        }
        assert!(narrow_positive < wide_positive);
    }

    #[test]
    fn pulse_dc_endpoints_are_stable_and_emit_no_sync_edges() {
        for (width, expected) in [(0.0, -PULSE_AC_GAIN), (1.0, PULSE_AC_GAIN)] {
            let mut oscillator = Vco::default();
            for _ in 0..2_000 {
                let sample = oscillator.next(440.0, 48_000.0, width, PULSE);
                assert!((sample.audio - expected).abs() < 1.0e-6);
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
            narrow_sum += narrow_sample.audio;
            wide_sum += wide_sample.audio;
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
        assert!((narrow_mean + wide_mean).abs() < 1.0e-4);
        assert!((narrow_mean / PULSE_AC_GAIN + 0.98).abs() < 0.01);
        assert!((wide_mean / PULSE_AC_GAIN - 0.98).abs() < 0.01);
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
        assert_eq!(sample.audio, 0.0);
        assert_eq!(sample.modulation, 0.0);
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
            band_limited_saw(0.20, 0.10) * (SAW_UPPER_VOLTS[0] - SAW_LOWER_VOLTS[0]) / 10.0;
        assert!((sample.audio - expected_audio).abs() < 1.0e-6);
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
            assert!(sample.audio.is_finite());
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
            assert!(sample.audio.is_finite());
            assert!(sample.modulation.is_finite());
            assert!(sample.audio.abs() <= 3.1);
            assert!(sample.modulation.abs() <= 4.8);
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
        }
    }

    #[test]
    fn triangle_is_level_shifted_bipolar_and_has_half_the_saw_excursion() {
        let profile = 4;
        let triangle_low = triangle(0.0, TRIANGLE_SYMMETRY[profile])
            * (TRIANGLE_UPPER_VOLTS[profile] - TRIANGLE_LOWER_VOLTS[profile])
            / 10.0;
        let triangle_high = triangle(TRIANGLE_SYMMETRY[profile], TRIANGLE_SYMMETRY[profile])
            * (TRIANGLE_UPPER_VOLTS[profile] - TRIANGLE_LOWER_VOLTS[profile])
            / 10.0;
        let saw_peak_to_peak = 2.0 * (SAW_UPPER_VOLTS[profile] - SAW_LOWER_VOLTS[profile]) / 10.0;
        let triangle_peak_to_peak = triangle_high - triangle_low;
        assert!((triangle_high + triangle_low).abs() < 1.0e-6);
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
        assert!(saw_sample.modulation >= 0.0);
        assert!(pulse_sample.modulation >= -0.1);
        assert_eq!(triangle_sample.audio, triangle_sample.modulation);
    }

    #[test]
    fn voice_board_resistors_make_pulse_slightly_hotter_than_saw() {
        let saw_peak = (SAW_UPPER_VOLTS[4] - SAW_LOWER_VOLTS[4]) / 10.0;
        assert!(PULSE_AC_GAIN > saw_peak);
        assert!(PULSE_AC_GAIN / saw_peak < 1.11);
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
