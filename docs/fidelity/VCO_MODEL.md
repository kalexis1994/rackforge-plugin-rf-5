# Dual VCO model

## Source-backed topology

TM1000D.2 section 2-4 and its CEM3340 appendix establish the Revision 3 voice
oscillator boundary:

- two CEM3340 oscillators per voice and one common CEM3340 LFO;
- a nominal 1 V/octave frequency scale;
- full 14-bit CV resolution for individual oscillator pitch;
- saw and pulse outputs for oscillator A;
- saw, triangle and pulse outputs for oscillator B;
- oscillator B can hard-synchronize oscillator A;
- selected waveforms are additive, not mutually exclusive.

The automatic tune system and low-frequency/keyboard modes are active parts of
this candidate.

## Active numerical model

Each physical voice owns two independent phase accumulators. They are seeded at
different phases, are not reset by note-on, and continue advancing after the
amplifier envelope becomes inactive. This preserves the free-running behaviour
of analog VCOs and avoids deterministic, phase-locked attacks.

Saw and pulse discontinuities use a PolyBLEP correction spanning two host
samples at the four-times internal rate. This wider window is required for the
documented 1%/99% pulse-width endpoints, where the two edges can share one
short reconstruction interval. The complete dual-VCO,
mixer, nonlinear filter and final-VCA path runs at four times the host sample
rate. A unity-DC, 127-tap low-pass then reconstructs one host-rate sample
instead of using the former four-sample box average. Triangle is generated
directly from phase because it is continuous, while its remaining
corner-bandwidth error is tracked below. The reconstruction boundary and its
measured numerical response are isolated in
[`OVERSAMPLING_AND_DECIMATION.md`](OVERSAMPLING_AND_DECIMATION.md).

Each seven-bit PULSE WIDTH pot maps monotonically to the owner's manual's
approximately 1-99% duty-cycle span, with the nearest code to 50% at the
physical midpoint. Wheel Mod and Poly Mod are summed after that panel law at
the shared board CV node. They may therefore overdrive a pulse to exactly 0%
or 100%, where the CEM3340 output becomes steady DC and stops generating sync
edges until modulation returns it to a finite pulse.

Hard sync is resolved at the same four-times internal rate but is not a generic
phase reset. The Revision 3 voice board takes oscillator B's pulse output before
its audio-selection switch, gates it through a 4016 and capacitively couples
both edges into oscillator A's CEM3340 hard-sync input. RF-5 retains each edge's
fractional position inside the internal sample, advances A exactly to that
instant, applies the polarity-dependent reflection and then advances the
remainder. Two edges inside one interval remain ordered. A rising B pulse
creates the positive sync polarity and can reverse A only while A's triangle is
rising; a falling B pulse creates the negative polarity and can reverse A only
while its triangle is falling. RF-5 reflects A onto the opposite triangle
branch at the same instantaneous voltage, which also creates the data-sheet saw
and pulse discontinuities. Sync therefore remains active even when no B
waveform is sent to the audio mixer. Detailed acceptance is documented in
[`HARD_SYNC_MODEL.md`](HARD_SYNC_MODEL.md).

Waveforms sum before their oscillator level. Enabling a second waveform can
therefore raise level and drive later blocks harder; RF-5 does not normalize
the selection count. Saw, triangle and pulse now retain their data-sheet
voltage relationships and the populated board's 150/200 kohm input weighting.
Oscillator B exposes separate physical mixer and Poly Mod voltages. The audio
mixer receives the raw positive-going triangle, while only U451's dedicated
Poly Mod route subtracts the 2.27 V reference; saw and pulse retain their
electrical bias in both paths.

Selecting oscillator B triangle also reproduces the CEM3340's load-dependent
frequency pull. SD431's 150 kohm mixer path loads the finite 65-150 ohm
triangle buffer, which also drives the internal comparator, lowering B by
approximately 0.75-1.73 cents according to its physical output profile. Saw's
buffer isolation and pulse's open-emitter output prevent the same pitch shift
when those waveforms are selected.

## Exposed controls in this block

- oscillator A and B levels;
- oscillator B fine detune;
- oscillator A and B frequency;
- oscillator A saw/pulse and pulse width;
- oscillator B saw/triangle/pulse and pulse width;
- oscillator B low-frequency and keyboard tracking switches;
- oscillator sync.

No public parameter was added in this block. The runtime state schema is
version 11 because the existing normalized PULSE WIDTH values now drive the
source-backed 1-99% panel law instead of the former protective 2-98% clamp.

## Residual uncertainty

This candidate does not yet claim final CEM3340 waveform equivalence. Open
items are:

- waveform curvature and high-frequency rounding at the actual board nodes;
- exact populated-chip PWM threshold, control-current loading and transition
  behavior at the 0/100% DC boundaries;
- analog bandwidth of hard-sync discontinuities after their now-fractional
  placement;
- final calibration of the frequency-knob and B fine-control laws;
- measured component populations, exact drift time evolution and exact
  low-octave tune extrapolation arithmetic;
- populated-unit band-limiting of triangle corners and sync discontinuities
  under extreme modulation.

The active CV-to-frequency mapping and its explicit hypotheses are maintained
separately in [`TUNING_MODEL.md`](TUNING_MODEL.md).
The ten-channel calibration pipeline is documented in
[`AUTOTUNE_MODEL.md`](AUTOTUNE_MODEL.md).
The ten independent post-tune trajectories and their data-sheet magnitude
limits are documented in [`VCO_DRIFT_MODEL.md`](VCO_DRIFT_MODEL.md).
The electrical output limits, board resistor weighting and separate audio/Poly
Mod polarities are documented in
[`VCO_OUTPUT_MODEL.md`](VCO_OUTPUT_MODEL.md).

Numerical spectral sweeps at 44.1, 48, 96 and 192 kHz now pass the -40 dB alias
threshold for saw, square, triangle, 1%/99% pulse and a periodic hard-sync
condition. The candidate still requires legally usable hardware measurements
to bound the remaining waveform curvature and analog sync-transient
hypotheses.
