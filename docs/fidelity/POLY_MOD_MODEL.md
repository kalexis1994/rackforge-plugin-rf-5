# Poly Mod model

## Accepted hardware contract

Poly Mod is generated independently inside every Rev 3 voice. Two separate
RCA/CA3280 amount VCAs add that voice's filter-envelope control voltage and
oscillator-B waveform bus. The resulting bus can reach oscillator-A frequency,
oscillator-A pulse width and filter cutoff through three independent switches.

The filter-envelope input has negative polarity in this path. The service test
therefore expects increasing FILTER ENV Poly Mod to make oscillator A descend
in frequency and to sweep a resonating filter downward. Oscillator B remains a
Poly Mod source even when its audio mixer level is zero, and any enabled B
waveforms contribute to the source bus. Oscillator hard sync is a separate route.

## Active candidate

- Every voice owns independent amplifier and filter ADSR state.
- Filter attack, decay, sustain, release and direct cutoff amount use the
  original 128-position panel quantization boundary.
- The two physical halves of U422 are modeled together on each voice: one
  controls direct filter-envelope amount and the other controls the Poly Mod
  envelope source. The direct half now produces U433 summing current from the
  populated Q301/5.1k IABC, 121k diode-bias, 475k/47.5k input and 100k
  common-CV reference networks. The latter is explicitly inverted at the
  summing node.
- SD333 Q304 turns the Poly Mod envelope amount's 0-10 V S/H output into its
  physical IABC curve through 3 kohm. Q303 independently applies 5.6 kohm to
  oscillator B. Both normalized source endpoints are therefore reached only
  after their distinct 2N4250 knees rather than by linear host multipliers.
- Oscillator B is evaluated first during each 4x internal substep. Its selected
  waveform sum passes through one profiled unlinearized CA3280 amount VCA per
  voice before the audio mixer level. Saw and pulse retain their board-level
  positive bias, while the dedicated DC level-shifter makes triangle bipolar
  about ground.
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
- The common bus is smoothly bounded at the CA3280 data sheet's guaranteed
  minimum +/-12 V output swing on +/-15 V rails. This is a conservative
  electrical compliance boundary, not a host-audio clamp.
- Oscillator-A frequency follows R4357 (30.1 kohm) relative to the calibrated
  100 kohm, one-volt-per-octave pitch path. One physical PMOD volt therefore
  spans approximately 39.867 semitones.
- Oscillator-A pulse width follows R4112 (30.1 kohm), U432 feedback R4162
  (52.3 kohm) and the CEM3340's 5 V duty-cycle range, producing approximately
  0.3475 normalized duty-cycle units per physical PMOD volt.
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
constant. The admitted 0-10 V DAC/S&H span crosses SD333 Q301 and 5.1 kohm,
then the populated U422/U433 network and CA3280 data-sheet equations produce
approximately eight octaves at a nominal 5 V CEM3310 peak. A populated-unit
measurement can refine transistor temperature and current without changing
the accepted circuit. Both envelopes use the CEM3310 true-RC candidate
documented in `ENVELOPE_MODEL.md`; exact mechanical panel taper remains
unmeasured.

## Acceptance tests

- full filter-envelope Poly Mod makes oscillator A descend rather than ascend;
- oscillator-B Poly Mod remains audible with oscillator-B mixer level at zero;
- paired direct/Poly Mod envelope halves remain close but non-identical;
- direct envelope depth follows U422/U433 current and resistor ratios rather
  than a free maximum-octaves constant;
- Poly Mod amount rises monotonically and its two CA3280 modes retain their
  distinct strong-signal ranges;
- Q303 and Q304 retain their distinct 5.6k and 3k current laws, including the
  shared silicon-junction knee;
- the two OTA currents add through one 30k load and remain bounded by the
  guaranteed CA3280 output swing;
- full one-saw oscillator-B modulation develops approximately 8.0-9.5 V
  across the five deterministic voice profiles, while a full nominal envelope
  approaches the conservative 12 V bus boundary;
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
and 2-5, schematics SD333 and SD431, service test 4-8 and the Intersil CA3280
data sheet. Provenance and hashes are recorded in `SOURCE_LEDGER.md`.
