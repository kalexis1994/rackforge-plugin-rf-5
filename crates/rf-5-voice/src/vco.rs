//! Band-limited numerical model of one CEM3340-class oscillator core.
//!
//! The chip topology and available outputs are source-backed. PolyBLEP edge
//! correction and four-times internal oversampling are RF-5's numerical
//! strategy, not claims about circuitry inside the physical IC.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WaveSelection {
    pub saw: bool,
    pub triangle: bool,
    pub pulse: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OscillatorSample {
    pub value: f32,
    pub wrapped: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Vco {
    phase: f32,
}

impl Default for Vco {
    fn default() -> Self {
        Self { phase: 0.0 }
    }
}

impl Vco {
    pub fn with_phase(phase: f32) -> Self {
        let phase = if phase.is_finite() { phase % 1.0 } else { 0.0 };
        Self {
            phase: if phase < 0.0 { phase + 1.0 } else { phase },
        }
    }

    pub fn hard_sync(&mut self) {
        self.phase = 0.0;
    }

    pub fn phase(self) -> f32 {
        self.phase
    }

    pub fn next(
        &mut self,
        frequency: f32,
        sample_rate: f32,
        pulse_width: f32,
        waves: WaveSelection,
    ) -> OscillatorSample {
        let increment = (frequency.max(0.0) / sample_rate.max(1.0)).clamp(0.0, 0.49);
        let pulse_width = pulse_width.clamp(0.02, 0.98);
        let phase = self.phase;
        let mut value = 0.0;

        if waves.saw {
            value += band_limited_saw(phase, increment);
        }
        if waves.triangle {
            value += triangle(phase);
        }
        if waves.pulse {
            value += band_limited_pulse(phase, increment, pulse_width);
        }

        let advanced = phase + increment;
        let wrapped = advanced >= 1.0;
        self.phase = if wrapped { advanced - 1.0 } else { advanced };

        OscillatorSample { value, wrapped }
    }
}

fn band_limited_saw(phase: f32, increment: f32) -> f32 {
    let naive = phase * 2.0 - 1.0;
    naive - poly_blep(phase, increment)
}

fn band_limited_pulse(phase: f32, increment: f32, pulse_width: f32) -> f32 {
    let naive = if phase < pulse_width { 1.0 } else { -1.0 };
    let falling_phase = if phase >= pulse_width {
        phase - pulse_width
    } else {
        phase + (1.0 - pulse_width)
    };
    naive + poly_blep(phase, increment) - poly_blep(falling_phase, increment)
}

fn triangle(phase: f32) -> f32 {
    1.0 - 4.0 * (phase - 0.5).abs()
}

fn poly_blep(phase: f32, increment: f32) -> f32 {
    if increment <= 0.0 {
        return 0.0;
    }
    if phase < increment {
        let x = phase / increment;
        return x + x - x * x - 1.0;
    }
    if phase > 1.0 - increment {
        let x = (phase - 1.0) / increment;
        return x * x + x + x + 1.0;
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAW: WaveSelection = WaveSelection {
        saw: true,
        triangle: false,
        pulse: false,
    };
    const PULSE: WaveSelection = WaveSelection {
        saw: false,
        triangle: false,
        pulse: true,
    };

    #[test]
    fn phase_wrap_is_reported_and_bounded() {
        let mut oscillator = Vco::with_phase(0.99);
        let sample = oscillator.next(1_000.0, 48_000.0, 0.5, SAW);
        assert!(sample.wrapped);
        assert!((0.0..1.0).contains(&oscillator.phase()));
    }

    #[test]
    fn pulse_width_changes_duty_cycle() {
        let mut narrow = Vco::default();
        let mut wide = Vco::default();
        let mut narrow_positive = 0;
        let mut wide_positive = 0;
        for _ in 0..1_000 {
            narrow_positive += (narrow.next(100.0, 10_000.0, 0.25, PULSE).value > 0.0) as usize;
            wide_positive += (wide.next(100.0, 10_000.0, 0.75, PULSE).value > 0.0) as usize;
        }
        assert!(narrow_positive < wide_positive);
    }

    #[test]
    fn hard_sync_resets_phase_without_reallocation() {
        let mut oscillator = Vco::with_phase(0.73);
        oscillator.hard_sync();
        assert_eq!(oscillator.phase(), 0.0);
    }

    #[test]
    fn all_wave_combinations_stay_finite() {
        let mut oscillator = Vco::default();
        let waves = WaveSelection {
            saw: true,
            triangle: true,
            pulse: true,
        };
        for _ in 0..10_000 {
            let sample = oscillator.next(12_000.0, 48_000.0, 0.37, waves).value;
            assert!(sample.is_finite());
            assert!(sample.abs() <= 3.1);
        }
    }
}
