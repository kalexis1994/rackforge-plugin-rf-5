# A-440 and front-panel Tune model

## Accepted Rev 3 behaviour

The Rev 3 CPU board assigns counter 1 of the 8253 to the reference tone. It
divides the 2.5 MHz clock by 5682, producing 439.985920 Hz rather than replacing
the circuit with an idealized 440.000 Hz oscillator. SD430 then routes that
square wave through the A-440 4016 switch and the populated R4498/C4183
10 kohm/0.1 uF network. R4559 feeds the resulting signal into the same common
node as the five 39 kohm voice inputs, ahead of the master CA3280 and Master
Volume. When deselected, U460 grounds the reference input.

The owner's manual describes TUNE as a momentary, non-programmable operation.
The panel is occupied for approximately two to eight seconds, depending on how
far the oscillators require correction. During the operation the CPU disconnects
Pitch, Master Tune and Unison CV sources from the oscillator measurement path.

## Active reconstruction

`rf_5_dsp::a440` keeps the counter free-running at the exact integer division,
applies the populated first-order RC network and weights its injection by the
39 kohm/20 kohm input-conductance ratio. The existing common output stage then
applies Master Volume, coupling-capacitor response and bounded host scaling.
Turning A-440 off drives zero into the RC model so its stored capacitor voltage
settles as the grounded hardware does.

The engine exposes A-440 as non-program patch-independent machine state. TUNE
is a momentary host control: its stored value is always zero, its queried value
reports whether calibration is busy, and neither a patch nor serialized state
can restore a half-completed operation. The active interval spans two to eight
seconds according to the largest normalized correction in the ten-VCO thermal
bank. Normal voice and reference output are suppressed while the CPU owns the
measurement path; analog voice, envelope and free-running-source state still
advances. Completion rebuilds the 200-byte calibration candidate, captures the
current ten-VCO thermal condition and refreshes all held pitch CVs.

## Isolated uncertainty

The counter ratio, switch topology and populated SD430 component values are
fixed. The absolute circuit-to-host level remains tied to the same unmeasured
gain staging as the voice summer. No published source gives the exact audible
transient at the output while the service algorithm visits each oscillator and
octave; RF-5 deliberately suppresses those internal sweeps rather than inventing
calibration tones. This boundary can be replaced by documented bench audio
without changing the tuning table or reference generator.
