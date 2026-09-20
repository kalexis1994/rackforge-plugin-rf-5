//! What a block costs before a single voice renders.
//!
//! `rf-5-profile` measures `next_sample`, which is preparation and five
//! voices together. On the appliance those two halves land in different
//! places: `prepare_next_sample` runs once per frame on the COORDINATOR, in
//! `begin_block`, and the voices run on worker cores. Measured there, a
//! 128-frame block spends 799 µs in preparation with nothing playing and
//! 840 µs with a four-note chord -- near enough a fixed cost, and a third of
//! the 2666 µs deadline before any voice is asked for a sample.
//!
//! It is also the part Amdahl charges for: splitting the voices across four
//! cores does nothing about it. So it wants measuring on its own.
//!
//! ```text
//! cargo run --release --bin rf-5-prepare-split
//! ```

use std::{hint::black_box, time::Instant};

use rf_5_dsp::{CommonVoiceFrame, Engine, PreparedSample, VoiceCalibration};

const SAMPLE_RATE: f64 = 48_000.0;
/// A second of audio, which is long enough that a stray scheduling hiccup
/// cannot move the mean much.
const FRAMES: usize = 48_000;
/// The appliance's block, so the per-block figures are comparable with what
/// the host measures.
const BLOCK_FRAMES: usize = 128;
const ROUNDS: usize = 5;

fn engine(notes: &[u8]) -> Engine {
    let mut engine = Engine::default();
    assert!(engine.prepare(SAMPLE_RATE));
    for &note in notes {
        engine.note_on(0, note, 100);
    }
    engine
}

/// Best of several rounds: the floor is the number and everything above it
/// is interference from the rest of the machine.
fn best_of(mut run: impl FnMut() -> f64) -> f64 {
    let mut best = f64::MAX;
    for _ in 0..ROUNDS {
        let each = run();
        if each < best {
            best = each;
        }
    }
    best
}

fn nanos_per_frame(label: &str, notes: &[u8], mut body: impl FnMut(&mut Engine)) -> f64 {
    let each = best_of(|| {
        let mut engine = engine(notes);
        let started = Instant::now();
        for _ in 0..FRAMES {
            body(&mut engine);
        }
        started.elapsed().as_nanos() as f64 / FRAMES as f64
    });
    println!(
        "  {label:<34} {:7.1} ns/frame   {:8.1} us por bloque de {BLOCK_FRAMES}",
        each,
        each * BLOCK_FRAMES as f64 / 1000.0
    );
    each
}

fn main() {
    // What a block hands the workers, which the host copies into every one
    // of them: the shared payload is per FRAME, so its size is multiplied by
    // the block length and again by the number of units.
    let common = core::mem::size_of::<CommonVoiceFrame>();
    let calibration = core::mem::size_of::<VoiceCalibration>();
    println!("=== lo que viaja ===");
    println!("  CommonVoiceFrame     {common:5} bytes por frame");
    println!("  VoiceCalibration     {calibration:5} bytes por frame y voz");
    println!("  PreparedSample       {:5} bytes", core::mem::size_of::<PreparedSample>());
    println!(
        "  el payload compartido de un bloque de {BLOCK_FRAMES}: {} KiB, 
           que el host copia a cada una de las 5 unidades: {} KiB por bloque",
        common * BLOCK_FRAMES / 1024,
        common * BLOCK_FRAMES * 6 / 1024
    );

    for (what, notes) in [
        ("en vacio", &[][..]),
        ("una nota", &[60][..]),
        ("cinco notas", &[48, 55, 60, 64, 67][..]),
    ] {
        println!("\n=== {what} ===");
        let prepared = nanos_per_frame("preparacion sola", notes, |engine| {
            black_box(engine.prepare_next_sample());
        });
        let whole = nanos_per_frame("preparacion + las cinco voces", notes, |engine| {
            black_box(engine.next_sample());
        });
        println!(
            "  {:<34} {:7.1} ns/frame   {:8.1} us por bloque   ({:.0} % del bloque)",
            "las voces, por diferencia",
            whole - prepared,
            (whole - prepared) * BLOCK_FRAMES as f64 / 1000.0,
            (whole - prepared) / whole * 100.0
        );
    }
}
