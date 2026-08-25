# Noise and mixer model

## Accepted hardware contract

Rev 3 contains two independent MM5837 pseudo-random sources. U375 on common
board schematic SD334 is the pink-noise source used only by Wheel Mod. Its
47 kohm input feeds an inverting LM348 stage with 100 kohm / 0.01 uF parallel
feedback: a 2.1277 low-frequency gain and an approximately 159 Hz pole.

U427 on output-board schematic SD430 is the separate white-noise source heard
in the audio path. C458's 0.1 uF coupling capacitor, R4131's 200 kohm series
resistor and R4132's 10 kohm shunt form an approximately 7.58 Hz high-pass
boundary before the common noise-level CA3280. That OTA output is buffered and
distributed to the dedicated noise input on all five voice cards. Audio noise
therefore neither shares a sequence nor a spectrum with Wheel Mod noise.

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
  network and feeds only the Wheel Mod crossfade.
- U427 passes through SD430's 0.1uF / 200k / 10k AC-coupling network and a
  profiled common unlinearized CA3280; its buffered output reaches all five
  filters.
- Noise level and source mix follow the original 128-position panel storage.
- Source mix is a complementary linear crossfade: zero is LFO, one is noise.
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

The output gain after the pinking network and the exact CA3280 drive at the
populated board are not numerically established by the service manual. The
current noise gain and normalized OTA drive are therefore isolated candidates.
Oscillator waveform voltage, 150/200 kohm input weighting and the common-noise
routing are source-bounded, but this block does not claim measured mixer
saturation fidelity.

## Acceptance tests

- the LFSR visits every non-zero 17-bit state before repeating;
- both sources are deterministic, finite, mutually distinct and bipolar;
- the populated networks produce their 159 Hz low-pass and 7.58 Hz high-pass
  corners, with substantially more difference energy in the white path;
- both RMS levels stay within 20% across all supported sample rates;
- both generators continue through silence and are not reset by notes;
- a noise-only patch reaches the per-voice signal path;
- Wheel Mod source mix audibly changes from the disabled LFO side to noise;
- package parameters and state expose both physical controls.

Primary evidence: Sequential Circuits TM1000D.2 sections 2-3, 2-5, service
tests 4-3 and 4-7, schematics SD334 and SD430, and the National Semiconductor
MM5837 data sheet. Provenance and hashes are recorded in `SOURCE_LEDGER.md`.
