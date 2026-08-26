use crate::{
    decimator::Decimator4x,
    vco::{Vco, WaveSelection},
};
use core::f32::consts::PI;

const FFT_SIZE: usize = 4_096;
const WARMUP_SAMPLES: usize = 1_024;

#[derive(Clone, Copy, Debug, Default)]
struct Complex {
    real: f32,
    imaginary: f32,
}

impl Complex {
    fn magnitude_squared(self) -> f32 {
        self.real * self.real + self.imaginary * self.imaginary
    }

    fn multiply(self, other: Self) -> Self {
        Self {
            real: self.real * other.real - self.imaginary * other.imaginary,
            imaginary: self.real * other.imaginary + self.imaginary * other.real,
        }
    }
}

fn fft(samples: &mut [Complex; FFT_SIZE]) {
    let mut reversed = 0;
    for index in 1..FFT_SIZE {
        let mut bit = FFT_SIZE >> 1;
        while reversed & bit != 0 {
            reversed ^= bit;
            bit >>= 1;
        }
        reversed ^= bit;
        if index < reversed {
            samples.swap(index, reversed);
        }
    }

    let mut span = 2;
    while span <= FFT_SIZE {
        let angle = -2.0 * PI / span as f32;
        let rotation = Complex {
            real: libm::cosf(angle),
            imaginary: libm::sinf(angle),
        };
        for base in (0..FFT_SIZE).step_by(span) {
            let mut twiddle = Complex {
                real: 1.0,
                imaginary: 0.0,
            };
            for offset in 0..span / 2 {
                let even = samples[base + offset];
                let odd = samples[base + offset + span / 2].multiply(twiddle);
                samples[base + offset] = Complex {
                    real: even.real + odd.real,
                    imaginary: even.imaginary + odd.imaginary,
                };
                samples[base + offset + span / 2] = Complex {
                    real: even.real - odd.real,
                    imaginary: even.imaginary - odd.imaginary,
                };
                twiddle = twiddle.multiply(rotation);
            }
        }
        span *= 2;
    }
}

fn alias_ratio(
    sample_rate: f32,
    fundamental_bin: usize,
    waves: WaveSelection,
    pulse_width: f32,
) -> f32 {
    let internal_rate = sample_rate * 4.0;
    let frequency = sample_rate * fundamental_bin as f32 / FFT_SIZE as f32;
    let mut oscillator = Vco::with_phase(0.137);
    let mut decimator = Decimator4x::default();
    let mut spectrum = [Complex::default(); FFT_SIZE];

    for host_index in 0..WARMUP_SAMPLES + FFT_SIZE {
        let mut output = 0.0;
        for _ in 0..4 {
            let internal = oscillator.next(frequency, internal_rate, pulse_width, waves);
            if let Some(sample) = decimator.push(internal.mixer_differential_source_volts()) {
                output = sample;
            }
        }
        if host_index >= WARMUP_SAMPLES {
            spectrum[host_index - WARMUP_SAMPLES].real = output;
        }
    }

    spectral_alias_ratio(spectrum, fundamental_bin)
}

fn hard_sync_alias_ratio(sample_rate: f32, fundamental_bin: usize) -> f32 {
    let internal_rate = sample_rate * 4.0;
    let master_frequency = sample_rate * fundamental_bin as f32 / FFT_SIZE as f32;
    let mut oscillator_a = Vco::with_phase_and_profile(0.137, 0);
    let mut oscillator_b = Vco::with_phase_and_profile(0.613, 1);
    let mut decimator = Decimator4x::default();
    let mut spectrum = [Complex::default(); FFT_SIZE];
    let saw = WaveSelection {
        saw: true,
        triangle: false,
        pulse: false,
    };

    for host_index in 0..WARMUP_SAMPLES + FFT_SIZE {
        let mut output = 0.0;
        for _ in 0..4 {
            let master = oscillator_b.next(
                master_frequency,
                internal_rate,
                0.37,
                WaveSelection::default(),
            );
            let slave = oscillator_a.next_with_sync(
                master_frequency * 3.0,
                internal_rate,
                0.5,
                saw,
                master.sync_events,
            );
            if let Some(sample) = decimator.push(slave.mixer_differential_source_volts()) {
                output = sample;
            }
        }
        if host_index >= WARMUP_SAMPLES {
            spectrum[host_index - WARMUP_SAMPLES].real = output;
        }
    }

    spectral_alias_ratio(spectrum, fundamental_bin)
}

fn spectral_alias_ratio(mut spectrum: [Complex; FFT_SIZE], fundamental_bin: usize) -> f32 {
    let mean = spectrum.iter().map(|sample| sample.real).sum::<f32>() / FFT_SIZE as f32;
    for sample in &mut spectrum {
        sample.real -= mean;
    }
    fft(&mut spectrum);

    let mut harmonic = [false; FFT_SIZE / 2 + 1];
    for bin in (fundamental_bin..FFT_SIZE / 2).step_by(fundamental_bin) {
        harmonic[bin] = true;
    }
    let mut total_power = 0.0;
    let mut alias_power = 0.0;
    for bin in 1..FFT_SIZE / 2 {
        let power = spectrum[bin].magnitude_squared();
        total_power += power;
        if !harmonic[bin] {
            alias_power += power;
        }
    }
    libm::sqrtf(alias_power / total_power.max(f32::MIN_POSITIVE))
}

#[test]
fn oscillator_alias_floor_is_bounded_at_all_supported_rates() {
    let saw = WaveSelection {
        saw: true,
        triangle: false,
        pulse: false,
    };
    let pulse = WaveSelection {
        saw: false,
        triangle: false,
        pulse: true,
    };
    let triangle = WaveSelection {
        saw: false,
        triangle: true,
        pulse: false,
    };

    for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
        for fundamental_bin in [37, 173, 509, 997] {
            for (name, waves, pulse_width) in [
                ("saw", saw, 0.5),
                ("square", pulse, 0.5),
                ("narrow pulse", pulse, 0.01),
                ("wide pulse", pulse, 0.99),
                ("triangle", triangle, 0.5),
            ] {
                let ratio = alias_ratio(sample_rate, fundamental_bin, waves, pulse_width);
                assert!(
                    ratio < 0.01,
                    "{name} alias ratio {ratio} exceeded -40 dB at bin {fundamental_bin} and {sample_rate} Hz"
                );
            }
        }
    }
}

#[test]
fn fractional_hard_sync_alias_floor_is_bounded_at_all_supported_rates() {
    for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
        for fundamental_bin in [37, 173, 509] {
            let ratio = hard_sync_alias_ratio(sample_rate, fundamental_bin);
            assert!(
                ratio < 0.01,
                "hard-sync alias ratio {ratio} exceeded -40 dB at bin {fundamental_bin} and {sample_rate} Hz"
            );
        }
    }
}
