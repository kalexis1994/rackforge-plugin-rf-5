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

The active candidate now contains dual oversampled oscillators, hard sync,
shared LFO and noise, audio-rate Poly Mod, a four-pole CEM3320-class filter,
separate CEM3310-class RC envelopes, five-voice Unison, Glide and both
performance wheels. The global programmable Release switch now drives both
envelopes, and every original RF-5 patch is packed through the recovered V8.1
24-byte program format with 24 seven-bit pots and 22 mapped switches. Separate
CA3280 mixer, final-voice and master-volume
transfers now feed the documented five-input summer and output buffer, while a
source-backed control scheduler recreates the held 6/11 ms panel cycle. It
also reconstructs the ten-VCO automatic-tune path with a 2.5 MHz period
counter, fourteen-bit successive approximation and per-semitone bias
interpolation, including the operating ROM's exact C4-C3 extrapolation for its
three lower octaves and its discrete twelve-position runtime arithmetic. Each
oscillator also uses the recovered 7-bit coarse-pitch assembly, including its
49 normal semitone positions and oscillator B's distinct nine-octave LO FREQ,
keyboard and analog fine paths. Each of those ten oscillators now has an
independent, sample-rate-stable post-tune drift path bounded by the CEM3340 data
sheet, and the engine exposes non-serialized retuning. Its original programmable
Scale Mode adds twelve global, patch-independent chromatic offsets with exact
V8.1 code steps. Their saw, triangle and pulse
outputs also retain the published voltage/symmetry ranges, populated-board
resistor weighting and distinct audio versus Poly Mod polarity. A unified 6/11
ms CPU cycle drives 38 independent sample-and-hold cells through the exact
five-bank V8.1 address order, including both physically unconnected timing
slots, for common, oscillator and per-voice filter CVs. The five filter ICs now
form a serviced CEM3320 population with 440/880 Hz scale calibration, bounded
warm-up motion,
physical resonance gain, clipping span and second-harmonic character. Its ten
CEM3310 envelope generators also retain bounded device-specific peak,
asymptote and RC timing curves. It remains a reverse-engineering candidate:
measured component populations, overload/output levels and original-instrument
measurements still pass through the evidence gates in
[`docs/IMPLEMENTATION_PLAN.md`](docs/IMPLEMENTATION_PLAN.md).

Until the graphical panel is added, the `RF-5 Audition` factory bank provides
eighteen immediately playable listening programs for Wheel/Poly Mod, LFO range,
Sync, filter drive, resonance, fast/slow envelope behaviour, CA3280 drive and
common noise, including an explicit global-Release override. They require no UI; see
[`docs/AUDITION_PROGRAMS.md`](docs/AUDITION_PROGRAMS.md).

## Repository layout

```text
crates/rf-5-contract/  Stable public parameter and state vocabulary
crates/rf-5-voice/     Per-voice synthesis path
crates/rf-5-dsp/       Five-voice allocator, MIDI and audio engine
crates/rf-5-audition/  Deterministic WAV renderer for evaluation without UI
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
cargo run --release -p rf-5-audition
rustup target add wasm32-unknown-unknown
bash tools/build-package.sh
```

The package is written to `artifacts/rf-5-0.1.0.rfplugin`. GitHub Actions tests
x86-64 and ARM64 before publishing the portable package as a workflow artifact.
The audition command writes twenty-one unnormalized listening files and their metrics
to `artifacts/auditions`; see
[`docs/AUDITION_RENDERER.md`](docs/AUDITION_RENDERER.md).

## Independence

RF-5 is not affiliated with or endorsed by any hardware manufacturer. The
repository and distributed package do not include third-party firmware, ROMs,
factory sound banks, product artwork or trademarks.

## License

RF-5 is distributed under GPL-3.0-only. See [LICENSE](LICENSE) and
[NOTICE.md](NOTICE.md).
