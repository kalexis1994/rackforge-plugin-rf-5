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
  Both its span and absolute anchor now come from SD334 rather than an admitted
  listening guess. C382 is the actual 1 uF timing capacitor; C381 belongs to
  soft sync. R3138's 2.21 Mohm feed establishes 6.787 uA of reference current,
  while the 681 kohm/+15 V and 101 kohm/+5 V paths establish the zero-code
  frequency-control current. The 0-10 V DAC joins them through R3136's 110
  kohm path. R3107, R3108 and R3137 populate the CEM3340 multiplier with 30.1
  kohm, 5.62 kohm and 1.82 kohm respectively.
- RF-5 evaluates the manufacturer's three linked equations rather than
  reducing this network to an ideal one-volt-per-octave approximation:
  `I_OM = 22 V_T/R_T * (1 - I_C R_Z/3 V)`, `V_B = I_OM R_S`,
  `I_EG = I_REF exp(-V_B/V_T)` and
  `f = 3 I_EG/(2 V_CC C_F)`. Thermal voltage cancels from the combined nominal
  law. The unbounded populated circuit spans 9.3753 octaves and requests
  approximately 0.908 uA to 603.2 uA from the exponential generator, placing
  the slow endpoint at approximately 0.09083 Hz.
- The CEM3340 data sheet publishes a 400/570/800 uA minimum/typical/maximum
  timing-capacitor current rather than an exact overload curve. RF-5 therefore
  applies a deliberately narrow, high-order continuous knee at the 570 uA
  typical point. It leaves the specified sub-100 uA accurate region unchanged,
  preserves all 128 distinct panel steps and rounds the nominal fast endpoint
  to approximately 55.8 Hz instead of imposing an invented hard clip.
- Saw, triangle and square are independently summable, but they do not share a
  generic bipolar normalization. The manual states that all raw CEM3340
  outputs are positive-going and that triangle alone must be level-shifted for
  smooth vibrato. SD334 implements that distinction directly: saw remains
  approximately 0-10 V through U377/R3133's 300 ohm/160 kohm path and loaded
  pulse remains 0-13.009 V through U377/R3132's 300 ohm/200 kohm path.
- Triangle alone crosses U377 into U380. R3148/R3147 are equal 100 kohm
  reference/feedback resistors, so U380 applies `2 * V_triangle - 4.97 V`.
  The nominal 0-5 V raw triangle consequently becomes approximately -4.97 to
  +5.03 V before R3131's 160 kohm path. Saw and square therefore produce the
  original upward modulation displacement, while triangle remains almost
  symmetric around the unmodulated pitch/filter/PWM position. All three paths
  are converted to one five-volt/160-kohm current coordinate before U378.
- The LFO and noise sources pass through the two profiled, unlinearized halves
  of common CA3280 U378. The 0-10 V source-mix CV drives grounded-base 2N4250
  Q307 through 8.2 kohm and drives Q309 in the opposite direction against the
  10k/20k divider's 10 V Thevenin source.
- Each U378 half uses the CA3280 data-sheet 16 mS/mA small-signal slope and
  0.82 peak-output-current ratio. The 160k/330-ohm LFO input, 20k/330-ohm
  noise input and shared R3113 10 kohm output load produce W-MOD circuit volts
  directly; the service balance trims retain zero feed-through at zero input.
- One LFO unit represents five circuit volts in R3131's 160 kohm current
  coordinate; it is a unit conversion, not a bipolar source assumption. Pink noise uses
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

The schematic and CEM3340 equations now close the absolute nominal LFO law;
there is no remaining free 20 Hz anchor. The slow endpoint is above the data
sheet's preferred 50 nA accurate-current boundary. At the opposite end, the
raw populated equation asks for approximately 603.2 uA, just above the 570 uA
typical timing-current ceiling but within its published 400-800 uA population
range. The data sheet says the oscillator flattens in its uppermost octaves but
does not publish the knee shape. The isolated continuous knee is consequently
a bounded device-overload candidate: a populated-unit timing sweep can refine
the last fast codes without changing the reconstructed reference, scale,
timing-capacitor or DAC networks. Wheel Mod destination ratios, U380 triangle
gain and the W-MOD source voltage are circuit-derived. Populated-unit
measurements can refine the transistor/OTA population without restoring a host
normalization boundary.

The original Wheel Mod source-mix control now current-mixes the LFO with the
shared MM5837-class noise candidate through its physical CA3280 rather than a
generic arithmetic blend. Populated transistor temperature, MM5837 rail
excursions and CA3280 matching remain bounded candidates. The noise circuit
and spectral assumptions are documented separately in
`NOISE_AND_MIXER_MODEL.md`.

## Acceptance tests

- the frequency mapping is monotonic and exposes 128 distinct panel steps;
- the populated scale network produces the 9.3753-octave unbounded sweep;
- the 2.21 Mohm reference feed, fixed-current inputs and 1 uF timing capacitor
  reproduce approximately 0.908 uA/0.09083 Hz at panel minimum and request
  approximately 603.2 uA at panel maximum;
- the published typical timing-current ceiling leaves the accurate region
  unchanged, continuously rounds only the fastest codes and produces an
  approximately 55.8 Hz nominal endpoint;
- square-wave high and zero intervals are equal within one sample and never
  become negative;
- simultaneously selected waveforms sum on one shared bus;
- saw and pulse retain their positive-going DC displacement while U380 alone
  level-shifts triangle to approximately -4.97/+5.03 V;
- source amplitudes follow the accepted CEM3340 voltages, 4016 on-resistance,
  U380 triangle conditioning, loaded pulse output and SD334 160k/200k paths;
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
