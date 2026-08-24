# RF-5 architecture

RF-5 keeps the public controls, per-voice circuit model, polyphonic allocator
and RackForge adapter in separate crates. This prevents UI assumptions or a
temporary DSP approximation from becoming the architecture by accident.

```text
RackForge MIDI + automation
            |
            v
  rackforge-rf-5 adapter
            |
            v
      rf-5-dsp engine
     /      |       |       |          \
allocator  MIDI   state   common LFO  pink noise
     |                       |          |
     +-----------+-----------+----------+
                 v
 five independent rf-5-voice instances
       |
       v
 voice mix -> output calibration -> stereo host output
```

The intended final voice path is:

```text
VCO A ----\
           mixer -> four-pole VCF -> VCA -> voice output
VCO B ----/          ^                ^
  |                  |                |
  +-- sync ----------+        amplifier envelope
  +-- Poly Mod ------+
filter envelope -----+
```

The first audible implementation is deliberately named a baseline. Individual
blocks are replaced only after their source, parameter mapping, numerical
model and acceptance test are recorded.

The frozen hardware boundary and first control-system contract are documented
in [`fidelity/REFERENCE_HARDWARE.md`](fidelity/REFERENCE_HARDWARE.md) and
[`fidelity/CONTROL_SCANNING_AND_CV.md`](fidelity/CONTROL_SCANNING_AND_CV.md).
The active oscillator candidate and its remaining uncertainty are recorded in
[`fidelity/VCO_MODEL.md`](fidelity/VCO_MODEL.md).
Its control-voltage law and calibration hypotheses are isolated in
[`fidelity/TUNING_MODEL.md`](fidelity/TUNING_MODEL.md).
The shared modulation topology, implemented Wheel Mod routes and remaining
depth/range uncertainty are recorded in
[`fidelity/LFO_AND_WHEEL_MOD_MODEL.md`](fidelity/LFO_AND_WHEEL_MOD_MODEL.md).
The shared MM5837-class source, its SD334 pinking stage and the current mixer
boundary are recorded in
[`fidelity/NOISE_AND_MIXER_MODEL.md`](fidelity/NOISE_AND_MIXER_MODEL.md).
