# LFO and Wheel Mod model

## Accepted hardware contract

The Rev 3 technical manual identifies one common CEM3340 LFO in addition to
the ten audio oscillators. It is not keyboard controlled and is not restarted
by note events. Saw, triangle and square outputs pass through independent
switches to the common modulation bus, so the selected shapes are additive.
The square wave has a fixed 50% duty cycle.

The Wheel Mod service test verifies five switch destinations: oscillator A
frequency, oscillator B frequency, oscillator A pulse width, oscillator B
pulse width and filter cutoff. MIDI CC1 represents the physical modulation
wheel in RF-5.

## Active candidate

- One engine-owned, free-running phase is evaluated once per output sample and
  distributed to every active and inactive voice.
- Frequency follows an exponential mapping from the scanned 7-bit panel value.
  Its span is no longer a free 0.08-20 Hz guess: the populated 110 kohm
  frequency-CV input versus the CEM3340's standard 100 kohm, one-volt-per-octave
  input and the 0-10 V DAC range establish 9.0909 octaves, or approximately
  545.3:1. The 20 Hz upper anchor remains an isolated calibration hypothesis,
  which currently places the lower endpoint near 0.0367 Hz.
- Saw, triangle and square are independently summable. Their AC-centered
  render now preserves the nominal CEM3340/SD334 current-domain relationship:
  saw is the 1.0 reference, triangle is 0.5 and pulse is approximately 1.176.
  The last figure combines the accepted 14.7 V clamped pulse excursion with
  its 200 kohm path versus the saw's nominal 10 V and 160 kohm path.
- The LFO and noise sources pass through the two profiled, unlinearized halves
  of common CA3280 U378. One 7-bit source-mix CV moves their bias currents in
  opposite directions; the service balance trims are represented as zero
  feed-through at zero input.
- Wheel Mod amount is the passive/live performance level after that dual-OTA
  source and is not stored in a program.
- Enabled frequency destinations currently span a candidate maximum of one
  octave at full wheel and unit waveform amplitude.
- Enabled pulse-width destinations span a candidate normalized depth of 0.48.
- The filter destination spans a candidate 4.5 octaves on the CEM3320
  exponential control-voltage path.
- Three diagnostic factory programs temporarily establish a known wheel
  position for vibrato, pulse-width and filter auditions. This override is
  deliberately not serialized; incoming MIDI CC1 replaces it immediately.

## Bounded uncertainty

The manual provides qualitative slow/faster-ramp checks but no absolute LFO
frequency endpoints. RF-5 therefore accepts the circuit-derived sweep width
while isolating the 20 Hz upper anchor in the LFO module rather than treating
it as measured hardware fact. A populated-unit timing measurement can replace
that one anchor without changing the control law. The three Wheel Mod depth
constants are similarly isolated calibration hypotheses.

The original Wheel Mod source-mix control now crossfades the LFO with the
shared MM5837-class noise candidate through its physical CA3280 rather than a
generic arithmetic blend. Populated-device matching and normalized overload
remain candidates. The noise circuit and spectral assumptions are documented separately in
`NOISE_AND_MIXER_MODEL.md`.

## Acceptance tests

- the frequency mapping is monotonic, exposes 128 distinct panel steps and
  spans the circuit-derived 9.0909 octaves;
- square-wave positive and negative intervals are equal within one sample;
- simultaneously selected waveforms sum on one shared bus;
- source amplitudes follow the accepted CEM3340 voltage and SD334 resistor
  ratios instead of ideal equal-amplitude shapes;
- source-mix endpoints completely isolate the opposite OTA half, intermediate
  settings remain complementary and zero input has no balance offset;
- silence and note events do not stop or retrigger the LFO;
- CC1 changes the render when a documented destination is enabled;
- audition wheel state is cleared by CC1, normal program loads and state loads;
- all supported sample rates remain finite and bounded.

Primary evidence: Sequential Circuits technical manual TM1000D.2, sections
2-2, 2-4 and service test 4-7. Provenance and hash are recorded in
`SOURCE_LEDGER.md`.
