use std::{env, f64::consts::PI, fmt::Write as _, fs, path::PathBuf};

const TABLE_SIZE: usize = 4_096;
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let harmonic_counts = harmonic_counts();
    let harmonic_levels = harmonic_counts.len();
    let mut phase_rows = vec![vec![0.0_f32; harmonic_levels]; TABLE_SIZE];
    for (index, row) in phase_rows.iter_mut().enumerate() {
        let phase = index as f64 / TABLE_SIZE as f64;
        let mut saw = 0.0_f64;
        let mut level = 0;
        for harmonic in 1..=*harmonic_counts.last().expect("harmonic levels") {
            saw -= 2.0 * (2.0 * PI * harmonic as f64 * phase).sin() / (PI * harmonic as f64);
            if harmonic == harmonic_counts[level] {
                row[level] = saw as f32;
                level += 1;
                if level == harmonic_levels {
                    break;
                }
            }
        }
    }

    let mut generated = String::new();
    writeln!(generated, "const PULSE_TABLE_SIZE: usize = {TABLE_SIZE};").unwrap();
    writeln!(
        generated,
        "const PULSE_HARMONIC_LEVELS: usize = {harmonic_levels};"
    )
    .unwrap();
    writeln!(
        generated,
        "const PULSE_HARMONIC_COUNTS: [u16; {harmonic_levels}] = {:?};",
        harmonic_counts
    )
    .unwrap();
    writeln!(
        generated,
        "#[allow(clippy::approx_constant)]\nstatic BAND_LIMITED_SAW_TABLES: [[f32; {TABLE_SIZE}]; {harmonic_levels}] = ["
    )
    .unwrap();

    for level in 0..harmonic_levels {
        writeln!(generated, "    [").unwrap();
        for row in &phase_rows {
            let sample = row[level];
            writeln!(generated, "        {sample:e}_f32,").unwrap();
        }
        writeln!(generated, "    ],").unwrap();
    }
    writeln!(generated, "];").unwrap();

    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("pulse_wavetable.rs");
    fs::write(output, generated).expect("write pulse wavetable");
}

fn harmonic_counts() -> Vec<usize> {
    let mut counts = (1..=32).collect::<Vec<_>>();
    counts.extend((34..=64).step_by(2));
    counts.extend((68..=128).step_by(4));
    counts.extend((136..=256).step_by(8));
    counts.extend((272..=512).step_by(16));
    counts.extend((544..=1_024).step_by(32));
    counts
}
