//! Compare two RF-5 audition suites under fixed perceptual acceptance limits.

use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use rustfft::{FftPlanner, num_complex::Complex32};
use serde::Serialize;

const FFT_SIZE: usize = 4096;
const FFT_HOP: usize = FFT_SIZE / 2;
const MAX_INTEGER_LAG: i32 = 32;
const ANALYSIS_START_SECONDS: f32 = 0.15;
const ANALYSIS_END_MARGIN_SECONDS: f32 = 0.10;
const MIN_LEVEL_DB: f32 = -120.0;

const MEAN_LEVEL_DELTA_LIMIT_DB: f32 = 0.50;
const MEAN_BARK_DELTA_LIMIT_DB: f32 = 1.00;
const AGGREGATE_ERROR_LIMIT_DB: f32 = -30.0;
const AGGREGATE_HF_EXCESS_LIMIT_DB: f32 = -50.0;

const OUTLIER_LEVEL_DELTA_DB: f32 = 1.00;
const OUTLIER_BARK_DELTA_DB: f32 = 2.00;
const OUTLIER_ERROR_DB: f32 = -20.0;
const OUTLIER_HF_EXCESS_DB: f32 = -35.0;

// Approximate critical-band edges. Equal weighting across active bands makes
// narrow high-frequency differences visible instead of letting bass energy
// dominate a single full-band RMS number.
const CRITICAL_BAND_EDGES_HZ: [f32; 27] = [
    20.0, 100.0, 200.0, 300.0, 400.0, 510.0, 630.0, 770.0, 920.0, 1_080.0, 1_270.0, 1_480.0,
    1_720.0, 2_000.0, 2_320.0, 2_700.0, 3_150.0, 3_700.0, 4_400.0, 5_300.0, 6_400.0, 7_700.0,
    9_500.0, 12_000.0, 15_500.0, 20_000.0, 24_000.0,
];

#[derive(Debug)]
struct Wav {
    sample_rate: u32,
    samples: Vec<f32>,
}

#[derive(Clone, Debug, Serialize)]
struct SceneComparison {
    id: String,
    integer_lag_samples: i32,
    fractional_lag_samples: f32,
    correlation: f32,
    level_delta_db: f32,
    aligned_error_db: f32,
    gain_matched_error_db: f32,
    critical_band_rms_delta_db: f32,
    critical_band_max_delta_db: f32,
    high_band_excess_db: f32,
    outlier: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct AcceptanceThresholds {
    mean_absolute_level_delta_db: f32,
    mean_critical_band_rms_delta_db: f32,
    aggregate_aligned_error_db: f32,
    aggregate_high_band_excess_db: f32,
}

#[derive(Debug, Serialize)]
struct SuiteSummary {
    schema_version: u32,
    candidate_profile: String,
    reference_profile: String,
    scene_count: usize,
    accepted: bool,
    thresholds: AcceptanceThresholds,
    mean_absolute_level_delta_db: f32,
    mean_critical_band_rms_delta_db: f32,
    aggregate_aligned_error_db: f32,
    aggregate_high_band_excess_db: f32,
    outlier_count: usize,
    scenes: Vec<SceneComparison>,
}

fn main() -> ExitCode {
    match run() {
        Ok(summary) => {
            println!(
                concat!(
                    "RF5_PROFILE_COMPARISON accepted={} scenes={} level_db={:.3} ",
                    "bark_db={:.3} error_db={:.3} hf_excess_db={:.3} outliers={}"
                ),
                summary.accepted,
                summary.scene_count,
                summary.mean_absolute_level_delta_db,
                summary.mean_critical_band_rms_delta_db,
                summary.aggregate_aligned_error_db,
                summary.aggregate_high_band_excess_db,
                summary.outlier_count
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("RF5_PROFILE_COMPARISON_ERROR {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<SuiteSummary> {
    let mut arguments = env::args_os().skip(1);
    let portable_directory = required_path(arguments.next(), "portable render directory")?;
    let reference_directory = required_path(arguments.next(), "reference render directory")?;
    let output_directory = required_path(arguments.next(), "comparison output directory")?;
    let candidate_profile = arguments.next().map_or_else(
        || "portable-realtime wasm-v1 DSP".to_owned(),
        |value| value.to_string_lossy().into_owned(),
    );
    let reference_profile = arguments.next().map_or_else(
        || "four-times nonlinear voice path with precise math and 127-tap decimation".to_owned(),
        |value| value.to_string_lossy().into_owned(),
    );
    if arguments.next().is_some() {
        return Err(io::Error::other(
            "usage: rf-5-compare CANDIDATE_DIR REFERENCE_DIR OUTPUT_DIR [CANDIDATE_LABEL] [REFERENCE_LABEL]",
        ));
    }
    fs::create_dir_all(&output_directory)?;

    let mut portable_paths = wav_paths(&portable_directory)?;
    portable_paths.sort();
    if portable_paths.is_empty() {
        return Err(io::Error::other(
            "portable render directory has no WAV files",
        ));
    }

    let mut scenes = Vec::with_capacity(portable_paths.len());
    for portable_path in portable_paths {
        let file_name = portable_path
            .file_name()
            .ok_or_else(|| io::Error::other("portable WAV has no file name"))?;
        let reference_path = reference_directory.join(file_name);
        if !reference_path.is_file() {
            return Err(io::Error::other(format!(
                "reference render is missing {}",
                reference_path.display()
            )));
        }
        let id = portable_path
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| io::Error::other("WAV file name is not valid UTF-8"))?;
        scenes.push(compare_scene(
            id,
            &read_pcm16_mono(&portable_path)?,
            &read_pcm16_mono(&reference_path)?,
        )?);
    }

    let reference_count = wav_paths(&reference_directory)?.len();
    if reference_count != scenes.len() {
        return Err(io::Error::other(format!(
            "render suites differ in size: portable={} reference={reference_count}",
            scenes.len()
        )));
    }

    let count = scenes.len() as f64;
    let mean_absolute_level_delta_db = (scenes
        .iter()
        .map(|scene| f64::from(scene.level_delta_db.abs()))
        .sum::<f64>()
        / count) as f32;
    let mean_critical_band_rms_delta_db = (scenes
        .iter()
        .map(|scene| f64::from(scene.critical_band_rms_delta_db))
        .sum::<f64>()
        / count) as f32;
    let aggregate_aligned_error_db = ratio_mean_db(
        scenes
            .iter()
            .map(|scene| db_to_power_ratio(scene.aligned_error_db)),
    );
    let aggregate_high_band_excess_db = ratio_mean_db(
        scenes
            .iter()
            .map(|scene| db_to_power_ratio(scene.high_band_excess_db)),
    );
    let thresholds = AcceptanceThresholds {
        mean_absolute_level_delta_db: MEAN_LEVEL_DELTA_LIMIT_DB,
        mean_critical_band_rms_delta_db: MEAN_BARK_DELTA_LIMIT_DB,
        aggregate_aligned_error_db: AGGREGATE_ERROR_LIMIT_DB,
        aggregate_high_band_excess_db: AGGREGATE_HF_EXCESS_LIMIT_DB,
    };
    let accepted = mean_absolute_level_delta_db <= MEAN_LEVEL_DELTA_LIMIT_DB
        && mean_critical_band_rms_delta_db <= MEAN_BARK_DELTA_LIMIT_DB
        && aggregate_aligned_error_db <= AGGREGATE_ERROR_LIMIT_DB
        && aggregate_high_band_excess_db <= AGGREGATE_HF_EXCESS_LIMIT_DB;
    let outlier_count = scenes.iter().filter(|scene| scene.outlier).count();
    let summary = SuiteSummary {
        schema_version: 1,
        candidate_profile,
        reference_profile,
        scene_count: scenes.len(),
        accepted,
        thresholds,
        mean_absolute_level_delta_db,
        mean_critical_band_rms_delta_db,
        aggregate_aligned_error_db,
        aggregate_high_band_excess_db,
        outlier_count,
        scenes,
    };
    write_outputs(&output_directory, &summary)?;
    Ok(summary)
}

fn required_path(value: Option<std::ffi::OsString>, name: &str) -> io::Result<PathBuf> {
    value.map(PathBuf::from).ok_or_else(|| {
        io::Error::other(format!(
            "missing {name}; usage: rf-5-compare CANDIDATE_DIR REFERENCE_DIR OUTPUT_DIR [CANDIDATE_LABEL] [REFERENCE_LABEL]"
        ))
    })
}

fn wav_paths(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("wav"))
        {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn compare_scene(id: &str, portable: &Wav, reference: &Wav) -> io::Result<SceneComparison> {
    if portable.sample_rate != reference.sample_rate {
        return Err(io::Error::other(format!(
            "scene {id} sample-rate mismatch: portable={} reference={}",
            portable.sample_rate, reference.sample_rate
        )));
    }
    if portable.samples.len() != reference.samples.len() {
        return Err(io::Error::other(format!(
            "scene {id} frame-count mismatch: portable={} reference={}",
            portable.samples.len(),
            reference.samples.len()
        )));
    }
    let start = (portable.sample_rate as f32 * ANALYSIS_START_SECONDS) as usize;
    let end_margin = (portable.sample_rate as f32 * ANALYSIS_END_MARGIN_SECONDS) as usize;
    let end = portable.samples.len().saturating_sub(end_margin);
    if end <= start + FFT_SIZE {
        return Err(io::Error::other(format!(
            "scene {id} is too short for comparison"
        )));
    }

    let integer_lag = best_integer_lag(&portable.samples, &reference.samples, start, end);
    let fractional_lag = best_fractional_lag(
        &portable.samples,
        &reference.samples,
        start,
        end,
        integer_lag,
    );
    let (portable_aligned, reference_aligned) = aligned_samples(
        &portable.samples,
        &reference.samples,
        start,
        end,
        fractional_lag,
    );
    let temporal = temporal_metrics(&portable_aligned, &reference_aligned);
    let spectral = spectral_metrics(&portable_aligned, &reference_aligned, portable.sample_rate);
    let outlier = temporal.level_delta_db.abs() > OUTLIER_LEVEL_DELTA_DB
        || spectral.critical_band_rms_delta_db > OUTLIER_BARK_DELTA_DB
        || temporal.aligned_error_db > OUTLIER_ERROR_DB
        || spectral.high_band_excess_db > OUTLIER_HF_EXCESS_DB;

    Ok(SceneComparison {
        id: id.to_owned(),
        integer_lag_samples: integer_lag,
        fractional_lag_samples: fractional_lag,
        correlation: temporal.correlation,
        level_delta_db: temporal.level_delta_db,
        aligned_error_db: temporal.aligned_error_db,
        gain_matched_error_db: temporal.gain_matched_error_db,
        critical_band_rms_delta_db: spectral.critical_band_rms_delta_db,
        critical_band_max_delta_db: spectral.critical_band_max_delta_db,
        high_band_excess_db: spectral.high_band_excess_db,
        outlier,
    })
}

fn best_integer_lag(portable: &[f32], reference: &[f32], start: usize, end: usize) -> i32 {
    let mut best_lag = 0_i32;
    let mut best_score = f32::NEG_INFINITY;
    for lag in -MAX_INTEGER_LAG..=MAX_INTEGER_LAG {
        let score = correlation_at_lag(portable, reference, start, end, lag as f32, 8);
        if score > best_score + 1.0e-7
            || ((score - best_score).abs() <= 1.0e-7 && lag.abs() < best_lag.abs())
        {
            best_lag = lag;
            best_score = score;
        }
    }
    best_lag
}

fn best_fractional_lag(
    portable: &[f32],
    reference: &[f32],
    start: usize,
    end: usize,
    integer_lag: i32,
) -> f32 {
    let mut best_lag = integer_lag as f32;
    let mut best_offset = 0.0_f32;
    let mut best_score = correlation_at_lag(portable, reference, start, end, best_lag, 4);
    for step in -20_i32..=20 {
        let offset = step as f32 * 0.05;
        let lag = integer_lag as f32 + offset;
        let score = correlation_at_lag(portable, reference, start, end, lag, 4);
        if score > best_score + 1.0e-7
            || ((score - best_score).abs() <= 1.0e-7 && offset.abs() < best_offset.abs())
        {
            best_lag = lag;
            best_offset = offset;
            best_score = score;
        }
    }
    best_lag
}

fn correlation_at_lag(
    portable: &[f32],
    reference: &[f32],
    start: usize,
    end: usize,
    lag: f32,
    stride: usize,
) -> f32 {
    let mut dot = 0.0_f64;
    let mut portable_energy = 0.0_f64;
    let mut reference_energy = 0.0_f64;
    for index in (start..end).step_by(stride) {
        let reference_index = index as f32 + lag;
        if reference_index < 0.0 || reference_index + 1.0 >= reference.len() as f32 {
            continue;
        }
        let left = f64::from(portable[index]);
        let right = f64::from(interpolate(reference, reference_index));
        dot += left * right;
        portable_energy += left * left;
        reference_energy += right * right;
    }
    if portable_energy <= f64::EPSILON || reference_energy <= f64::EPSILON {
        0.0
    } else {
        (dot / (portable_energy * reference_energy).sqrt()) as f32
    }
}

fn aligned_samples(
    portable: &[f32],
    reference: &[f32],
    start: usize,
    end: usize,
    lag: f32,
) -> (Vec<f32>, Vec<f32>) {
    let mut portable_aligned = Vec::with_capacity(end - start);
    let mut reference_aligned = Vec::with_capacity(end - start);
    for (index, &portable_sample) in portable.iter().enumerate().take(end).skip(start) {
        let reference_index = index as f32 + lag;
        if reference_index < 0.0 || reference_index + 1.0 >= reference.len() as f32 {
            continue;
        }
        portable_aligned.push(portable_sample);
        reference_aligned.push(interpolate(reference, reference_index));
    }
    (portable_aligned, reference_aligned)
}

fn interpolate(samples: &[f32], index: f32) -> f32 {
    let lower = index.floor() as usize;
    let fraction = index - lower as f32;
    samples[lower] + (samples[lower + 1] - samples[lower]) * fraction
}

#[derive(Clone, Copy)]
struct TemporalMetrics {
    correlation: f32,
    level_delta_db: f32,
    aligned_error_db: f32,
    gain_matched_error_db: f32,
}

fn temporal_metrics(portable: &[f32], reference: &[f32]) -> TemporalMetrics {
    let mut portable_energy = 0.0_f64;
    let mut reference_energy = 0.0_f64;
    let mut error_energy = 0.0_f64;
    let mut dot = 0.0_f64;
    for (&left, &right) in portable.iter().zip(reference) {
        let left = f64::from(left);
        let right = f64::from(right);
        portable_energy += left * left;
        reference_energy += right * right;
        error_energy += (left - right) * (left - right);
        dot += left * right;
    }
    let gain = if portable_energy <= f64::EPSILON {
        1.0
    } else {
        dot / portable_energy
    };
    let gain_matched_error = portable
        .iter()
        .zip(reference)
        .map(|(&left, &right)| {
            let difference = f64::from(left) * gain - f64::from(right);
            difference * difference
        })
        .sum::<f64>();
    TemporalMetrics {
        correlation: safe_ratio(dot, (portable_energy * reference_energy).sqrt()) as f32,
        level_delta_db: power_db(safe_ratio(portable_energy, reference_energy)),
        aligned_error_db: power_db(safe_ratio(error_energy, reference_energy)),
        gain_matched_error_db: power_db(safe_ratio(gain_matched_error, reference_energy)),
    }
}

#[derive(Clone, Copy)]
struct SpectralMetrics {
    critical_band_rms_delta_db: f32,
    critical_band_max_delta_db: f32,
    high_band_excess_db: f32,
}

fn spectral_metrics(portable: &[f32], reference: &[f32], sample_rate: u32) -> SpectralMetrics {
    let portable_power = average_power_spectrum(portable);
    let reference_power = average_power_spectrum(reference);
    let bin_hz = sample_rate as f64 / FFT_SIZE as f64;
    let mut band_deltas = Vec::new();
    let mut reference_bands = Vec::new();
    let mut portable_bands = Vec::new();
    for edges in CRITICAL_BAND_EDGES_HZ.windows(2) {
        let start = ((f64::from(edges[0]) / bin_hz).ceil() as usize)
            .min(reference_power.len().saturating_sub(1));
        let end = ((f64::from(edges[1]) / bin_hz).ceil() as usize).min(reference_power.len());
        let reference_band = reference_power[start..end].iter().sum::<f64>();
        let portable_band = portable_power[start..end].iter().sum::<f64>();
        reference_bands.push(reference_band);
        portable_bands.push(portable_band);
    }
    let maximum_reference_band = reference_bands.iter().copied().fold(0.0_f64, f64::max);
    for (&portable_band, &reference_band) in portable_bands.iter().zip(&reference_bands) {
        if reference_band >= maximum_reference_band * 1.0e-6 {
            band_deltas.push(power_db(safe_ratio(portable_band, reference_band)));
        }
    }
    let critical_band_rms_delta_db = if band_deltas.is_empty() {
        0.0
    } else {
        (band_deltas
            .iter()
            .map(|value| f64::from(*value) * f64::from(*value))
            .sum::<f64>()
            / band_deltas.len() as f64)
            .sqrt() as f32
    };
    let critical_band_max_delta_db = band_deltas
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f32, f32::max);

    let high_start = (12_000.0 / bin_hz).ceil() as usize;
    let high_end = (20_000.0 / bin_hz).ceil() as usize;
    let portable_high = portable_power[high_start..high_end].iter().sum::<f64>();
    let reference_high = reference_power[high_start..high_end].iter().sum::<f64>();
    let reference_total = reference_power.iter().sum::<f64>();
    let high_excess = (portable_high - reference_high).max(0.0);
    SpectralMetrics {
        critical_band_rms_delta_db,
        critical_band_max_delta_db,
        high_band_excess_db: power_db(safe_ratio(high_excess, reference_total)),
    }
}

fn average_power_spectrum(samples: &[f32]) -> Vec<f64> {
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);
    let mut buffer = vec![Complex32::default(); FFT_SIZE];
    let mut power = vec![0.0_f64; FFT_SIZE / 2 + 1];
    let mut windows = 0_u64;
    for offset in (0..=samples.len().saturating_sub(FFT_SIZE)).step_by(FFT_HOP) {
        for (index, value) in buffer.iter_mut().enumerate() {
            let phase = 2.0 * core::f32::consts::PI * index as f32 / (FFT_SIZE - 1) as f32;
            let window = 0.5 - 0.5 * phase.cos();
            *value = Complex32::new(samples[offset + index] * window, 0.0);
        }
        fft.process(&mut buffer);
        for (destination, value) in power.iter_mut().zip(&buffer) {
            *destination += f64::from(value.norm_sqr());
        }
        windows += 1;
    }
    if windows != 0 {
        for value in &mut power {
            *value /= windows as f64;
        }
    }
    power
}

fn read_pcm16_mono(path: &Path) -> io::Result<Wav> {
    let bytes = fs::read(path)?;
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(io::Error::other(format!(
            "{} is not a RIFF/WAVE file",
            path.display()
        )));
    }
    let mut cursor = 12_usize;
    let mut format = None;
    let mut data = None;
    while cursor + 8 <= bytes.len() {
        let id = &bytes[cursor..cursor + 4];
        let length = u32::from_le_bytes(bytes[cursor + 4..cursor + 8].try_into().unwrap()) as usize;
        let start = cursor + 8;
        let end = start
            .checked_add(length)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| io::Error::other("WAV chunk exceeds file length"))?;
        if id == b"fmt " {
            if length < 16 {
                return Err(io::Error::other("WAV fmt chunk is too short"));
            }
            format = Some((
                u16::from_le_bytes(bytes[start..start + 2].try_into().unwrap()),
                u16::from_le_bytes(bytes[start + 2..start + 4].try_into().unwrap()),
                u32::from_le_bytes(bytes[start + 4..start + 8].try_into().unwrap()),
                u16::from_le_bytes(bytes[start + 14..start + 16].try_into().unwrap()),
            ));
        } else if id == b"data" {
            data = Some(&bytes[start..end]);
        }
        cursor = end + (length & 1);
    }
    let (encoding, channels, sample_rate, bits) =
        format.ok_or_else(|| io::Error::other("WAV has no fmt chunk"))?;
    if encoding != 1 || channels != 1 || bits != 16 {
        return Err(io::Error::other(format!(
            "{} must be mono 16-bit PCM",
            path.display()
        )));
    }
    let data = data.ok_or_else(|| io::Error::other("WAV has no data chunk"))?;
    if data.len() % 2 != 0 {
        return Err(io::Error::other("WAV PCM payload is not sample-aligned"));
    }
    let (sample_bytes, remainder) = data.as_chunks::<2>();
    debug_assert!(remainder.is_empty());
    let samples = sample_bytes
        .iter()
        .map(|bytes| f32::from(i16::from_le_bytes(*bytes)) / f32::from(i16::MAX))
        .collect();
    Ok(Wav {
        sample_rate,
        samples,
    })
}

fn write_outputs(output_directory: &Path, summary: &SuiteSummary) -> io::Result<()> {
    let json = serde_json::to_vec_pretty(summary).map_err(io::Error::other)?;
    fs::write(output_directory.join("comparison.json"), json)?;

    let mut report = fs::File::create(output_directory.join("REPORT.md"))?;
    writeln!(report, "# RF-5 portable/reference comparison")?;
    writeln!(report)?;
    writeln!(report, "Candidate: `{}`.", summary.candidate_profile)?;
    writeln!(report, "Reference: `{}`.", summary.reference_profile)?;
    writeln!(report)?;
    writeln!(
        report,
        "Overall decision: **{}**.",
        if summary.accepted {
            "accepted"
        } else {
            "not accepted"
        }
    )?;
    writeln!(report)?;
    writeln!(
        report,
        "The decision uses suite averages fixed before this render:"
    )?;
    writeln!(report)?;
    writeln!(
        report,
        "- mean absolute level delta: {:.3} dB (limit {:.2} dB);",
        summary.mean_absolute_level_delta_db, summary.thresholds.mean_absolute_level_delta_db
    )?;
    writeln!(
        report,
        "- mean critical-band RMS delta: {:.3} dB (limit {:.2} dB);",
        summary.mean_critical_band_rms_delta_db, summary.thresholds.mean_critical_band_rms_delta_db
    )?;
    writeln!(
        report,
        "- aggregate aligned error: {:.3} dB relative to reference (limit {:.2} dB);",
        summary.aggregate_aligned_error_db, summary.thresholds.aggregate_aligned_error_db
    )?;
    writeln!(
        report,
        "- aggregate 12-20 kHz excess: {:.3} dB relative to reference (limit {:.2} dB).",
        summary.aggregate_high_band_excess_db, summary.thresholds.aggregate_high_band_excess_db
    )?;
    writeln!(report)?;
    writeln!(
        report,
        "Integer and fractional lag are estimated before temporal subtraction so the reference FIR delay is not counted as a timbral difference. Critical-band results use 4096-sample Hann-windowed spectra. These are engineering limits, not a substitute for a controlled ABX test."
    )?;
    writeln!(report)?;
    writeln!(
        report,
        "| Scene | Lag | Corr. | Level dB | Error dB | Gain-matched dB | Band RMS dB | Band max dB | HF excess dB | Outlier |"
    )?;
    writeln!(
        report,
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | :---: |"
    )?;
    for scene in &summary.scenes {
        writeln!(
            report,
            "| {} | {:.2} | {:.5} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {} |",
            scene.id,
            scene.fractional_lag_samples,
            scene.correlation,
            scene.level_delta_db,
            scene.aligned_error_db,
            scene.gain_matched_error_db,
            scene.critical_band_rms_delta_db,
            scene.critical_band_max_delta_db,
            scene.high_band_excess_db,
            if scene.outlier { "yes" } else { "no" }
        )?;
    }
    report.flush()
}

fn safe_ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator <= f64::EPSILON {
        0.0
    } else {
        numerator / denominator
    }
}

fn power_db(ratio: f64) -> f32 {
    if ratio <= 0.0 {
        MIN_LEVEL_DB
    } else {
        (10.0 * ratio.log10()).max(f64::from(MIN_LEVEL_DB)) as f32
    }
}

fn db_to_power_ratio(value: f32) -> f64 {
    10.0_f64.powf(f64::from(value) / 10.0)
}

fn ratio_mean_db(values: impl Iterator<Item = f64>) -> f32 {
    let values = values.collect::<Vec<_>>();
    power_db(values.iter().sum::<f64>() / values.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deterministic_signal(length: usize) -> Vec<f32> {
        let mut state = 0x1234_5678_u32;
        (0..length)
            .map(|index| {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let noise = (state >> 8) as f32 / 16_777_215.0 - 0.5;
                let tone = (index as f32 * 0.071).sin();
                tone * 0.75 + noise * 0.25
            })
            .collect()
    }

    #[test]
    fn identical_signal_prefers_zero_lag_and_zero_error() {
        let signal = deterministic_signal(16_384);
        let lag = best_integer_lag(&signal, &signal, 128, signal.len() - 128);
        let fractional = best_fractional_lag(&signal, &signal, 128, signal.len() - 128, lag);
        let (candidate, reference) =
            aligned_samples(&signal, &signal, 128, signal.len() - 128, fractional);
        let metrics = temporal_metrics(&candidate, &reference);
        assert_eq!(lag, 0);
        assert_eq!(fractional, 0.0);
        assert_eq!(metrics.aligned_error_db, MIN_LEVEL_DB);
        assert!((metrics.correlation - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn alignment_recovers_a_known_reference_delay() {
        let candidate = deterministic_signal(16_384);
        let mut reference = vec![0.0_f32; 16];
        reference.extend_from_slice(&candidate[..candidate.len() - 16]);
        let lag = best_integer_lag(&candidate, &reference, 128, candidate.len() - 128);
        assert_eq!(lag, 16);
    }

    #[test]
    fn identical_spectra_have_no_band_or_high_frequency_delta() {
        let signal = deterministic_signal(16_384);
        let metrics = spectral_metrics(&signal, &signal, 48_000);
        assert_eq!(metrics.critical_band_rms_delta_db, 0.0);
        assert_eq!(metrics.critical_band_max_delta_db, 0.0);
        assert_eq!(metrics.high_band_excess_db, MIN_LEVEL_DB);
    }
}
