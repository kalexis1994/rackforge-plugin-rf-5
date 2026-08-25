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
  the documented self-oscillation calibration region through the populated
  0-10 V/200 kohm CEM3320 resonance-control path.
- `Audition - Envelope Punch` uses short filter/amplifier decays so repeated
  notes expose the ten CEM3310 profiles;
- `Audition - Envelope Slow` holds a pad while the independent filter and
  amplifier CEM3310 trajectories evolve through the populated timing network.
- `Audition - Release Switch Off` deliberately stores both Release pots at
  maximum while the V8.1 program bit forces their common minimum, making the
  global switch audible without a UI.
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
- `Audition - Unison Glide Circuit` uses the service manual's medium panel-6
  position so the Q309/CA3280/C376 linear pitch transitions can be heard.
- `Audition - LFO Slow Range` and `Audition - LFO Fast Range` expose two
  widely separated points of the circuit-derived common-LFO sweep with the
  same restrained vibrato routing.
- `Audition - Oscillator B Fine Zero` and `Audition - Oscillator B Fine
  Semitone` use the same two-saw setup at both documented FINE endpoints, so
  the unison start and one-semitone rise can be compared directly.
- `Audition - Pulse Width 1%`, `50%` and `99%` isolate oscillator A's pulse at
  both documented panel endpoints and the nearest stored square-wave code.

These are diagnostic listening conditions rather than emulated factory
patches. Each one temporarily places the modulation wheel at a documented
fixed audition level because RF-5 currently has no front panel. The temporary
level is engine machine state: it is not a public parameter, does not enlarge
or alter the serialized patch format, is cleared by loading a normal program
or state, survives audio-device preparation order, and is replaced by the
first physical MIDI CC1 message.

All twenty-three audition programs are covered by deterministic render probes for
finite output, usable level and bounded headroom. All catalog programs are
also contract-validated, and the filter population is swept at every supported
sample rate.
