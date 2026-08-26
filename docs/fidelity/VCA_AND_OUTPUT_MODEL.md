# CA3280 mixer, VCA and output model

## Accepted hardware contract

The Revision 3 signal path uses several RCA CA3280 operational
transconductance amplifiers rather than one generic gain stage. Each voice
routes the selected oscillator-A and oscillator-B waveforms through separate
CA3280 mixer VCAs with the linearizing-diode terminal cut off. CEM3320 OUT D is
AC-coupled by C4164, loaded by R4460 and amplified 3.4 times by U474 before the
filtered signal enters a CA3280 final VCA whose linearizing terminal is active
and whose bias current is controlled by the amplifier envelope through Q410
and two populated 3.3 kohm resistors. Q410 is a grounded-base Fairchild 2N4250 PNP;
its emitter junction therefore converts CEM3310 voltage to CA3280 IABC current
rather than passing a normalized envelope value directly.

Five equal 39 kohm input resistors feed the high-impedance U480 follower, so
the common node is a passive one-fifth average of the five low-impedance voice
outputs. A second linearized CA3280 applies the physical volume control,
followed by an NE5534 buffer and the back-panel output network. Balance
trimmers remove OTA DC offset and the service procedure separately calibrates
final-VCA balance and per-voice volume.

The same OTA family also controls modulation. U378 crossfades common LFO and
noise in opposite directions, while each voice has a linearized dual-envelope
amount device and an unlinearized oscillator-B Poly Mod amount device.

## Active candidate

- Each voice has one deterministic dual-OTA mixer profile. Its oscillator A
  and B halves retain close but non-identical transconductance and overload
  knees inside the CA3280 data-sheet output-current bounds.
- Each oscillator mix reconstructs the manual's approximately 100 kohm
  unlinearized input impedance against the selected 150 kohm saw/triangle and
  200 kohm pulse paths plus the populated 330 ohm input shunt. Parallel
  waveform selections produce the source-backed passive loading before the
  nonlinear current transfer. U464's output currents then enter the first
  CEM3320 cell directly and develop voltage through its 100k feedback in
  parallel with the nominal 1M output impedance.
- One separate unlinearized CA3280 sets common noise level before the result is
  developed across R4129's 10k, buffered by U474 and distributed through five
  100k paths to the CEM3320 inputs; noise does not pass through a fictitious
  third mixer OTA on every voice card.
- The 0-10 V oscillator A/B level cells reach the paired voice-mixer VCAs
  through SD333 Q306/Q302 and 33k emitter resistors. The common noise cell
  reaches its OTA through Q305 and 75k. Their physical IABC currents now drive
  CA3280 current limits and the populated filter-input transimpedances; the
  intermediate settings follow the common 2N4250 junction equation instead of
  linear host multipliers.
- The five final VCAs use substantially wider diode-linearized transfers and
  are evaluated inside the same four-times-oversampled loop as the filters.
  Their small-signal gain is equal after the documented per-voice service
  adjustment, while their strong-signal knees remain distinct.
- The final-VCA audio input is the same stateful C4164/U474 node that drives
  the CEM3320 resonance return. Its approximately 1.064 Hz DC-blocking corner
  and 3.4 non-inverting gain are applied once inside the filter model. U474's
  reconstructed load exceeds 10 kohm, and its profiled late knee is bounded by
  the TL082's published +/-12 V minimum and +/-13.5 V typical swing while
  retaining the published 20 Vpp linearity. No separate normalized pre-VCA
  gain or duplicate high-pass is added here.
- The final-VCA signal transfer is separate from the generic linearized
  modulation OTA. Intersil Figure 3A supplies two bounded landmarks at 650 uA
  IABC and 200 uA diode current: a long linear centre and rounded current limit
  at approximately four horizontal 1 V divisions with 10 kohm inputs. SD431's
  populated 20 kohm/20 kohm inputs double the source-voltage span. The filter
  and VCA exchange circuit volts directly, so a sixth-order smooth norm places
  the asymptote at eight volts and starts measurable compression inside the
  CEM3320 population's 10-14 Vpp output range.
- The nominal 0-5 V amplifier envelope is converted once per host sample by
  the populated R4495/Q410/R4533 network. A Fairchild 2N4250 junction fit uses
  the original approximately 0.56 V at 100 uA and 26 mV thermal slope, solving
  the implicit diode-plus-6.6-kohm equation directly. The nominal peak reaches
  approximately 665 uA IABC, closely matching the CA3280 data sheet's 650 uA
  linearized-transfer plot; the result then remains fixed across all four
  oversampled audio evaluations.
- The IABC result is normalized only after the physical conversion so a 5 V
  envelope preserves the serviced level anchor. The admitted 4.7-5.3 V CEM3310
  population can extend slightly above nominal instead of being digitally
  clamped at one.
- Each complete nonlinear voice path crosses a 127-tap anti-alias low-pass
  before it enters the host-rate common summer. This keeps filter resonance,
  VCA curvature and sync products from folding through the former box-average
  stopband.
- The five post-VCA voice signals cross equal 39 kohm resistors into U480. The
  resulting common signal is their exact passive average; this replaces the
  former unexplained 0.18 host gain with the populated one-fifth network.
- The master CA3280 is distinct from the per-voice VCAs and follows the
  physical master-volume control. PCB1's R113 10 kohm linear pot is loaded by
  SD430 R4555's 100 kohm and C4184's 0.22 uF before the U480 buffer. Its loaded
  wiper voltage is smoothed with the position-dependent Thevenin resistance,
  rather than treating the control as an instantaneous digital multiplier.
- Q411 is a second grounded-base 2N4250 converter. The buffered volume voltage
  crosses R4542 and R4541, both 4.7 kohm, before reaching U479 IABC. The same
  room-temperature junction law used for Q410 reconstructs approximately
  468 uA at the five-volt endpoint. This turns the physical linear pot into a
  useful audio taper: the loaded midpoint produces approximately 2.439 V and
  42.3% of maximum control current.
- U479 is reconstructed from the Figure 3A centre slope as a separate master
  signal transfer. Its 68 kohm diode feed produces approximately 212 uA,
  close to the graph's 200 uA condition. Scaling the approximate 100 uA/V
  graph slope from 10 kohm/650 uA to the populated 15 kohm/468 uA point and
  developing current across 20 kohm in parallel with 100 kohm gives
  approximately 0.80 small-signal voltage gain at full volume. The same
  rounded sixth-order graph envelope places its source-input asymptote at
  six circuit volts.
- The master-VCA output is AC-coupled by the populated 2.2 uF C4189 into the
  parallel 20 kohm/100 kohm load formed by R4562 and R4541. The resulting
  first-order high-pass corner is approximately 4.34 Hz. C4189 is represented
  by its stored physical capacitor voltage and advanced with the exact
  exponential RC solution, so its elapsed-time decay does not inherit a host
  sample-rate approximation.
- Five paired envelope-amount profiles, five oscillator-B Poly Mod profiles
  and the common dual Wheel Mod source profile preserve the modulation-side
  CA3280 boundaries and documented diode modes.
- The three stored amount controls reach their per-voice CA3280s through the
  separate grounded-base SD333 converters: Q301/5.1k for direct filter
  envelope, Q303/5.6k for oscillator-B Poly Mod and Q304/3k for envelope Poly
  Mod. Their 0-10 V held controls now follow the same source-backed 2N4250
  junction equation as the audio VCAs instead of linear normalized gains.
- The two Poly Mod amount stages produce physical CA3280 output currents.
  U422's envelope half uses populated 22k signal/return and 120k diode-bias
  paths; U428's oscillator half retains the enabled 150k/200k waveform-source
  loading. Their currents meet at R4108's 30k load and the resulting voltage
  is bounded at the data sheet's guaranteed minimum +/-12 V output swing
  before U431's follower and the three destination networks.
- U481 is now an explicit NE5534 voltage follower on the populated +/-15 V
  rails. R4544 permanently loads its output with 1 kohm and R4543 contributes
  the measured 560 ohm jack source resistance. The accepted manufacturer
  boundaries are 24 Vpp guaranteed and 26 Vpp typical into at least 600 ohm,
  38 mA typical output current and 13 V/us typical slew rate. The high-
  impedance RackForge input leaves R4543 unloaded; a finite external-load
  fixture verifies the divider without silently assuming a particular mixer.
- All modeled audio stages exchange circuit volts through the jack. One
  explicit candidate conversion maps four jack volts to one host unit only
  after U481. The mapping is strictly linear and replaces the former host
  `tanh`, which compressed strong chords before any physical stage reached its
  own overload boundary.
- Master volume is direct, is not delayed by the CPU control scheduler and is
  preserved when programs change.

## Bounded uncertainty

The device modes, routing, approximate input impedances, waveform-source
resistors, final-VCA 20 kohm input network, Q410/Q411 identities and bias
networks, R113/R4555/C4184 master-volume network, the 2N4250 room-temperature
base-emitter curve, equal-resistor summer, linear gain-versus-bias law,
AC-coupling values and output topology are source-backed.
The deterministic population is bounded by the published 0.70-1.30
peak-output-current ratio and kept deliberately narrower. Figure 3A is a
printed bitmap, so the approximately four-division limit and sixth-order knee
are explicit bounded interpretations rather than digitized device
measurements. The oscillator and noise current networks no longer require a
one-saw loading normalization. Q410 temperature, overload-knee spread,
populated-device matching, external output load and the final four-volts-per-
host-unit conversion remain hypotheses. Neither that conversion nor a digital
full-scale limiter is present inside the analog path.
The five-volt volume reference follows the documented analog control domain,
but its populated rail and R113 end-to-end tolerance are unmeasured. U479's
approximately 100 uA/V centre slope is read from the printed Figure 3A bitmap,
not a numerical manufacturer table. Exact THD, gain law and overload require
recorded sweeps from a serviced reference instrument.

## Acceptance tests

- zero bias current closes every physical VCA boundary exactly;
- gain rises monotonically with control current;
- Q306/Q302 reach approximately 280 uA at full oscillator level and Q305
  reaches approximately 125 uA at full noise level; all three preserve their
  transistor knees and calibrated endpoints;
- the 5 V envelope reconstructs 650-680 uA IABC, has a silicon-junction knee,
  and accepts the complete bounded 4.7-5.3 V CEM3310 population;
- the linearized transfer retains more strong-signal range than the mixer VCA;
- the populated 20 kohm inputs double Figure 3A's 10 kohm source-voltage span,
  preserve calibrated small-signal gain and put the final-VCA knee inside the
  CEM3320's 10-14 Vpp output range;
- all mixer profiles remain inside published output bounds, paired halves stay
  close but distinct, and serviced final-voice small-signal gains match;
- a single 150 kohm mixer path develops approximately 10.9 mV at U464's input,
  the first filter cell presents approximately 90.909 kohm transimpedance, and
  one full saw reaches 4.0-4.8 circuit volts across all five profiles;
- parallel paths and the 200 kohm pulse path follow the finite-input loading
  law, while the complete common-noise path reaches 0.84-1.0 circuit volts;
- all transfers are finite and odd-symmetric and reject non-finite input;
- the two Poly Mod currents share one populated 30k current-to-voltage load,
  preserve waveform-dependent input loading and remain within guaranteed
  CA3280 voltage compliance;
- the master stage remains bounded while retaining multi-voice headroom;
- the loaded linear volume pot reaches zero and five volts, its midpoint is
  approximately 2.439 V, Q411 reaches 460-475 uA, and the resulting current law
  is monotonic with an approximately 42% midpoint;
- C4184 smoothing is monotonic and produces the same elapsed-time response at
  48 and 96 kHz;
- five equal 39 kohm paths produce exactly one fifth of the voice sum, U479's
  diode current remains within 7% of Figure 3A's condition and the populated
  full-volume small-signal voltage gain remains between 0.79 and 0.81;
- the 4.34 Hz coupling network rejects steady DC at every supported sample
  rate, its 100 ms capacitor decay matches the analytic value at
  44.1/48/96/192 kHz, and it retains the expected approximately 97.7%
  amplitude at 20 Hz;
- the loaded NE5534 remains exactly linear through its guaranteed +/-12 V
  span at 44.1/48/96/192 kHz, is bounded by its typical +/-13 V swing, draws
  less than the published current capability through R4544 and preserves the
  documented 560 ohm source resistance under a finite-load fixture;
- the final circuit-volts-to-host conversion is linear above unity and cannot
  masquerade as analog saturation;
- program changes preserve the physical master volume.
- the decimator preserves DC gain, produces exactly one output per four
  internal samples, passes 10 kHz essentially flat and rejects a 60 kHz
  internal tone below -80 dB relative to its input RMS.

Primary evidence: TM1000D.2 sections 2-5 and 2-8, schematics SD431-SD435 and
SD430, service adjustments 4-21 and 4-22, the CA3280 and NE5534 data sheets and
Fairchild's 2N4248/2N4249/2N4250 data. Provenance is recorded in
`SOURCE_LEDGER.md`.
