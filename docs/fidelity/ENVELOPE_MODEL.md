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
- Every voice owns two deterministic physical profiles: amplifier then filter.
  Across all ten CEM3310 candidates, control sensitivity remains inside
  58.5-61.5 mV/decade, peak inside 4.7-5.3 V and attack asymptote inside
  6.1-6.9 V.
- Attack converges toward its device-specific asymptote and changes to decay
  at its device-specific peak.
- Decay and release converge exponentially toward sustain and zero.
- Sustain is linear, includes the published -3 to +23 mV final-value error,
  and all four panel controls retain their 128 positions.
- Nominal time constants are mapped logarithmically from 2 ms to 20 s. Each
  device's complete curve applies its control sensitivity and a bounded RC
  ratio inside the documented +/-15% practical unit-to-unit tracking window,
  which also represents the external 0.02 uF mylar capacitor population.
- Retrigger changes the charging phase without digitally clearing the stored
  capacitor voltage; note release likewise changes only the active phase.

## Bounded uncertainty

The RC shape, asymptote and data-sheet bounds are accepted. The ten selected
points inside those bounds are a deterministic validation population, not
measurements from one instrument. The exact panel taper, individual capacitor
values, small internal phase thresholds, gate/trigger timing and the mapping
needed to reproduce the service dial-6 observation remain unmeasured.

## Acceptance tests

- both generators advance and release independently;
- all ten profiles stay inside the admitted voltage, control-scale and
  unit-tracking limits;
- each filter/amplifier pair follows a distinct time curve;
- attack is observably curved rather than a linear ramp;
- control endpoints cover the documented time range;
- retrigger preserves the existing capacitor voltage;
- a complete attack-decay-sustain-release lifecycle reaches idle safely.

Primary evidence: TM1000D.2 sections 2-7 and 4-11, voice schematic SD431 and
the original CEM3310 data sheet. Provenance is recorded in `SOURCE_LEDGER.md`.
