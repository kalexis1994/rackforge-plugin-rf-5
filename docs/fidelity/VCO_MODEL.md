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

The automatic tune system and low-frequency/keyboard modes are documented but
are not part of this candidate yet.

## Active numerical model

Each physical voice owns two independent phase accumulators. They are seeded at
different phases, are not reset by note-on, and continue advancing after the
amplifier envelope becomes inactive. This preserves the free-running behaviour
of analog VCOs and avoids deterministic, phase-locked attacks.

Saw and pulse discontinuities use PolyBLEP correction. The complete dual-VCO
path runs at four times the host sample rate and is averaged back to the audio
rate. Triangle is generated directly from phase because it is continuous,
while its remaining corner-bandwidth error is tracked below. Hard sync is
resolved at the same four-times internal rate: a wrap from oscillator B resets
oscillator A for the following internal interval.

Waveforms sum before their oscillator level. Enabling a second waveform can
therefore raise level and drive later blocks harder; RF-5 does not normalize
the selection count.

## Exposed controls in this block

- oscillator A and B levels;
- oscillator B fine detune;
- oscillator A and B frequency;
- oscillator A saw/pulse and pulse width;
- oscillator B saw/triangle/pulse and pulse width;
- oscillator B low-frequency and keyboard tracking switches;
- oscillator sync.

The state schema is now version 2 because oscillator A's former baseline
crossfade parameter has become its physical level and thirteen hardware controls
were added.

## Residual uncertainty

This candidate does not yet claim final CEM3340 waveform equivalence. Open
items are:

- output amplitudes, offsets and waveform curvature at the actual board nodes;
- pulse-width transfer limits and behavior at both extremes;
- exact hard-sync edge, reset phase and transient shape;
- final calibration of the frequency-knob and B fine-control laws;
- 14-bit tune-bias interpolation, temperature drift and per-oscillator scale;
- band-limiting of triangle corners and sync discontinuities under extreme
modulation.

The active CV-to-frequency mapping and its explicit hypotheses are maintained
separately in [`TUNING_MODEL.md`](TUNING_MODEL.md).

The candidate becomes accepted only after spectral sweeps at 44.1, 48, 96 and
192 kHz pass the alias threshold and legally usable hardware measurements bound
the remaining waveform and sync hypotheses.
