//! Deterministic per-note level scan for keyboard-dependent patch regressions.

use rf_5_contract::Parameter;
use rf_5_dsp::{Engine, VOICE_COUNT};
use std::io::Write as _;

const SAMPLE_RATE: usize = 48_000;
const SETTLE_FRAMES: usize = SAMPLE_RATE * 2;
const DEFAULT_STRIKE_FRAMES: usize = SAMPLE_RATE;
const RMS_START: usize = SAMPLE_RATE / 200;
const RMS_END: usize = SAMPLE_RATE * 3 / 10;
const ATTACK_ANALYSIS_FRAMES: usize = SAMPLE_RATE / 10;

fn main() {
    let program = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "original-16-harpsichord".to_owned());
    let isolation = std::env::args().nth(2).unwrap_or_else(|| "both".to_owned());
    let selected_note = std::env::args()
        .nth(3)
        .map(|note| note.parse::<u8>().expect("note must be a MIDI number"));
    let modifier = std::env::args().nth(4).unwrap_or_default();
    let wave_output = std::env::args().nth(5);
    let render_frames = std::env::args()
        .nth(6)
        .map(|seconds| {
            (seconds.parse::<f32>().expect("duration must be seconds") * SAMPLE_RATE as f32).round()
                as usize
        })
        .unwrap_or(DEFAULT_STRIKE_FRAMES);
    let gate_frames = std::env::args().nth(7).map(|seconds| {
        (seconds
            .parse::<f32>()
            .expect("gate duration must be seconds")
            * SAMPLE_RATE as f32)
            .round() as usize
    });

    let mut source = Engine::default();
    assert!(source.prepare(SAMPLE_RATE as f64));
    assert!(source.load_program(&program), "unknown program {program}");
    let settings = source.settings();
    println!(
        "RF5_NOTE_SCAN program={program} isolation={isolation} cutoff={:.6} resonance={:.6} keyboard={:.0} filter_env={:.6} a_level={:.6} a_freq={:.6} a_saw={:.0} a_pulse={:.0} a_pw={:.6} b_level={:.6} b_freq={:.6} b_fine={:.6} b_saw={:.0} b_triangle={:.0} b_pulse={:.0} b_pw={:.6} sync={:.0} poly={:.6}/{:.6}/{:.0}/{:.0}/{:.0} amp_adsr={:.6}/{:.6}/{:.6}/{:.6} filter_adsr={:.6}/{:.6}/{:.6}/{:.6} release_switch={:.0} glide={:.6} unison={:.0} lfo_freq={:.6} lfo_shapes={:.0}/{:.0}/{:.0} wheel_mix={:.6} wheel_dest={:.0}/{:.0}/{:.0}/{:.0}/{:.0}",
        settings.get(Parameter::FilterCutoff),
        settings.get(Parameter::FilterResonance),
        settings.get(Parameter::FilterKeyboard),
        settings.get(Parameter::FilterEnvelopeAmount),
        settings.get(Parameter::OscillatorALevel),
        settings.get(Parameter::OscillatorAFrequency),
        settings.get(Parameter::OscillatorASaw),
        settings.get(Parameter::OscillatorAPulse),
        settings.get(Parameter::OscillatorAPulseWidth),
        settings.get(Parameter::OscillatorBLevel),
        settings.get(Parameter::OscillatorBFrequency),
        settings.get(Parameter::OscillatorBDetune),
        settings.get(Parameter::OscillatorBSaw),
        settings.get(Parameter::OscillatorBTriangle),
        settings.get(Parameter::OscillatorBPulse),
        settings.get(Parameter::OscillatorBPulseWidth),
        settings.get(Parameter::OscillatorSync),
        settings.get(Parameter::PolyModFilterEnvelopeAmount),
        settings.get(Parameter::PolyModOscillatorBAmount),
        settings.get(Parameter::PolyModOscillatorAFrequency),
        settings.get(Parameter::PolyModOscillatorAPulseWidth),
        settings.get(Parameter::PolyModFilter),
        settings.get(Parameter::AmpAttack),
        settings.get(Parameter::AmpDecay),
        settings.get(Parameter::AmpSustain),
        settings.get(Parameter::AmpRelease),
        settings.get(Parameter::FilterAttack),
        settings.get(Parameter::FilterDecay),
        settings.get(Parameter::FilterSustain),
        settings.get(Parameter::FilterRelease),
        settings.get(Parameter::ReleaseSwitch),
        settings.get(Parameter::Glide),
        settings.get(Parameter::Unison),
        settings.get(Parameter::LfoFrequency),
        settings.get(Parameter::LfoSaw),
        settings.get(Parameter::LfoTriangle),
        settings.get(Parameter::LfoSquare),
        settings.get(Parameter::WheelModSourceMix),
        settings.get(Parameter::WheelModOscillatorAFrequency),
        settings.get(Parameter::WheelModOscillatorBFrequency),
        settings.get(Parameter::WheelModOscillatorAPulseWidth),
        settings.get(Parameter::WheelModOscillatorBPulseWidth),
        settings.get(Parameter::WheelModFilter),
    );

    for note in 24_u8..=96 {
        if selected_note.is_some_and(|selected| selected != note) {
            continue;
        }
        if selected_note.is_none() && (note - 24) % 3 != 0 {
            continue;
        }
        let mut engine = Engine::default();
        assert!(engine.prepare(SAMPLE_RATE as f64));
        assert!(engine.load_program(&program));
        let mut precondition_pitch = false;
        let mut glide_target = None;
        for modifier in modifier.split(',').filter(|value| !value.is_empty()) {
            if modifier == "open" {
                for (parameter, value) in [
                    (Parameter::FilterCutoff, 1.0),
                    (Parameter::FilterResonance, 0.0),
                    (Parameter::FilterEnvelopeAmount, 0.0),
                    (Parameter::AmpAttack, 0.0),
                    (Parameter::AmpDecay, 0.0),
                    (Parameter::AmpSustain, 1.0),
                ] {
                    assert!(engine.set_parameter(parameter as u32, value));
                }
            } else if modifier == "nosync" {
                assert!(engine.set_parameter(Parameter::OscillatorSync as u32, 0.0));
            } else if modifier == "levels" {
                assert!(engine.set_parameter(Parameter::OscillatorALevel as u32, 1.0));
                assert!(engine.set_parameter(Parameter::OscillatorBLevel as u32, 1.0));
            } else if modifier == "b-saw" {
                assert!(engine.set_parameter(Parameter::OscillatorBSaw as u32, 1.0));
            } else if modifier == "b-triangle" {
                assert!(engine.set_parameter(Parameter::OscillatorBTriangle as u32, 1.0));
            } else if modifier == "b-pulse" {
                assert!(engine.set_parameter(Parameter::OscillatorBPulse as u32, 1.0));
            } else if modifier == "nopolymod" {
                assert!(engine.set_parameter(Parameter::PolyModOscillatorAFrequency as u32, 0.0));
                assert!(engine.set_parameter(Parameter::PolyModFilterEnvelopeAmount as u32, 0.0));
                assert!(engine.set_parameter(Parameter::PolyModOscillatorBAmount as u32, 0.0));
            } else if modifier == "release-off" {
                assert!(engine.set_parameter(Parameter::ReleaseSwitch as u32, 0.0));
            } else if modifier == "precondition" {
                precondition_pitch = true;
            } else if modifier == "unison-off" {
                assert!(engine.set_parameter(Parameter::Unison as u32, 0.0));
            } else if let Some(target) = modifier.strip_prefix("glide-to=") {
                glide_target = Some(
                    target
                        .parse::<u8>()
                        .expect("glide target must be a MIDI note"),
                );
            } else if let Some(amount) = modifier.strip_prefix("poly=") {
                let amount = amount.parse::<f64>().expect("poly amount must be numeric");
                assert!(
                    engine.set_parameter(Parameter::PolyModFilterEnvelopeAmount as u32, amount)
                );
            } else if let Some(amount) = modifier.strip_prefix("wheel=") {
                let amount = amount.parse::<f32>().expect("wheel amount must be numeric");
                let midi_value = (amount.clamp(0.0, 1.0) * 127.0).round() as u8;
                engine.handle_midi([0xb0, 1, midi_value]);
            } else if let Some(amount) = modifier.strip_prefix("amp-release=") {
                let amount = amount
                    .parse::<f64>()
                    .expect("amplifier release amount must be numeric");
                assert!(engine.set_parameter(Parameter::AmpRelease as u32, amount));
            } else if let Some(amount) = modifier.strip_prefix("amp-attack=") {
                let amount = amount
                    .parse::<f64>()
                    .expect("amplifier attack amount must be numeric");
                assert!(engine.set_parameter(Parameter::AmpAttack as u32, amount));
            } else if let Some(amount) = modifier.strip_prefix("filter-release=") {
                let amount = amount
                    .parse::<f64>()
                    .expect("filter release amount must be numeric");
                assert!(engine.set_parameter(Parameter::FilterRelease as u32, amount));
            } else if let Some(amount) = modifier.strip_prefix("cutoff=") {
                let amount = amount
                    .parse::<f64>()
                    .expect("filter cutoff amount must be numeric");
                assert!(engine.set_parameter(Parameter::FilterCutoff as u32, amount));
            } else if let Some(amount) = modifier.strip_prefix("resonance=") {
                let amount = amount
                    .parse::<f64>()
                    .expect("filter resonance amount must be numeric");
                assert!(engine.set_parameter(Parameter::FilterResonance as u32, amount));
            } else if let Some(amount) = modifier.strip_prefix("filter-env=") {
                let amount = amount
                    .parse::<f64>()
                    .expect("filter envelope amount must be numeric");
                assert!(engine.set_parameter(Parameter::FilterEnvelopeAmount as u32, amount));
            } else {
                panic!("unknown modifier {modifier}");
            }
        }
        match isolation.as_str() {
            "a" => assert!(engine.set_parameter(Parameter::OscillatorBLevel as u32, 0.0)),
            "b" => assert!(engine.set_parameter(Parameter::OscillatorALevel as u32, 0.0)),
            "both" => {}
            _ => panic!("isolation must be one of: both, a, b"),
        }
        if precondition_pitch {
            engine.note_on(0, note, 127);
            for _ in 0..SETTLE_FRAMES {
                let _ = engine.next_sample();
            }
            engine.note_off(0, note);
            engine.reset_voices();
        }
        for _ in 0..SETTLE_FRAMES {
            let _ = engine.next_sample();
        }

        engine.note_on(0, note, 127);
        let mut peak = 0.0_f32;
        let mut energy = 0.0_f64;
        let mut voice_energy = 0.0_f64;
        let mut rms_frames = 0;
        let mut previous = 0.0_f32;
        let mut previous_delta = 0.0_f32;
        let mut maximum_delta = (0.0_f32, 0_usize);
        let mut maximum_second_difference = (0.0_f32, 0_usize);
        let mut samples = wave_output
            .as_ref()
            .map(|_| Vec::with_capacity(render_frames));
        for frame in 0..render_frames {
            if let Some(target) = glide_target.filter(|_| frame == SAMPLE_RATE / 2) {
                engine.note_on(0, target, 127);
                engine.note_off(0, note);
            }
            if gate_frames == Some(frame) {
                if let Some(target) = glide_target {
                    engine.note_off(0, target);
                }
                engine.note_off(0, note);
            }
            let prepared = engine.prepare_next_sample();
            let mut voice_sum = 0.0_f32;
            for voice in 0..VOICE_COUNT {
                voice_sum += engine.render_prepared_voice(voice, prepared);
            }
            let sample = engine.finish_prepared_sample(prepared, voice_sum);
            assert!(sample.is_finite(), "non-finite sample at MIDI note {note}");
            peak = peak.max(sample.abs());
            let delta = sample - previous;
            let second_difference = delta - previous_delta;
            if frame < ATTACK_ANALYSIS_FRAMES && delta.abs() > maximum_delta.0 {
                maximum_delta = (delta.abs(), frame);
            }
            if frame < ATTACK_ANALYSIS_FRAMES
                && second_difference.abs() > maximum_second_difference.0
            {
                maximum_second_difference = (second_difference.abs(), frame);
            }
            previous = sample;
            previous_delta = delta;
            if let Some(samples) = &mut samples {
                samples.push(sample);
            }
            if (RMS_START..RMS_END).contains(&frame) {
                energy += f64::from(sample) * f64::from(sample);
                voice_energy += f64::from(voice_sum) * f64::from(voice_sum);
                rms_frames += 1;
            }
        }
        let rms = (energy / rms_frames as f64).sqrt();
        let voice_rms = (voice_energy / rms_frames as f64).sqrt();
        println!(
            "note={note:3} peak={peak:.7} rms={rms:.7} voice_rms={voice_rms:.7} max_delta={:.7}@{} max_d2={:.7}@{}",
            maximum_delta.0,
            maximum_delta.1,
            maximum_second_difference.0,
            maximum_second_difference.1,
        );
        if let (Some(path), Some(samples)) = (&wave_output, samples) {
            write_pcm16(path, &samples);
        }
    }
}

fn write_pcm16(path: &str, samples: &[f32]) {
    let data_bytes = u32::try_from(samples.len() * 2).expect("WAV data length");
    let mut file = std::fs::File::create(path).expect("create WAV output");
    file.write_all(b"RIFF").unwrap();
    file.write_all(&(36 + data_bytes).to_le_bytes()).unwrap();
    file.write_all(b"WAVEfmt ").unwrap();
    file.write_all(&16_u32.to_le_bytes()).unwrap();
    file.write_all(&1_u16.to_le_bytes()).unwrap();
    file.write_all(&1_u16.to_le_bytes()).unwrap();
    file.write_all(&(SAMPLE_RATE as u32).to_le_bytes()).unwrap();
    file.write_all(&(SAMPLE_RATE as u32 * 2).to_le_bytes())
        .unwrap();
    file.write_all(&2_u16.to_le_bytes()).unwrap();
    file.write_all(&16_u16.to_le_bytes()).unwrap();
    file.write_all(b"data").unwrap();
    file.write_all(&data_bytes.to_le_bytes()).unwrap();
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
        file.write_all(&value.to_le_bytes()).unwrap();
    }
}
