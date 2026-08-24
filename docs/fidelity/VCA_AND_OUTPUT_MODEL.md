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

- Unlinearized mixer VCAs use an odd-symmetric differential-pair transfer and
  linear control-current gain. Oscillator A, oscillator B and common noise are
  shaped independently before summing into each filter.
- The final VCA uses a substantially wider linearized transfer and is evaluated
  inside the same four-times-oversampled loop as the filter.
- The five post-VCA voice signals are summed with equal gain.
- The master CA3280 is distinct from the per-voice VCAs and follows the
  physical master-volume control.
- The NE5534/output network is treated as linear inside its headroom. A smooth
  host full-scale boundary replaces the previous hard digital clamp.
- Master volume is direct, is not delayed by the CPU control scheduler and is
  preserved when programs change.

## Bounded uncertainty

The device modes, routing, equal-resistor summer and output topology are
source-backed. Normalized voltage scale, OTA drive constants, control-current
taper, resistor/capacitor tolerances, calibrated per-voice level and the
circuit-volts-to-host-full-scale conversion are hypotheses. Exact THD and
overload require recorded sweeps from a serviced reference instrument.

## Acceptance tests

- zero bias current closes both VCA candidates exactly;
- gain rises monotonically with control current;
- the linearized transfer retains more strong-signal range than the mixer VCA;
- all transfers are finite and odd-symmetric;
- the master stage remains bounded while retaining multi-voice headroom;
- program changes preserve the physical master volume.

Primary evidence: TM1000D.2 sections 2-5 and 2-8, schematics SD431-SD435 and
SD430, service adjustments 4-21 and 4-22, and the CA3280 data sheet. Provenance
is recorded in `SOURCE_LEDGER.md`.
