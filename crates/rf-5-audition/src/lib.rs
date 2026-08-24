//! Deterministic listening scenes for evaluating RF-5 without a graphical UI.

use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};

use rf_5_dsp::Engine;

pub const SAMPLE_RATE: u32 = 48_000;
pub const SCENE_SECONDS: u32 = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MidiAction {
    frame: u32,
    data: [u8; 3],
}

#[derive(Clone, Debug)]
struct Scene {
    id: &'static str,
    program: &'static str,
    description: &'static str,
    events: Vec<MidiAction>,
}

#[derive(Clone, Debug)]
pub struct RenderMetrics {
    pub id: &'static str,
    pub program: &'static str,
    pub description: &'static str,
    pub frames: u32,
    pub peak: f32,
    pub rms: f32,
    pub dc: f32,
    pub clipped_samples: u32,
    pub path: PathBuf,
}

pub fn render_suite(output_directory: &Path) -> io::Result<Vec<RenderMetrics>> {
    fs::create_dir_all(output_directory)?;
    let scenes = scenes();
    let mut metrics = Vec::with_capacity(scenes.len());
    for scene in scenes {
        let scene_metrics = render_scene(output_directory, &scene)?;
        validate_metrics(&scene_metrics)?;
        metrics.push(scene_metrics);
    }
    write_manifest(output_directory, &metrics)?;
    Ok(metrics)
}

fn validate_metrics(metrics: &RenderMetrics) -> io::Result<()> {
    if metrics.peak <= 0.01 || metrics.rms <= 0.001 {
        return Err(io::Error::other(format!(
            "scene {} is not audibly above the acceptance floor",
            metrics.id
        )));
    }
    if metrics.peak >= 0.999 || metrics.clipped_samples != 0 {
        return Err(io::Error::other(format!(
            "scene {} exhausted host headroom",
            metrics.id
        )));
    }
    if metrics.dc.abs() >= 0.02 {
        return Err(io::Error::other(format!(
            "scene {} exceeds the bounded DC window",
            metrics.id
        )));
    }
    Ok(())
}

fn render_scene(output_directory: &Path, scene: &Scene) -> io::Result<RenderMetrics> {
    let mut engine = Engine::default();
    if !engine.prepare(f64::from(SAMPLE_RATE)) || !engine.load_program(scene.program) {
        return Err(io::Error::other(format!(
            "could not prepare scene {} with program {}",
            scene.id, scene.program
        )));
    }

    let frame_count = SAMPLE_RATE * SCENE_SECONDS;
    let mut samples = Vec::with_capacity(frame_count as usize);
    let mut event_index = 0;
    let mut peak = 0.0_f32;
    let mut energy = 0.0_f64;
    let mut sum = 0.0_f64;
    let mut clipped_samples = 0_u32;
    for frame in 0..frame_count {
        while let Some(event) = scene.events.get(event_index)
            && event.frame == frame
        {
            engine.handle_midi(event.data);
            event_index += 1;
        }
        let sample = engine.next_sample();
        if !sample.is_finite() {
            return Err(io::Error::other(format!(
                "non-finite output in scene {} at frame {frame}",
                scene.id
            )));
        }
        peak = peak.max(sample.abs());
        energy += f64::from(sample) * f64::from(sample);
        sum += f64::from(sample);
        if sample.abs() >= 0.999 {
            clipped_samples += 1;
        }
        samples.push(sample);
    }
    if event_index != scene.events.len() {
        return Err(io::Error::other(format!(
            "scene {} contains events beyond its render window",
            scene.id
        )));
    }

    let path = output_directory.join(format!("{}.wav", scene.id));
    write_pcm16_mono(&path, SAMPLE_RATE, &samples)?;
    Ok(RenderMetrics {
        id: scene.id,
        program: scene.program,
        description: scene.description,
        frames: frame_count,
        peak,
        rms: (energy / f64::from(frame_count)).sqrt() as f32,
        dc: (sum / f64::from(frame_count)) as f32,
        clipped_samples,
        path,
    })
}

fn write_pcm16_mono(path: &Path, sample_rate: u32, samples: &[f32]) -> io::Result<()> {
    let data_bytes = u32::try_from(samples.len())
        .ok()
        .and_then(|frames| frames.checked_mul(2))
        .ok_or_else(|| io::Error::other("WAV data exceeds RIFF size"))?;
    let mut writer = BufWriter::new(File::create(path)?);
    writer.write_all(b"RIFF")?;
    writer.write_all(&(36_u32 + data_bytes).to_le_bytes())?;
    writer.write_all(b"WAVEfmt ")?;
    writer.write_all(&16_u32.to_le_bytes())?;
    writer.write_all(&1_u16.to_le_bytes())?;
    writer.write_all(&1_u16.to_le_bytes())?;
    writer.write_all(&sample_rate.to_le_bytes())?;
    writer.write_all(&(sample_rate * 2).to_le_bytes())?;
    writer.write_all(&2_u16.to_le_bytes())?;
    writer.write_all(&16_u16.to_le_bytes())?;
    writer.write_all(b"data")?;
    writer.write_all(&data_bytes.to_le_bytes())?;
    for sample in samples {
        let pcm = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
        writer.write_all(&pcm.to_le_bytes())?;
    }
    writer.flush()
}

fn write_manifest(output_directory: &Path, metrics: &[RenderMetrics]) -> io::Result<()> {
    let mut writer = BufWriter::new(File::create(output_directory.join("manifest.json"))?);
    writeln!(writer, "{{")?;
    writeln!(writer, "  \"schema_version\": 1,")?;
    writeln!(writer, "  \"sample_rate\": {SAMPLE_RATE},")?;
    writeln!(writer, "  \"scene_seconds\": {SCENE_SECONDS},")?;
    writeln!(writer, "  \"normalization\": \"none\",")?;
    writeln!(writer, "  \"scenes\": [")?;
    for (index, metric) in metrics.iter().enumerate() {
        let comma = if index + 1 == metrics.len() { "" } else { "," };
        writeln!(
            writer,
            concat!(
                "    {{ \"id\": \"{}\", \"program\": \"{}\", ",
                "\"description\": \"{}\", \"frames\": {}, \"peak\": {:.9}, ",
                "\"rms\": {:.9}, \"dc\": {:.9}, \"clipped_samples\": {} }}{}"
            ),
            metric.id,
            metric.program,
            metric.description,
            metric.frames,
            metric.peak,
            metric.rms,
            metric.dc,
            metric.clipped_samples,
            comma
        )?;
    }
    writeln!(writer, "  ]")?;
    writeln!(writer, "}}")?;
    writer.flush()
}

fn scenes() -> Vec<Scene> {
    vec![
        Scene {
            id: "01_baseline_warm_chords",
            program: "baseline-warm",
            description: "Three dry polyphonic chords for global tone and level",
            events: chord_sequence(&[
                (0.20, 1.55, &[48, 55, 60]),
                (1.85, 3.20, &[44, 51, 56]),
                (3.50, 5.10, &[41, 48, 55]),
            ]),
        },
        Scene {
            id: "02_filter_drive",
            program: "audition-filter-drive",
            description: "High-level dual-VCO chords through the five filter profiles",
            events: chord_sequence(&[
                (0.20, 1.55, &[36, 43, 48]),
                (1.90, 3.25, &[39, 46, 51]),
                (3.60, 5.15, &[41, 48, 53]),
            ]),
        },
        Scene {
            id: "03_filter_resonance",
            program: "audition-filter-resonance",
            description: "Ascending notes near the documented resonance calibration region",
            events: monophonic_sequence(&[
                (0.20, 52),
                (1.15, 57),
                (2.10, 60),
                (3.05, 64),
                (4.00, 69),
            ]),
        },
        Scene {
            id: "04_wheel_vibrato",
            program: "audition-wheel-vibrato",
            description: "Common triangle LFO routed to both oscillator frequencies",
            events: chord_sequence(&[(0.25, 5.10, &[57])]),
        },
        Scene {
            id: "05_wheel_pwm",
            program: "audition-wheel-pwm",
            description: "Common triangle LFO routed to both pulse-width summing nodes",
            events: chord_sequence(&[(0.25, 5.10, &[45, 52, 57])]),
        },
        Scene {
            id: "06_wheel_filter",
            program: "audition-wheel-filter",
            description: "Common triangle LFO routed to filter cutoff",
            events: chord_sequence(&[(0.25, 5.10, &[48, 55, 60])]),
        },
        Scene {
            id: "07_envelope_punch",
            program: "audition-envelope-punch",
            description: "Short repeated notes expose ten CEM3310 attack and decay profiles",
            events: chord_sequence(&[
                (0.20, 0.58, &[36]),
                (0.85, 1.23, &[36]),
                (1.50, 1.88, &[43]),
                (2.15, 2.53, &[43]),
                (2.80, 3.18, &[48]),
                (3.45, 3.83, &[48]),
                (4.10, 4.48, &[36, 43, 48]),
            ]),
        },
        Scene {
            id: "08_envelope_slow",
            program: "audition-envelope-slow",
            description: "Slow chord exposes independent filter and amplifier RC trajectories",
            events: chord_sequence(&[(0.10, 4.70, &[48, 55, 60])]),
        },
        Scene {
            id: "09_ca3280_drive",
            program: "audition-ca3280-drive",
            description: "Strong five-voice chords traverse all mixer, final and master CA3280 stages",
            events: chord_sequence(&[
                (0.20, 1.75, &[36, 43, 48, 52, 55]),
                (2.05, 3.60, &[41, 48, 53, 57, 60]),
                (3.90, 5.25, &[43, 50, 55, 59, 62]),
            ]),
        },
        Scene {
            id: "10_common_noise_vca",
            program: "audition-common-noise-vca",
            description: "Common noise-level CA3280 feeding the five filter noise inputs",
            events: chord_sequence(&[
                (0.20, 1.45, &[36]),
                (1.75, 3.00, &[43, 48]),
                (3.30, 5.05, &[36, 43, 48, 52, 55]),
            ]),
        },
        Scene {
            id: "11_poly_mod_oscillator_b",
            program: "audition-poly-mod-oscillator-b",
            description: "Audio-rate oscillator-B triangle through five unlinearized Poly Mod amount VCAs",
            events: monophonic_sequence(&[
                (0.20, 43),
                (1.20, 48),
                (2.20, 52),
                (3.20, 55),
                (4.20, 60),
            ]),
        },
        Scene {
            id: "12_poly_mod_filter_envelope",
            program: "audition-poly-mod-filter-envelope",
            description: "Descending resonant sweeps through five linearized Poly Mod envelope VCAs",
            events: chord_sequence(&[
                (0.20, 1.35, &[36]),
                (1.60, 2.75, &[43]),
                (3.00, 4.15, &[48]),
                (4.40, 5.35, &[36, 43, 48]),
            ]),
        },
        Scene {
            id: "13_wheel_noise_filter",
            program: "audition-wheel-noise-filter",
            description: "Noise half of the complementary Wheel Mod source OTA routed to filter cutoff",
            events: chord_sequence(&[(0.20, 5.20, &[48, 55, 60])]),
        },
        Scene {
            id: "14_bipolar_hard_sync",
            program: "audition-hard-sync",
            description: "Both capacitively coupled oscillator-B pulse edges reverse the matching oscillator-A triangle branch",
            events: monophonic_sequence(&[
                (0.20, 36),
                (1.20, 41),
                (2.20, 48),
                (3.20, 53),
                (4.20, 60),
            ]),
        },
    ]
}

fn chord_sequence(entries: &[(f32, f32, &[u8])]) -> Vec<MidiAction> {
    let mut events = Vec::new();
    for (start, end, notes) in entries {
        for note in *notes {
            events.push(MidiAction {
                frame: seconds_to_frame(*start),
                data: [0x90, *note, 112],
            });
        }
        for note in *notes {
            events.push(MidiAction {
                frame: seconds_to_frame(*end),
                data: [0x80, *note, 0],
            });
        }
    }
    events.sort_by_key(|event| event.frame);
    events
}

fn monophonic_sequence(entries: &[(f32, u8)]) -> Vec<MidiAction> {
    let mut events = Vec::new();
    for (index, (start, note)) in entries.iter().enumerate() {
        let end = entries
            .get(index + 1)
            .map_or(5.10, |(next_start, _)| next_start - 0.10);
        events.push(MidiAction {
            frame: seconds_to_frame(*start),
            data: [0x90, *note, 112],
        });
        events.push(MidiAction {
            frame: seconds_to_frame(end),
            data: [0x80, *note, 0],
        });
    }
    events.sort_by_key(|event| event.frame);
    events
}

fn seconds_to_frame(seconds: f32) -> u32 {
    (seconds * SAMPLE_RATE as f32).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("rf-5-{label}-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn scene_events_are_ordered_and_inside_the_render_window() {
        for scene in scenes() {
            assert!(!scene.events.is_empty());
            assert!(
                scene
                    .events
                    .windows(2)
                    .all(|pair| pair[0].frame <= pair[1].frame)
            );
            assert!(
                scene
                    .events
                    .iter()
                    .all(|event| event.frame < SAMPLE_RATE * SCENE_SECONDS)
            );
        }
    }

    #[test]
    fn wav_writer_emits_a_valid_mono_pcm_header() {
        let directory = temporary_directory("wav-header");
        fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("test.wav");
        write_pcm16_mono(&path, 48_000, &[0.0, 0.5, -0.5]).expect("write WAV");
        let bytes = fs::read(&path).expect("read WAV");
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[36..40], b"data");
        assert_eq!(u32::from_le_bytes(bytes[40..44].try_into().unwrap()), 6);
        fs::remove_dir_all(directory).expect("remove temporary directory");
    }

    #[test]
    fn program_probes_are_deterministic_finite_and_bounded() {
        fn probe(program: &str) -> Vec<f32> {
            let mut engine = Engine::default();
            assert!(engine.prepare(f64::from(SAMPLE_RATE)));
            assert!(engine.load_program(program));
            engine.handle_midi([0x90, 48, 112]);
            let mut samples = Vec::with_capacity(16_384);
            for frame in 0..16_384 {
                if frame == 12_000 {
                    engine.handle_midi([0x80, 48, 0]);
                }
                samples.push(engine.next_sample());
            }
            samples
        }

        for scene in scenes() {
            let first = probe(scene.program);
            let second = probe(scene.program);
            assert_eq!(first, second, "non-deterministic program {}", scene.program);
            let peak = first.iter().fold(0.0_f32, |peak, sample| {
                assert!(sample.is_finite());
                peak.max(sample.abs())
            });
            assert!(peak > 0.01, "silent program {}", scene.program);
            assert!(peak < 0.999, "unbounded program {}", scene.program);
        }
    }
}
