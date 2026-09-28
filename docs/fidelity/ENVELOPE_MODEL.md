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
- The panel-to-time law is the SD430 circuit, not a fit to manual dial
  observations. V8.1 writes each time pot as `0x7a - pot` through the 7-bit
  DAC step (10.67 V / 128), so a short time is a high held voltage. That CV
  enters one node shared by all five voice cards through 24.3 kohm (R415,
  R413, R414, R410, R412, R411), with 13 kohm to the -5 V rail (R407-class)
  and 806 ohm to ground (R402-class). The node therefore spans approximately
  +25 mV at pot 0 to -278 mV at the V8.1 panel ceiling (code 120) and drives
  the CEM3310 Va/Vd/Vr pins directly. The data sheet's
  `tau = Rx*Cx*exp(-Vc/VT)` then makes time exactly exponential in the stored
  code: every code multiplies the time constant by about 1.095.
- The populated 24.3 kohm/0.039 uF network sets the 0.9477 ms time constant
  reached at 0 V on the pin, near stored code 10. Pot 0 is about 2.5 times
  faster, and code 120 is about 22,000 times slower.
- The data sheet's 60 mV/decade is kT/q at 25 C and carries its published
  +3300 ppm/C coefficient. RF-5 evaluates it at a 45 C operating die, the one
  explicit operating-point assumption. That places the nominal full-scale
  attack at the owner's manual's "approximately 30 seconds" and keeps the
  service manual's longer-than-20-second release at 10. Factory 1-4's Attack
  code 30 also reaches half of the final-VCA current in about 3.2 ms, inside
  the 3-5 ms measured on the Rev 3 recording. The earlier dial-5/dial-6
  landmarks (0.5 s and 1 s) are not reproduced exactly. The circuit gives
  about 0.13 s and 0.39 s to peak at codes 60 and 72. Those landmarks are
  approximate knob readings through a pot whose taper is undocumented, while
  stored programs carry exact codes.
- Sustain is written uncomplemented and halved by R405/R406 (amplifier) or
  R426/R433 (filter), each 4.75 kohm. Code 120, the V8.1 10 V ceiling, is
  therefore exactly the 5 V peak. A held sustain above the peak threshold is
  approached after the peak at the fastest attack rate, as the data sheet
  specifies.
- Each device's curve applies its 58.5-61.5 mV/decade control sensitivity, a
  bounded component/time-tracking ratio and distinct charge/discharge current
  ratios inside the published 0.75-1.30 and 0.83-1.20 limits.
- Retrigger changes the charging phase without digitally clearing the stored
  capacitor voltage; note release likewise changes only the active phase.
- The external timing capacitor and ENV OUT are separate state domains. The
  capacitor remains continuous, while each device's bounded 100-350 ohm
  internal buffer resistance converts the instantaneous charge/discharge
  current through 24.3 kohm into the small output steps shown by the original
  CEM3310 waveforms. The nominal attack step at 0 V control is 53.5 mV (the
  data sheet's `(Ro/Rx)*Vz`), and the sign
  reverses naturally on entering decay or release.
- The amplifier generator's voltage is not used as a digital amplitude
  multiplier. Its nominal 0-5 V output drives the populated
  grounded-base R4496/Q410 voltage-to-current stage documented in
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

The RC shape, asymptote, populated nominal components, the SD430 control
network, the V8.1 time complement and the data-sheet bounds are accepted. The
ten selected component/current points inside those bounds are a deterministic
validation population, not measurements from one instrument. The operating
die temperature is the only calibrated quantity: 45 C is chosen from the
owner's-manual full-scale time and bounded by the service manual's
longer-than-20-second release. Exact device values, small internal phase
thresholds, exact gate/trigger timing, Q410 temperature, buffer-resistance
correlation and the correlated populated-device spread remain unmeasured.
The DAC step is taken from SD332's approximately 10.67 V full scale; a
serviced reference error scales every envelope time by the same power law.

An earlier candidate fitted a monotone cubic through the dial-5/6/10 manual
observations and the factory 1-4 fast region. Compared with the circuit, that
fit made the attack codes most used by the factory programs (45-75) 1.5-2.8
times too slow and the common release codes (52-77) 20-45 percent too long. It
is superseded.

## Acceptance tests

- both generators advance and release independently;
- all ten profiles stay inside the admitted voltage, control-scale and
  unit-tracking limits;
- each filter/amplifier pair follows a distinct time curve;
- attack is observably curved rather than a linear ramp;
- the populated components produce a 0.9477 ms time constant at 0 V control;
- the SD430 node spans +24-26 mV at pot 0 and -278 to -280 mV at code 120,
  crossing 0 V between codes 9 and 11;
- every stored code multiplies the time constant by the same factor, and pots
  above 0x7a saturate V8.1's complement at zero;
- the full-scale attack lies within 28-33 s, the full-scale release to ten
  percent exceeds 20 s and the fastest attack stays near one millisecond;
- factory 1-4 Attack code 30 reaches half of the final-VCA current within
  3-5 ms, and RELEASE-off code 22 remains a few-millisecond time constant
  slower than pot 0;
- sustain code 120 equals the 5 V peak and code 60 half of it;
- charge and discharge currents remain distinct and inside their separate
  electrical bounds;
- retrigger preserves the existing capacitor voltage;
- phase-current polarity creates bounded output steps without moving the
  stored capacitor voltage, including the source-equation 53.5 mV step at
  0 V control;
- a complete attack-decay-sustain-release lifecycle reaches idle safely.
- RELEASE off overrides both programmed release pots with exact equivalent
  code 22, traverses both physical S/H cells and reaches idle far sooner than a
  maximum-release patch without collapsing to the minimum time.
- the nominal 5 V amplifier peak reaches the final VCA's source-backed IABC
  operating region without changing the filter-envelope voltage domain.

Primary evidence: TM1000D.2 sections 2-7 and service tests 4-5/4-6, CV
distribution schematic SD430, voice schematic SD431, the owner's manual and
the original CEM3310 data sheet. Provenance is recorded in `SOURCE_LEDGER.md`.
