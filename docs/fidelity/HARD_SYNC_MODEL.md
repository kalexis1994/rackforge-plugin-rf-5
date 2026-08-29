# Conventional saw-reset hard-sync model

## Hardware contract

Revision 3 routes oscillator B's saw output through U446, one 4016 switch,
independently of the audio waveform switches. SD431 leaves oscillator A's
CEM3340 hard-sync pin 6 explicitly unconnected. Instead, C4107 (200 pF),
R4296 (47 kohm), R4297 (10 kohm) and Q401 (2N4250) copy the CEM3340 data
sheet's Figure 5 external circuit: the master's positive-going saw reset is
AC-coupled into the PNP base while Q401 acts between triangle output pin 10
and the threshold network on pin 9. This is not oscillator B's PWM-comparator
edge, so changing B pulse width cannot move or disable the sync clock.

Curtis labels this topology "conventional hard sync" and states that it
produces the same waveforms as conventionally synchronized sawtooth
oscillators. It is distinct both from direct bidirectional pulses on hard-sync
pin 6 and from the separate soft-sync method of applying small negative pulses
only to threshold pin 9.

## Active numerical model

Oscillator B reports at most one saw-reset event per active oscillator sample.
The event retains its normalized fractional offset. Oscillator A
advances to that instant, starts a new cycle at its lower endpoint and then
advances the remaining fraction.
Invalid offsets are bounded and cannot poison oscillator phase. B continues
producing the event when none of its waveforms is selected for audio.

The reset remains a discontinuity, as it is in the conventional analog
hard-sync circuit. RF-5 does not place an arbitrary multi-sample smoothing
envelope around it: that approximation produced two audible attack clicks
when the factory Sync I sweep crossed high oscillator ratios in the portable
host-rate profile. Fractional phase placement supplies sub-sample timing while
the existing band-limited oscillator outputs and the fidelity reference's
four-times reconstruction bound numerical folding without changing the reset
trajectory.

## Acceptance tests

- the negative external pulse resets either A triangle branch to cycle start;
- B's saw reset reports the analytically known fractional crossing;
- changing B pulse width, including its 0/100% DC endpoints, does not alter
  the saw-derived sync clock;
- segmented advancement matches the analytical reset phase;
- invalid offsets remain finite and bounded;
- sync remains independent of oscillator B's audio selection;
- periodic hard-sync renders stay below -40 dB non-harmonic energy at three
  pitch regions for 44.1, 48, 96 and 192 kHz host rates.

## Residual uncertainty

The board routing, saw source, unused pin 6 and manufacturer Figure 5
topology are source-backed. The event placement and alias bound are numerically
verified. C4107 and its populated resistor/transistor network bound the pulse
to a short analog transient, but the exact Q401 switching trajectory, pin-9/
pin-10 interaction, populated oscillator bandwidth and resulting microsecond
pulse shape remain unmeasured and therefore candidate-level.
