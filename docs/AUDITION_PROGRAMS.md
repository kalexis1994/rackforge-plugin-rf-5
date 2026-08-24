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
- `Audition - Envelope Punch` uses short filter/amplifier decays so repeated
  notes expose the ten CEM3310 profiles;
- `Audition - Envelope Slow` holds a pad while the independent filter and
  amplifier RC trajectories evolve.
- `Audition - CA3280 Drive` sends high-level five-note chords through all ten
  oscillator mixer halves, five final VCAs, the voice summer and master VCA;
- `Audition - Common Noise VCA` mutes both oscillators and exposes the one
  shared noise-level OTA feeding all five filter noise inputs.
- `Audition - Poly Mod Oscillator B` keeps oscillator B out of the audio mixer
  while its triangle drives oscillator A through the five unlinearized Poly
  Mod amount VCAs;
- `Audition - Poly Mod Filter Envelope` produces descending resonant sweeps
  through the five linearized envelope amount paths;
- `Audition - Wheel Noise Filter` selects the noise endpoint of common U378
  and routes the physical wheel output to filter cutoff.
- `Audition - Bipolar Hard Sync` removes oscillator B from the audio mixer but
  keeps its pulse output connected to oscillator A's physical sync circuit, so
  both edge polarities and the resulting direction reversals can be heard.
- `Audition - Unison Low Note` exposes the five simultaneous voices, low-note
  priority and legato pitch changes without restarting either envelope.
- `Audition - LFO Slow Range` and `Audition - LFO Fast Range` expose two
  widely separated points of the circuit-derived common-LFO sweep with the
  same restrained vibrato routing.

These are diagnostic listening conditions rather than emulated factory
patches. Each one temporarily places the modulation wheel at a documented
fixed audition level because RF-5 currently has no front panel. The temporary
level is engine machine state: it is not a public parameter, does not enlarge
or alter the serialized patch format, is cleared by loading a normal program
or state, survives audio-device preparation order, and is replaced by the
first physical MIDI CC1 message.

All sixteen audition programs are covered by deterministic render probes for
finite output, usable level and bounded headroom. All catalog programs are
also contract-validated, and the filter population is swept at every supported
sample rate.
