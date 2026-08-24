# CA3280 mixer, VCA and output model

## Accepted hardware contract

The Revision 3 signal path uses several RCA CA3280 operational
transconductance amplifiers rather than one generic gain stage. Each voice
routes the selected oscillator-A and oscillator-B waveforms through separate
CA3280 mixer VCAs with the linearizing-diode terminal cut off. The filtered
signal then enters a CA3280 final VCA whose linearizing terminal is active and
whose bias current is controlled directly by the amplifier envelope.

Five equal 39 kohm input resistors feed the common inverting voice summer. A
second linearized CA3280 applies the physical volume control, followed by an
NE5534 buffer and the back-panel output network. Balance trimmers remove OTA
DC offset and the service procedure separately calibrates final-VCA balance
and per-voice volume.

## Active candidate

- Each voice has one deterministic dual-OTA mixer profile. Its oscillator A
  and B halves retain close but non-identical transconductance and overload
  knees inside the CA3280 data-sheet output-current bounds.
- One separate unlinearized CA3280 sets common noise level before the result is
  distributed to the five CEM3320 noise inputs; noise does not pass through a
  fictitious third mixer OTA on every voice card.
- The five final VCAs use substantially wider diode-linearized transfers and
  are evaluated inside the same four-times-oversampled loop as the filters.
  Their small-signal gain is equal after the documented per-voice service
  adjustment, while their strong-signal knees remain distinct.
- The five post-VCA voice signals are summed with equal gain.
- The master CA3280 is distinct from the per-voice VCAs and follows the
  physical master-volume control.
- The NE5534/output network is treated as linear inside its headroom. A smooth
  host full-scale boundary replaces the previous hard digital clamp.
- Master volume is direct, is not delayed by the CPU control scheduler and is
  preserved when programs change.

## Bounded uncertainty

The device modes, routing, equal-resistor summer, linear gain-versus-bias law
and output topology are source-backed. The deterministic population is bounded
by the published 0.70-1.30 peak-output-current ratio and kept deliberately
narrower. Normalized voltage scale, overload-knee spread, populated-device
matching and circuit-volts-to-host-full-scale conversion remain hypotheses.
Exact THD and overload require recorded sweeps from a serviced reference
instrument.

## Acceptance tests

- zero bias current closes every physical VCA boundary exactly;
- gain rises monotonically with control current;
- the linearized transfer retains more strong-signal range than the mixer VCA;
- all mixer profiles remain inside published output bounds, paired halves stay
  close but distinct, and serviced final-voice small-signal gains match;
- all transfers are finite and odd-symmetric and reject non-finite input;
- the master stage remains bounded while retaining multi-voice headroom;
- program changes preserve the physical master volume.

Primary evidence: TM1000D.2 sections 2-5 and 2-8, schematics SD431-SD435 and
SD430, service adjustments 4-21 and 4-22, and the CA3280 data sheet. Provenance
is recorded in `SOURCE_LEDGER.md`.
