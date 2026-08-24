//! Four-pole CEM3320 low-pass candidate.
//!
//! The reference voice wires the four independent cells as cascaded low-pass
//! stages and returns the fourth output through the chip's resonance VCA. This
//! model preserves that topology and evaluates it at the voice oversampling
//! rate. Exact cell overload and resonance transconductance remain calibration
//! hypotheses.

const STAGE_COUNT: usize = 4;
const MAXIMUM_NORMALIZED_CUTOFF: f32 = 0.45;
const MAXIMUM_RESONANCE_FEEDBACK: f32 = 4.4;
const INPUT_DRIVE: f32 = 0.35;
const CELL_DRIVE: f32 = 0.08;
const THERMAL_NOISE_LEVEL: f32 = 2.0e-7;

#[derive(Clone, Copy, Debug, Default)]
struct TptStage {
    state: f32,
}

impl TptStage {
    fn next(&mut self, input: f32, coefficient: f32) -> f32 {
        let delta = (input - self.state) * coefficient;
        let output = delta + self.state;
        self.state = output + delta;
        soft_clip(output, CELL_DRIVE)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cem3320Filter {
    stages: [TptStage; STAGE_COUNT],
    noise_state: u32,
}

impl Default for Cem3320Filter {
    fn default() -> Self {
        Self {
            stages: [TptStage::default(); STAGE_COUNT],
            noise_state: 0x51f1_5e5d,
        }
    }
}

impl Cem3320Filter {
    pub fn next(&mut self, input: f32, cutoff_hz: f32, resonance: f32, sample_rate: f32) -> f32 {
        let sample_rate = sample_rate.max(1.0);
        let cutoff_hz = cutoff_hz.clamp(1.0, sample_rate * MAXIMUM_NORMALIZED_CUTOFF);
        let g = libm::tanf(core::f32::consts::PI * cutoff_hz / sample_rate);
        let coefficient = g / (1.0 + g);
        let feedback = resonance_feedback(resonance);
        let thermal_noise = self.next_thermal_noise();
        let mut signal = soft_clip(
            input + thermal_noise - self.stages[3].state * feedback,
            INPUT_DRIVE,
        );
        for stage in &mut self.stages {
            signal = stage.next(signal, coefficient);
        }
        if signal.is_finite() {
            signal
        } else {
            self.stages = [TptStage::default(); STAGE_COUNT];
            0.0
        }
    }

    fn next_thermal_noise(&mut self) -> f32 {
        let mut state = self.noise_state;
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        self.noise_state = state;
        let normalized = state as f32 / u32::MAX as f32;
        (normalized * 2.0 - 1.0) * THERMAL_NOISE_LEVEL
    }

    #[cfg(test)]
    fn energy(self) -> f32 {
        self.stages.iter().map(|stage| stage.state.abs()).sum()
    }
}

fn resonance_feedback(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    // The CEM3320 resonance cell is described as modified-linear and its
    // transconductance curve flattens near maximum current. This gentle bend
    // retains a near-linear panel response while placing oscillation inside
    // the 7-9.5 service-manual dial window.
    let modified_linear = value * (1.08 - 0.08 * value);
    modified_linear * MAXIMUM_RESONANCE_FEEDBACK
}

fn soft_clip(value: f32, drive: f32) -> f32 {
    libm::tanhf(value * drive) / drive
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_pole_candidate_is_finite_across_supported_rates() {
        for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
            let mut filter = Cem3320Filter::default();
            for index in 0..32_768 {
                let input = libm::sinf(index as f32 * 0.173) * 4.0;
                let cutoff = 10.0 + (index % 20_000) as f32;
                let resonance = (index % 128) as f32 / 127.0;
                assert!(
                    filter
                        .next(input, cutoff, resonance, sample_rate)
                        .is_finite()
                );
            }
        }
    }

    #[test]
    fn low_cutoff_rejects_more_high_frequency_energy() {
        fn render(cutoff: f32) -> f32 {
            let mut filter = Cem3320Filter::default();
            let mut energy = 0.0;
            for index in 0..16_384 {
                let input =
                    libm::sinf(2.0 * core::f32::consts::PI * 6_000.0 * index as f32 / 48_000.0);
                let output = filter.next(input, cutoff, 0.0, 48_000.0);
                if index > 4_096 {
                    energy += output * output;
                }
            }
            energy
        }
        assert!(render(8_000.0) > render(500.0) * 100.0);
    }

    #[test]
    fn resonance_extends_the_impulse_decay() {
        fn tail_energy(resonance: f32) -> f32 {
            let mut filter = Cem3320Filter::default();
            let mut energy = 0.0;
            for index in 0..24_000 {
                let input = if index == 0 { 0.1 } else { 0.0 };
                let output = filter.next(input, 1_000.0, resonance, 48_000.0);
                if index > 1_000 {
                    energy += output * output;
                }
            }
            energy
        }
        assert!(tail_energy(0.9) > tail_energy(0.0) * 100.0);
    }

    #[test]
    fn zero_input_and_zero_resonance_decay_to_rest() {
        let mut filter = Cem3320Filter::default();
        let _ = filter.next(1.0, 4_000.0, 0.0, 48_000.0);
        for _ in 0..32_768 {
            let _ = filter.next(0.0, 4_000.0, 0.0, 48_000.0);
        }
        assert!(filter.energy() < 2.0e-6);
    }

    #[test]
    fn self_oscillation_begins_inside_the_service_manual_window() {
        fn late_peak(resonance: f32) -> f32 {
            let mut filter = Cem3320Filter::default();
            let mut peak = 0.0_f32;
            for index in 0..192_000 {
                let output = filter.next(0.0, 1_000.0, resonance, 48_000.0);
                if index > 144_000 {
                    peak = peak.max(output.abs());
                }
            }
            peak
        }

        let below_window = late_peak(0.65);
        let inside_window = late_peak(0.95);
        assert!(below_window < 0.001, "oscillated too early: {below_window}");
        assert!(
            inside_window > 0.01,
            "resonance did not sustain: {inside_window}"
        );
    }
}
