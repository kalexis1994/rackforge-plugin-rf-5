//! Post-tune thermal motion for the ten independent CEM3340-class VCOs.
//!
//! The data sheet bounds oscillator drift but does not specify a warm-up curve
//! or noise spectrum. This model therefore keeps the published magnitude as a
//! hard boundary and isolates the unknown time evolution in a deterministic,
//! low-rate process.

use crate::autotune::Oscillator;

pub const VCO_COUNT: usize = 10;

const UPDATE_RATE_HZ: f32 = 20.0;
const TYPICAL_DRIFT_PPM: f32 = 50.0;
const MAXIMUM_DRIFT_PPM: f32 = 200.0;
const COMMON_WEIGHT: f32 = 0.28;
const INDIVIDUAL_WEIGHT: f32 = 1.0 - COMMON_WEIGHT;
const COMMON_TIME_CONSTANT_SECONDS: f32 = 180.0;
const VCO_TIME_CONSTANT_SECONDS: [f32; VCO_COUNT] =
    [37.0, 53.0, 71.0, 43.0, 89.0, 61.0, 47.0, 79.0, 59.0, 97.0];
const INITIAL_POSITION: [f32; VCO_COUNT] = [
    -0.44, 0.19, 0.61, -0.12, 0.34, -0.57, 0.08, 0.47, -0.29, 0.73,
];
const INITIAL_TARGET: [f32; VCO_COUNT] = [
    0.38, -0.52, 0.17, 0.69, -0.31, 0.11, -0.73, 0.42, 0.58, -0.16,
];
const INITIAL_TARGET_STEPS: [u32; VCO_COUNT] = [143, 227, 311, 181, 409, 263, 197, 353, 239, 431];
const INITIAL_SEEDS: [u32; VCO_COUNT] = [
    0x91e1_0da5,
    0x6ac6_902d,
    0xc3ef_57a1,
    0x4b79_25d3,
    0xe8d4_163f,
    0x72ab_c981,
    0xad35_84e7,
    0x5f02_d16b,
    0xd197_3ca9,
    0x386e_f425,
];

#[derive(Clone, Copy, Debug)]
pub struct VcoDriftBank {
    position: [f32; VCO_COUNT],
    target: [f32; VCO_COUNT],
    reference: [f32; VCO_COUNT],
    target_steps: [u32; VCO_COUNT],
    seeds: [u32; VCO_COUNT],
    common_position: f32,
    common_target: f32,
    common_reference: f32,
    common_target_steps: u32,
    common_seed: u32,
    sample_phase: f64,
}

impl Default for VcoDriftBank {
    fn default() -> Self {
        Self {
            position: INITIAL_POSITION,
            target: INITIAL_TARGET,
            reference: INITIAL_POSITION,
            target_steps: INITIAL_TARGET_STEPS,
            seeds: INITIAL_SEEDS,
            common_position: 0.21,
            common_target: -0.47,
            common_reference: 0.21,
            common_target_steps: 1_117,
            common_seed: 0x63d8_3595,
            sample_phase: 0.0,
        }
    }
}

impl VcoDriftBank {
    /// Advances the slow process in real time while doing its work at 20 Hz.
    /// The accumulator gives equal elapsed-time behaviour at every audio rate.
    pub fn advance(&mut self, sample_rate: f32) {
        let sample_rate = f64::from(sample_rate.max(1.0));
        self.sample_phase += f64::from(UPDATE_RATE_HZ);
        while self.sample_phase >= sample_rate {
            self.sample_phase -= sample_rate;
            self.step_control_rate();
        }
    }

    /// Captures the present analog condition as the new automatic-tune
    /// reference. This is machine state and never becomes part of a patch.
    pub fn retune(&mut self) {
        self.reference = self.position;
        self.common_reference = self.common_position;
    }

    /// Returns the post-tune error for one physical VCO in parts per million.
    /// `character` expands the data-sheet typical bound toward its stated max.
    pub fn correction_ppm(self, voice_index: usize, oscillator: Oscillator, character: f32) -> f32 {
        let channel = channel_index(voice_index, oscillator);
        let individual = self.position[channel] - self.reference[channel];
        let common = self.common_position - self.common_reference;
        let normalized = (COMMON_WEIGHT * common + INDIVIDUAL_WEIGHT * individual).clamp(-1.0, 1.0);
        let limit =
            TYPICAL_DRIFT_PPM + character.clamp(0.0, 1.0) * (MAXIMUM_DRIFT_PPM - TYPICAL_DRIFT_PPM);
        normalized * limit
    }

    pub fn correction_semitones(
        self,
        voice_index: usize,
        oscillator: Oscillator,
        character: f32,
    ) -> f32 {
        ppm_to_semitones(self.correction_ppm(voice_index, oscillator, character))
    }

    fn step_control_rate(&mut self) {
        if self.common_target_steps == 0 {
            self.common_target = signed_unit(&mut self.common_seed);
            self.common_target_steps = hold_steps(&mut self.common_seed, 45, 121);
        } else {
            self.common_target_steps -= 1;
        }
        self.common_position = smooth_toward(
            self.common_position,
            self.common_target,
            COMMON_TIME_CONSTANT_SECONDS,
        );

        for (channel, time_constant) in VCO_TIME_CONSTANT_SECONDS.iter().enumerate() {
            if self.target_steps[channel] == 0 {
                self.target[channel] = signed_unit(&mut self.seeds[channel]);
                self.target_steps[channel] = hold_steps(&mut self.seeds[channel], 6, 26);
            } else {
                self.target_steps[channel] -= 1;
            }
            self.position[channel] =
                smooth_toward(self.position[channel], self.target[channel], *time_constant)
                    .clamp(-1.0, 1.0);
        }
    }
}

fn channel_index(voice_index: usize, oscillator: Oscillator) -> usize {
    (voice_index % (VCO_COUNT / 2)) * 2
        + match oscillator {
            Oscillator::A => 0,
            Oscillator::B => 1,
        }
}

fn smooth_toward(position: f32, target: f32, time_constant_seconds: f32) -> f32 {
    let alpha = 1.0 - libm::expf(-1.0 / (UPDATE_RATE_HZ * time_constant_seconds));
    position + (target - position) * alpha
}

fn hold_steps(seed: &mut u32, minimum_seconds: u32, maximum_seconds: u32) -> u32 {
    let span = maximum_seconds - minimum_seconds;
    let seconds = minimum_seconds + next_u32(seed) % span;
    seconds * UPDATE_RATE_HZ as u32
}

fn signed_unit(seed: &mut u32) -> f32 {
    let fraction = (next_u32(seed) >> 8) as f32 / 16_777_215.0;
    fraction * 2.0 - 1.0
}

fn next_u32(state: &mut u32) -> u32 {
    let mut value = *state;
    value ^= value << 13;
    value ^= value >> 17;
    value ^= value << 5;
    *state = value;
    value
}

fn ppm_to_semitones(ppm: f32) -> f32 {
    12.0 * libm::log2f(1.0 + ppm * 1.0e-6)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn advance_control_steps(bank: &mut VcoDriftBank, steps: usize) {
        for _ in 0..steps {
            bank.step_control_rate();
        }
    }

    fn corrections(bank: VcoDriftBank, character: f32) -> [f32; VCO_COUNT] {
        core::array::from_fn(|channel| {
            let oscillator = if channel % 2 == 0 {
                Oscillator::A
            } else {
                Oscillator::B
            };
            bank.correction_ppm(channel / 2, oscillator, character)
        })
    }

    #[test]
    fn published_ppm_limits_convert_to_sub_cent_motion() {
        let typical_cents = ppm_to_semitones(50.0) * 100.0;
        let maximum_cents = ppm_to_semitones(200.0) * 100.0;
        assert!((typical_cents - 0.086_56).abs() < 0.000_1);
        assert!((maximum_cents - 0.346_2).abs() < 0.000_2);
    }

    #[test]
    fn all_ten_vcos_develop_distinct_post_tune_offsets() {
        let mut bank = VcoDriftBank::default();
        advance_control_steps(&mut bank, 2_400);
        let values = corrections(bank, 1.0);
        for (index, value) in values.iter().enumerate() {
            assert!(value.abs() > 0.001);
            assert!(
                values[..index]
                    .iter()
                    .all(|previous| (previous - value).abs() > 0.000_1)
            );
        }
    }

    #[test]
    fn process_is_reproducible_and_independent_of_audio_sample_rate() {
        let mut at_44k = VcoDriftBank::default();
        let mut at_96k = VcoDriftBank::default();
        for _ in 0..441_000 {
            at_44k.advance(44_100.0);
        }
        for _ in 0..960_000 {
            at_96k.advance(96_000.0);
        }
        assert_eq!(corrections(at_44k, 0.5), corrections(at_96k, 0.5));
    }

    #[test]
    fn data_sheet_typical_and_maximum_bounds_are_hard_limits() {
        let mut bank = VcoDriftBank::default();
        for _ in 0..40_000 {
            bank.step_control_rate();
            assert!(
                corrections(bank, 0.0)
                    .into_iter()
                    .all(|ppm| ppm.abs() <= TYPICAL_DRIFT_PPM)
            );
            assert!(
                corrections(bank, 1.0)
                    .into_iter()
                    .all(|ppm| ppm.abs() <= MAXIMUM_DRIFT_PPM)
            );
        }
    }

    #[test]
    fn retune_captures_current_condition_without_resetting_the_process() {
        let mut bank = VcoDriftBank::default();
        advance_control_steps(&mut bank, 2_400);
        assert!(
            corrections(bank, 1.0)
                .into_iter()
                .any(|ppm| ppm.abs() > 0.1)
        );
        bank.retune();
        assert!(
            corrections(bank, 1.0)
                .into_iter()
                .all(|ppm| ppm.abs() <= f32::EPSILON)
        );
        advance_control_steps(&mut bank, 400);
        assert!(
            corrections(bank, 1.0)
                .into_iter()
                .any(|ppm| ppm.abs() > 0.1)
        );
    }
}
