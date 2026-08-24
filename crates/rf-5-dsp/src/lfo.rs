//! One free-running modulation oscillator shared by all five voices.
//!
//! The Rev 3 service manual establishes the single-oscillator topology, the
//! three additive shapes and the square wave's 50% duty cycle. Its absolute
//! frequency endpoints are not specified, so they remain isolated candidate
//! constants until measurements can narrow them.

use rf_5_contract::hardware::quantize_analog_pot;

const CANDIDATE_MINIMUM_HZ: f32 = 0.08;
const CANDIDATE_MAXIMUM_HZ: f32 = 20.0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LfoWaveSelection {
    pub saw: bool,
    pub triangle: bool,
    pub square: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Lfo {
    phase: f32,
}

impl Lfo {
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    pub fn next(
        &mut self,
        sample_rate: f32,
        frequency_control: f32,
        waves: LfoWaveSelection,
    ) -> f32 {
        let phase = self.phase;
        let saw = phase * 2.0 - 1.0;
        let triangle = 1.0 - 4.0 * (phase - 0.5).abs();
        let square = if phase < 0.5 { 1.0 } else { -1.0 };
        let mut output = 0.0;
        if waves.saw {
            output += saw;
        }
        if waves.triangle {
            output += triangle;
        }
        if waves.square {
            output += square;
        }

        let increment = frequency_hz(frequency_control) / sample_rate.max(1.0);
        let advanced = self.phase + increment;
        self.phase = advanced - libm::floorf(advanced);
        output
    }

    #[cfg(test)]
    pub(crate) fn phase(self) -> f32 {
        self.phase
    }
}

pub fn frequency_hz(control: f32) -> f32 {
    let control = quantize_analog_pot(control);
    CANDIDATE_MINIMUM_HZ * libm::powf(CANDIDATE_MAXIMUM_HZ / CANDIDATE_MINIMUM_HZ, control)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_frequency_mapping_is_monotonic_and_bounded() {
        assert!((frequency_hz(0.0) - CANDIDATE_MINIMUM_HZ).abs() < 1.0e-6);
        assert!((frequency_hz(1.0) - CANDIDATE_MAXIMUM_HZ).abs() < 1.0e-4);
        let mut previous = frequency_hz(0.0);
        for step in 1..=127 {
            let current = frequency_hz(step as f32 / 127.0);
            assert!(current > previous);
            previous = current;
        }
    }

    #[test]
    fn square_has_equal_positive_and_negative_halves() {
        let mut lfo = Lfo::default();
        let waves = LfoWaveSelection {
            square: true,
            ..LfoWaveSelection::default()
        };
        let sample_rate = 20_000.0;
        let mut positive: i32 = 0;
        let mut negative: i32 = 0;
        for _ in 0..1_000 {
            let sample = lfo.next(sample_rate, 1.0, waves);
            if sample > 0.0 {
                positive += 1;
            } else {
                negative += 1;
            }
        }
        assert!((positive - negative).abs() <= 2);
    }

    #[test]
    fn enabled_shapes_are_summed_on_the_shared_bus() {
        let mut lfo = Lfo::default();
        let all = LfoWaveSelection {
            saw: true,
            triangle: true,
            square: true,
        };
        assert!((lfo.next(48_000.0, 0.5, all) + 1.0).abs() < 1.0e-6);
    }
}
