# Bipolar hard-sync model

## Hardware contract

Revision 3 routes oscillator B's pulse output to a 4016 independently of the
audio waveform switches. The following capacitor converts both pulse edges
into opposite-polarity transients at oscillator A's CEM3340 hard-sync input.
The CEM3340 behavior is direction-sensitive: a positive transient reverses a
rising triangle and a negative transient reverses a falling triangle. It is
therefore neither a conventional phase reset nor a one-edge digital sync.

## Active numerical model

Oscillator B reports up to two ordered events per four-times internal sample.
Each event carries its physical polarity and a normalized fractional offset.
Oscillator A advances to the event, reflects onto the opposite triangle branch
at the same voltage when polarity and direction agree, and then advances the
remaining fraction. Invalid offsets are bounded and cannot poison oscillator
phase. B continues producing sync events when none of its waveforms is selected
for audio.

Saw and pulse edges use the same two-host-sample PolyBLEP window as unsynced
operation. The complete voice path is reconstructed by the 127-tap FIR after
the nonlinear filter and final VCA.

## Acceptance tests

- positive and negative events reflect only the matching triangle direction;
- event offsets match analytically known pulse crossings;
- two crossings in one internal sample remain ordered;
- segmented advancement matches the analytical reflected phase;
- invalid or reversed offsets remain finite and bounded;
- sync remains independent of oscillator B's audio selection;
- periodic hard-sync renders stay below -40 dB non-harmonic energy at three
  pitch regions for 44.1, 48, 96 and 192 kHz host rates.

## Residual uncertainty

The board routing, edge polarity and CEM3340 direction rule are source-backed.
The digital event placement and alias bound are numerically verified. The
capacitor/4016 transient shape, populated oscillator bandwidth and circuit
voltage-to-host scaling remain unmeasured and therefore candidate-level.
