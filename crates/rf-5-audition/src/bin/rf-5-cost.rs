//! What one frame of a program costs, on this machine, with every voice busy.
//!
//! The appliance runs the same arithmetic under wasmtime on a slower core, so
//! the absolute number is not the appliance's; the RATIO between two builds
//! of this crate is what transfers. It exists so an ablation can be ranked in
//! seconds here before one build is sent to the appliance to confirm it.
//!
//! ```text
//! cargo run --release --bin rf-5-cost -- [program-id] [notes...]
//! ```

use std::{hint::black_box, time::Instant};

use rf_5_dsp::Engine;

const SAMPLE_RATE: f64 = 48_000.0;
const FRAMES: usize = 96_000;
const BLOCK_FRAMES: usize = 256;
const ROUNDS: usize = 5;

fn main() {
    let mut arguments = std::env::args().skip(1);
    let program = arguments
        .next()
        .unwrap_or_else(|| "original-14-percussive-e-piano".to_owned());
    let notes: Vec<u8> = arguments.map(|n| n.parse().expect("nota MIDI")).collect();
    let notes = if notes.is_empty() {
        vec![48, 55, 60, 64, 67]
    } else {
        notes
    };

    let mut best = f64::MAX;
    for _ in 0..ROUNDS {
        let mut engine = Engine::default();
        assert!(engine.prepare(SAMPLE_RATE));
        assert!(
            engine.load_program(&program),
            "programa desconocido: {program}"
        );
        for &note in &notes {
            engine.note_on(0, note, 100);
        }
        // Let the envelopes leave their attacks before the clock starts, so
        // the figure is the sustained cost and not the first millisecond's.
        for _ in 0..4_800 {
            black_box(engine.next_sample());
        }
        let started = Instant::now();
        for _ in 0..FRAMES {
            black_box(engine.next_sample());
        }
        best = best.min(started.elapsed().as_nanos() as f64 / FRAMES as f64);
    }
    println!(
        "{program} con {} notas: {best:7.1} ns/frame = {:7.1} us por bloque de {BLOCK_FRAMES}",
        notes.len(),
        best * BLOCK_FRAMES as f64 / 1000.0
    );
}
