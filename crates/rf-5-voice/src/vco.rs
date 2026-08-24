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

    pub fn hard_sync(&mut self) {
        self.phase = 0.0;
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
        let increment = (frequency.max(0.0) / sample_rate.max(1.0)).clamp(0.0, 0.49);
        let pulse_width = pulse_width.clamp(0.02, 0.98);
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

        let advanced = phase + increment;
        let wrapped = advanced >= 1.0;
        self.phase = if wrapped { advanced - 1.0 } else { advanced };

        OscillatorSample {
            audio,
            modulation,
            wrapped,
        }
    }
}

fn band_limited_saw(phase: f32, increment: f32) -> f32 {
    let naive = phase * 2.0 - 1.0;
    naive - poly_blep(phase, increment)
}

fn band_limited_pulse(phase: f32, increment: f32, pulse_width: f32) -> f32 {
    let naive = if phase < pulse_width { 1.0 } else { -1.0 };
    let falling_phase = if phase >= pulse_width {
        phase - pulse_width
    } else {
        phase + (1.0 - pulse_width)
    };
    naive + poly_blep(phase, increment) - poly_blep(falling_phase, increment)
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
    fn hard_sync_resets_phase_without_reallocation() {
        let mut oscillator = Vco::with_phase(0.73);
        oscillator.hard_sync();
        assert_eq!(oscillator.phase(), 0.0);
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
    fn polyblep_saw_is_closer_than_a_naive_edge_to_the_band_limited_reference() {
        let increment = 0.18;
        let harmonic_count = libm::floorf(0.5 / increment) as usize;
        let mut phase = 0.137;
        let mut corrected_error = 0.0;
        let mut naive_error = 0.0;
        for _ in 0..4_096 {
            let mut ideal = 0.0;
            for harmonic in 1..=harmonic_count {
                ideal -= 2.0 / core::f32::consts::PI
                    * libm::sinf(2.0 * core::f32::consts::PI * harmonic as f32 * phase)
                    / harmonic as f32;
            }
            let corrected = band_limited_saw(phase, increment);
            let naive = phase * 2.0 - 1.0;
            corrected_error += (corrected - ideal) * (corrected - ideal);
            naive_error += (naive - ideal) * (naive - ideal);
            phase += increment;
            if phase >= 1.0 {
                phase -= 1.0;
            }
        }
        assert!(corrected_error < naive_error * 0.7);
    }
}
