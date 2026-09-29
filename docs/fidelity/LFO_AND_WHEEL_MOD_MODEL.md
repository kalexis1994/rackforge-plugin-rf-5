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
  listening guess. C382 is the actual timing capacitor, marked ".1 mylar 5%"
  (0.1 uF); C381 belongs to soft sync. R3138's 2.21 Mohm feed establishes
  6.787 uA of reference current. R3135 (487 kohm 1% from +15 V, not +5 V)
  supplies the only fixed SUM current, 30.80 uA. R3110 is not a SUM input: it
  runs 10 kohm from +15 V to PW MOD and, with R3111's 2 kohm, holds the
  comparator at 2.5 V for the fixed 50% square. The 1/12 V-per-code DAC joins
  the SUM node through R3136's 110 kohm path. R3107, R3108 and R3137 populate
  the CEM3340 multiplier with 30.1 kohm, 5.62 kohm and 1.82 kohm
  respectively.
- RF-5 evaluates the manufacturer's three linked equations rather than
  reducing this network to an ideal one-volt-per-octave approximation:
  `I_OM = K V_T/R_T * (1 - I_C R_Z/V_M)`, `V_B = I_OM R_S`,
  `I_EG = I_REF exp(-V_B/V_T)` and
  `f = 3 I_EG/(2 V_CC C_F)`. Thermal voltage cancels from the combined nominal
  law. The data sheet gives the tempco-generator factor `K` as a typical 22
  and the multiplier's internal reference `V_M` as a typical 3.0 V. It expects
  `R_Z` to be trimmed +/-20% to set a scale, and the LFO has no such trim, so
  the chip's own values set both the LFO's scale and its offset. RF-5 takes them
  from the one Rev 3 unit measured across the panel: the Synthmania factory
  recordings give stable LFO fundamentals of 0.0785 Hz for 4-4, 1.282 Hz for
  4-7, 5.76 Hz for 1-3, 7.60 Hz for 4-5 and 10.06 Hz for 5-5. At the codes
  the Rev 3's factory tapes store for those programs (16, 66, 93, 98 and 103;
  see ORIGINAL_FACTORY_PROGRAMS.md) a single exponential fits all five within
  +/-6 cents with `K = 23.52` (+7%) and `V_M = 3.115 V` (+3.8%). The populated
  circuit then spans 10.22 octaves from code 0 to code 127 and requests about
  32.2 nA to 38.3 uA from the exponential generator: 0.0322 Hz at code 0,
  3.90 Hz at code 86, 5.45 Hz at code 92, 26.0 Hz at the panel's code-120
  ceiling and 38.3 Hz at code 127. The upper current remains far below the
  data sheet's 400 uA minimum timing-capacitor capability, so RF-5 does not
  invent an overload limiter.
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
  Q307 through 8.2 kohm and drives the LFO-side converter Q308 in the opposite
  direction against the 10k/20k divider's 10 V Thevenin source.
- Each U378 half uses the CA3280 data-sheet 16 mS/mA small-signal slope and
  0.82 peak-output-current ratio. The 160k/330-ohm LFO input, 20k/330-ohm
  noise input and shared output load produce W-MOD circuit volts directly.
  That load is R3113 (10 kohm) in parallel with the 100 kohm MOD wheel pot R2,
  whose wiper feeds the U374 follower: 9.09 kohm. The service balance trims
  retain zero feed-through at zero input.
- One LFO unit represents five circuit volts in R3131's 160 kohm current
  coordinate; it is a unit conversion, not a bipolar source assumption. Pink noise uses
  the MM5837's guaranteed 12 Vpp logic separation before SD334's already
  modeled 100k/47k low-pass gain.
- Wheel Mod amount is the passive/live performance level after that dual-OTA
  source and is not stored in a program.
- The MOD wheel sweeps only part of R2. R2 is a 100 kohm linear pot (BOM
  R-207, Allen-Bradley JA1G040S104UA, taper letter U). It has W-MOD SOURCE at
  one end and ground at the other, and its wiper feeds the U374 follower
  unloaded. R1 and R2 are that same part, on the same wheel (M-200) and
  bracket (M-204). The PITCH wheel's owner's-manual "about a 5th", across its
  +/-15 V track, fixes that assembly at 4.83% of the track either side of the
  detent. The MOD wheel runs from its grounded stop through both halves of that
  travel, so a full wheel places 9.67% of W-MOD on the destinations. The full
  wheel then gives +/-1.34 semitones of LFO vibrato, +/-1.5 octaves of filter
  and +/-14% of pulse width. Those are the ranges SD334's x7.5 filter and
  0.70-per-volt PW gains are sized for, and they agree with the factory
  sheets, where effects "engage" at one quarter to one third of the wheel. The
  former whole-track reading gave +/-13.9 semitones, +/-15.5 octaves and
  +/-144% of pulse width, and put an ordinary vibrato within the wheel's first
  4%.
- MIDI CC1's 128 positions are reconstructed as a continuous wheel trajectory
  with a three-millisecond, sample-rate-invariant dezipper. This transport
  filter removes digital controller steps only; zero, full travel and every
  downstream circuit-derived modulation ratio remain unchanged.
- The five destination switches no longer multiply three unrelated depth
  guesses or require a normalized-bus voltage anchor. They consume U378's
  reconstructed voltage and follow the populated SD334 networks: 182 kohm/100 kohm for
  oscillator frequency, 15 kohm/100 kohm followed by 100 kohm/52.3 kohm for
  pulse width, and 13.3 kohm/100 kohm for filter cutoff. Each switch is a
  CD4016 section whose typical 300 ohm on-resistance sits in series with
  R3103/R3104, R397/R398 and R399, lowering the filter depth by 2.2%, pulse
  width by 2% and oscillator depth by 0.16%.
- One volt at R3113 produces approximately 6.583 oscillator semitones, 0.684
  normalized pulse-width units and 7.353 filter octaves. Actual depth now
  follows the selected waveform, the Q307/Q308 currents and U378 saturation.
- FILT MSUM's first LM348 stage (U367) adds FILT CUTOFF, the Unison keyboard
  (through FILT KBD) and the x7.5 Wheel Mod filter path before its output can
  swing past the op-amp's +/-13 V (`FILTER_SUMMER_SWING_VOLTS`). The engine
  carries only the Wheel Mod share onward, so that rail limits what remains of
  it above a high cutoff setting. The PW summers saturate only after the
  CEM3340 duty is already 0/100%, and the pitch summers never reach their
  rails, so neither summer's swing is modeled.
- SD334's summers are not ideal. R378/C368 (100 kohm/0.1 uF) give the MASTER
  TUNE/PITCH summer U367 a 10 ms lag, and R363/C364, R360/C362 and R357/C363
  (100 kohm/0.01 uF) give the A SUM, B SUM and FILT SUM output stages 1 ms
  each. Wheel Mod's oscillator and filter routes, pitch bend and Glide all
  pass through these one-pole lags; the PW summers have no feedback capacitor,
  so pulse-width modulation is unfiltered. C366/C367 are supply decoupling,
  not signal-path capacitors.
- Three diagnostic factory programs temporarily establish a known wheel
  position for vibrato, pulse-width and filter auditions. This override is
  deliberately not serialized; incoming MIDI CC1 replaces it immediately.

## Bounded uncertainty

The schematic and CEM3340 equations close the absolute nominal LFO law apart
from the device's two internal multiplier values. The parts list (service
manual pp. 75 and 85) confirms R3135 = R-123 = 487k 1% and R3136 = R-159 =
110k 1%, so the network is not the source of any rate difference. `K = 23.52`
and `V_M = 3.115 V` describe the measured Synthmania unit, not a typical
chip. That unit spans about 0.032-26 Hz up to code 120, near the owner's
manual's approximate 0.04-20 Hz. Both lie inside the data sheet's untrimmed
tolerance.

The constants were first fitted to the codes of Sequential's Rev 4
recreation of the programs (19, 66, 90, 95 and 100), which fitted only within
+/-53 cents with `K = 24.9` and `V_M = 3.053 V` and left the Brass vibrato
below disagreeing. The Rev 4 set lowers LFO FREQUENCY by two to five codes in
29 programs to suit its own LFO; the recorded Rev 3 held the tape's codes.

The previous single value, `V_M = 3.135 V`, was fitted to that manual range
and to the one clean vibrato stretch of the same unit's 1-1 Brass recording
(5.25-5.29 Hz at code 92). Under the unit law code 92 is 5.45 Hz, about 60
cents above that short, rougher reading, and Sequential's "approximately
5 Hz" for Brass agrees. A different unit is a
two-constant change; it touches neither the reference, scale, timing
capacitor nor DAC networks.

The slow endpoint's 32.2 nA is below the data sheet's preferred 50 nA
accurate-current boundary but inside its 10 nA generator capability. The
38.3 uA maximum stays inside its tightest exponential-scale accuracy through
100 uA. Wheel Mod destination ratios, U380 triangle gain and
the W-MOD source voltage are circuit-derived. Populated-unit measurements can
refine the transistor/OTA population without restoring a host normalization
boundary.

The original Wheel Mod source-mix control now current-mixes the LFO with the
shared MM5837-class noise candidate through its physical CA3280 rather than a
generic arithmetic blend. Populated transistor temperature, MM5837 rail
excursions and CA3280 matching remain bounded candidates. The noise circuit
and spectral assumptions are documented separately in
`NOISE_AND_MIXER_MODEL.md`.

## Acceptance tests

- the frequency mapping is monotonic and exposes 128 distinct panel steps;
- the populated scale network produces the 10.22-octave code 0-127 sweep;
- the 2.21 Mohm reference feed, R3135's single fixed current and C382's
  0.1 uF reproduce approximately 32.2 nA/0.0322 Hz at code 0, 26.0 Hz at code
  120 and 38.3 uA/38.3 Hz at code 127;
- the five measured Rev 3 program rates, at the factory tapes' codes 16, 66,
  93, 98 and 103, lie within 10 cents of the law, and 1-1 Brass's vibrato
  within 80;
- square-wave high and zero intervals are equal within one sample and never
  become negative;
- simultaneously selected waveforms sum on one shared bus;
- saw and pulse retain their positive-going DC displacement while U380 alone
  level-shifts triangle to approximately -4.97/+5.03 V;
- source amplitudes follow the accepted CEM3340 voltages, 4016 on-resistance,
  U380 triangle conditioning, loaded pulse output and SD334 160k/200k paths;
- source-mix endpoints completely isolate the opposite OTA half, intermediate
  Q307/Q308 currents move monotonically in opposite directions and zero input
  has no balance offset;
- a nominal selected saw produces the finite W-MOD voltage predicted by the
  populated input divider, CA3280 current limit and 9.09 kohm R3113/R2 load;
- oscillator, pulse-width and filter depths retain the populated SD334
  resistor ratios, each with its 300 ohm CD4016 switch in series, while
  consuming the reconstructed W-MOD voltage directly;
- with FILT CUTOFF high, a unison keyboard CV and full Wheel Mod, the filter
  route never lifts the first FILT MSUM stage past its +/-13 V swing;
- a full MOD wheel sweeps 9.67% of R2, twice the PITCH wheel's half-travel;
- silence and note events do not stop or retrigger the LFO;
- CC1 changes the render when a documented destination is enabled;
- CC1 steps converge continuously with the same three-millisecond time
  constant at every supported sample rate;
- audition wheel state is cleared by CC1, normal program loads and state loads;
- all supported sample rates remain finite and bounded.

Primary evidence: Sequential Circuits technical manual TM1000D.2, sections
2-2, 2-4, schematic SD334 and service test 4-7; the CEM3340 data sheet's
multiplier, exponential-generator and frequency equations; the owner's
manual, Sequential's 1-1 Brass patch sheet and the Synthmania Rev 3
recording for the LFO rate. Provenance and hashes are recorded in
`SOURCE_LEDGER.md`.
