# RF-5 implementation plan

## Objective

Deliver a self-contained, plug-and-play five-voice instrument whose audible
behaviour is bounded by published circuitry, component data and repeatable
measurements. The existing baseline proves the RackForge path only and is not a
fidelity reference.

## Acceptance rule

Each block carries an uncertainty envelope. A candidate replaces the active
model only when its error remains within the documented threshold across the
entire accepted stimulus set. Conflicting hypotheses remain experiments; they
do not become user-selectable authenticity modes.

## Work blocks

1. **Evidence foundation**
   - Freeze one reference revision and enumerate every board/block.
   - Collect service documentation, schematics, calibration procedures and
     primary component data.
   - Record provenance, redistribution status and confidence in the ledger.
2. **Control and program contract**
   - Map physical ranges, tapers, switches and modulation routings.
   - Define deterministic state and original RF-5 factory programs.
3. **Dual VCO voice core**
   - Model tuning law, waveform geometry, pulse width, oscillator sync, drift and
     per-voice dispersion without aliasing shortcuts becoming audible.
4. **Mixer and Poly Mod**
   - Calibrate oscillator/noise gain staging, overload and audio-rate routes.
   - Verify every source/destination combination and compound routing.
5. **Filter**
   - Derive the four-pole resonant path, keyboard tracking, envelope depth,
     self-oscillation boundary and component/voice spread.
6. **Envelopes and amplifier**
   - Reconstruct attack, decay, sustain and release laws separately for filter
     and amplifier paths, including retrigger and voice-steal behaviour.
7. **Performance system**
   - Implement unison, glide, pitch/mod wheels, sustain and all-notes-off with
     sample-accurate RackForge events; explicitly ignore velocity and
     aftertouch because the reference keyboard does not provide them.
8. **Output and calibration**
   - Match headroom, saturation, DC behaviour and level across single notes,
     five-note chords and unison.
9. **Rust/WebAssembly panel**
   - Recreate the RF-5 physical control language using original graphics.
   - Expose every musical control to RackForge mapping and touch interaction.
10. **Release evidence**
    - Golden renders, parameter sweeps, cross-sample-rate tests, ARM/x86 CI,
      package reproducibility and a published residual uncertainty report.

## Definition of 1.0

- Every front-panel musical control is implemented and mappable.
- Five-voice allocation, unison and performance MIDI are deterministic.
- No known fidelity gap exceeds its agreed uncertainty threshold.
- Factory programs are original, balanced for live use and cover bass, brass,
  keys, pads, leads, sync and Poly Mod sounds.
- The `.rfplugin` contains everything required to play immediately.
