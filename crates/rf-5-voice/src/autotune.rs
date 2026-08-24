//! Ten-VCO automatic tuning and runtime bias interpolation.
//!
//! The topology follows the Revision 3 service description: each audio VCO is
//! selected by the tune multiplexer, measured by the 2.5 MHz interval timer,
//! searched with the 14 writable DAC bits at C3-C9, and represented by ten
//! octave bias points. C0-C2 are extrapolated from the measured curve.

use rf_5_contract::hardware::{
    CONTROL_DAC_WRITABLE_BITS, TUNE_CPU_CLOCK_HZ, TUNE_DIRECT_MEASUREMENT_FIRST_OCTAVE,
    TUNE_DIRECT_MEASUREMENT_LAST_OCTAVE, TUNE_OCTAVE_BIAS_COUNT, TUNE_OSCILLATOR_COUNT,
};

const DAC_MAX_CODE: i32 = (1_i32 << CONTROL_DAC_WRITABLE_BITS) - 1;
// V8.1 adds 0x0100 to its sparse internal DAC word for every semitone. The
// tune writer rotates the low byte and never sets internal bit zero, so that
// interval is exactly 128 of the fourteen CPU-writable DAC positions.
const DAC_CODES_PER_SEMITONE: f32 = 128.0;
const C0_FREQUENCY_HZ: f32 = 16.351_599;

// Deterministic component-tolerance fixtures. They exercise the documented
// calibration mechanism without claiming measurements from a particular unit.
const INITIAL_OFFSET_CODES: [f32; TUNE_OSCILLATOR_COUNT] =
    [-11.0, 7.0, -5.0, 13.0, -8.0, 4.0, -14.0, 9.0, -3.0, 12.0];
const SCALE_ERROR: [f32; TUNE_OSCILLATOR_COUNT] = [
    0.000_72, -0.000_55, 0.000_31, -0.000_81, 0.000_46, -0.000_28, 0.000_88, -0.000_63, 0.000_19,
    -0.000_39,
];
const CURVATURE_SEMITONES: [f32; TUNE_OSCILLATOR_COUNT] = [
    0.034, -0.027, 0.019, -0.041, 0.025, -0.016, 0.044, -0.031, 0.013, -0.022,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Oscillator {
    A,
    B,
}

impl Oscillator {
    const fn offset(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct BiasTable {
    codes: [i16; TUNE_OCTAVE_BIAS_COUNT],
}

impl BiasTable {
    const ZERO: Self = Self {
        codes: [0; TUNE_OCTAVE_BIAS_COUNT],
    };
}

/// Calibrated scratchpad-equivalent state. It is machine state, not patch
/// state, and is therefore rebuilt on engine preparation rather than saved.
#[derive(Clone, Copy, Debug)]
pub struct AutoTune {
    tables: [BiasTable; TUNE_OSCILLATOR_COUNT],
}

impl Default for AutoTune {
    fn default() -> Self {
        Self::calibrated()
    }
}

impl AutoTune {
    pub fn calibrated() -> Self {
        let mut result = Self {
            tables: [BiasTable::ZERO; TUNE_OSCILLATOR_COUNT],
        };
        let mut channel = 0;
        while channel < TUNE_OSCILLATOR_COUNT {
            let mut octave = TUNE_DIRECT_MEASUREMENT_FIRST_OCTAVE;
            while octave <= TUNE_DIRECT_MEASUREMENT_LAST_OCTAVE {
                let tuned = successive_approximation_code(channel, octave);
                result.tables[channel].codes[octave] = (tuned - ideal_code_for_octave(octave))
                    .clamp(i16::MIN as i32, i16::MAX as i32)
                    as i16;
                octave += 1;
            }
            extrapolate_lower_octaves(&mut result.tables[channel].codes);
            channel += 1;
        }
        result
    }

    /// Residual pitch error after the CPU-style bias correction, in semitones.
    pub fn residual_semitones(
        self,
        voice_index: usize,
        oscillator: Oscillator,
        ideal_semitones: f32,
    ) -> f32 {
        if !ideal_semitones.is_finite() {
            return 0.0;
        }
        let channel = channel_index(voice_index, oscillator);
        let ideal_code = ideal_semitones * DAC_CODES_PER_SEMITONE;
        let bias = interpolated_bias(self.tables[channel].codes, ideal_semitones);
        let applied_code = libm::roundf(ideal_code + bias);
        physical_semitones(channel, applied_code) - ideal_semitones
    }

    #[cfg(test)]
    fn bias_code(self, channel: usize, octave: usize) -> i16 {
        self.tables[channel].codes[octave]
    }
}

const fn channel_index(voice_index: usize, oscillator: Oscillator) -> usize {
    (voice_index % (TUNE_OSCILLATOR_COUNT / 2)) * 2 + oscillator.offset()
}

fn ideal_code_for_octave(octave: usize) -> i32 {
    octave as i32 * 12 * DAC_CODES_PER_SEMITONE as i32
}

fn physical_semitones(channel: usize, dac_code: f32) -> f32 {
    let ideal = dac_code / DAC_CODES_PER_SEMITONE;
    let normalized = dac_code / DAC_MAX_CODE as f32;
    let centered = normalized - 0.5;
    ideal * (1.0 + SCALE_ERROR[channel])
        + INITIAL_OFFSET_CODES[channel] / DAC_CODES_PER_SEMITONE
        + CURVATURE_SEMITONES[channel] * centered * centered
}

fn cycles_to_measure(octave: usize) -> u32 {
    if octave <= 4 { 1 } else { 1 << (octave - 4) }
}

fn measured_cpu_cycles(channel: usize, dac_code: i32, oscillator_cycles: u32) -> u32 {
    let semitones = physical_semitones(channel, dac_code as f32);
    let frequency = C0_FREQUENCY_HZ * libm::exp2f(semitones / 12.0);
    libm::roundf(TUNE_CPU_CLOCK_HZ as f32 * oscillator_cycles as f32 / frequency) as u32
}

fn reference_cpu_cycles(octave: usize, oscillator_cycles: u32) -> u32 {
    let frequency = C0_FREQUENCY_HZ * libm::exp2f(octave as f32);
    libm::roundf(TUNE_CPU_CLOCK_HZ as f32 * oscillator_cycles as f32 / frequency) as u32
}

fn successive_approximation_code(channel: usize, octave: usize) -> i32 {
    let oscillator_cycles = cycles_to_measure(octave);
    let reference = reference_cpu_cycles(octave, oscillator_cycles);
    let mut code = 0_i32;
    let mut bit = CONTROL_DAC_WRITABLE_BITS;
    while bit > 0 {
        bit -= 1;
        let candidate = code | (1_i32 << bit);
        let measured = measured_cpu_cycles(channel, candidate, oscillator_cycles);
        if measured >= reference {
            code = candidate;
        }
    }
    code
}

fn extrapolate_lower_octaves(codes: &mut [i16; TUNE_OCTAVE_BIAS_COUNT]) {
    // Rev 3 V8.1 operating-ROM offsets 0x0101-0x0125 recover the exact
    // arithmetic left implicit by the manual: form the signed 16-bit C4-C3
    // difference once, then subtract that same difference successively to
    // obtain C2, C1 and C0. The admitted ROM is evidence only; these operations
    // are the independent Rust reconstruction and require no firmware at run
    // time.
    let c4_minus_c3 = i32::from(codes[4]) - i32::from(codes[3]);
    for octave in (0..TUNE_DIRECT_MEASUREMENT_FIRST_OCTAVE).rev() {
        let extrapolated = i32::from(codes[octave + 1]) - c4_minus_c3;
        codes[octave] = extrapolated.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
    }
}

fn interpolated_bias(codes: [i16; TUNE_OCTAVE_BIAS_COUNT], semitone: f32) -> f32 {
    // Operating-ROM offsets 0x03ee-0x0483 divide the integer pitch coordinate
    // by twelve, select the surrounding octave words and multiply their bias
    // difference by the remainder. The lookup multiply is deliberately kept
    // discrete: the original does not blend continuously between keys.
    let last_semitone = (TUNE_OCTAVE_BIAS_COUNT - 1) * 12;
    let semitone = libm::floorf(semitone.clamp(0.0, last_semitone as f32)) as usize;
    let lower = semitone / 12;
    let remainder = (semitone % 12) as u8;
    let upper = (lower + 1).min(TUNE_OCTAVE_BIAS_COUNT - 1);
    let delta = i32::from(codes[upper]) - i32::from(codes[lower]);
    (i32::from(codes[lower]) + firmware_bias_fraction(delta, remainder)) as f32
}

fn firmware_bias_fraction(delta: i32, remainder: u8) -> i32 {
    if delta == 0 || remainder == 0 {
        return 0;
    }

    // The ROM halves its sparse sixteen-bit difference first. Because tune
    // words always have bit zero clear, this leaves the dense fourteen-bit
    // code used here. Only the resulting low byte addresses the two multiply
    // tables, matching the Z80 register path rather than silently saturating.
    let magnitude = (delta.unsigned_abs() as usize) & 0xff;
    let coarse = magnitude >> 5;
    let fine = magnitude & 0x1f;
    let interpolated = coarse_product(coarse, remainder) + fine_product(fine, remainder);
    if delta.is_negative() {
        -(interpolated as i32)
    } else {
        interpolated as i32
    }
}

fn rounded_twelfths(value: usize, remainder: u8) -> usize {
    (value * usize::from(remainder) + 6) / 12
}

fn coarse_product(coarse: usize, remainder: u8) -> usize {
    let nearest = rounded_twelfths(coarse << 5, remainder);
    // Four entries in V8.1 choose the lower neighbor instead of the ordinary
    // nearest result. Keeping the quirk as arithmetic avoids carrying either
    // of the original firmware lookup tables in the plug-in.
    if coarse == 2 && remainder % 3 == 2 {
        nearest - 1
    } else {
        nearest
    }
}

fn fine_product(fine: usize, remainder: u8) -> usize {
    let nearest = rounded_twelfths(fine, remainder);
    let downward_quirk = match remainder {
        3 | 9 => matches!(fine, 2 | 10 | 14 | 22 | 26),
        6 => fine < 27 && fine % 2 == 1 && fine % 6 != 3,
        _ => false,
    };
    if downward_quirk {
        nearest - 1
    } else if remainder == 4 && fine == 1 {
        nearest + 1
    } else {
        nearest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cents(value: f32) -> f32 {
        value * 100.0
    }

    #[test]
    fn tune_mux_maps_five_voices_to_ten_distinct_vcos() {
        let mut seen = [false; TUNE_OSCILLATOR_COUNT];
        for voice in 0..5 {
            for oscillator in [Oscillator::A, Oscillator::B] {
                let channel = channel_index(voice, oscillator);
                assert!(!seen[channel]);
                seen[channel] = true;
            }
        }
        assert!(seen.into_iter().all(|mapped| mapped));
    }

    #[test]
    fn high_octaves_measure_more_cycles_to_preserve_counter_resolution() {
        assert_eq!(cycles_to_measure(3), 1);
        assert_eq!(cycles_to_measure(4), 1);
        assert_eq!(cycles_to_measure(5), 2);
        assert_eq!(cycles_to_measure(9), 32);
    }

    #[test]
    fn calibration_populates_all_two_hundred_bias_bytes() {
        let tune = AutoTune::calibrated();
        for channel in 0..TUNE_OSCILLATOR_COUNT {
            assert!((0..TUNE_OCTAVE_BIAS_COUNT).any(|octave| tune.bias_code(channel, octave) != 0));
        }
        assert_eq!(core::mem::size_of_val(&tune.tables), 200);
    }

    #[test]
    fn calibration_bounds_all_ten_vcos_across_the_playing_table() {
        let tune = AutoTune::calibrated();
        let mut worst = 0.0_f32;
        let mut worst_case = (0, Oscillator::A, 0);
        let mut absolute_error_sum = 0.0_f32;
        let mut samples = 0_u32;
        for voice in 0..5 {
            for oscillator in [Oscillator::A, Oscillator::B] {
                for semitone in 0..=108 {
                    let error =
                        cents(tune.residual_semitones(voice, oscillator, semitone as f32)).abs();
                    if error > worst {
                        worst = error;
                        worst_case = (voice, oscillator, semitone);
                    }
                    absolute_error_sum += error;
                    samples += 1;
                }
            }
        }
        let mean = absolute_error_sum / samples as f32;
        assert!(mean < 0.75, "mean calibrated error: {mean} cents");
        assert!(
            worst < 4.0,
            "worst calibrated error: {worst} cents at {worst_case:?}"
        );
    }

    #[test]
    fn direct_octave_search_uses_only_fourteen_bit_dac_codes() {
        for channel in 0..TUNE_OSCILLATOR_COUNT {
            for octave in TUNE_DIRECT_MEASUREMENT_FIRST_OCTAVE..=TUNE_DIRECT_MEASUREMENT_LAST_OCTAVE
            {
                let code = successive_approximation_code(channel, octave);
                assert!((0..=DAC_MAX_CODE).contains(&code));
            }
        }
    }

    #[test]
    fn lower_octaves_repeat_the_operating_rom_c4_minus_c3_slope() {
        let mut codes = [0, 0, 0, 100, 112, -900, 700, -400, 300, -200];
        extrapolate_lower_octaves(&mut codes);
        assert_eq!(codes[..5], [64, 76, 88, 100, 112]);
    }

    #[test]
    fn lower_octave_extrapolation_ignores_later_measured_curvature() {
        let mut first = [0, 0, 0, -40, -33, -20, 2, 31, 69, 116];
        let mut second = [0, 0, 0, -40, -33, 900, -800, 700, -600, 500];
        extrapolate_lower_octaves(&mut first);
        extrapolate_lower_octaves(&mut second);
        assert_eq!(first[..3], second[..3]);
        assert_eq!(first[..5], [-61, -54, -47, -40, -33]);
    }

    #[test]
    fn calibrated_tables_preserve_one_lower_octave_difference() {
        let tune = AutoTune::calibrated();
        for channel in 0..TUNE_OSCILLATOR_COUNT {
            let expected = tune.bias_code(channel, 4) - tune.bias_code(channel, 3);
            for octave in 0..3 {
                assert_eq!(
                    tune.bias_code(channel, octave + 1) - tune.bias_code(channel, octave),
                    expected
                );
            }
        }
    }

    #[test]
    fn firmware_semitone_scale_is_exact_in_writable_dac_codes() {
        assert_eq!(ideal_code_for_octave(1), 12 * 128);
        assert_eq!(ideal_code_for_octave(9), 9 * 12 * 128);
    }

    #[test]
    fn runtime_interpolation_matches_emulated_v81_landmarks() {
        let positive: [i32; 11] =
            core::array::from_fn(|index| firmware_bias_fraction(50, index as u8 + 1));
        assert_eq!(positive, [5, 8, 13, 17, 21, 25, 30, 33, 38, 42, 46]);
        for remainder in 1..12 {
            assert_eq!(
                firmware_bias_fraction(-50, remainder),
                -firmware_bias_fraction(50, remainder)
            );
        }
        assert_eq!(firmware_bias_fraction(6, 1), 1);
        assert_eq!(firmware_bias_fraction(6, 2), 1);
        assert_eq!(firmware_bias_fraction(6, 11), 6);
    }

    #[test]
    fn lookup_rounding_quirks_are_preserved_without_rom_tables() {
        assert_eq!(coarse_product(2, 2), 10);
        assert_eq!(coarse_product(2, 5), 26);
        assert_eq!(fine_product(2, 3), 0);
        assert_eq!(fine_product(6, 3), 2);
        assert_eq!(fine_product(1, 4), 1);
        assert_eq!(fine_product(1, 6), 0);
        assert_eq!(fine_product(3, 6), 2);
    }

    #[test]
    fn interpolation_uses_the_twelve_firmware_key_positions() {
        let mut codes = [0; TUNE_OCTAVE_BIAS_COUNT];
        codes[0] = 100;
        codes[1] = 150;
        assert_eq!(interpolated_bias(codes, 0.0), 100.0);
        assert_eq!(interpolated_bias(codes, 1.99), 105.0);
        assert_eq!(interpolated_bias(codes, 6.0), 125.0);
        assert_eq!(interpolated_bias(codes, 12.0), 150.0);
    }
}
