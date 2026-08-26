# Noise and mixer model

## Accepted hardware contract

Rev 3 contains two independent MM5837 pseudo-random sources. U375 on common
board schematic SD334 is the pink-noise source used only by Wheel Mod. Its
47 kohm input feeds an inverting LM348 stage with 100 kohm / 0.01 uF parallel
feedback: a 2.1277 low-frequency gain and an approximately 159 Hz pole.

U427 on output-board schematic SD430 is the separate white-noise source heard
in the audio path. C458's 0.1 uF coupling capacitor, R4131's 200 kohm series
resistor and R4132's 10 kohm shunt form an approximately 7.58 Hz high-pass
boundary before the common noise-level CA3280. Its output current develops
voltage across R4129's 10 kohm, U474 buffers it, and five 100 kohm paths inject
the result into the first CEM3320 cell on the voice cards. Audio noise therefore
neither shares a sequence nor a spectrum with Wheel Mod noise.

The MM5837 data sheet identifies a self-clocked 17-bit maximal-length shift
register with feedback at stages 17 and 14. Its specified sequence cycle is
1.1-2.4 seconds and its half-power point is 24-56 kHz. The chip starts in a
random non-zero state at power-up.

Noise level is one common patch control and one common CA3280 OTA. It does not
traverse the two per-voice oscillator mixer OTAs. Oscillator A and B levels use
the two halves of one separate CA3280 on each voice. The Wheel Mod source
control drives its independent pink-noise and LFO amount VCAs in opposite
directions, producing one continuous crossfade before the modulation wheel VCA
and destination switches.

## Active candidate

- Two engine-owned 17-bit LFSRs advance from distinct fixed non-zero seeds,
  representing U375 and U427 rather than one artificially shared sequence.
- Independent phase accumulators hold both candidate chip clocks constant
  across host sample rates; four-times internal processing integrates their
  transitions and analog networks without making sequence timing host-rate
  dependent.
- U375 passes through SD334's exact 47k / (100k parallel 0.01uF) pinking
  network and feeds only the Wheel Mod current mixer. Its voltage boundary
  uses the MM5837 data-sheet minimum 12 Vpp logic-level separation.
- U427 passes through SD430's 0.1uF / 200k / 10k AC-coupling network and a
  profiled common unlinearized CA3280. The candidate uses the MM5837's
  guaranteed 12 Vpp separation, U430 output current, R4129's 10k load, U474,
  each 100k distribution path and the first filter cell's 90.909k
  transimpedance instead of a normalized noise gain.
- Per-voice oscillator waveform paths retain their populated conductances:
  saw/triangle are 150k, pulse is 200k, and simultaneous selections load the
  330-ohm-shunted, approximately 100k unlinearized CA3280 input before its
  nonlinear transfer. Both U464 output currents feed CEM3320 `IN A` directly;
  the populated 100k feedback in parallel with the nominal 1M output impedance
  converts them through the same 90.909k first-cell transimpedance.
- The three audio-level cells retain their original 128-position storage and
  0-10 V output domain. Q306 and Q302 convert oscillator A and B level through
  33k each; Q305 converts common noise level through 75k. Their normalized
  endpoints preserve the serviced full-level boundary only after the physical
  2N4250 knees and distinct current laws.
- Wheel Mod source mix follows the original 128-position panel storage.
- Source mix recreates the complementary Q307/Q309 current controls: zero is
  LFO, one is noise, while intermediate gains follow their unequal populated
  8.2k and 10k-parallel-20k emitter networks rather than a linear crossfade.
- Noise joins the two-OTA oscillator mix only at each filter input, matching
  the SD431-SD435 routing.

## Bounded uncertainty

The MM5837 data sheet specifies ranges rather than a typical internal clock.
RF-5 isolates an 80 kHz candidate inside the documented limits for each IC.
Two physical self-contained oscillators will not be exactly frequency locked;
until populated devices are measured, independent state/phase removes the
false shared sequence without inventing an unsupported clock mismatch. Both
physical ICs power up from random non-zero states, while RF-5 deliberately uses
distinct fixed seeds so tests, saved sessions and live renders are reproducible.

The MM5837 sheet bounds its loaded output levels but does not publish typical
rail error, and the exact CA3280/Q307 population remains unmeasured. RF-5 uses
the guaranteed 12 Vpp separation, the populated U374 gain, both U378 input
dividers, the data-sheet OTA slope/current limit and R3113 load rather than a
normalized drive. These are bounded nominal reconstructions, not measurements
of one populated instrument.
Oscillator waveform voltage, 150/200 kohm input weighting, the 330 ohm shunt,
the manual's approximate 100 kohm unlinearized input, U464's direct current
connection and the complete common-noise routing are source-bounded. The
SD333 audio-level current converters, R4129, the distribution resistors and
the first-cell transimpedance are accepted. The candidate still uses the
MM5837's guaranteed minimum separation rather than a populated output level;
transistor temperature, the two-circuit-volts-per-internal-unit filter scale
and measured mixer saturation remain unavailable.

## Acceptance tests

- the LFSR visits every non-zero 17-bit state before repeating;
- both sources are deterministic, finite, mutually distinct and bipolar;
- the populated networks produce their 159 Hz low-pass and 7.58 Hz high-pass
  corners, with substantially more difference energy in the white path;
- both RMS levels stay within 20% across all supported sample rates;
- both generators continue through silence and are not reset by notes;
- a noise-only patch reaches the per-voice signal path;
- Q306/Q302/Q305 produce the expected 33k/33k/75k full-level currents, retain
  a silicon-junction knee and rise monotonically through all 128 positions;
- one full saw develops approximately 10.9 mV at U464's shunted input and
  reaches 2.0-2.4 internal filter units through the populated current path;
- full common white noise reaches 0.42-0.50 internal filter units through
  R4129, U474, the 100k distribution path and first-cell transimpedance;
- Wheel Mod source mix audibly changes from the disabled LFO side to noise;
- package parameters and state expose both physical controls.

Primary evidence: Sequential Circuits TM1000D.2 sections 2-3, 2-5, service
tests 4-3 and 4-7, schematics SD334 and SD430, and the National Semiconductor
MM5837 data sheet. Provenance and hashes are recorded in `SOURCE_LEDGER.md`.
