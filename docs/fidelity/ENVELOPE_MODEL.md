# CEM3310 envelope model

## Accepted hardware contract

Every voice has separate CEM3310 generators for filter and amplifier. The
data sheet describes a true external-capacitor RC envelope, exponential time
control, linear sustain, a nominal 2 ms to 20 s range and a 6.5 V attack
asymptote for a 5 V peak. The service procedure checks roughly one-second
attack, decay and release around dial position 6 and a release longer than
20 seconds at position 10.

## Active candidate

- Filter and amplifier envelopes own completely independent capacitor state.
- Attack converges toward a normalized 1.3 asymptote and changes to decay at
  the normalized 1.0 peak.
- Decay and release converge exponentially toward sustain and zero.
- Sustain is linear and all four panel controls retain their 128 positions.
- Time constants are mapped logarithmically from 2 ms to 20 s.
- Retrigger changes the charging phase without digitally clearing the stored
  capacitor voltage; note release likewise changes only the active phase.

## Bounded uncertainty

The RC shape, asymptote and data-sheet endpoints are accepted. The exact panel
taper, capacitor tolerance, small internal phase thresholds, gate/trigger
timing and the mapping needed to reproduce the service dial-6 observation are
not measured. Filter and amplifier currently share one candidate mapping even
though physical parts may differ slightly.

## Acceptance tests

- both generators advance and release independently;
- attack is observably curved rather than a linear ramp;
- control endpoints cover the documented time range;
- retrigger preserves the existing capacitor voltage;
- a complete attack-decay-sustain-release lifecycle reaches idle safely.

Primary evidence: TM1000D.2 sections 2-7 and 4-11, voice schematic SD431 and
the original CEM3310 data sheet. Provenance is recorded in `SOURCE_LEDGER.md`.
