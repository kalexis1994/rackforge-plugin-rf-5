# CEM3310 envelope model

## Accepted hardware contract

Every voice has separate CEM3310 generators for filter and amplifier. The
data sheet describes a true external-capacitor RC envelope, exponential time
control, linear sustain, at least a 50,000:1 time-control range and a 6.5 V
attack asymptote for a 5 V peak. The populated voice cards use a 24.3 kohm 1%
timing resistor and 0.039 uF 5% timing capacitor; their separate 0.02 uF mylar
part is compensation rather than the principal RC capacitor. The service
procedure checks roughly one-second attack, decay and release around dial
position 6 and a release longer than 20 seconds at position 10.

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
- The populated 24.3 kohm/0.039 uF network sets a 0.9477 ms nominal fastest
  time constant. The nominal dial-6 attack reaches the 5 V peak in one second;
  this isolates one service-backed anchor and derives a 285.71 mV full control
  excursion, 4.762 decades and a 57,786:1 range.
- Position 10 therefore exceeds a 50-second nominal time constant. Its release
  comfortably satisfies the documented greater-than-20-second observation
  instead of being forced to an arbitrary 20-second endpoint.
- Each device's curve applies its 58.5-61.5 mV/decade control sensitivity, a
  bounded component/time-tracking ratio and distinct charge/discharge current
  ratios inside the published 0.75-1.30 and 0.83-1.20 limits.
- Retrigger changes the charging phase without digitally clearing the stored
  capacitor voltage; note release likewise changes only the active phase.
- The stored RELEASE switch governs both filter and amplifier generators. When
  on, each uses its own programmed Release pot. When off, both use the global
  minimum-time setting, matching the owner's-manual behavior. V8.1's fixed
  `0x64` release-CV code remains recorded at the firmware boundary; until an
  instrument measurement fixes its absolute CEM3310 offset, the active analog
  model maps that serviced minimum to its fastest admitted time.

## Bounded uncertainty

The RC shape, asymptote, populated nominal components, data-sheet bounds and
service observations are accepted. The ten selected component/current points
inside those bounds are a deterministic validation population, not
measurements from one instrument. Exact device values, the meaning of
"approximately" in the dial-6 listening test, small internal phase thresholds,
gate/trigger timing and temperature remain unmeasured. The dial-6 second is one
explicit replaceable absolute anchor; the remaining panel law is derived from
the chip equation and populated circuit.

## Acceptance tests

- both generators advance and release independently;
- all ten profiles stay inside the admitted voltage, control-scale and
  unit-tracking limits;
- each filter/amplifier pair follows a distinct time curve;
- attack is observably curved rather than a linear ramp;
- the populated components produce a 0.9477 ms fastest time constant;
- dial position 6 produces a one-second nominal attack;
- the derived control span stays inside the published time-control range and
  position 10 exceeds the service release requirement;
- charge and discharge currents remain distinct and inside their separate
  electrical bounds;
- retrigger preserves the existing capacitor voltage;
- a complete attack-decay-sustain-release lifecycle reaches idle safely.
- RELEASE off overrides both programmed release pots and reaches idle far
  sooner than a maximum-release patch.

Primary evidence: TM1000D.2 sections 2-7 and service tests 4-5/4-6, voice schematic SD431 and
the original CEM3310 data sheet. Provenance is recorded in `SOURCE_LEDGER.md`.
