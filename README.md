# RackForge RF-5

RF-5 is an independent, portable five-voice analog polyphonic synthesizer for
RackForge. It is being built as a self-contained Rust/WebAssembly instrument:
no firmware, sample ROM, wave ROM or external sound bank is required.

The target architecture combines two voltage-controlled oscillators per voice,
hard sync, audio-rate polyphonic modulation, a four-pole resonant low-pass
filter, separate filter and amplifier envelopes, unison and performance
controls. All factory programs and interface assets will be original RF-5
content.

## Current status

Milestone 0 establishes the portable package, five-voice allocator, parameter
contract, deterministic tests and CI. Its audible oscillator/filter path is a
technical baseline only; it is not accepted as the final circuit model. Every
fidelity block must replace that baseline through the source and measurement
gates documented in [`docs/IMPLEMENTATION_PLAN.md`](docs/IMPLEMENTATION_PLAN.md).

## Repository layout

```text
crates/rf-5-contract/  Stable public parameter and state vocabulary
crates/rf-5-voice/     Per-voice synthesis path
crates/rf-5-dsp/       Five-voice allocator, MIDI and audio engine
plugin/                RackForge wasm-v1 adapter and package resources
docs/                  Architecture, evidence and implementation gates
tools/                 Reproducible package builders
```

## Build and test

A RackForge checkout is expected next to this repository. For local development,
copy `.cargo/config.toml.example` to `.cargo/config.toml` to use that checkout.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --release --workspace
rustup target add wasm32-unknown-unknown
bash tools/build-package.sh
```

The package is written to `artifacts/rf-5-0.1.0.rfplugin`. GitHub Actions tests
x86-64 and ARM64 before publishing the portable package as a workflow artifact.

## Independence

RF-5 is not affiliated with or endorsed by any hardware manufacturer. The
repository and distributed package do not include third-party firmware, ROMs,
factory sound banks, product artwork or trademarks.

## License

RF-5 is distributed under GPL-3.0-only. See [LICENSE](LICENSE) and
[NOTICE.md](NOTICE.md).
