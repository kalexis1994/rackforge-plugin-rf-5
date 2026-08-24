# Audition programs without a front panel

RF-5 includes a separate `RF-5 Audition` factory bank so the reconstructed
performance paths can be heard before a graphical interface exists. Select a
program in RackForge's program browser and play normally:

- `Audition - Wheel Vibrato` routes the common triangle LFO to both VCO
  frequency summing nodes;
- `Audition - Wheel PWM` uses both pulse outputs and routes the LFO to both
  pulse-width summing nodes;
- `Audition - Wheel Filter` routes the LFO to the filter cutoff summing node
  with moderate resonance.
- `Audition - Filter Drive` drives all four CEM3320 cells with both VCOs and
  their high-level waveform combinations;
- `Audition - Filter Resonance` places the five physical filter profiles near
  the documented self-oscillation calibration region.

These are diagnostic listening conditions rather than emulated factory
patches. Each one temporarily places the modulation wheel at a documented
fixed audition level because RF-5 currently has no front panel. The temporary
level is engine machine state: it is not a public parameter, does not enlarge
or alter the serialized patch format, is cleared by loading a normal program
or state, survives audio-device preparation order, and is replaced by the
first physical MIDI CC1 message.

The Wheel Mod and filter patches are covered by full-second render tests for
finite output, usable level and distinct audio signatures. All catalog
programs are also contract-validated, and the filter population is swept at
every supported sample rate.
