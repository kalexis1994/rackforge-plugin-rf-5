#[cfg(all(not(feature = "host-rate"), feature = "two-times"))]
use crate::decimator::Decimator2x;
use crate::{
    decimator::Decimator4x,
    vco::{
        Vco, WaveSelection, band_limited_pulse, band_limited_triangle, naive_triangle,
        triangle_load_frequency_ratio,
    },
};
use core::f32::consts::PI;

const FFT_SIZE: usize = 4_096;
const WARMUP_SAMPLES: usize = 1_024;

#[cfg(feature = "host-rate")]
const ACTIVE_OVERSAMPLING: usize = 1;
#[cfg(all(not(feature = "host-rate"), feature = "two-times"))]
const ACTIVE_OVERSAMPLING: usize = 2;
#[cfg(not(any(feature = "host-rate", feature = "two-times")))]
const ACTIVE_OVERSAMPLING: usize = 4;

#[derive(Default)]
struct ActiveDecimator {
    #[cfg(all(not(feature = "host-rate"), feature = "two-times"))]
    two_times: Decimator2x,
    #[cfg(not(any(feature = "host-rate", feature = "two-times")))]
    four_times: Decimator4x,
}

impl ActiveDecimator {
    fn push(&mut self, sample: f32) -> Option<f32> {
        #[cfg(feature = "host-rate")]
        {
            Some(sample)
        }
        #[cfg(all(not(feature = "host-rate"), feature = "two-times"))]
        {
            self.two_times.push(sample)
        }
        #[cfg(not(any(feature = "host-rate", feature = "two-times")))]
        {
            self.four_times.push(sample)
        }
    }
}

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
    if !waves.pulse {
        return four_times_alias_ratio(sample_rate, fundamental_bin, waves, pulse_width);
    }
    let internal_rate = sample_rate * ACTIVE_OVERSAMPLING as f32;
    let target_frequency = sample_rate * fundamental_bin as f32 / FFT_SIZE as f32;
    // Keep the probe exactly periodic after the real triangle-output load
    // pull; otherwise FFT leakage would be misclassified as alias energy.
    let frequency = if waves.triangle {
        target_frequency / triangle_load_frequency_ratio(0)
    } else {
        target_frequency
    };
    let mut oscillator = Vco::with_phase(0.137);
    let mut decimator = ActiveDecimator::default();
    let mut spectrum = [Complex::default(); FFT_SIZE];

    for host_index in 0..WARMUP_SAMPLES + FFT_SIZE {
        let mut output = 0.0;
        for _ in 0..ACTIVE_OVERSAMPLING {
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

fn four_times_alias_ratio(
    sample_rate: f32,
    fundamental_bin: usize,
    waves: WaveSelection,
    pulse_width: f32,
) -> f32 {
    let internal_rate = sample_rate * 4.0;
    let target_frequency = sample_rate * fundamental_bin as f32 / FFT_SIZE as f32;
    let frequency = if waves.triangle {
        target_frequency / triangle_load_frequency_ratio(0)
    } else {
        target_frequency
    };
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
                master_frequency * 2.0,
                internal_rate,
                0.5,
                saw,
                true,
                master.hard_sync_event,
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

fn direct_triangle_alias_ratio(
    sample_rate: f32,
    fundamental_bin: usize,
    symmetry: f32,
    corrected: bool,
) -> f32 {
    let internal_rate = sample_rate * 4.0;
    let frequency = sample_rate * fundamental_bin as f32 / FFT_SIZE as f32;
    let increment = frequency / internal_rate;
    let mut decimator = Decimator4x::default();
    let mut spectrum = [Complex::default(); FFT_SIZE];

    for host_index in 0..WARMUP_SAMPLES + FFT_SIZE {
        let mut output = 0.0;
        for sub_sample in 0..4 {
            let internal_index = host_index * 4 + sub_sample;
            let phase = (0.137_f64
                + internal_index as f64 * f64::from(frequency) / f64::from(internal_rate))
            .fract() as f32;
            let triangle = if corrected {
                band_limited_triangle(phase, increment, symmetry)
            } else {
                naive_triangle(phase, symmetry)
            };
            if let Some(sample) = decimator.push(triangle) {
                output = sample;
            }
        }
        if host_index >= WARMUP_SAMPLES {
            spectrum[host_index - WARMUP_SAMPLES].real = output;
        }
    }

    spectral_alias_ratio(spectrum, fundamental_bin)
}

fn audio_rate_pwm_alias_ratio(sample_rate: f32, depth: f32) -> f32 {
    const MODULATOR_BIN: usize = 37;
    const CARRIER_BIN: usize = MODULATOR_BIN * 5;
    let internal_rate = sample_rate * ACTIVE_OVERSAMPLING as f32;
    let carrier_frequency = sample_rate * CARRIER_BIN as f32 / FFT_SIZE as f32;
    let modulator_frequency = sample_rate * MODULATOR_BIN as f32 / FFT_SIZE as f32;
    let carrier_increment = carrier_frequency / internal_rate;
    let mut decimator = ActiveDecimator::default();
    let mut spectrum = [Complex::default(); FFT_SIZE];
    let mut previous_width = 0.5;

    for host_index in 0..WARMUP_SAMPLES + FFT_SIZE {
        let mut output = 0.0;
        for sub_sample in 0..ACTIVE_OVERSAMPLING {
            let internal_index = host_index * ACTIVE_OVERSAMPLING + sub_sample;
            let elapsed = internal_index as f64 / f64::from(internal_rate);
            let carrier_phase = (0.137_f64 + elapsed * f64::from(carrier_frequency)).fract() as f32;
            let modulator_phase =
                (0.319_f64 + elapsed * f64::from(modulator_frequency)).fract() as f32;
            let width = 0.5 + depth * libm::sinf(2.0 * PI * modulator_phase);
            let pulse = band_limited_pulse(carrier_phase, carrier_increment, previous_width, width);
            previous_width = width;
            if let Some(sample) = decimator.push(pulse) {
                output = sample;
            }
        }
        if host_index >= WARMUP_SAMPLES {
            spectrum[host_index - WARMUP_SAMPLES].real = output;
        }
    }

    spectral_alias_ratio(spectrum, MODULATOR_BIN)
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
                ("harpsichord-a pulse", pulse, 0.056_3),
                ("harpsichord-b pulse", pulse, 0.920_5),
                ("five-percent pulse", pulse, 0.05),
                ("ninety-five-percent pulse", pulse, 0.95),
                ("narrow pulse", pulse, 0.01),
                ("wide pulse", pulse, 0.99),
                ("triangle", triangle, 0.5),
            ] {
                let ratio = alias_ratio(sample_rate, fundamental_bin, waves, pulse_width);
                assert!(
                    ratio < 0.01,
                    "{name} alias ratio {ratio} exceeded its bound at bin {fundamental_bin} and {sample_rate} Hz"
                );
            }
        }
    }
}

#[test]
fn triangle_blamp_reduces_non_harmonic_energy_across_the_accepted_matrix() {
    for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
        for fundamental_bin in [37, 173, 509, 997] {
            for symmetry in [0.45, 0.50, 0.55] {
                let naive =
                    direct_triangle_alias_ratio(sample_rate, fundamental_bin, symmetry, false);
                let corrected =
                    direct_triangle_alias_ratio(sample_rate, fundamental_bin, symmetry, true);
                assert!(
                    corrected < naive && corrected < 0.01,
                    "corrected={corrected}, naive={naive}, symmetry={symmetry}, bin={fundamental_bin}, rate={sample_rate}"
                );
            }
        }
    }
}

#[test]
fn audio_rate_pwm_alias_floor_is_bounded() {
    for sample_rate in [44_100.0, 48_000.0, 96_000.0, 192_000.0] {
        for depth in [0.20, 0.45] {
            let ratio = audio_rate_pwm_alias_ratio(sample_rate, depth);
            assert!(
                ratio < 0.01,
                "audio-rate PWM alias ratio={ratio}, depth={depth}, rate={sample_rate}"
            );
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
