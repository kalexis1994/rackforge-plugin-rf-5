# One-edge conventional hard-sync model

## Hardware contract

Revision 3 routes oscillator B's pulse output through U446, one 4016 switch,
independently of the audio waveform switches. SD431 leaves oscillator A's
CEM3340 hard-sync pin 6 explicitly unconnected. Instead, C4107 (200 pF),
R4296 (47 kohm), R4297 (10 kohm) and Q401 (2N4250) copy the CEM3340 data
sheet's Figure 5 external circuit: the selected falling pulse edge drives a
negative transient into the PNP base while Q401 acts between triangle output
pin 10 and the threshold network on pin 9. The opposite positive transient is
suppressed.

Curtis labels this topology "conventional hard sync" and states that it
produces the same waveforms as conventionally synchronized sawtooth
oscillators. It is distinct both from direct bidirectional pulses on hard-sync
pin 6 and from the separate soft-sync method of applying small negative pulses
only to threshold pin 9.

## Active numerical model

Oscillator B reports at most one falling-pulse event per four-times internal
sample. The event retains its normalized fractional offset, including a fall
that occurs after phase wrap. Oscillator A advances to that instant, starts a
new cycle at its lower endpoint and then advances the remaining fraction.
Invalid offsets are bounded and cannot poison oscillator phase. B continues
producing the event when none of its waveforms is selected for audio.

Saw and pulse edges use the same two-host-sample PolyBLEP window as unsynced
operation. The complete voice path is reconstructed by the 127-tap FIR after
the nonlinear filter and final VCA.

## Acceptance tests

- the negative external pulse resets either A triangle branch to cycle start;
- a falling B pulse reports the analytically known fractional crossing;
- B's rising edge and phase wrap produce no second sync event;
- a falling edge after wrap retains its correct fractional position;
- segmented advancement matches the analytical reset phase;
- invalid offsets remain finite and bounded;
- sync remains independent of oscillator B's audio selection;
- periodic hard-sync renders stay below -40 dB non-harmonic energy at three
  pitch regions for 44.1, 48, 96 and 192 kHz host rates.

## Residual uncertainty

The board routing, one-edge polarity, unused pin 6 and manufacturer Figure 5
topology are source-backed. The event placement and alias bound are numerically
verified. C4107 and its populated resistor/transistor network bound the pulse
to a short analog transient, but the exact Q401 switching trajectory, pin-9/
pin-10 interaction, populated oscillator bandwidth and resulting microsecond
pulse shape remain unmeasured and therefore candidate-level.
