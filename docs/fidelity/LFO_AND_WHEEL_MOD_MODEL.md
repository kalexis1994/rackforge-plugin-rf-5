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
  which currently places the lower endpoint near 0.0367 Hz. SD334's populated
  0.1 uF C381 timing capacitor and the CEM3340 equation
  `f = 3 I_EG / (2 V_CC C_F)` make the same candidate equivalent to 20 uA at
  the upper endpoint and 36.7 nA at the lower endpoint; the latter is slightly
  below the data sheet's preferred 50 nA accurate-range boundary.
- Saw, triangle and square are independently summable. Their AC-centered
  render now preserves the nominal CEM3340/SD334 current-domain relationship:
  saw and triangle are both the 1.0 reference, while pulse is approximately
  1.041. U380's equal 100 kohm input and feedback resistors double the raw 5 V
  triangle span around its reference before its 160 kohm input path, matching
  the saw's 10 V span and 160 kohm path. The pulse figure solves the CEM3340's
  published high-current output equation against SD334's populated 10 kohm
  pull-down to ground, producing a 13.009 V excursion before its 200 kohm path.
- The LFO and noise sources pass through the two profiled, unlinearized halves
  of common CA3280 U378. The 0-10 V source-mix CV drives grounded-base 2N4250
  Q307 through 8.2 kohm and drives Q309 in the opposite direction against the
  10k/20k divider's 10 V Thevenin source.
- Each U378 half uses the CA3280 data-sheet 16 mS/mA small-signal slope and
  0.82 peak-output-current ratio. The 160k/330-ohm LFO input, 20k/330-ohm
  noise input and shared R3113 10 kohm output load produce W-MOD circuit volts
  directly; the service balance trims retain zero feed-through at zero input.
- One LFO unit represents the nominal 5 V saw half-excursion. Pink noise uses
  the MM5837's guaranteed 12 Vpp logic separation before SD334's already
  modeled 100k/47k low-pass gain.
- Wheel Mod amount is the passive/live performance level after that dual-OTA
  source and is not stored in a program.
- The five destination switches no longer multiply three unrelated depth
  guesses or require a normalized-bus voltage anchor. They consume U378's
  reconstructed voltage and follow the populated SD334 networks: 182 kohm/100 kohm for
  oscillator frequency, 15 kohm/100 kohm followed by 100 kohm/52.3 kohm for
  pulse width, and 13.3 kohm/100 kohm for filter cutoff.
- One volt at R3113 produces approximately 6.593 oscillator semitones, 0.697
  normalized pulse-width units and 7.519 filter octaves. Actual depth now
  follows the selected waveform, the Q307/Q309 currents and U378 saturation;
  pulse width and cutoff remain naturally limited by their later physical
  model boundaries.
- Three diagnostic factory programs temporarily establish a known wheel
  position for vibrato, pulse-width and filter auditions. This override is
  deliberately not serialized; incoming MIDI CC1 replaces it immediately.

## Bounded uncertainty

The manual provides qualitative slow/faster-ramp checks but no absolute LFO
frequency endpoints. RF-5 therefore accepts the circuit-derived sweep width
and populated timing capacitor while isolating the 20 Hz/20 uA upper anchor in
the LFO module rather than treating it as measured hardware fact. A populated-
unit timing measurement can replace that one anchor without changing the
control law. The lowest approximately 0.45 octave lies below the CEM3340 data
sheet's preferred 50 nA accurate-current range and is correspondingly more
device-sensitive. Wheel Mod destination ratios, U380 triangle gain and the
W-MOD source voltage are now circuit-derived. Populated-unit measurements can
refine the transistor/OTA population without restoring a host normalization
boundary.

The original Wheel Mod source-mix control now current-mixes the LFO with the
shared MM5837-class noise candidate through its physical CA3280 rather than a
generic arithmetic blend. Populated transistor temperature, MM5837 rail
excursions and CA3280 matching remain bounded candidates. The noise circuit
and spectral assumptions are documented separately in
`NOISE_AND_MIXER_MODEL.md`.

## Acceptance tests

- the frequency mapping is monotonic, exposes 128 distinct panel steps and
  spans the circuit-derived 9.0909 octaves;
- the populated 0.1 uF timing capacitor maps the isolated endpoint candidate
  to 20 uA at maximum and approximately 36.7 nA at minimum;
- square-wave positive and negative intervals are equal within one sample;
- simultaneously selected waveforms sum on one shared bus;
- source amplitudes follow the accepted CEM3340 voltage, U380's 2x triangle
  conditioning, loaded pulse output and SD334 resistor ratios instead of raw
  chip amplitudes;
- source-mix endpoints completely isolate the opposite OTA half, intermediate
  Q307/Q309 currents move monotonically in opposite directions and zero input
  has no balance offset;
- a nominal selected saw produces the finite R3113 voltage predicted by the
  populated input divider, CA3280 current limit and 10 kohm load;
- oscillator, pulse-width and filter depths retain the populated SD334
  resistor ratios while consuming the reconstructed W-MOD voltage directly;
- silence and note events do not stop or retrigger the LFO;
- CC1 changes the render when a documented destination is enabled;
- audition wheel state is cleared by CC1, normal program loads and state loads;
- all supported sample rates remain finite and bounded.

Primary evidence: Sequential Circuits technical manual TM1000D.2, sections
2-2, 2-4 and service test 4-7. Provenance and hash are recorded in
`SOURCE_LEDGER.md`.
