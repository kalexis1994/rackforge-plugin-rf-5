//! Portable oversampling reconstruction filters.
//!
//! The four-times reference uses a 127-tap Kaiser-windowed sinc. The portable
//! two-times candidate uses a 159-tap half-band FIR. Its ideal half-band zeros
//! are omitted from the convolution, leaving only forty symmetric pairs and
//! the centre tap. Both filters remove nonlinear ultrasonic products before
//! selecting the host-rate sample.

// Each feature profile links only one decimator into the voice path. Unit and
// spectral tests intentionally compile both implementations side by side.
#![cfg_attr(not(test), allow(dead_code))]

const TWO_TIMES_FACTOR: u8 = 2;
const TWO_TIMES_TAP_COUNT: usize = 159;
const TWO_TIMES_CENTRE_INDEX: usize = TWO_TIMES_TAP_COUNT / 2;

// Kaiser-windowed half-band FIR (beta 10), unity DC and linear phase. At a
// 44.1 kHz host rate it is effectively flat at 20 kHz and exceeds 100 dB of
// attenuation by 24.1 kHz. Only the non-zero taps from the first half are
// stored; their symmetric partners are applied by the convolution.
const TWO_TIMES_PAIR_TAPS: [f32; 40] = [
    -1.430_982_5e-6,
    3.962_513e-6,
    -8.257_42e-6,
    1.502_696_6e-5,
    -2.515_76e-5,
    3.973_343e-5,
    -6.005_856_5e-5,
    8.767_896e-5,
    -1.244_035_3e-4,
    1.723_243_4e-4,
    -2.338_359_2e-4,
    3.116_539_2e-4,
    -4.088_337_6e-4,
    5.287_9e-4,
    -6.753_183_5e-4,
    8.526_218_4e-4,
    -1.065_345e-3,
    1.318_619_8e-3,
    -1.618_129_1e-3,
    1.970_197e-3,
    -2.381_916e-3,
    2.861_327_4e-3,
    -3.417_680_6e-3,
    4.061_802e-3,
    -4.806_626_6e-3,
    5.667_972e-3,
    -6.665_678e-3,
    7.825_319e-3,
    -9.180_827e-3,
    1.077_864_7e-2,
    -1.268_452_6e-2,
    1.499_511_6e-2,
    -1.785_887_7e-2,
    2.151_626_5e-2,
    -2.638_376_8e-2,
    3.324_996_4e-2,
    -4.380_762e-2,
    6.246_276_7e-2,
    -1.053_798_6e-1,
    3.180_682_4e-1,
];
const TWO_TIMES_CENTRE_TAP: f32 = 5.000_002_4e-1;

const WIDE_TRANSITION_TAP_COUNT: usize = 31;
const WIDE_TRANSITION_PAIR_TAPS: [f32; 8] = [
    -2.827_624e-5,
    4.745_791_4e-4,
    -2.254_86e-3,
    7.110_714e-3,
    -1.793_624_3e-2,
    4.013_244e-2,
    -9.012_399e-2,
    3.126_308_3e-1,
];
const WIDE_TRANSITION_CENTRE_TAP: f32 = 4.999_896_3e-1;

#[derive(Clone, Copy, Debug)]
pub struct Decimator2x {
    history: [f32; TWO_TIMES_TAP_COUNT],
    write_index: usize,
    phase: u8,
}

impl Default for Decimator2x {
    fn default() -> Self {
        Self {
            history: [0.0; TWO_TIMES_TAP_COUNT],
            write_index: 0,
            phase: 0,
        }
    }
}

impl Decimator2x {
    #[cfg(test)]
    fn reset(&mut self) {
        self.history = [0.0; TWO_TIMES_TAP_COUNT];
        self.write_index = 0;
        self.phase = 0;
    }

    pub fn push(&mut self, sample: f32) -> Option<f32> {
        self.history[self.write_index] = if sample.is_finite() { sample } else { 0.0 };
        self.write_index = (self.write_index + 1) % TWO_TIMES_TAP_COUNT;
        self.phase += 1;
        if self.phase != TWO_TIMES_FACTOR {
            return None;
        }
        self.phase = 0;

        let mut newer_index = if self.write_index == 0 {
            TWO_TIMES_TAP_COUNT - 1
        } else {
            self.write_index - 1
        };
        let mut older_index = self.write_index;
        let mut output = 0.0;
        for coefficient in TWO_TIMES_PAIR_TAPS {
            output += coefficient * (self.history[newer_index] + self.history[older_index]);
            newer_index = if newer_index < 2 {
                newer_index + TWO_TIMES_TAP_COUNT - 2
            } else {
                newer_index - 2
            };
            older_index += 2;
            if older_index >= TWO_TIMES_TAP_COUNT {
                older_index -= TWO_TIMES_TAP_COUNT;
            }
        }
        let centre_index = (self.write_index + TWO_TIMES_TAP_COUNT - 1 - TWO_TIMES_CENTRE_INDEX)
            % TWO_TIMES_TAP_COUNT;
        output += TWO_TIMES_CENTRE_TAP * self.history[centre_index];
        Some(output)
    }
}

/// Efficient 4x-to-2x bridge for the hybrid profile.
///
/// The following 2x-to-1x stage defines the final 20 kHz passband, so this
/// stage can use the broad 0.125-to-0.375 normalized transition. The 31-tap
/// half-band response is flat at the future host Nyquist and rejects content
/// that could alias into the final host band by more than 100 dB.
#[derive(Clone, Copy, Debug)]
pub struct WideTransitionDecimator2x {
    history: [f32; WIDE_TRANSITION_TAP_COUNT],
    write_index: usize,
    phase: u8,
}

impl Default for WideTransitionDecimator2x {
    fn default() -> Self {
        Self {
            history: [0.0; WIDE_TRANSITION_TAP_COUNT],
            write_index: 0,
            phase: 0,
        }
    }
}

impl WideTransitionDecimator2x {
    pub fn push(&mut self, sample: f32) -> Option<f32> {
        self.history[self.write_index] = if sample.is_finite() { sample } else { 0.0 };
        self.write_index = (self.write_index + 1) % WIDE_TRANSITION_TAP_COUNT;
        self.phase += 1;
        if self.phase != TWO_TIMES_FACTOR {
            return None;
        }
        self.phase = 0;

        let mut newer_index = if self.write_index == 0 {
            WIDE_TRANSITION_TAP_COUNT - 1
        } else {
            self.write_index - 1
        };
        let mut older_index = self.write_index;
        let mut output = 0.0;
        for coefficient in WIDE_TRANSITION_PAIR_TAPS {
            output += coefficient * (self.history[newer_index] + self.history[older_index]);
            newer_index = if newer_index < 2 {
                newer_index + WIDE_TRANSITION_TAP_COUNT - 2
            } else {
                newer_index - 2
            };
            older_index += 2;
            if older_index >= WIDE_TRANSITION_TAP_COUNT {
                older_index -= WIDE_TRANSITION_TAP_COUNT;
            }
        }
        let centre_index =
            (self.write_index + WIDE_TRANSITION_TAP_COUNT / 2) % WIDE_TRANSITION_TAP_COUNT;
        output += WIDE_TRANSITION_CENTRE_TAP * self.history[centre_index];
        Some(output)
    }
}

const FACTOR: u8 = 4;
const TAP_COUNT: usize = 127;

// Unity-DC, linear-phase low-pass. The ideal cutoff is host Nyquist (0.125
// cycles per internal sample); the finite transition is -0.36 dB at 90% of
// host Nyquist and exceeds 100 dB attenuation by 120% of host Nyquist.
const TAPS: [f32; TAP_COUNT] = [
    -4.760_669_5e-6,
    -1.144_388_6e-5,
    -1.246_452_4e-5,
    0.0,
    2.512_603_5e-5,
    4.789_446e-5,
    4.455_304_6e-5,
    0.0,
    -7.293_100_5e-5,
    -1.290_762_4e-4,
    -1.128_548_2e-4,
    0.0,
    1.673_582_3e-4,
    2.844_721e-4,
    2.399_414_7e-4,
    0.0,
    -3.345_981_8e-4,
    -5.537_976_4e-4,
    -4.558_444_8e-4,
    0.0,
    6.087_535e-4,
    9.882_95e-4,
    7.990_012e-4,
    0.0,
    -1.032_932_1e-3,
    -1.652_462e-3,
    -1.317_655_9e-3,
    0.0,
    1.661_194e-3,
    2.627_363e-3,
    2.072_698_2e-3,
    0.0,
    -2.562_783_4e-3,
    -4.017_963e-3,
    -3.144_026_3e-3,
    0.0,
    3.831_725_8e-3,
    5.969_815e-3,
    4.645_066_3e-3,
    0.0,
    -5.608_934e-3,
    -8.707_781e-3,
    -6.756_813_3e-3,
    0.0,
    8.135_56e-3,
    1.263_134_2e-2,
    9.813_582e-3,
    0.0,
    -1.189_560_4e-2,
    -1.857_89e-2,
    -1.455_108_6e-2,
    0.0,
    1.807_364_8e-2,
    2.873_186_6e-2,
    2.301_971_6e-2,
    0.0,
    -3.058_481_4e-2,
    -5.113_824_5e-2,
    -4.388_28e-2,
    0.0,
    7.434_182_6e-2,
    1.585_084_5e-1,
    2.248_508_5e-1,
    2.500_009_5e-1,
    2.248_508_5e-1,
    1.585_084_5e-1,
    7.434_182_6e-2,
    0.0,
    -4.388_28e-2,
    -5.113_824_5e-2,
    -3.058_481_4e-2,
    0.0,
    2.301_971_6e-2,
    2.873_186_6e-2,
    1.807_364_8e-2,
    0.0,
    -1.455_108_6e-2,
    -1.857_89e-2,
    -1.189_560_4e-2,
    0.0,
    9.813_582e-3,
    1.263_134_2e-2,
    8.135_56e-3,
    0.0,
    -6.756_813_3e-3,
    -8.707_781e-3,
    -5.608_934e-3,
    0.0,
    4.645_066_3e-3,
    5.969_815e-3,
    3.831_725_8e-3,
    0.0,
    -3.144_026_3e-3,
    -4.017_963e-3,
    -2.562_783_4e-3,
    0.0,
    2.072_698_2e-3,
    2.627_363e-3,
    1.661_194e-3,
    0.0,
    -1.317_655_9e-3,
    -1.652_462e-3,
    -1.032_932_1e-3,
    0.0,
    7.990_012e-4,
    9.882_95e-4,
    6.087_535e-4,
    0.0,
    -4.558_444_8e-4,
    -5.537_976_4e-4,
    -3.345_981_8e-4,
    0.0,
    2.399_414_7e-4,
    2.844_721e-4,
    1.673_582_3e-4,
    0.0,
    -1.128_548_2e-4,
    -1.290_762_4e-4,
    -7.293_100_5e-5,
    0.0,
    4.455_304_6e-5,
    4.789_446e-5,
    2.512_603_5e-5,
    0.0,
    -1.246_452_4e-5,
    -1.144_388_6e-5,
    -4.760_669_5e-6,
];

const NONZERO_PAIR_OFFSETS: [usize; 48] = [
    0, 1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, 16, 17, 18, 20, 21, 22, 24, 25, 26, 28, 29, 30, 32, 33,
    34, 36, 37, 38, 40, 41, 42, 44, 45, 46, 48, 49, 50, 52, 53, 54, 56, 57, 58, 60, 61, 62,
];

#[derive(Clone, Copy, Debug)]
pub struct Decimator4x {
    // Mirroring the ring removes two wrap branches from every symmetric tap
    // pair. The first and second halves always contain the same logical ring.
    history: [f32; TAP_COUNT * 2],
    write_index: usize,
    phase: u8,
}

impl Default for Decimator4x {
    fn default() -> Self {
        Self {
            history: [0.0; TAP_COUNT * 2],
            write_index: 0,
            phase: 0,
        }
    }
}

impl Decimator4x {
    #[cfg(test)]
    fn reset(&mut self) {
        self.history = [0.0; TAP_COUNT * 2];
        self.write_index = 0;
        self.phase = 0;
    }

    pub fn push(&mut self, sample: f32) -> Option<f32> {
        let sample = if sample.is_finite() { sample } else { 0.0 };
        self.history[self.write_index] = sample;
        self.history[self.write_index + TAP_COUNT] = sample;
        self.write_index = (self.write_index + 1) % TAP_COUNT;
        self.phase += 1;
        if self.phase != FACTOR {
            return None;
        }
        self.phase = 0;

        let base = self.write_index;
        let mut output = 0.0;
        for offset in NONZERO_PAIR_OFFSETS {
            output += TAPS[offset]
                * (self.history[base + TAP_COUNT - 1 - offset] + self.history[base + offset]);
        }
        output += TAPS[TAP_COUNT / 2] * self.history[base + TAP_COUNT / 2];
        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f32::consts::PI;

    fn sine_rms_2x(frequency: f32, host_rate: f32) -> f32 {
        let internal_rate = host_rate * f32::from(TWO_TIMES_FACTOR);
        let mut decimator = Decimator2x::default();
        let mut energy = 0.0;
        let mut count = 0;
        let phase_increment = 2.0 * PI * frequency / internal_rate;
        let mut phase = 0.0;
        for index in 0..(internal_rate as usize * 2) {
            let input = libm::sinf(phase);
            phase += phase_increment;
            if phase >= 2.0 * PI {
                phase -= 2.0 * PI;
            }
            if let Some(output) = decimator.push(input)
                && index >= internal_rate as usize
            {
                energy += output * output;
                count += 1;
            }
        }
        libm::sqrtf(energy / count as f32)
    }

    fn sine_rms(frequency: f32, host_rate: f32) -> f32 {
        let internal_rate = host_rate * f32::from(FACTOR);
        let mut decimator = Decimator4x::default();
        let mut energy = 0.0;
        let mut count = 0;
        let phase_increment = 2.0 * PI * frequency / internal_rate;
        let mut phase = 0.0;
        for index in 0..(internal_rate as usize * 2) {
            let input = libm::sinf(phase);
            phase += phase_increment;
            if phase >= 2.0 * PI {
                phase -= 2.0 * PI;
            }
            if let Some(output) = decimator.push(input)
                && index >= internal_rate as usize
            {
                energy += output * output;
                count += 1;
            }
        }
        libm::sqrtf(energy / count as f32)
    }

    #[test]
    fn exactly_one_output_is_produced_for_each_four_inputs() {
        let mut decimator = Decimator4x::default();
        for cycle in 0..32 {
            for phase in 0..4 {
                assert_eq!(decimator.push(cycle as f32).is_some(), phase == 3);
            }
        }
    }

    #[test]
    fn two_times_produces_exactly_one_output_for_each_two_inputs() {
        let mut decimator = Decimator2x::default();
        for cycle in 0..32 {
            assert!(decimator.push(cycle as f32).is_none());
            assert!(decimator.push(cycle as f32).is_some());
        }
    }

    #[test]
    fn two_times_preserves_dc_after_group_delay() {
        let mut decimator = Decimator2x::default();
        let mut output = 0.0;
        for _ in 0..2_000 {
            if let Some(sample) = decimator.push(1.0) {
                output = sample;
            }
        }
        assert!((output - 1.0).abs() < 1.0e-6, "DC gain was {output}");
    }

    #[test]
    fn wide_transition_produces_one_output_for_each_two_inputs() {
        let mut decimator = WideTransitionDecimator2x::default();
        for cycle in 0..32 {
            assert!(decimator.push(cycle as f32).is_none());
            assert!(decimator.push(cycle as f32).is_some());
        }
    }

    #[test]
    fn wide_transition_preserves_dc_after_group_delay() {
        let mut decimator = WideTransitionDecimator2x::default();
        let mut output = 0.0;
        for _ in 0..2_000 {
            if let Some(sample) = decimator.push(1.0) {
                output = sample;
            }
        }
        assert!((output - 1.0).abs() < 1.0e-6, "DC gain was {output}");
    }

    #[test]
    fn two_times_half_band_preserves_audio_and_rejects_alias_sources() {
        let reference_rms = 1.0 / libm::sqrtf(2.0);
        let twenty_khz = sine_rms_2x(20_000.0, 44_100.0);
        let twenty_three_khz = sine_rms_2x(23_000.0, 44_100.0);
        let twenty_four_point_one_khz = sine_rms_2x(24_100.0, 44_100.0);
        assert!(
            twenty_khz / reference_rms > 0.99,
            "20 kHz gain was {}",
            twenty_khz / reference_rms
        );
        assert!(
            twenty_three_khz / reference_rms < 0.05,
            "23 kHz gain was {}",
            twenty_three_khz / reference_rms
        );
        assert!(
            twenty_four_point_one_khz / reference_rms < 1.0e-4,
            "24.1 kHz gain was {}",
            twenty_four_point_one_khz / reference_rms
        );
    }

    #[test]
    fn two_times_reset_removes_history_and_contains_non_finite_input() {
        let mut decimator = Decimator2x::default();
        for _ in 0..256 {
            let _ = decimator.push(1.0);
        }
        decimator.reset();
        assert!(decimator.push(f32::NAN).is_none());
        assert_eq!(decimator.push(0.0), Some(0.0));
    }

    #[test]
    fn unity_dc_gain_is_preserved_after_group_delay() {
        let mut decimator = Decimator4x::default();
        let mut output = 0.0;
        for _ in 0..2_000 {
            if let Some(sample) = decimator.push(1.0) {
                output = sample;
            }
        }
        assert!((output - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn audible_band_survives_while_ultrasonic_alias_source_is_rejected() {
        let reference_rms = 1.0 / libm::sqrtf(2.0);
        let ten_khz = sine_rms(10_000.0, 44_100.0);
        let twenty_khz = sine_rms(20_000.0, 44_100.0);
        let ultrasonic = sine_rms(60_000.0, 48_000.0);
        assert!(
            (ten_khz / reference_rms - 1.0).abs() < 0.001,
            "10 kHz gain was {}",
            ten_khz / reference_rms
        );
        assert!(
            twenty_khz / reference_rms > 0.93,
            "20 kHz gain was {}",
            twenty_khz / reference_rms
        );
        assert!(
            ultrasonic / reference_rms < 1.0e-4,
            "60 kHz gain was {}",
            ultrasonic / reference_rms
        );
    }

    #[test]
    fn reset_removes_filter_history_and_non_finite_input_is_contained() {
        let mut decimator = Decimator4x::default();
        for _ in 0..256 {
            let _ = decimator.push(1.0);
        }
        decimator.reset();
        assert!(decimator.push(f32::NAN).is_none());
        assert!(decimator.push(0.0).is_none());
        assert!(decimator.push(0.0).is_none());
        assert_eq!(decimator.push(0.0), Some(0.0));
    }
}
