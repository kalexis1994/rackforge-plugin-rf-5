use std::{hint::black_box, time::Instant};

use rf_5_contract::Parameter;
use rf_5_dsp::Engine;

const SAMPLE_RATE: f64 = 48_000.0;
const FRAMES: usize = 48_000;

fn main() {
    measure("idle", configure(|_| {}), false);
    measure("one_voice_baseline", configure(|_| {}), true);
    measure_polyphony(
        "five_voice_baseline",
        configure(|_| {}),
        &[48, 55, 60, 64, 67],
    );
    measure(
        "one_voice_no_resonance",
        configure(|engine| set(engine, Parameter::FilterResonance, 0.0)),
        true,
    );
    measure(
        "one_voice_static_cutoff",
        configure(|engine| {
            set(engine, Parameter::FilterResonance, 0.0);
            set(engine, Parameter::FilterEnvelopeAmount, 0.0);
            set(engine, Parameter::PolyModFilterEnvelopeAmount, 0.0);
            set(engine, Parameter::PolyModFilter, 0.0);
        }),
        true,
    );
    measure(
        "one_voice_silent_mixers",
        configure(|engine| {
            set(engine, Parameter::FilterResonance, 0.0);
            set(engine, Parameter::FilterEnvelopeAmount, 0.0);
            set(engine, Parameter::OscillatorALevel, 0.0);
            set(engine, Parameter::OscillatorBLevel, 0.0);
            set(engine, Parameter::NoiseLevel, 0.0);
        }),
        true,
    );

    for wheel in [0_u8, 32, 64, 96, 127] {
        let mut engine = configure(|engine| {
            assert!(engine.load_diagnostic_program("baseline-pad"));
            engine.handle_midi([0xb0, 1, wheel]);
        });
        for note in [48, 55, 60, 64, 67] {
            engine.note_on(0, note, 100);
        }
        measure_engine(&format!("baseline_pad_wheel_{wheel}"), engine);
    }

    measure_baseline_pad_wheel_ramp(48);
    measure_baseline_pad_wheel_ramp(8);
}

fn measure_baseline_pad_wheel_ramp(frames_per_step: usize) {
    let mut engine = configure(|engine| assert!(engine.load_diagnostic_program("baseline-pad")));
    for note in [48, 55, 60, 64, 67] {
        engine.note_on(0, note, 100);
    }

    let started = Instant::now();
    let mut checksum = 0.0_f32;
    let mut peak = 0.0_f32;
    let mut max_delta = 0.0_f32;
    let mut previous = 0.0_f32;
    let mut non_finite = 0_u32;
    for frame in 0..FRAMES {
        if frame % frames_per_step == 0 {
            let step = (frame / frames_per_step) % 254;
            let wheel = if step <= 127 { step } else { 254 - step } as u8;
            engine.handle_midi([0xb0, 1, wheel]);
        }
        let sample = black_box(engine.next_sample());
        if sample.is_finite() {
            checksum += sample;
            peak = peak.max(sample.abs());
            max_delta = max_delta.max((sample - previous).abs());
            previous = sample;
        } else {
            non_finite += 1;
        }
    }
    let elapsed = started.elapsed();
    println!(
        "RF5_PROFILE label=baseline_pad_wheel_ramp_{frames_per_step} frames={FRAMES} elapsed_ms={:.3} realtime_ratio={:.3} checksum={checksum:.6} peak={peak:.6} max_delta={max_delta:.6} non_finite={non_finite}",
        elapsed.as_secs_f64() * 1_000.0,
        SAMPLE_RATE / FRAMES as f64 * elapsed.as_secs_f64(),
    );
}

fn measure_polyphony(label: &str, mut engine: Engine, notes: &[u8]) {
    for &note in notes {
        engine.note_on(0, note, 100);
    }
    measure_engine(label, engine);
}

fn configure(change: impl FnOnce(&mut Engine)) -> Engine {
    let mut engine = Engine::default();
    assert!(engine.prepare(SAMPLE_RATE));
    change(&mut engine);
    engine
}

fn set(engine: &mut Engine, parameter: Parameter, value: f64) {
    assert!(engine.set_parameter(parameter as u32, value));
}

fn measure(label: &str, mut engine: Engine, note_on: bool) {
    if note_on {
        engine.note_on(0, 60, 100);
    }
    measure_engine(label, engine);
}

fn measure_engine(label: &str, mut engine: Engine) {
    let started = Instant::now();
    let mut checksum = 0.0_f32;
    let mut peak = 0.0_f32;
    let mut max_delta = 0.0_f32;
    let mut previous = 0.0_f32;
    let mut non_finite = 0_u32;
    for _ in 0..FRAMES {
        let sample = black_box(engine.next_sample());
        if sample.is_finite() {
            checksum += sample;
            peak = peak.max(sample.abs());
            max_delta = max_delta.max((sample - previous).abs());
            previous = sample;
        } else {
            non_finite += 1;
        }
    }
    let elapsed = started.elapsed();
    println!(
        "RF5_PROFILE label={label} frames={FRAMES} elapsed_ms={:.3} realtime_ratio={:.3} checksum={checksum:.6} peak={peak:.6} max_delta={max_delta:.6} non_finite={non_finite}",
        elapsed.as_secs_f64() * 1_000.0,
        SAMPLE_RATE / FRAMES as f64 * elapsed.as_secs_f64(),
    );
}
