# Portable/reference comparison

## Decision

The current host-rate distributed profile is **not accepted as perceptually
negligible** relative to the four-times fidelity reference. The fast bounded
elementary functions are accepted; removing four-times processing is the
dominant source of the measured difference.

This decision does not mean that the portable profile is unstable or unusable.
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

## 2026-08-27 result

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

Subsequent selective- and two-times experiments confirmed the tradeoff: moving
more of the nonlinear boundary toward the oracle can reduce the render delta,
but the candidates that did so were not real-time safe under five-voice
Raspberry Pi 4 stress. The released profile consequently keeps all modeled
blocks at host rate and retains the four-times version as an offline oracle.
This is an explicit engineering boundary, not a claim that the two profiles are
indistinguishable. Portable arithmetic and solver optimizations are admitted by
a separate fixed portable-to-portable gate documented in
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
