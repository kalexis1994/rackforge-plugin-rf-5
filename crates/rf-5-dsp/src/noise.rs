//! The two independent MM5837 noise paths fitted to the Rev 3 hardware.
//!
//! U375 on SD334 feeds the Wheel Mod source through a 47 kohm input and a
//! 100 kohm / 0.01 uF parallel-feedback pinking stage. U427 on SD430 is a
//! separate device: its AC-coupled white output drives the common noise-level
//! CA3280 and is then distributed to all five voice filters.

const LFSR_MASK: u32 = (1 << 17) - 1;
const PINK_RESET_SEED: u32 = 0x1_5a4d;
const WHITE_RESET_SEED: u32 = 0x0_c2b7;

// The data sheet does not publish a typical self-clock frequency. It instead
// bounds one complete maximal-length sequence to 1.1-2.4 seconds. The
// geometric centre minimizes the largest relative error to either limit and
// remains an explicit RF-5 candidate rather than a claimed device typical.
#[cfg(test)]
const MM5837_MINIMUM_CYCLE_SECONDS: f32 = 1.1;
#[cfg(test)]
const MM5837_MAXIMUM_CYCLE_SECONDS: f32 = 2.4;
const MM5837_CANDIDATE_CYCLE_SECONDS: f32 = 1.624_807_7;
const MM5837_CLOCK_HZ: f32 = LFSR_MASK as f32 / MM5837_CANDIDATE_CYCLE_SECONDS;

// A held random-bit stream has a sinc-squared power response. Its -3 dB
// frequency is this fraction of the update clock, providing an independent
// check against the data sheet's published 24-56 kHz half-power range.
#[cfg(test)]
const ZERO_ORDER_HOLD_HALF_POWER_RATIO: f32 = 0.442_946_46;
#[cfg(test)]
const MM5837_MINIMUM_HALF_POWER_HZ: f32 = 24_000.0;
#[cfg(test)]
const MM5837_MAXIMUM_HALF_POWER_HZ: f32 = 56_000.0;
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
    clock_phase: f64,
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

    /// Visit every constant-output segment inside one internal sample.
    ///
    /// The MM5837 clock is asynchronous to the audio clock. Keeping the
    /// fractional edge position lets the following analog RC network evolve
    /// for the correct time on each side of a random-bit transition.
    fn for_each_segment(&mut self, internal_rate: f32, mut visit: impl FnMut(f32, f64)) {
        let internal_rate = f64::from(internal_rate.max(1.0));
        let phase_increment = f64::from(MM5837_CLOCK_HZ) / internal_rate;
        let mut remaining_fraction = 1.0_f64;

        while remaining_fraction > 0.0 {
            let fraction_to_edge = (1.0 - self.clock_phase) / phase_increment;
            if fraction_to_edge > remaining_fraction {
                visit(self.held, remaining_fraction);
                self.clock_phase += phase_increment * remaining_fraction;
                break;
            }

            visit(self.held, fraction_to_edge);
            remaining_fraction = (remaining_fraction - fraction_to_edge).max(0.0);
            self.clock_phase = 0.0;
            self.advance_lfsr();
        }
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
    low_pass: f64,
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
        let time_constant =
            f64::from(PINK_FEEDBACK_RESISTANCE_OHMS * PINK_FEEDBACK_CAPACITANCE_FARADS);
        let full_interval_retained = libm::exp(-1.0 / (f64::from(internal_rate) * time_constant));
        let mut output = 0.0;
        for _ in 0..INTERNAL_OVERSAMPLING {
            output = self.next_internal(internal_rate, full_interval_retained);
        }
        output
    }

    fn next_internal(&mut self, internal_rate: f32, full_interval_retained: f64) -> f32 {
        let low_pass = &mut self.low_pass;
        self.source
            .for_each_segment(internal_rate, |raw, interval_fraction| {
                *low_pass = advance_rc_fraction(
                    *low_pass,
                    f64::from(raw),
                    interval_fraction,
                    full_interval_retained,
                );
            });
        // The inverting sign is immaterial for noise, but the populated
        // closed-loop magnitude fixes its level relative to the other Wheel
        // Mod source before the still-isolated absolute calibration anchor.
        self.low_pass as f32 * PINK_FEEDBACK_RESISTANCE_OHMS / PINK_INPUT_RESISTANCE_OHMS
    }

    #[cfg(test)]
    pub(crate) fn state(self) -> u32 {
        self.source.lfsr
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WhiteNoise {
    source: Mm5837,
    coupling_low_pass: f64,
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
        let time_constant = f64::from(
            (WHITE_SERIES_RESISTANCE_OHMS + WHITE_SHUNT_RESISTANCE_OHMS)
                * WHITE_COUPLING_CAPACITANCE_FARADS,
        );
        let full_interval_retained = libm::exp(-1.0 / (f64::from(internal_rate) * time_constant));
        let mut output = 0.0;
        for _ in 0..INTERNAL_OVERSAMPLING {
            output = self.next_internal(internal_rate, full_interval_retained);
        }
        output
    }

    fn next_internal(&mut self, internal_rate: f32, full_interval_retained: f64) -> f32 {
        let coupling_low_pass = &mut self.coupling_low_pass;
        self.source
            .for_each_segment(internal_rate, |raw, interval_fraction| {
                *coupling_low_pass = advance_rc_fraction(
                    *coupling_low_pass,
                    f64::from(raw),
                    interval_fraction,
                    full_interval_retained,
                );
            });
        // Absolute MM5837 swing and CA3280 input drive remain one candidate
        // boundary. Preserve unity pass-band normalization while reproducing
        // C458/R4131/R4132's documented 7.58 Hz AC-coupling corner.
        self.source.held - self.coupling_low_pass as f32
    }

    #[cfg(test)]
    pub(crate) fn state(self) -> u32 {
        self.source.lfsr
    }
}

fn advance_rc_fraction(
    value: f64,
    target: f64,
    interval_fraction: f64,
    full_interval_retained: f64,
) -> f64 {
    let retained = if interval_fraction == 1.0 {
        full_interval_retained
    } else {
        libm::exp(libm::log(full_interval_retained) * interval_fraction)
    };
    target + (value - target) * retained
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_self_clock_obeys_both_published_ranges() {
        let cycle_seconds = LFSR_MASK as f32 / MM5837_CLOCK_HZ;
        let half_power_hz = MM5837_CLOCK_HZ * ZERO_ORDER_HOLD_HALF_POWER_RATIO;

        assert!(
            (MM5837_MINIMUM_CYCLE_SECONDS..=MM5837_MAXIMUM_CYCLE_SECONDS).contains(&cycle_seconds)
        );
        assert!(
            (MM5837_MINIMUM_HALF_POWER_HZ..=MM5837_MAXIMUM_HALF_POWER_HZ).contains(&half_power_hz)
        );
        assert!(
            (cycle_seconds * cycle_seconds
                - MM5837_MINIMUM_CYCLE_SECONDS * MM5837_MAXIMUM_CYCLE_SECONDS)
                .abs()
                < 1.0e-5
        );
    }

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
    fn fractional_mm5837_edge_splits_both_analog_networks() {
        let internal_rate = 192_000.0;
        let phase_increment = f64::from(MM5837_CLOCK_HZ) / f64::from(internal_rate);
        let initial = 0.2;

        let mut pink = PinkNoise {
            source: Mm5837::new(1),
            low_pass: initial,
        };
        pink.source.clock_phase = 1.0 - 0.25 * phase_increment;
        let pink_time_constant =
            f64::from(PINK_FEEDBACK_RESISTANCE_OHMS * PINK_FEEDBACK_CAPACITANCE_FARADS);
        let pink_retained = libm::exp(-1.0 / (f64::from(internal_rate) * pink_time_constant));
        let expected_pink = advance_rc_fraction(
            advance_rc_fraction(initial, 1.0, 0.25, pink_retained),
            -1.0,
            0.75,
            pink_retained,
        );
        let pink_gain = PINK_FEEDBACK_RESISTANCE_OHMS / PINK_INPUT_RESISTANCE_OHMS;
        let actual_pink = f64::from(pink.next_internal(internal_rate, pink_retained) / pink_gain);
        assert!((actual_pink - expected_pink).abs() < 1.0e-7);

        let mut white = WhiteNoise {
            source: Mm5837::new(1),
            coupling_low_pass: initial,
        };
        white.source.clock_phase = 1.0 - 0.25 * phase_increment;
        let white_time_constant = f64::from(
            (WHITE_SERIES_RESISTANCE_OHMS + WHITE_SHUNT_RESISTANCE_OHMS)
                * WHITE_COUPLING_CAPACITANCE_FARADS,
        );
        let white_retained = libm::exp(-1.0 / (f64::from(internal_rate) * white_time_constant));
        let expected_white_low_pass = advance_rc_fraction(
            advance_rc_fraction(initial, 1.0, 0.25, white_retained),
            -1.0,
            0.75,
            white_retained,
        );
        let actual_white = white.next_internal(internal_rate, white_retained);
        assert!((f64::from(actual_white) - (-1.0 - expected_white_low_pass)).abs() < 1.0e-7);
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
