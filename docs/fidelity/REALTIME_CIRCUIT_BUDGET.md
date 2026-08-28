# Real-time circuit budget

## Decision

The distributed RF-5 component keeps every modeled signal block in the audible
path. No voice, oscillator, nonlinear filter cell, envelope, OTA stage,
resonance return, master VCA or output coupling network is removed on Raspberry
Pi or any other host. The package contains one architecture-independent
WebAssembly component.

The release path is evaluated at host sample rate. The source tree also retains
a complete four-times profile with 127-tap reconstruction as an offline
fidelity oracle. The oracle is intentionally not shipped as a selectable mode:
it exceeds the real-time budget of the Raspberry Pi 4 in worst-case polyphony.
This sample-rate boundary is the largest known difference between the portable
component and the oracle; it is not presented as inaudible.

## Retained physical model

Each of the five voices still evaluates:

- two free-running CEM3340 candidates, fractional one-edge hard sync and the
  loaded waveform outputs;
- the dual-CA3280 oscillator/noise mixer and Poly Mod current paths;
- four distinct nonlinear CEM3320 pole cells, the populated resonance return
  capacitors and the TL082 output-buffer slew state;
- separate CEM3310 filter and amplifier timing capacitors;
- the final CA3280 voice VCA.

After the passive five-input sum, the common master CA3280, output coupling
capacitor, loaded NE5534 follower and jack isolation are also retained. Control
scanning, sample-and-hold state, drift, automatic tune, LFO/noise sources and
performance modulation remain independent physical or firmware-derived state.
The five oscillator-mixer/filter paths continue evolving while their final
VCAs are closed; this preserves pulse-wave DC operating points and prevents a
stale nonlinear filter state from appearing at the next key-down.

## Bounded numerical transformations

The portable profile reduces arithmetic cost without deleting topology:

- reduced-domain polynomial functions replace general logarithm, exponential,
  hyperbolic tangent and fixed fractional-root calls; their maximum errors are
  bounded by unit tests over the actual circuit domains;
- the CEM3320's contractive region below panel resonance 0.16 reuses the
  physical return capacitors as a one-sample state prediction;
- the normal resonant region uses one Newton correction when the TPT
  coefficient is below 0.35, while the extreme high-cutoff region retains the
  converged three-correction solve;
- fixed sixteenth- and thirty-second-order soft knees use degree-six inverse
  root polynomials rather than four or five square roots;
- pulse spectra come from build-time mipmapped Fourier tables; the audio
  thread performs bounded interpolation and no trigonometric series, with the
  harmonic boundary scaled to the active one-, two- or four-times profile;
- the committed audio path evaluates only transfer values; slope derivatives
  remain available to the Newton predictor but are not calculated and then
  discarded for each of the four committed cells;
- Binaryen 132 `wasm-opt -O4` is a required, checksum-pinned packaging step.

The last value-only split is sample-identical across all 34 deterministic
scenes. The combined real-time math, feedback and soft-knee candidate was
compared with the prior stable host-rate motor under fixed limits recorded
before the render:

| Metric | Measured | Limit | Result |
| --- | ---: | ---: | --- |
| Mean absolute level delta | 0.013 dB | 0.50 dB | Pass |
| Mean critical-band RMS delta | 0.102 dB | 1.00 dB | Pass |
| Aggregate aligned error | -37.064 dB | -30.00 dB | Pass |
| 12-20 kHz excess | -81.209 dB | -50.00 dB | Pass |
| Scene outliers | 0 / 34 | 0 | Pass |

These figures accept the optimizations relative to the previous stable
portable motor. They do not override the separately documented host-rate versus
four-times-oracle result.

## Hardware validation

The optimized universal package was exercised on a Raspberry Pi 4 Model B Rev
1.4, 64-bit Cortex-A72 at 1.8 GHz, through the normal RackForge WebAssembly
runtime and a USB Scarlett Solo. The active profile was 48 kHz, 256 period
frames and 512 buffer frames.

- five simultaneous voices, the high-resonance audition program and continuous
  CC1 motion for 30 seconds: zero XRUNs;
- five simultaneous voices, `Baseline Pad` and a full repeated CC1 sweep for 30
  seconds: zero XRUNs.

The same high-resonance test produced sustained XRUNs before the value-only
transfer split. Raising the buffer to 1024 had not solved that earlier build,
so the accepted result comes from reducing redundant computation rather than
hiding it with extra latency.

## Remaining uncertainty

This is a schematic-, firmware- and data-sheet-constrained emulation, not a
calibrated clone of one measured original unit. Populated component
correlations, individual IC overload curves, power-rail interaction,
temperature coefficients, parasitics and original-unit spectra remain evidence
gaps. They are tracked per block in `GAP_MATRIX.md`. A future change may narrow
those uncertainties, but it must pass both the fixed render limits and the
real-time hardware stress matrix before replacing the distributed profile.
