# Master Tune model

## Accepted hardware path

The Rev 3 technical manual identifies R104 MASTER TUNE as a direct analog
control. It is neither scanned by the CPU nor stored in a program. During the
automatic-tune measurement its 4016 switch opens so it cannot contaminate the
ten oscillator bias tables. During normal playing it reaches both oscillator
master summers and does not reach the filter path.

Schematic SD334 fixes the populated nominal network:

- R104 is a 100 kohm linear potentiometer from the +5 V analog rail to ground;
- its wiper feeds the first inverting summer through R377, 1 Mohm;
- R378, 100 kohm, gives that stage a nominal gain magnitude of 0.1;
- the matched 100 kohm input/feedback pair in each final A/B master summer has
  unity gain;
- the CEM3340 pitch law is one volt per octave, or twelve semitones per volt.

RF-5 evaluates the loaded potentiometer rather than treating it as an ideal
unloaded slider. At normalized position `u`, its Thevenin source is
`5u` volts in series with `100k * u * (1-u)` ohms. Loading that source with
R377 creates a small, deterministic asymmetry around the centre detent. RF-5
subtracts the centre value so position 0.5 remains concert pitch, producing
approximately -2.927 semitones at 0 and +3.073 semitones at 1.

## State and routing boundary

MASTER TUNE is appended to the public contract so all earlier parameter
indices remain stable. It is continuous, sample-accurate host state and bypasses
the emulated 24-pot scan loop. Program recall preserves it. States written
before its introduction migrate to the centre detent.

The control is applied after the automatic-tune table and equally to oscillator
A and B. It does not alter oscillator-B fine detune, Scale Mode, filter keyboard
tracking, pitch-wheel state or the stored program codec.

## Bounded uncertainty

The topology, nominal component values, direct/non-programmable status and
one-volt-per-octave destination are accepted from the original manufacturer
manual. Switch on-resistance, resistor tolerance, rail error, potentiometer
end resistance and a populated instrument's mechanical centre are not measured.
They are deliberately excluded rather than represented as random per-note
error; the existing per-VCO drift and calibration models own that variability.

## Acceptance tests

- the centre detent contributes exactly zero relative pitch;
- both endpoints exceed the manual's stated correction of more than one
  semitone flat or sharp;
- the complete transfer is finite, continuous and monotonic;
- the non-programmable value bypasses the CPU control scanner;
- program recall preserves it;
- old 60-parameter state migrates to the centre detent.

Primary evidence: original owner's manual control classification and initial
tuning instructions; TM1000D.2 sections 2-3/2-13 and schematic SD334. Provenance
is recorded in `SOURCE_LEDGER.md`.
