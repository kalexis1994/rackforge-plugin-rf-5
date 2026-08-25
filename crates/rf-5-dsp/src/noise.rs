//! The two independent MM5837 noise paths fitted to the Rev 3 hardware.
//!
//! U375 on SD334 feeds the Wheel Mod source through a 47 kohm input and a
//! 100 kohm / 0.01 uF parallel-feedback pinking stage. U427 on SD430 is a
//! separate device: its AC-coupled white output drives the common noise-level
//! CA3280 and is then distributed to all five voice filters.

const LFSR_MASK: u32 = (1 << 17) - 1;
const PINK_RESET_SEED: u32 = 0x1_5a4d;
const WHITE_RESET_SEED: u32 = 0x0_c2b7;
const CANDIDATE_CLOCK_HZ: f32 = 80_000.0;
const INTERNAL_OVERSAMPLING: usize = 4;

const PINK_INPUT_RESISTANCE_OHMS: f32 = 47_000.0;
const PINK_FEEDBACK_RESISTANCE_OHMS: f32 = 100_000.0;
const PINK_FEEDBACK_CAPACITANCE_FARADS: f32 = 0.01e-6;

const WHITE_COUPLING_CAPACITANCE_FARADS: f32 = 0.1e-6;
const WHITE_SERIES_RESISTANCE_OHMS: f32 = 200_000.0;
const WHITE_SHUNT_RESISTANCE_OHMS: f32 = 10_000.0;

#[derive(Clone, Copy, Debug)]
struct Mm5837 {
    lfsr: u32,
    reset_seed: u32,
    clock_phase: f32,
    held: f32,
}

impl Mm5837 {
    const fn new(seed: u32) -> Self {
        Self {
            lfsr: seed,
            reset_seed: seed,
            clock_phase: 0.0,
            held: if seed & 1 == 0 { -1.0 } else { 1.0 },
        }
    }

    fn reset(&mut self) {
        *self = Self::new(self.reset_seed);
    }

    fn next_subsample(&mut self, internal_rate: f32) -> f32 {
        self.clock_phase += CANDIDATE_CLOCK_HZ / internal_rate;
        while self.clock_phase >= 1.0 {
            self.clock_phase -= 1.0;
            self.advance_lfsr();
        }
        self.held
    }

    fn advance_lfsr(&mut self) {
        let feedback = ((self.lfsr >> 16) ^ (self.lfsr >> 13)) & 1;
        self.lfsr = ((self.lfsr << 1) | feedback) & LFSR_MASK;
        debug_assert_ne!(self.lfsr, 0);
        self.held = if self.lfsr & 1 == 0 { -1.0 } else { 1.0 };
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PinkNoise {
    source: Mm5837,
    low_pass: f32,
}

impl Default for PinkNoise {
    fn default() -> Self {
        Self {
            source: Mm5837::new(PINK_RESET_SEED),
            low_pass: 0.0,
        }
    }
}

impl PinkNoise {
    pub fn reset(&mut self) {
        self.source.reset();
        self.low_pass = 0.0;
    }

    pub fn next(&mut self, sample_rate: f32) -> f32 {
        let internal_rate = sample_rate.max(1.0) * INTERNAL_OVERSAMPLING as f32;
        let cutoff_hz = 1.0
            / (2.0
                * core::f32::consts::PI
                * PINK_FEEDBACK_RESISTANCE_OHMS
                * PINK_FEEDBACK_CAPACITANCE_FARADS);
        let coefficient =
            1.0 - libm::expf(-2.0 * core::f32::consts::PI * cutoff_hz / internal_rate);
        for _ in 0..INTERNAL_OVERSAMPLING {
            let raw = self.source.next_subsample(internal_rate);
            self.low_pass += coefficient * (raw - self.low_pass);
        }
        // The inverting sign is immaterial for noise, but the populated
        // closed-loop magnitude fixes its level relative to the other Wheel
        // Mod source before the still-isolated absolute calibration anchor.
        self.low_pass * PINK_FEEDBACK_RESISTANCE_OHMS / PINK_INPUT_RESISTANCE_OHMS
    }

    #[cfg(test)]
    pub(crate) fn state(self) -> u32 {
        self.source.lfsr
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WhiteNoise {
    source: Mm5837,
    coupling_low_pass: f32,
}

impl Default for WhiteNoise {
    fn default() -> Self {
        Self {
            source: Mm5837::new(WHITE_RESET_SEED),
            coupling_low_pass: 0.0,
        }
    }
}

impl WhiteNoise {
    pub fn reset(&mut self) {
        self.source.reset();
        self.coupling_low_pass = 0.0;
    }

    pub fn next(&mut self, sample_rate: f32) -> f32 {
        let internal_rate = sample_rate.max(1.0) * INTERNAL_OVERSAMPLING as f32;
        let time_constant = (WHITE_SERIES_RESISTANCE_OHMS + WHITE_SHUNT_RESISTANCE_OHMS)
            * WHITE_COUPLING_CAPACITANCE_FARADS;
        let coefficient = 1.0 - libm::expf(-1.0 / (time_constant * internal_rate));
        let mut output = 0.0;
        for _ in 0..INTERNAL_OVERSAMPLING {
            let raw = self.source.next_subsample(internal_rate);
            self.coupling_low_pass += coefficient * (raw - self.coupling_low_pass);
            output = raw - self.coupling_low_pass;
        }
        // Absolute MM5837 swing and CA3280 input drive remain one candidate
        // boundary. Preserve unity pass-band normalization while reproducing
        // C458/R4131/R4132's documented 7.58 Hz AC-coupling corner.
        output
    }

    #[cfg(test)]
    pub(crate) fn state(self) -> u32 {
        self.source.lfsr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lfsr_has_the_documented_maximal_period() {
        let mut source = Mm5837::new(PINK_RESET_SEED);
        let initial = source.lfsr;
        for _ in 1..LFSR_MASK {
            source.advance_lfsr();
            assert_ne!(source.lfsr, initial);
            assert_ne!(source.lfsr, 0);
        }
        source.advance_lfsr();
        assert_eq!(source.lfsr, initial);
    }

    #[test]
    fn two_physical_sources_are_deterministic_but_independent() {
        let mut first_pink = PinkNoise::default();
        let mut second_pink = PinkNoise::default();
        let mut first_white = WhiteNoise::default();
        let mut second_white = WhiteNoise::default();
        let mut paths_differ = false;
        for _ in 0..96_000 {
            let pink_a = first_pink.next(48_000.0);
            let pink_b = second_pink.next(48_000.0);
            let white_a = first_white.next(48_000.0);
            let white_b = second_white.next(48_000.0);
            assert_eq!(pink_a, pink_b);
            assert_eq!(white_a, white_b);
            assert!(pink_a.is_finite() && white_a.is_finite());
            paths_differ |= (pink_a - white_a).abs() > 1.0e-3;
        }
        assert!(paths_differ);
        assert_ne!(first_pink.state(), first_white.state());
    }

    #[test]
    fn populated_networks_produce_pink_and_white_spectra() {
        let pink_corner = 1.0
            / (2.0
                * core::f32::consts::PI
                * PINK_FEEDBACK_RESISTANCE_OHMS
                * PINK_FEEDBACK_CAPACITANCE_FARADS);
        let white_corner = 1.0
            / (2.0
                * core::f32::consts::PI
                * (WHITE_SERIES_RESISTANCE_OHMS + WHITE_SHUNT_RESISTANCE_OHMS)
                * WHITE_COUPLING_CAPACITANCE_FARADS);
        assert!((159.0..160.0).contains(&pink_corner));
        assert!((7.5..7.7).contains(&white_corner));

        let mut pink = PinkNoise::default();
        let mut white = WhiteNoise::default();
        let mut previous_pink = 0.0;
        let mut previous_white = 0.0;
        let mut pink_difference_energy = 0.0;
        let mut white_difference_energy = 0.0;
        for _ in 0..96_000 {
            let pink_sample = pink.next(48_000.0);
            let white_sample = white.next(48_000.0);
            pink_difference_energy += (pink_sample - previous_pink).powi(2);
            white_difference_energy += (white_sample - previous_white).powi(2);
            previous_pink = pink_sample;
            previous_white = white_sample;
        }
        assert!(white_difference_energy > pink_difference_energy * 100.0);
    }

    #[test]
    fn both_render_levels_are_stable_across_supported_sample_rates() {
        let mut reference_pink = None;
        let mut reference_white = None;
        for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
            let mut pink = PinkNoise::default();
            let mut white = WhiteNoise::default();
            let frames = sample_rate as usize * 3;
            let mut pink_energy = 0.0;
            let mut white_energy = 0.0;
            for _ in 0..frames {
                let pink_sample = pink.next(sample_rate);
                let white_sample = white.next(sample_rate);
                pink_energy += pink_sample * pink_sample;
                white_energy += white_sample * white_sample;
            }
            let pink_rms = libm::sqrtf(pink_energy / frames as f32);
            let white_rms = libm::sqrtf(white_energy / frames as f32);
            assert!(pink_rms > 0.1 && white_rms > 0.5);
            if let (Some(pink_reference), Some(white_reference)) = (reference_pink, reference_white)
            {
                assert!((0.8..1.2).contains(&(pink_rms / pink_reference)));
                assert!((0.8..1.2).contains(&(white_rms / white_reference)));
            } else {
                reference_pink = Some(pink_rms);
                reference_white = Some(white_rms);
            }
        }
    }
}
