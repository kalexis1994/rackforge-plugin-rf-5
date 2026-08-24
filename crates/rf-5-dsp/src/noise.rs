//! Shared MM5837-class pseudo-random source and its analog pinking stage.
//!
//! The hardware uses one self-clocked 17-bit pseudo-random generator for the
//! common noise bus. The exact chip clock and final analog level vary by part
//! and calibration, so those values remain isolated candidate constants.

const LFSR_MASK: u32 = (1 << 17) - 1;
const RESET_SEED: u32 = 0x1_5a4d;
const CANDIDATE_CLOCK_HZ: f32 = 80_000.0;
const PINKING_RESISTANCE_OHMS: f32 = 100_000.0;
const PINKING_CAPACITANCE_FARADS: f32 = 0.01e-6;
const CANDIDATE_OUTPUT_GAIN: f32 = 5.5;
const INTERNAL_OVERSAMPLING: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct PinkNoise {
    lfsr: u32,
    clock_phase: f32,
    held: f32,
    pink: f32,
}

impl Default for PinkNoise {
    fn default() -> Self {
        Self {
            lfsr: RESET_SEED,
            clock_phase: 0.0,
            held: 1.0,
            pink: 0.0,
        }
    }
}

impl PinkNoise {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn next(&mut self, sample_rate: f32) -> f32 {
        let internal_rate = sample_rate.max(1.0) * INTERNAL_OVERSAMPLING as f32;
        let clock_increment = CANDIDATE_CLOCK_HZ / internal_rate;
        let cutoff_hz = 1.0
            / (2.0 * core::f32::consts::PI * PINKING_RESISTANCE_OHMS * PINKING_CAPACITANCE_FARADS);
        let coefficient =
            1.0 - libm::expf(-2.0 * core::f32::consts::PI * cutoff_hz / internal_rate);

        for _ in 0..INTERNAL_OVERSAMPLING {
            self.clock_phase += clock_increment;
            while self.clock_phase >= 1.0 {
                self.clock_phase -= 1.0;
                self.advance_lfsr();
            }
            self.pink += coefficient * (self.held - self.pink);
        }

        self.pink * CANDIDATE_OUTPUT_GAIN
    }

    fn advance_lfsr(&mut self) {
        let feedback = ((self.lfsr >> 16) ^ (self.lfsr >> 13)) & 1;
        self.lfsr = ((self.lfsr << 1) | feedback) & LFSR_MASK;
        debug_assert_ne!(self.lfsr, 0);
        self.held = if self.lfsr & 1 == 0 { -1.0 } else { 1.0 };
    }

    #[cfg(test)]
    pub(crate) fn state(self) -> u32 {
        self.lfsr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lfsr_has_the_documented_maximal_period() {
        let mut noise = PinkNoise::default();
        let initial = noise.lfsr;
        for _ in 1..LFSR_MASK {
            noise.advance_lfsr();
            assert_ne!(noise.lfsr, initial);
            assert_ne!(noise.lfsr, 0);
        }
        noise.advance_lfsr();
        assert_eq!(noise.lfsr, initial);
    }

    #[test]
    fn output_is_deterministic_finite_and_bipolar() {
        let mut first = PinkNoise::default();
        let mut second = PinkNoise::default();
        let mut minimum = f32::MAX;
        let mut maximum = f32::MIN;
        for _ in 0..96_000 {
            let a = first.next(48_000.0);
            let b = second.next(48_000.0);
            assert_eq!(a, b);
            assert!(a.is_finite());
            minimum = minimum.min(a);
            maximum = maximum.max(a);
        }
        assert!(minimum < -0.1);
        assert!(maximum > 0.1);
    }

    #[test]
    fn render_level_is_stable_across_supported_sample_rates() {
        let mut reference_rms = None;
        for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
            let mut noise = PinkNoise::default();
            let frames = sample_rate as usize * 3;
            let mut energy = 0.0;
            for _ in 0..frames {
                let sample = noise.next(sample_rate);
                energy += sample * sample;
            }
            let rms = libm::sqrtf(energy / frames as f32);
            assert!(rms > 0.2 && rms < 1.0, "unexpected RMS at {sample_rate} Hz");
            if let Some(reference) = reference_rms {
                let ratio = rms / reference;
                assert!((0.8..1.2).contains(&ratio));
            } else {
                reference_rms = Some(rms);
            }
        }
    }
}
