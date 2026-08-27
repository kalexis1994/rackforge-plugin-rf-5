# RackForge RF-5

RF-5 is an independent, portable five-voice analog polyphonic synthesizer for
RackForge. It is being built as a self-contained Rust/WebAssembly instrument:
no firmware, sample ROM, wave ROM or external sound bank is required.

The target architecture combines two voltage-controlled oscillators per voice,
the original one-edge CEM3340 hard-sync path, audio-rate polyphonic modulation,
a four-pole resonant low-pass filter, separate filter and amplifier envelopes,
unison and performance controls. All factory programs and interface assets
will be original RF-5 content.

## Current status

The active candidate now contains dual four-times-oversampled oscillators,
fractional hard sync and a 127-tap anti-alias decimator after the nonlinear
per-voice filter/VCA path. Saw and pulse edges use a two-host-sample PolyBLEP
correction, while oscillator B's asymmetric triangle uses a one-internal-sample
PolyBLAMP at both slope transitions. SD431's single falling-edge sync transient
retains its fractional position inside each internal sample and drives the
external Figure 5 reset network, one shared LFO, independent pink Wheel-Mod
and white audio-noise generators,
audio-rate Poly Mod, a four-pole CEM3320-class filter,
separate CEM3310-class RC envelopes, five-voice Unison, Glide and both
performance wheels, including the pitch wheel's SD334 diode deadband. The
direct R104 Master Tune path now reaches both VCO master summers continuously,
outside the scanned and programmable control domain. The global programmable
Release switch now sends V8.1's exact fixed `0x64` write—equivalent to physical
pot code `0x16`—through both release sample/hold cells and envelopes, and every
original RF-5 patch is packed through the recovered V8.1
24-byte program format with 24 seven-bit pots and 22 mapped switches. Separate
CA3280 mixer, final-voice and master-volume
transfers now feed the documented five-input summer, C4189 coupling network and
1-kohm-loaded NE5534 output follower. The final host mapping is linear, so it
cannot compress before the reconstructed analog stages, while a
source-backed control scheduler recreates the held 6/11 ms panel cycle. It
retains the physical window ADC's 34 mV hysteresis boundary and the V8.1
two-scan same-direction pot qualification, while program and state recalls
synchronize immediately through their separate path. It
also reconstructs the ten-VCO automatic-tune path with a 2.5 MHz period
counter, fourteen-bit successive approximation and per-semitone bias
interpolation, including the operating ROM's exact C4-C3 extrapolation for its
three lower octaves and its discrete twelve-position runtime arithmetic. Each
oscillator also uses the recovered 7-bit coarse-pitch assembly, including its
49 normal semitone positions and oscillator B's distinct nine-octave LO FREQ,
keyboard and one-sided 0-to-1-semitone FINE paths. Each of those ten oscillators
now has an independent, sample-rate-stable post-tune drift path bounded by the
CEM3340 data sheet, and the engine exposes the original momentary two-to-eight-second
retuning operation. The exact 2.5 MHz/5682 A-440 counter now crosses its
grounded-off 4016 switch and populated output RC network before joining the
five-voice pre-volume summer. Its original programmable
Scale Mode adds twelve global, patch-independent chromatic offsets with exact
V8.1 code steps. Their saw, triangle and pulse
outputs also retain the published voltage/symmetry ranges, populated-board
resistor weighting, 128-step 1-99% panel pulse-width law, modulation overtravel
to stable 0/100% DC and distinct audio versus Poly Mod polarity. The shared LFO
now follows SD334's complete CEM3340 reference, multiplier,
1-uF timing-capacitor and 0-10 V DAC network instead of a provisional 20 Hz
anchor: its nominal 128-step range is approximately 0.0908-55.8 Hz, with the
published finite timing-current ceiling rounding only the fastest codes. Its
saw and loaded square retain their positive-going Wheel Mod displacement;
U380 alone centres triangle around ground for symmetric vibrato. A unified 6/11
ms CPU cycle drives 38 independent sample-and-hold cells through the exact
five-bank V8.1 address order, including both physically unconnected timing
slots, for common, oscillator and per-voice filter CVs. Scheduled visits use
the populated 0.01 uF cell, a conservative 4051 resistance bound and the
recovered 25.6 us firmware dwell, settling by more than 99.9999% per visit.
New gates precede their A/B/filter CV refreshes exactly as in the V8.1 loop,
while Unison keyboard pitch now travels through its dedicated common S/H and
Glide path rather than being mistaken for the digital switch state.
The five filter ICs now
form a serviced CEM3320 population with 440/880 Hz scale calibration, bounded
warm-up motion,
physical resonance gain, clipping span, TL082 large-signal slew and
second-harmonic character. Their
nonlinear four-pole feedback loops are solved without inserting a digital
sample of delay, so self-oscillation follows the calibrated cutoff consistently
from 44.1 through 192 kHz. Its ten
CEM3310 envelope generators also retain bounded device-specific peak,
asymptote and RC timing curves, including finite-buffer steps between physical
phases without discontinuously moving the timing capacitor. It remains a reverse-engineering candidate:
measured component populations, overload/output levels and original-instrument
measurements still pass through the evidence gates in
[`docs/IMPLEMENTATION_PLAN.md`](docs/IMPLEMENTATION_PLAN.md).

The first native RF-5 control surface is now active. It is rendered by a Rust
WebAssembly module, binds every one of the sixty-three public controls exactly once,
keeps both program banks below the hardware panel and reorganizes its five
sections at phone, tablet and desktop widths. Pointer capture gives knobs the
same relative vertical drag on mouse and touch, while RackForge parameter
attributes keep host-owned context menus and MIDI Link available.

The `RF-5 Audition` factory bank provides
twenty-nine immediately playable listening programs for Wheel/Poly Mod, LFO range and polarity,
Sync, filter drive, resonance, fast/slow envelope behaviour, CA3280 drive and
common noise, including explicit global-Release and oscillator-B FINE endpoint
comparisons plus pulse-width endpoints. They require no UI; see
[`docs/AUDITION_PROGRAMS.md`](docs/AUDITION_PROGRAMS.md).

## Repository layout

```text
crates/rf-5-contract/  Stable public parameter and state vocabulary
crates/rf-5-voice/     Per-voice synthesis path
crates/rf-5-dsp/       Five-voice allocator, MIDI and audio engine
crates/rf-5-audition/  Deterministic WAV renderer for evaluation without UI
plugin/                RackForge wasm-v1 adapter and package resources
plugin-ui/             Rust/WebAssembly responsive control surface
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
powershell -ExecutionPolicy Bypass -File tools/build-web-ui.ps1
bash tools/build-package.sh
```

The package is written to `artifacts/rf-5-0.1.0.rfplugin`. GitHub Actions tests
x86-64 and ARM64 before publishing the portable package as a workflow artifact.
The audition command writes thirty-three unnormalized listening files and their metrics
to `artifacts/auditions`; see
[`docs/AUDITION_RENDERER.md`](docs/AUDITION_RENDERER.md).

## Independence

RF-5 is not affiliated with or endorsed by any hardware manufacturer. The
repository and distributed package do not include third-party firmware, ROMs,
factory sound banks, product artwork or trademarks.

## License

RF-5 is distributed under GPL-3.0-only. See [LICENSE](LICENSE) and
[NOTICE.md](NOTICE.md).
