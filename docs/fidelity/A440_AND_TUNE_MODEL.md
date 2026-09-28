# A-440 and front-panel Tune model

## Accepted Rev 3 behaviour

The Rev 3 CPU board assigns counter 1 of the 8253 to the reference tone. It
divides the 2.5 MHz clock by 5682, producing 439.985920 Hz rather than replacing
the circuit with an idealized 440.000 Hz oscillator. While A-440 is selected,
U459 passes that 0/5 V square to TP401; otherwise U460 grounds TP401. SD430
then routes TP401 through R4498 10 kohm into C4183 0.1 uF to ground, and
R4519 20 kohm joins C4183 to the U480 node where the five voice cards meet
through their 25 kohm VOL rheostats and 39 kohm summing resistors, ahead of
the master CA3280 and Master Volume. The branch therefore loads the voices
whether or not the reference is selected.

The owner's manual describes TUNE as a momentary, non-programmable operation.
The panel is occupied for approximately two to eight seconds, depending on how
far the oscillators require correction. During the operation the CPU disconnects
Pitch, Master Tune and Unison CV sources from the oscillator measurement path.

## Active reconstruction

`rf_5_dsp::a440` keeps the counter free-running at the exact integer division
and returns only the mean TP401 voltage over each host sample, placing every
counter transition at its exact fractional position inside that sample; the
selected mean is 2.5 V at every supported rate, and deselection returns
exactly 0 V. Because R4519 couples C4183 to a node the voices also drive, the
network is not solved here: `summing_node` in `output.rs` treats C4183 as the
node's single state and advances it exactly for the held TP401 and voice
inputs, so the A-440 injection is the physical network response and the same
capacitor gives the voice path its approximately +1.2 dB low-frequency shelf
(see [`VCA_AND_OUTPUT_MODEL.md`](VCA_AND_OUTPUT_MODEL.md)). The common output
stage then applies Master Volume, coupling-capacitor response and linear host
scaling. With A-440 off, the grounded TP401 lets C4183 settle as the hardware
does.

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
fixed. The 5 V logic high assumes an unloaded 8253/4016 output on the +5 V
supply, and the absolute circuit-to-host level remains tied to the same
unmeasured host boundary as the voice summer. No published source gives the exact audible
transient at the output while the service algorithm visits each oscillator and
octave; RF-5 deliberately suppresses those internal sweeps rather than inventing
calibration tones. This boundary can be replaced by documented bench audio
without changing the tuning table or reference generator.
