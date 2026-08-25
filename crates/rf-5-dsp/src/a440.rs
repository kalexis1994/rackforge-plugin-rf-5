//! Rev 3 A-440 reference generator and its SD430 output-network shaping.
//!
//! Counter 1 of the 8253 divides the 2.5 MHz CPU clock by 5682. The selected
//! square wave then crosses R4498/C4183 before R4559 feeds the five-voice
//! summer. Keeping that path here also makes the reference obey Master Volume.

const CPU_CLOCK_HZ: f64 = 2_500_000.0;
const COUNTER_DIVISOR: f64 = 5_682.0;
pub const FREQUENCY_HZ: f64 = CPU_CLOCK_HZ / COUNTER_DIVISOR;

const SERIES_RESISTANCE_OHMS: f64 = 10_000.0;
const SHUNT_CAPACITANCE_FARADS: f64 = 0.1e-6;
const VOICE_INPUT_RESISTANCE_OHMS: f32 = 39_000.0;
const REFERENCE_INPUT_RESISTANCE_OHMS: f32 = 20_000.0;
const SUMMER_INJECTION_GAIN: f32 = VOICE_INPUT_RESISTANCE_OHMS / REFERENCE_INPUT_RESISTANCE_OHMS;

#[derive(Clone, Copy, Debug, Default)]
pub struct ReferenceTone {
    phase: f64,
    filtered: f64,
}

impl ReferenceTone {
    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.filtered = 0.0;
    }

    /// The counter remains free-running. With A-440 deselected U460 grounds
    /// the network input, so its capacitor settles without leaking a clock.
    pub fn next(&mut self, enabled: bool, sample_rate: f32) -> f32 {
        if !sample_rate.is_finite() || sample_rate <= 0.0 {
            self.reset();
            return 0.0;
        }

        self.phase += FREQUENCY_HZ / f64::from(sample_rate);
        self.phase -= libm::floor(self.phase);
        let input = if enabled {
            if self.phase < 0.5 { 1.0 } else { -1.0 }
        } else {
            0.0
        };
        let time_constant = SERIES_RESISTANCE_OHMS * SHUNT_CAPACITANCE_FARADS;
        let coefficient = 1.0 - libm::exp(-1.0 / (f64::from(sample_rate) * time_constant));
        self.filtered += coefficient * (input - self.filtered);
        self.filtered as f32 * SUMMER_INJECTION_GAIN
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_division_is_the_documented_reference_frequency() {
        assert!((FREQUENCY_HZ - 439.985_920_45).abs() < 1.0e-8);
    }

    #[test]
    fn measured_frequency_is_independent_of_audio_rate() {
        for sample_rate in [44_100.0, 48_000.0, 96_000.0] {
            let mut tone = ReferenceTone::default();
            let duration = sample_rate as usize * 4;
            let mut crossings = 0_u32;
            let mut previous = tone.next(true, sample_rate);
            for _ in 1..duration {
                let sample = tone.next(true, sample_rate);
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
    fn deselection_grounds_and_settles_the_rc_input() {
        let mut tone = ReferenceTone::default();
        for _ in 0..48_000 {
            let _ = tone.next(true, 48_000.0);
        }
        let mut sample = 1.0;
        for _ in 0..4_800 {
            sample = tone.next(false, 48_000.0);
        }
        assert!(sample.abs() < 1.0e-6);
    }

    #[test]
    fn invalid_rate_resets_the_generator() {
        let mut tone = ReferenceTone::default();
        let _ = tone.next(true, 48_000.0);
        assert_eq!(tone.next(true, f32::NAN), 0.0);
        assert_eq!(tone.next(false, 48_000.0), 0.0);
    }
}
