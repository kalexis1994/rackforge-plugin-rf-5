# CA3280 mixer, VCA and output model

## Accepted hardware contract

The Revision 3 signal path uses several RCA CA3280 operational
transconductance amplifiers rather than one generic gain stage. Each voice
routes the selected oscillator-A and oscillator-B waveforms through separate
CA3280 mixer VCAs with the linearizing-diode terminal cut off. The filtered
signal then enters a CA3280 final VCA whose linearizing terminal is active and
whose bias current is controlled by the amplifier envelope through Q410 and
two populated 3.3 kohm resistors. Q410 is a grounded-base Fairchild 2N4250 PNP;
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
  200 kohm pulse paths. A single saw remains the level anchor; parallel
  waveform selections produce the source-backed passive loading before the
  nonlinear current transfer.
- One separate unlinearized CA3280 sets common noise level before the result is
  distributed to the five CEM3320 noise inputs; noise does not pass through a
  fictitious third mixer OTA on every voice card.
- The five final VCAs use substantially wider diode-linearized transfers and
  are evaluated inside the same four-times-oversampled loop as the filters.
  Their small-signal gain is equal after the documented per-voice service
  adjustment, while their strong-signal knees remain distinct.
- The final-VCA signal transfer is separate from the generic linearized
  modulation OTA. Intersil Figure 3A supplies two bounded landmarks at 650 uA
  IABC and 200 uA diode current: a long linear centre and rounded current limit
  at approximately four horizontal 1 V divisions with 10 kohm inputs. SD431's
  populated 20 kohm/20 kohm inputs double the source-voltage span. With the
  filter candidate's explicit 2 V/internal-unit conversion, a sixth-order
  smooth norm places the asymptote at four internal units and starts measurable
  compression inside the CEM3320 population's 10-14 Vpp output range.
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
  three internal units.
- The master-VCA output is AC-coupled by the populated 2.2 uF C4189 into the
  parallel 20 kohm/100 kohm load formed by R4562 and R4541. The resulting
  first-order high-pass corner is approximately 4.34 Hz.
- Five paired envelope-amount profiles, five oscillator-B Poly Mod profiles
  and the common dual Wheel Mod source profile preserve the modulation-side
  CA3280 boundaries and documented diode modes.
- The NE5534 and its 1 kohm shunt/560 ohm output-isolation network are treated
  as linear inside their headroom. The 560 ohm resistor is not modeled as a
  fixed divider because the external load is unspecified. A smooth host
  full-scale boundary replaces the previous hard digital clamp.
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
measurements. The one-saw loading normalization, normalized voltage scale,
Q410 temperature, overload-knee spread, populated-device matching, external
output load and circuit-volts-to-host-full-scale conversion remain hypotheses.
The five-volt volume reference follows the documented analog control domain,
but its populated rail and R113 end-to-end tolerance are unmeasured. U479's
approximately 100 uA/V centre slope is read from the printed Figure 3A bitmap,
not a numerical manufacturer table. Exact THD, gain law and overload require
recorded sweeps from a serviced reference instrument.

## Acceptance tests

- zero bias current closes every physical VCA boundary exactly;
- gain rises monotonically with control current;
- the 5 V envelope reconstructs 650-680 uA IABC, has a silicon-junction knee,
  and accepts the complete bounded 4.7-5.3 V CEM3310 population;
- the linearized transfer retains more strong-signal range than the mixer VCA;
- the populated 20 kohm inputs double Figure 3A's 10 kohm source-voltage span,
  preserve calibrated small-signal gain and put the final-VCA knee inside the
  CEM3320's 10-14 Vpp output range;
- all mixer profiles remain inside published output bounds, paired halves stay
  close but distinct, and serviced final-voice small-signal gains match;
- a single 150 kohm mixer path preserves the existing level anchor, while
  parallel paths and the 200 kohm pulse path follow the finite-input loading
  law;
- all transfers are finite and odd-symmetric and reject non-finite input;
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
  rate while retaining the expected approximately 97.7% amplitude at 20 Hz;
- program changes preserve the physical master volume.
- the decimator preserves DC gain, produces exactly one output per four
  internal samples, passes 10 kHz essentially flat and rejects a 60 kHz
  internal tone below -80 dB relative to its input RMS.

Primary evidence: TM1000D.2 sections 2-5 and 2-8, schematics SD431-SD435 and
SD430, service adjustments 4-21 and 4-22, the CA3280 data sheet and Fairchild's
2N4248/2N4249/2N4250 data. Provenance is recorded in `SOURCE_LEDGER.md`.
