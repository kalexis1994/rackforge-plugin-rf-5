# Post-tune VCO drift model

## Accepted physical boundary

The CEM3340 data sheet describes a fully temperature-compensated VCO and gives
two limits at its stated electrical-characteristic conditions:

- oscillator drift is typically within +/-50 ppm and at most +/-200 ppm;
- temperature-coefficient cancellation is specified from -150 to +150 ppm;
- trimmed exponential-scale error is typically 0.05% and at most 0.3%.

The first limit is the boundary used by this model. A 50 ppm frequency error is
about 0.087 cent and 200 ppm is about 0.346 cent. It is therefore incorrect to
replace post-tune drift with a several-cent static detune and still present it
as normal CEM3340 temperature behaviour.

The technical manual establishes that Tune measures all ten audio VCOs and
that oscillator parameters change with age and temperature despite the IC's
temperature compensation. Tune state belongs to the instrument, not to an
individual stored program.

## Active reconstruction

RF-5 owns ten independent slow states: oscillator A and B for each of five
physical voices. A correlated board-temperature component is combined with an
independent component for each IC. The process advances at a fixed 20 Hz
control rate, so it cannot create audio-rate pitch noise, and an elapsed-time
accumulator makes its trajectory independent of the host sample rate.

Automatic tune captures the present value of all ten states as a reference.
Only subsequent motion reaches the oscillator frequency. The public engine
action `tune_oscillators` recalculates the automatic-tune table, captures that
thermal reference and refreshes the ten pitch sample/holds without changing or
serializing any patch parameter.

`VintageSpread` no longer applies a fixed opposing detune to oscillator A and
B. It now expands the hard drift envelope from the data-sheet typical limit of
50 ppm toward the maximum 200 ppm. At zero it retains the typical residual
motion because a real compensated oscillator is not mathematically static.

## Isolated hypothesis

Neither admitted source publishes the VCO warm-up curve, thermal time
constants, stochastic spectrum, correlation between adjacent chips or the
temperature present at the moment Tune is pressed. RF-5 therefore uses:

- deterministic target sequences rather than nondeterministic randomness;
- individual time constants between 37 and 97 seconds;
- a slower 180-second common component;
- target hold times of 6-25 seconds per IC and 45-120 seconds for the common
  component;
- a 28% common / 72% individual mixture.

Those values define time evolution only. They cannot increase the accepted
50/200 ppm magnitude limits. They are replaceable as one isolated model when
measurements from a real Revision 3 instrument become available.

## Acceptance tests

- all ten VCO trajectories are distinct and deterministic;
- equal elapsed time produces the same state at 44.1 and 96 kHz;
- typical and maximum modes never exceed 50 and 200 ppm respectively;
- Tune makes every current post-tune correction zero without changing Settings;
- both published drift limits convert to the expected sub-cent values.
