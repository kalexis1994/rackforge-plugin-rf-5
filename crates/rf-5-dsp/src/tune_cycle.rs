//! Front-panel TUNE operation timing.
//!
//! The original CPU occupies the panel and oscillator measurement hardware
//! for roughly two to eight seconds. Duration grows with the correction that
//! the current thermal state requires; the calibration table itself remains
//! machine state rather than patch data.

const MINIMUM_SECONDS: f32 = 2.0;
const MAXIMUM_SECONDS: f32 = 8.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct TuneCycle {
    remaining_samples: u64,
    total_samples: u64,
}

impl TuneCycle {
    pub fn start(&mut self, sample_rate: f32, normalized_error: f32) -> bool {
        if self.is_active() || !sample_rate.is_finite() || sample_rate <= 0.0 {
            return false;
        }
        let duration = MINIMUM_SECONDS
            + normalized_error.clamp(0.0, 1.0) * (MAXIMUM_SECONDS - MINIMUM_SECONDS);
        self.total_samples = libm::roundf(duration * sample_rate).max(1.0) as u64;
        self.remaining_samples = self.total_samples;
        true
    }

    pub fn is_active(self) -> bool {
        self.remaining_samples != 0
    }

    pub fn duration_seconds(self, sample_rate: f32) -> f32 {
        if !sample_rate.is_finite() || sample_rate <= 0.0 {
            return 0.0;
        }
        self.total_samples as f32 / sample_rate
    }

    /// Returns true exactly once, on the sample that completes calibration.
    pub fn advance(&mut self) -> bool {
        if self.remaining_samples == 0 {
            return false;
        }
        self.remaining_samples -= 1;
        self.remaining_samples == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_spans_the_documented_two_to_eight_seconds() {
        for (error, expected) in [(0.0, 2.0), (0.5, 5.0), (1.0, 8.0)] {
            let mut cycle = TuneCycle::default();
            assert!(cycle.start(48_000.0, error));
            assert_eq!(cycle.duration_seconds(48_000.0), expected);
        }
    }

    #[test]
    fn active_cycle_cannot_be_retriggered_and_completes_once() {
        let mut cycle = TuneCycle::default();
        assert!(cycle.start(10.0, 0.0));
        assert!(!cycle.start(10.0, 1.0));
        for _ in 0..19 {
            assert!(!cycle.advance());
        }
        assert!(cycle.advance());
        assert!(!cycle.advance());
        assert!(!cycle.is_active());
    }

    #[test]
    fn invalid_rate_does_not_start() {
        let mut cycle = TuneCycle::default();
        assert!(!cycle.start(0.0, 0.5));
        assert!(!cycle.start(f32::NAN, 0.5));
    }
}
