# Noise and mixer model

## Accepted hardware contract

The Rev 3 common-analog schematic SD334 contains one MM5837 pseudo-random
noise source. The source is shared by the five voices and by Wheel Mod. The
technical manual calls the resulting signal pink noise and shows a 47 kohm
input feeding an inverting LM348 stage with a 100 kohm / 0.01 uF parallel
feedback network. The resulting first-order pole is approximately 159 Hz.

The MM5837 data sheet identifies a self-clocked 17-bit maximal-length shift
register with feedback at stages 17 and 14. Its specified sequence cycle is
1.1-2.4 seconds and its half-power point is 24-56 kHz. The chip starts in a
random non-zero state at power-up.

Noise level is one common patch control, but the resulting signal enters every
voice mixer ahead of its CEM3320 filter. Oscillator A and B levels use separate
RCA/CA3280 OTA stages in each voice. The Wheel Mod source control drives the
noise and LFO amount VCAs in opposite directions, producing one continuous
crossfade before the modulation wheel VCA and destination switches.

## Active candidate

- One engine-owned 17-bit LFSR advances from a fixed non-zero seed.
- A phase accumulator holds the candidate chip clock constant across host
  sample rates; four-times internal processing integrates transitions and the
  analog pole without making sequence timing host-rate dependent.
- The SD334 100 kohm / 0.01 uF feedback pole is evaluated as a one-pole analog
  low-pass approximation.
- One filtered sample is distributed to all voices and to the Wheel Mod
  crossfade each host sample.
- Noise level and source mix follow the original 128-position panel storage.
- Source mix is a complementary linear crossfade: zero is LFO, one is noise.
- Noise joins oscillator A and B before the current baseline filter.

## Bounded uncertainty

The MM5837 data sheet specifies ranges rather than a typical internal clock.
RF-5 isolates an 80 kHz candidate inside the documented limits. The physical
unit powers up from a random non-zero state, while RF-5 deliberately uses a
fixed seed so tests, saved sessions and live renders are reproducible.

The output gain after the pinking network and the CA3280 gain/overload curves
are not numerically established by the service manual. The current noise gain
is therefore an isolated candidate, and oscillator/noise mixer gains remain
linear until controlled measurements justify OTA nonlinearity. This block does
not claim final mixer saturation fidelity.

## Acceptance tests

- the LFSR visits every non-zero 17-bit state before repeating;
- the source is deterministic, finite and bipolar;
- RMS level stays within 20% across all supported sample rates;
- the shared generator continues through silence and is not reset by notes;
- a noise-only patch reaches the per-voice signal path;
- Wheel Mod source mix audibly changes from the disabled LFO side to noise;
- package parameters and state expose both physical controls.

Primary evidence: Sequential Circuits TM1000D.2 sections 2-3, 2-5, service
tests 4-3 and 4-7, schematic SD334, and the National Semiconductor MM5837 data
sheet. Provenance and hashes are recorded in `SOURCE_LEDGER.md`.
