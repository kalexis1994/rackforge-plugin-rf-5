# Portable/reference comparison

## Decision

The former host-rate distributed profile was **not accepted as perceptually
negligible** relative to the four-times fidelity reference. The fast bounded
elementary functions were accepted; removing internal-rate processing was the
dominant source of the measured difference.

That decision did not mean that the host-rate profile was unstable or unusable.
All thirty-four renders are finite, audible, unclipped and within host
headroom. It means only that the present 1x reduction cannot be described as
indistinguishable from the admitted reference under the limits below.

## Fixed limits

The limits were recorded before rendering the comparison suite:

- mean absolute level delta no greater than 0.50 dB;
- mean RMS delta across active critical bands no greater than 1.00 dB;
- aggregate aligned error no greater than -30 dB relative to the reference;
- aggregate excess energy from 12-20 kHz no greater than -50 dB relative to
  total reference energy.

Acceptance follows the suite averages. Per-scene outliers are also reported at
1 dB level delta, 2 dB critical-band RMS delta, -20 dB aligned error or -35 dB
high-band excess, but they do not silently change the aggregate limits.

## Method

The deterministic renderer executes thirty-three existing diagnostic scenes
plus a five-voice `Baseline Pad` scene with a full ascending and descending
MIDI CC1 sweep. Each six-second scene is rendered at 48 kHz without
normalization in four configurations:

1. host rate plus bounded fast math, which is the distributed profile;
2. host rate plus precise `libm`, isolating sample-rate reduction;
3. four times plus bounded fast math, isolating elementary-function error;
4. four times plus precise `libm` and 127-tap decimation, the reference.

Before temporal subtraction, cross-correlation estimates integer and 0.05
sample fractional lag so the reference FIR's approximately 15.75-sample group
delay is not counted as a timbral difference. Spectral comparison uses
4096-sample Hann windows and approximate critical bands. The comparator also
reports raw and gain-matched temporal error, correlation, maximum band delta
and 12-20 kHz excess for every scene.

## 2026-08-27 host-rate result

| Candidate against four-times precise reference | Level | Critical bands | Aligned error | 12-20 kHz excess | Decision |
| --- | ---: | ---: | ---: | ---: | --- |
| Host rate + fast math | 0.372 dB | 4.596 dB | -6.044 dB | -38.120 dB | Not accepted |
| Host rate + precise math | 0.372 dB | 4.596 dB | -6.044 dB | -38.121 dB | Not accepted |
| Four times + fast math | 0.000 dB | 0.000 dB | -67.438 dB | -73.442 dB | Accepted |

The near-identical first two rows isolate host-rate processing as the cause.
The third row establishes that bounded math remains far inside every admitted
limit when topology and internal rate are held constant.

Twenty-seven of thirty-four host-rate scenes cross at least one outlier
threshold. The largest critical-band differences occur at the 1% and 99%
pulse-width endpoints. Wheel Filter, hard sync, audio-rate PWM, LFO routes and
the CC1 Baseline Pad stress scene also remain outside the admitted window.
Baseline Pad stays finite and unclipped throughout its full wheel sweep, so the
former catastrophic failure is fixed even though its 1x/reference timbral
delta is not yet accepted.

Those selective- and two-times experiments established the tradeoff while the
five cards still had to run serially on one audio core. RackForge's later
host-owned per-voice worker contract changed that constraint without changing
the plugin or duplicating shared control state. The released profile now keeps
the oscillators/mixer at four times and the held/interpolated nonlinear
filter/final VCA at two times the host rate. A targeted factory 2-1 comparison
places all five broad spectral bands within 0.03 dB of the complete four-times oracle; the former
host-rate path was approximately 6.2 dB low in the 3-8 kHz band. A complete
suite and Raspberry Pi stress rerun remain required before extending that
targeted result into a universal indistinguishability claim. Portable
arithmetic and solver optimizations are admitted by a separate gate documented in
[`REALTIME_CIRCUIT_BUDGET.md`](REALTIME_CIRCUIT_BUDGET.md).

## Reproduction

From the repository root on PowerShell:

```powershell
.\tools\compare-portable-reference.ps1 -OutputRoot artifacts\portable-reference-comparison
```

The command refuses to overwrite an earlier result. Each comparison directory
contains `comparison.json` and `REPORT.md`. These measurements are an
engineering acceptance gate, not proof from a controlled listening ABX test or
measurements of an original populated instrument.
