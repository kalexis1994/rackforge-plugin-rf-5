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
- The external timing capacitor and ENV OUT are separate state domains. The
  capacitor remains continuous, while each device's bounded 100-350 ohm
  internal buffer resistance converts the instantaneous charge/discharge
  current through 24.3 kohm into the small output steps shown by the original
  CEM3310 waveforms. The nominal fastest-attack step is 53.5 mV, and the sign
  reverses naturally on entering decay or release.
- The amplifier generator's voltage is not used as a digital amplitude
  multiplier. Its nominal 0-5 V output drives the populated
  R4495/Q410/R4533 voltage-to-current stage documented in
  `VCA_AND_OUTPUT_MODEL.md`, giving the final VCA a physical silicon-junction
  knee while leaving the filter envelope in its separate CV paths.
- The stored RELEASE switch governs both filter and amplifier generators. When
  on, each uses its own programmed Release pot. When off, both use the global
  fixed-time setting, matching the owner's-manual behavior. The V8.1 loop
  normally writes envelope times as `0x7a - pot`; its fixed `0x64` Release
  write therefore equals physical pot code `0x16`, or 22/127 of the panel
  domain. RF-5 applies that code before both release sample/hold cells, so the
  normal acquisition and leakage paths remain active. It is deliberately not
  collapsed to the absolute fastest CEM3310 time.

## Bounded uncertainty

The RC shape, asymptote, populated nominal components, data-sheet bounds and
service observations are accepted. The ten selected component/current points
inside those bounds are a deterministic validation population, not
measurements from one instrument. Exact device values, the meaning of
"approximately" in the dial-6 listening test, small internal phase thresholds,
exact gate/trigger timing, Q410 temperature, buffer-resistance correlation and
the correlated populated-device spread remain unmeasured. The disabled Release
code and its pot-equivalent position are exact firmware behavior and no longer
carry a separate analog-offset hypothesis. The dial-6 second is
one explicit replaceable absolute anchor; the remaining panel law is derived
from the chip equation and populated circuit.

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
- phase-current polarity creates bounded output steps without moving the
  stored capacitor voltage, including the source-equation 53.5 mV nominal
  fastest-attack endpoint;
- a complete attack-decay-sustain-release lifecycle reaches idle safely.
- RELEASE off overrides both programmed release pots with exact equivalent
  code 22, traverses both physical S/H cells and reaches idle far sooner than a
  maximum-release patch without collapsing to the minimum time.
- the nominal 5 V amplifier peak reaches the final VCA's source-backed IABC
  operating region without changing the filter-envelope voltage domain.

Primary evidence: TM1000D.2 sections 2-7 and service tests 4-5/4-6, voice schematic SD431 and
the original CEM3310 data sheet. Provenance is recorded in `SOURCE_LEDGER.md`.
