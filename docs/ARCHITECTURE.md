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
       /     |      \
 allocator  MIDI   state
       |
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
