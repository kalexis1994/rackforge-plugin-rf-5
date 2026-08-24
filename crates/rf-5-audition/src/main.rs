use std::{env, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let output_directory = env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("artifacts/auditions"), PathBuf::from);
    match rf_5_audition::render_suite(&output_directory) {
        Ok(metrics) => {
            for scene in metrics {
                println!(
                    "AUDITION_RENDERED id={} peak={:.6} rms={:.6} dc={:.6} clipped={} path={}",
                    scene.id,
                    scene.peak,
                    scene.rms,
                    scene.dc,
                    scene.clipped_samples,
                    scene.path.display()
                );
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("AUDITION_RENDER_ERROR {error}");
            ExitCode::FAILURE
        }
    }
}
