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
- The Poly Mod filter-envelope source is explicitly inverted before summing.
- Oscillator B is evaluated first during each 4x internal substep. Its raw
  selected waveform sum feeds Poly Mod before the audio mixer level.
- Oscillator-A frequency and pulse-width destinations are evaluated at that
  internal rate, preserving audio-rate modulation.
- The filter destination enters the four-pole CEM3320 candidate independently
  on every internal substep, preserving its audio-rate content.
- Source amounts are additive and destinations can be enabled independently.

## Bounded uncertainty

The circuit and service procedure establish routing and polarity, but not the
complete RCA/CA3280 gain, overload or feed-through curves. Candidate maximum
depths are therefore isolated constants: 48 semitones for oscillator-A
frequency, 0.48 normalized units for pulse width and 4.5 octaves for filter
cutoff. They are hypotheses to calibrate, not measured specifications.

The direct filter-envelope cutoff depth is also an isolated candidate. Both
envelopes now use the CEM3310 true-RC candidate documented in
`ENVELOPE_MODEL.md`; exact panel taper remains unmeasured.

## Acceptance tests

- full filter-envelope Poly Mod makes oscillator A descend rather than ascend;
- oscillator-B Poly Mod remains audible with oscillator-B mixer level at zero;
- frequency and filter destinations produce distinct renders;
- both envelopes trigger and release independently;
- the expanded parameter state round-trips exactly;
- all workspace tests remain finite and deterministic.

Primary evidence: Sequential Circuits technical manual TM1000D.2 section 2-4,
voice schematic SD431 and service test 4-8. Provenance and hash are recorded in
`SOURCE_LEDGER.md`.
