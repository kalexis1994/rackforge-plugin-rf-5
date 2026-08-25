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

Five equal 39 kohm input resistors feed the common inverting voice summer. A
second linearized CA3280 applies the physical volume control, followed by an
NE5534 buffer and the back-panel output network. Balance trimmers remove OTA
DC offset and the service procedure separately calibrates final-VCA balance
and per-voice volume.

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
- The five post-VCA voice signals are summed with equal gain.
- The master CA3280 is distinct from the per-voice VCAs and follows the
  physical master-volume control.
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
resistors, Q410 identity and bias network, the 2N4250 room-temperature
base-emitter curve, equal-resistor summer, linear gain-versus-bias law,
AC-coupling values and output topology are source-backed. The deterministic population is
bounded by the published 0.70-1.30 peak-output-current ratio and kept
deliberately narrower. The one-saw loading normalization, normalized voltage
scale, Q410 temperature, overload-knee spread, populated-device matching,
external output load and circuit-volts-to-host-full-scale conversion remain
hypotheses. Exact THD and overload require recorded sweeps from a serviced
reference instrument.

## Acceptance tests

- zero bias current closes every physical VCA boundary exactly;
- gain rises monotonically with control current;
- the 5 V envelope reconstructs 650-680 uA IABC, has a silicon-junction knee,
  and accepts the complete bounded 4.7-5.3 V CEM3310 population;
- the linearized transfer retains more strong-signal range than the mixer VCA;
- all mixer profiles remain inside published output bounds, paired halves stay
  close but distinct, and serviced final-voice small-signal gains match;
- a single 150 kohm mixer path preserves the existing level anchor, while
  parallel paths and the 200 kohm pulse path follow the finite-input loading
  law;
- all transfers are finite and odd-symmetric and reject non-finite input;
- the master stage remains bounded while retaining multi-voice headroom;
- the 4.34 Hz coupling network rejects steady DC at every supported sample
  rate while retaining the expected approximately 97.7% amplitude at 20 Hz;
- program changes preserve the physical master volume.
- the decimator preserves DC gain, produces exactly one output per four
  internal samples, passes 10 kHz essentially flat and rejects a 60 kHz
  internal tone below -80 dB relative to its input RMS.

Primary evidence: TM1000D.2 sections 2-5 and 2-8, schematics SD431-SD435 and
SD430, service adjustments 4-21 and 4-22, the CA3280 data sheet and Fairchild's
2N4248/2N4249/2N4250 data. Provenance is recorded in `SOURCE_LEDGER.md`.
