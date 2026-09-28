# Poly Mod model

## Accepted hardware contract

Poly Mod is generated independently inside every Rev 3 voice. Two separate
RCA/CA3280 amount VCAs add that voice's filter-envelope control voltage and
oscillator-B waveform bus. The resulting bus can reach oscillator-A frequency,
oscillator-A pulse width and filter cutoff through three independent switches.

The filter envelope enters U422 pin 16, the positive input, and pin 13 sources
its output into the shared R4108 load. A positive envelope therefore raises
oscillator-A frequency and filter cutoff. Service test 4-8's "DESCENDING FREQ"
and "DESCENDING RES FILT SWEEP" indications describe the audible trajectory as
that positive voltage decays, not an inverted source polarity. Oscillator B
remains a Poly Mod source even when its audio mixer level is zero, and any
enabled B waveforms contribute to the source bus. Oscillator hard sync is a
separate route.

## Active candidate

- Every voice owns independent amplifier and filter ADSR state.
- Filter attack, decay, sustain, release and direct cutoff amount use the
  original 128-position panel quantization boundary.
- The two physical halves of U422 are modeled together on each voice: one
  controls direct filter-envelope amount and the other controls the Poly Mod
  envelope source. The direct half now produces U433 summing current from the
  populated Q301/5.1k IABC, 121k diode-bias, R452/R450 47.5k/47.5k input,
  R453 475k balance and 100k common-CV reference networks. R452 (R462/R492 on
  voices 2/5) is printed compactly as "47.5K", like R450 and unlike the spaced
  "475 K" of R453, so the direct half mirrors the Poly Mod half's equal
  22k/22k pair. The Poly Mod half retains the positive polarity
  established by U422 pins 16 and 13 at the summing node.
- SD333 Q304 turns the Poly Mod envelope amount's S/H output (1/12 V per
  stored code, 10.000 V at the panel's code-120 ceiling) into its
  physical IABC curve through 3 kohm. Q303 independently applies 5.6 kohm to
  oscillator B. Each collector line feeds the five corresponding CA3280 IABC
  pins in parallel, so its total current is divided nominally five ways rather
  than duplicated on every voice card. Both source endpoints are therefore
  reached only after their distinct 2N4250 knees and the physical fanout rather
  than by linear host multipliers.
- Oscillator B is evaluated first during each 4x internal substep. Its selected
  waveform sum passes through one profiled unlinearized CA3280 amount VCA per
  voice before the audio mixer level. Saw and pulse retain their board-level
  positive bias, while U451 doubles the raw 0-5 V triangle and subtracts the
  4.57 V TRI REF, making it approximately +/-5 V about ground; the same shifted
  wave also feeds the audio mixer (see
  [`VCO_OUTPUT_MODEL.md`](VCO_OUTPUT_MODEL.md)).
- Oscillator-A frequency and pulse-width destinations are evaluated at that
  internal rate, preserving audio-rate modulation. The PWM destination now
  advances the CEM3340 comparator edge from its velocity relative to oscillator
  phase rather than applying a static-width correction independently on every
  substep.
- The filter destination enters the four-pole CEM3320 candidate independently
  on every internal substep, preserving its audio-rate content.
- Source amounts are additive and destinations can be enabled independently.
- U422 and U428 now contribute physical output currents to their shared PMOD
  node. The sum develops voltage across populated R4108 (30 kohm), and U431 is
  treated as the voltage follower shown by SD431. This removes the former
  normalized bus and its inferred volts-per-unit anchor.
- The envelope half uses the populated 22k source and return paths, R4146's
  120k linearizing-diode feed and the serviced 470k/100k balance return. The
  oscillator-B half reuses the selected 150k saw/triangle and 200k pulse
  conductances, the approximately 100k unlinearized CA3280 input and the 330
  ohm shunt. Its current therefore changes with the actual enabled waveform
  combination rather than only with their normalized sum.
- Both U422 halves follow the CA3280 data sheet's linearized transfer with no
  fitted gain. Intersil's Figure 3A (ID 200 uA, IABC 650 uA, 10k + 10k inputs)
  is linear at approximately 125 uA/V until the input current reaches +/-ID,
  where the output settles at the published 0.82 IABC peak. The populated
  source loop and the diodes' small dynamic impedance set the signal current
  Is, and the output is Iout = 0.776 Is IABC / ID
  (`CA3280_LINEARIZED_CURRENT_TRANSFER`), bounded by the same 0.82 IABC peak.
  The ideal LM13700-style law, 2 Is IABC / ID, was considered and rejected
  because Figure 3A contradicts it. Intersil's functional diagram shows the
  ID pin feeding an internal current mirror referred to the V- pin, so ID is
  programmed across approximately 28.8 V (`LINEARIZED_ID_PROGRAMMING_VOLTS`,
  two junctions above -15 V). The final and master VCAs U477/U479 follow the
  same law and ID reference (see
  [`VCA_AND_OUTPUT_MODEL.md`](VCA_AND_OUTPUT_MODEL.md)). The
  former 3.15 direct and 0.84 Poly Mod reference gains are removed: the 475
  kohm misreading of R452, the former transconductance-times-diode-impedance
  linearization and the full 30 V ID span accounted for them.
- The Poly Mod envelope bus consequently reaches approximately 4.5-4.7 V at
  factory 1-7's (Sync I) code 86 and approximately 6.4-6.7 V at code 120 for a
  nominal 5 V envelope peak across the five voice profiles.
- The common bus is smoothly bounded at the CA3280 data sheet's guaranteed
  minimum +/-12 V output swing on +/-15 V rails. This is a conservative
  electrical compliance boundary, not a host-audio clamp.
- Oscillator-A frequency follows R4357 (301 kohm) relative to the calibrated
  100 kohm, one-volt-per-octave pitch path. One physical PMOD volt therefore
  spans approximately 3.9867 semitones.
- Oscillator-A pulse width follows R4172 (30.1 kohm), U432 feedback R4162
  (52.3 kohm) and the CEM3340's 5 V duty-cycle range. The panel/Wheel-Mod PW A
  SUM CV reaches the same summer already inverted by SD334's U366, so it lands
  on pin 5 positive, while PMOD crosses only U432's inversion. Positive Poly
  Mod therefore narrows the pulse, by approximately 0.3475 normalized
  duty-cycle units per physical PMOD volt.
- Filter cutoff follows R4181 (54.9 kohm) relative to the 100 kohm calibrated
  common filter input. The shared per-voice FIL 1 SCALE stage cancels from
  that ratio, producing approximately 1.8215 octaves per physical PMOD volt.

## Bounded uncertainty

The circuit and service procedure establish routing, polarity, active versus
cut-off linearizing terminals, destination resistance ratios, R4108's shared
load and the balance trims. The CA3280 data sheet establishes small-signal
transconductance, peak-output-current bounds and at least +/-12 V output
swing. The exact populated current transfer, transistor temperature, actual
U422/U428 matching and U431 bus swing remain unmeasured. The typical data-sheet
output limits extend to approximately +13.7/-14.3 V, but RF-5 deliberately
uses the guaranteed minimum magnitude until a serviced unit is measured.

The direct filter-envelope path no longer owns an isolated octave-depth
constant or fitted gain. The 1/12 V-per-code DAC/S&H voltage crosses SD333
Q301 and 5.1 kohm, divides across the five voice-card IABC inputs, then
reaches the populated U422/U433 network and the Figure 3A transfer. A nominal
5 V CEM3310 peak at the code-120 panel ceiling moves cutoff by approximately
6.3-6.6 octaves across the five voice profiles. At factory 1-4's exact amount
34 this is approximately 1.5-1.6 octaves, which exposes the octave overtone
explicitly described by Sequential's patch sheet. A populated-unit measurement can refine transistor
temperature and current without changing the accepted circuit. Both envelopes use the CEM3310 true-RC candidate
documented in `ENVELOPE_MODEL.md`; exact mechanical panel taper remains
unmeasured.

## Acceptance tests

- full filter-envelope Poly Mod raises oscillator A at attack and then descends
  as the envelope decays, matching service test 4-8;
- oscillator-B Poly Mod remains audible with oscillator-B mixer level at zero;
- paired direct/Poly Mod envelope halves remain close but non-identical;
- direct envelope depth follows U422/U433 current, Figure 3A's linearized
  transfer and resistor ratios rather than a free maximum-octaves constant or
  fitted gain, reaching approximately 6.2-6.7 octaves at code 120 and
  1.45-1.65 octaves at 1-4's amount 34;
- factory 1-4 keeps its exact 34/127 amount while its attack octave remains
  within a bounded ratio of the fundamental;
- Poly Mod amount rises monotonically and its two CA3280 modes retain their
  distinct strong-signal ranges;
- Q303 and Q304 retain their distinct 5.6k and 3k total-current laws, including
  the shared silicon-junction knee and five-way IABC fanout;
- the two OTA currents add through one 30k load and remain bounded by the
  guaranteed CA3280 output swing;
- full one-saw oscillator-B modulation develops approximately 1.6-1.95 V
  across the five deterministic voice profiles, while a nominal envelope
  reaches approximately 4.5-4.7 V at factory 1-7's code 86 and stays below the
  conservative 12 V bus boundary at code 127;
- positive PMOD narrows oscillator A's pulse, opposite in sign to the
  frequency and filter destinations;
- frequency and filter destinations produce distinct renders;
- all three destination depths retain the populated SD431 resistor ratios and
  consume the same physical PMOD voltage;
- audio-rate PWM reduces non-harmonic energy against the former static-width
  correction across moderate and near-full modulation depths at every accepted
  host rate;
- both envelopes trigger and release independently;
- the expanded parameter state round-trips exactly;
- all workspace tests remain finite and deterministic.

Primary evidence: Sequential Circuits technical manual TM1000D.2 sections 2-4
and 2-5, schematics SD333, SD334 and SD431, service test 4-8, the Intersil
CA3280 data sheet including its Figure 3A linearized transfer, and the CEM3340
data sheet's 0-5 V PWM input. Provenance and hashes are recorded in
`SOURCE_LEDGER.md`.
