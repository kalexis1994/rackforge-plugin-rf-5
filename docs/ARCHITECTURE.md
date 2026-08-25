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
     /      |       |       |          |          |          \
allocator  MIDI   state   common LFO  pink noise  auto-tune  38-cell CV S/H
     |                       |          |          |          |
     +-----------+-----------+----------+----------+----------+
                 v
 five independent rf-5-voice instances
       |
       v
 voice mix -> master VCA -> 4.34 Hz AC coupling -> stereo host output
```

The active per-voice routing is:

```text
VCO A ----\
           dual OTA mixer --\
VCO B ----/                    four-pole VCF -> final VCA -> voice output
common noise OTA ------------/       ^               ^
  |                                  |               |
  +-- sync / Poly Mod ---------------+       amplifier envelope
filter envelope ---------------------+---> Poly Mod bus (inverted)
```

The first audible implementation is deliberately named a baseline. Individual
blocks are replaced only after their source, parameter mapping, numerical
model and acceptance test are recorded.

The frozen hardware boundary and first control-system contract are documented
in [`fidelity/REFERENCE_HARDWARE.md`](fidelity/REFERENCE_HARDWARE.md) and
[`fidelity/CONTROL_SCANNING_AND_CV.md`](fidelity/CONTROL_SCANNING_AND_CV.md).
The active oscillator candidate and its remaining uncertainty are recorded in
[`fidelity/VCO_MODEL.md`](fidelity/VCO_MODEL.md).
Its CEM3340 endpoint limits, board-level waveform weighting and distinct audio
and modulation domains are documented in
[`fidelity/VCO_OUTPUT_MODEL.md`](fidelity/VCO_OUTPUT_MODEL.md).
Its control-voltage law and calibration hypotheses are isolated in
[`fidelity/TUNING_MODEL.md`](fidelity/TUNING_MODEL.md).
The direct, non-programmable R104 path shared by oscillator A and B is isolated
in [`fidelity/MASTER_TUNE_MODEL.md`](fidelity/MASTER_TUNE_MODEL.md).
The ten-channel counter, DAC search and runtime bias tables are documented in
[`fidelity/AUTOTUNE_MODEL.md`](fidelity/AUTOTUNE_MODEL.md).
The independent post-tune motion of all ten VCOs, including its published
magnitude bounds and hypothesized time evolution, is documented in
[`fidelity/VCO_DRIFT_MODEL.md`](fidelity/VCO_DRIFT_MODEL.md).
The unified CPU service cycle, destination map and bounded sample/hold leakage
are documented in
[`fidelity/SAMPLE_HOLD_MODEL.md`](fidelity/SAMPLE_HOLD_MODEL.md).
The Q309/CA3280/C376 Unison Glide path consumes its held common CV after that
distribution step and is documented with the other performance controls.
The shared modulation topology, implemented Wheel Mod routes and remaining
depth/range uncertainty are recorded in
[`fidelity/LFO_AND_WHEEL_MOD_MODEL.md`](fidelity/LFO_AND_WHEEL_MOD_MODEL.md).
The shared MM5837-class source, its SD334 pinking stage and the current mixer
boundary are recorded in
[`fidelity/NOISE_AND_MIXER_MODEL.md`](fidelity/NOISE_AND_MIXER_MODEL.md).
The per-voice Poly Mod bus, independent filter envelope, destination routing
and bounded depth uncertainty are recorded in
[`fidelity/POLY_MOD_MODEL.md`](fidelity/POLY_MOD_MODEL.md).
The active five-profile four-pole filter and ten-profile true-RC envelope candidates are documented in
[`fidelity/FILTER_MODEL.md`](fidelity/FILTER_MODEL.md) and
[`fidelity/ENVELOPE_MODEL.md`](fidelity/ENVELOPE_MODEL.md).
Unison, Glide, pitch wheel and the explicit absence of velocity response are
documented in
[`fidelity/PERFORMANCE_MODEL.md`](fidelity/PERFORMANCE_MODEL.md).
The distinct mixer, per-voice, master and output stages are documented in
[`fidelity/VCA_AND_OUTPUT_MODEL.md`](fidelity/VCA_AND_OUTPUT_MODEL.md).
Deterministic listening scenes that exercise this architecture without a UI
are documented in [`AUDITION_RENDERER.md`](AUDITION_RENDERER.md).
