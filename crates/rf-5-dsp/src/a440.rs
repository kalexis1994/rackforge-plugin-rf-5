//! Rev 3 A-440 reference generator.
//!
//! Counter 1 of the 8253 divides the 2.5 MHz CPU clock by 5682. U459 passes
//! the resulting 0/5 V square to TP401 while A-440 is selected; otherwise
//! U460 grounds TP401 so the network cannot leak the free-running counter.
//! TP401 then feeds R4498 10k, C4183 0.1 uF to ground and R4519 20k into the
//! five-voice summer. That network loads the voices as well, so it is solved
//! with the summer node in `output.rs`; this module supplies TP401 alone.

const CPU_CLOCK_HZ: f64 = 2_500_000.0;
const COUNTER_DIVISOR: f64 = 5_682.0;
pub const FREQUENCY_HZ: f64 = CPU_CLOCK_HZ / COUNTER_DIVISOR;
// 8253 output through a 4016 on the +5 V logic supply.
const REFERENCE_HIGH_VOLTS: f64 = 5.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct ReferenceTone {
    phase: f64,
}

impl ReferenceTone {
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    /// Mean TP401 voltage over the next host sample. The counter keeps
    /// running while deselected; its edges are placed at their exact
    /// fractional positions inside the sample.
    pub fn next(&mut self, enabled: bool, sample_rate: f32) -> f32 {
        if !sample_rate.is_finite() || sample_rate <= 0.0 {
            self.reset();
            return 0.0;
        }

        let sample_period_seconds = 1.0 / f64::from(sample_rate);
        let phase_increment = FREQUENCY_HZ * sample_period_seconds;
        let start_phase = self.phase;
        self.phase += phase_increment;
        self.phase -= libm::floor(self.phase);

        if !enabled {
            return 0.0;
        }
        (REFERENCE_HIGH_VOLTS * high_fraction(start_phase, phase_increment)) as f32
    }
}

/// Fraction of `[phase, phase + increment)` spent in the high first half of
/// the counter cycle.
fn high_fraction(phase: f64, increment: f64) -> f64 {
    if increment <= 0.0 {
        return if phase < 0.5 { 1.0 } else { 0.0 };
    }
    let high_time_until = |end: f64| {
        let whole = libm::floor(end);
        whole * 0.5 + (end - whole).min(0.5)
    };
    (high_time_until(phase + increment) - high_time_until(phase)) / increment
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_division_is_the_documented_reference_frequency() {
        assert!((FREQUENCY_HZ - 439.985_920_45).abs() < 1.0e-8);
    }

    #[test]
    fn mean_level_is_half_the_logic_swing_at_every_rate() {
        for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
            let mut tone = ReferenceTone::default();
            let samples = sample_rate as usize * 2;
            let mean = (0..samples)
                .map(|_| f64::from(tone.next(true, sample_rate)))
                .sum::<f64>()
                / samples as f64;
            assert!((mean - 2.5).abs() < 0.01, "rate={sample_rate} mean={mean}");
        }
    }

    #[test]
    fn fractional_counter_edge_splits_the_sample() {
        let sample_rate = 10_000.0_f32;
        let increment = FREQUENCY_HZ / f64::from(sample_rate);
        let mut tone = ReferenceTone { phase: 0.49 };
        let expected = REFERENCE_HIGH_VOLTS * (0.01 / increment);
        assert!((f64::from(tone.next(true, sample_rate)) - expected).abs() < 1.0e-5);
    }

    #[test]
    fn measured_frequency_is_independent_of_audio_rate() {
        for sample_rate in [44_100.0, 48_000.0, 96_000.0] {
            let mut tone = ReferenceTone::default();
            let duration = sample_rate as usize * 4;
            let mut crossings = 0_u32;
            let mut previous = tone.next(true, sample_rate) - 2.5;
            for _ in 1..duration {
                let sample = tone.next(true, sample_rate) - 2.5;
                crossings += u32::from(previous <= 0.0 && sample > 0.0);
                previous = sample;
            }
            let measured = f64::from(crossings) / 4.0;
            assert!(
                (measured - FREQUENCY_HZ).abs() <= 0.25,
                "rate={sample_rate}"
            );
        }
    }

    #[test]
    fn deselection_grounds_tp401() {
        let mut tone = ReferenceTone::default();
        for _ in 0..4_800 {
            let _ = tone.next(true, 48_000.0);
        }
        assert_eq!(tone.next(false, 48_000.0), 0.0);
    }

    #[test]
    fn invalid_rate_resets_the_generator() {
        let mut tone = ReferenceTone::default();
        let _ = tone.next(true, 48_000.0);
        assert_eq!(tone.next(true, f32::NAN), 0.0);
        assert_eq!(tone.phase, 0.0);
    }
}
