//! Four-times oversampling reconstruction filter.
//!
//! Nonlinear VCF/VCA and sync processing happens at four times the host rate.
//! A 127-tap Kaiser-windowed sinc removes content above the host Nyquist limit
//! before selecting one sample in four. The previous box average had only a
//! shallow first-order stopband and allowed ultrasonic products to fold back.

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

#[derive(Clone, Copy, Debug)]
pub struct Decimator4x {
    history: [f32; TAP_COUNT],
    write_index: usize,
    phase: u8,
}

impl Default for Decimator4x {
    fn default() -> Self {
        Self {
            history: [0.0; TAP_COUNT],
            write_index: 0,
            phase: 0,
        }
    }
}

impl Decimator4x {
    #[cfg(test)]
    fn reset(&mut self) {
        self.history = [0.0; TAP_COUNT];
        self.write_index = 0;
        self.phase = 0;
    }

    pub fn push(&mut self, sample: f32) -> Option<f32> {
        self.history[self.write_index] = if sample.is_finite() { sample } else { 0.0 };
        self.write_index = (self.write_index + 1) % TAP_COUNT;
        self.phase += 1;
        if self.phase != FACTOR {
            return None;
        }
        self.phase = 0;

        let mut newer_index = if self.write_index == 0 {
            TAP_COUNT - 1
        } else {
            self.write_index - 1
        };
        let mut older_index = self.write_index;
        let mut output = 0.0;
        for coefficient in &TAPS[..TAP_COUNT / 2] {
            output += coefficient * (self.history[newer_index] + self.history[older_index]);
            newer_index = if newer_index == 0 {
                TAP_COUNT - 1
            } else {
                newer_index - 1
            };
            older_index += 1;
            if older_index == TAP_COUNT {
                older_index = 0;
            }
        }
        output += TAPS[TAP_COUNT / 2] * self.history[newer_index];
        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f32::consts::PI;

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
