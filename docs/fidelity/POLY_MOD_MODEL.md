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
waveforms contribute to the source bus. Hard sync is a separate route.

## Active candidate

- Every voice owns independent amplifier and filter ADSR state.
- Filter attack, decay, sustain, release and direct cutoff amount use the
  original 128-position panel quantization boundary.
- The two physical halves of U422 are modeled together on each voice: one
  linearized transfer controls direct filter-envelope amount and the other
  controls the Poly Mod envelope source. The latter is explicitly inverted at
  the summing node.
- Oscillator B is evaluated first during each 4x internal substep. Its selected
  waveform sum passes through one profiled unlinearized CA3280 amount VCA per
  voice before the audio mixer level. Saw and pulse retain their board-level
  positive bias, while the dedicated DC level-shifter makes triangle bipolar
  about ground.
- Oscillator-A frequency and pulse-width destinations are evaluated at that
  internal rate, preserving audio-rate modulation.
- The filter destination enters the four-pole CEM3320 candidate independently
  on every internal substep, preserving its audio-rate content.
- Source amounts are additive and destinations can be enabled independently.
- The two amount VCAs now feed one normalized PMOD bus boundary. One candidate
  1.2 V/unit conversion is applied before every destination, rather than
  assigning three unrelated maximum depths.
- Oscillator-A frequency follows R4357 (30.1 kohm) relative to the calibrated
  100 kohm, one-volt-per-octave pitch path. A unit PMOD bus therefore spans
  approximately 47.84 semitones with the current voltage anchor.
- Oscillator-A pulse width follows R4112 (30.1 kohm), U432 feedback R4162
  (52.3 kohm) and the CEM3340's 5 V duty-cycle range, producing approximately
  0.417 normalized duty-cycle units per unit bus.
- Filter cutoff follows R4181 (54.9 kohm) relative to the 100 kohm calibrated
  common filter input. The shared per-voice FIL 1 SCALE stage cancels from
  that ratio, producing approximately 2.186 octaves per unit bus.

## Bounded uncertainty

The circuit and service procedure establish routing, polarity, active versus
cut-off linearizing terminals, destination resistance ratios and balance
trims. The CA3280 population is bounded by its data-sheet output-current
ratios, but the loaded voltage at U431's PMOD buffer remains unmeasured. RF-5
therefore retains one explicit 1.2 V/unit bus anchor. A populated-unit PMOD
voltage measurement can replace it and recalibrate all three destinations
together without changing their accepted circuit ratios.

The direct filter-envelope cutoff depth is also an isolated candidate. Both
envelopes now use the CEM3310 true-RC candidate documented in
`ENVELOPE_MODEL.md`; exact panel taper remains unmeasured.

## Acceptance tests

- full filter-envelope Poly Mod makes oscillator A descend rather than ascend;
- oscillator-B Poly Mod remains audible with oscillator-B mixer level at zero;
- paired direct/Poly Mod envelope halves remain close but non-identical;
- Poly Mod amount rises monotonically and its two CA3280 modes retain their
  distinct strong-signal ranges;
- frequency and filter destinations produce distinct renders;
- all three destination depths retain the populated SD431 resistor ratios and
  share one replaceable PMOD-bus voltage anchor;
- both envelopes trigger and release independently;
- the expanded parameter state round-trips exactly;
- all workspace tests remain finite and deterministic.

Primary evidence: Sequential Circuits technical manual TM1000D.2 section 2-4,
voice schematic SD431 and service test 4-8. Provenance and hash are recorded in
`SOURCE_LEDGER.md`.
