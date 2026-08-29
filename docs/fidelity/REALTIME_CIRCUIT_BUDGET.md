# Real-time circuit budget

## Decision

The distributed RF-5 component keeps every modeled signal block in every
audible path. No active voice, oscillator, nonlinear filter cell, envelope,
OTA stage, resonance return, master VCA or output coupling network is removed
on Raspberry Pi or any other host. Once a card's amplifier envelope is fully
idle, its VCO, sync, PWM, Poly Mod phase and both envelope states continue, but
the electrically hidden filter/VCA/FIR evaluation is suspended until the next
gate. The package contains one architecture-independent WebAssembly component.

The release path evaluates the oscillators, audio-rate Poly Mod and mixer at
four times the host rate, then the nonlinear filter and final voice VCA at two
times the host rate with held/interpolated samples into the four-to-one voice
decimator. The source tree also retains a complete four-times profile
with 127-tap reconstruction as an offline fidelity oracle. The oracle is not a
selectable mode. The distributed hybrid profile is the closest candidate that
can exploit RackForge's host-owned per-voice workers without changing the one
portable component or duplicating shared control state.

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
The ten VCO cores and their modulation-dependent phase paths continue evolving
while their final VCAs are closed. Filter capacitor state is held only after
the amplifier envelope reaches its idle floor; it resumes beneath the next
attack from zero. The purely numerical reconstruction history is then cleared,
and a regression verifies that dormant-card reuse introduces no larger sample
step than a fresh hardware-style attack.

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
- consecutive four-times mixer/cutoff samples are averaged into each two-times
  filter/VCA evaluation and the intervening output is linearly reconstructed
  into the four-to-one voice decimator;
- pulse spectra come from build-time mipmapped Fourier tables; the audio
  thread performs bounded interpolation and no trigonometric series, with the
  harmonic boundary scaled to the active one-, two- or four-times profile;
- cards whose amplifier VCA has reached its zero-current idle state advance
  their free-running VCO/sync/PWM/Poly-Mod and envelope state without computing
  the unobservable nonlinear filter, final VCA and reconstruction FIR;
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

These figures accepted the arithmetic optimizations relative to the previous
stable host-rate motor. They do not override the separately documented sample-
rate comparison. The same bounded arithmetic is retained in the hybrid path.

## Hardware validation

The former host-rate universal package was exercised on a Raspberry Pi 4 Model B Rev
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
hiding it with extra latency. These measurements remain the historical lower-
cost baseline; the hybrid release path requires a fresh hardware stress pass
under RackForge's per-voice workers before receiving the same zero-XRUN claim.

## Remaining uncertainty

This is a schematic-, firmware- and data-sheet-constrained emulation, not a
calibrated clone of one measured original unit. Populated component
correlations, individual IC overload curves, power-rail interaction,
temperature coefficients, parasitics and original-unit spectra remain evidence
gaps. They are tracked per block in `GAP_MATRIX.md`. A future change may narrow
those uncertainties, but it must pass both the fixed render limits and the
real-time hardware stress matrix before replacing the distributed profile.
