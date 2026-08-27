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
filter envelope ---------------------+---> Poly Mod bus (positive; decay sweeps down)
```

The first audible implementation is deliberately named a baseline. Individual
blocks are replaced only after their source, parameter mapping, numerical
model and acceptance test are recorded.

The frozen hardware boundary and first control-system contract are documented
in [`fidelity/REFERENCE_HARDWARE.md`](fidelity/REFERENCE_HARDWARE.md) and
[`fidelity/CONTROL_SCANNING_AND_CV.md`](fidelity/CONTROL_SCANNING_AND_CV.md).
The active oscillator candidate and its remaining uncertainty are recorded in
[`fidelity/VCO_MODEL.md`](fidelity/VCO_MODEL.md).
The fractional dual-edge synchronization path and its spectral acceptance
matrix are isolated in
[`fidelity/HARD_SYNC_MODEL.md`](fidelity/HARD_SYNC_MODEL.md).
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
The unified CPU service cycle, destination map, finite RC acquisition and
bounded sample/hold leakage are documented in
[`fidelity/SAMPLE_HOLD_MODEL.md`](fidelity/SAMPLE_HOLD_MODEL.md).
That model also preserves V8.1's gate-before-next-pitch-sweep ordering. The
Q309/CA3280/C376 Unison Glide path consumes lowest-key voltage from common S/H
destination 21 after that distribution step; the digital Unison latch remains
separate. Both are documented with the other performance controls.
The shared modulation topology, implemented Wheel Mod routes and remaining
depth/range uncertainty are recorded in
[`fidelity/LFO_AND_WHEEL_MOD_MODEL.md`](fidelity/LFO_AND_WHEEL_MOD_MODEL.md).
The shared MM5837-class source, its SD334 pinking stage and the current mixer
boundary are recorded in
[`fidelity/NOISE_AND_MIXER_MODEL.md`](fidelity/NOISE_AND_MIXER_MODEL.md).
The per-voice Poly Mod bus, independent filter envelope, destination routing
and bounded depth uncertainty are recorded in
[`fidelity/POLY_MOD_MODEL.md`](fidelity/POLY_MOD_MODEL.md).
The active five-profile four-pole filter uses its physical return-capacitor
state in the contractive low-Q region, one Newton correction through the normal
resonant region and a converged nonlinear solve at the extreme high-cutoff
boundary. It and the ten-profile
true-RC envelope candidates are documented in
[`fidelity/FILTER_MODEL.md`](fidelity/FILTER_MODEL.md) and
[`fidelity/ENVELOPE_MODEL.md`](fidelity/ENVELOPE_MODEL.md).
Unison, Glide, pitch wheel and the explicit absence of velocity response are
documented in
[`fidelity/PERFORMANCE_MODEL.md`](fidelity/PERFORMANCE_MODEL.md).
The distinct mixer, per-voice, master and output stages are documented in
[`fidelity/VCA_AND_OUTPUT_MODEL.md`](fidelity/VCA_AND_OUTPUT_MODEL.md).
The single portable real-time path and the non-distributed four-times fidelity
reference are documented in
[`fidelity/OVERSAMPLING_AND_DECIMATION.md`](fidelity/OVERSAMPLING_AND_DECIMATION.md).
Its retained topology, bounded numerical reductions and Raspberry Pi stress
gate are documented in
[`fidelity/REALTIME_CIRCUIT_BUDGET.md`](fidelity/REALTIME_CIRCUIT_BUDGET.md).
Deterministic listening scenes that exercise this architecture without a UI
are documented in [`AUDITION_RENDERER.md`](AUDITION_RENDERER.md).
The five physical voice cards can be dispatched to RackForge-owned workers
without duplicating shared control state. State ownership, frame-exact command
delivery, deterministic mixing and the bit-identical sequential fallback are
documented in [`PARALLEL_RENDER.md`](PARALLEL_RENDER.md).
