# Automatic VCO tuning model

## Accepted hardware behaviour

The Revision 3 technical manual fixes the automatic-tune architecture:

- the tune multiplexer selects each of the ten audio VCOs independently;
- the tune comparator converts the selected sawtooth into counter pulses;
- an 8253 interval timer measures the period against the 2.5 MHz CPU clock;
- successive approximation finds the required 14-bit oscillator CV;
- C3-C9 are measured directly, using 1, 1, 2, 4, 8, 16 and 32 oscillator
  cycles respectively;
- C0-C2 are extrapolated from the measured curve;
- ten octave biases for each of ten VCOs occupy a 200-byte scratchpad table;
- while playing, the CPU interpolates the surrounding octave biases for the
  requested semitone.

The calibration is machine state, not program data. RF-5 rebuilds it when the
audio engine is prepared and does not serialize it into patches or host state.
The common LFO is not one of the ten tune-multiplexer channels.

## Active reconstruction

`rf_5_voice::autotune` implements the complete data path above. Every physical
voice maps to independent A and B tune channels. The virtual counter quantizes
period measurements at 2.5 MHz, and the search evaluates all fourteen writable
DAC bits from most to least significant. Its result is stored as a signed bias
relative to the ideal 83 mV semitone step.

The V8.1 operating ROM resolves two arithmetic details omitted by the manual.
Offsets `0x0101-0x0125` form the signed 16-bit difference between the measured
C4 and C3 entries, then subtract that same difference successively to write C2,
C1 and C0. Offsets `0x03ee-0x0483` divide the pitch coordinate by twelve, select
the surrounding octave points and apply the signed bias difference through
coarse and fine multiplication lookups. RF-5 independently reduces those
lookups to nearest-twelfth arithmetic plus their small set of original rounding
quirks; no firmware table is copied into the plug-in.

The ROM's internal DAC word advances by `0x0100` per semitone. Its tune writer
keeps bit zero clear and rotates the low byte before the DAC latch, proving that
one semitone is exactly 128 of the fourteen writable DAC positions. This agrees
with the manual's nominal 83 mV step and 651 uV writable resolution without
mixing their rounded decimal values in the pitch calculation. The ROM is
neither distributed nor loaded at runtime; its admitted hash and evidence
boundary are recorded in `SOURCE_LEDGER.md`.

At render time, oscillator A and B each receive the residual error produced by
their own calibrated table. Bias is reconstructed at the twelve discrete key
positions between adjacent octave points, with the same split multiply and
signed application as V8.1, then applied at writable-DAC resolution and retained
by the corresponding physical sample/hold cell. This keeps performance
wheel and audio-rate modulation outside the digital tune table, matching their
separate paths.

The acceptance gate evaluates every semitone from C0 through C9 on all ten
VCOs. The deterministic validation population must remain below 0.75 cent mean
absolute error and 4 cents worst-case error. The latter admits the slightly
larger low-octave residual produced by the authentic first-order extrapolation
instead of tuning the test around the former quadratic fit. These are gates for
this reconstruction, not claimed factory specifications.

## Isolated uncertainty

The C0-C2 arithmetic, runtime interpolation and writable-DAC scale are no longer
uncertain for the Rev 3 V8.1 target. They are independently tied to the manual's
tune architecture and the admitted operating ROM. Remaining uncertainty begins
at the physical VCO population and any revision-specific difference outside
this target.

Temperature motion after this measurement is deliberately outside the held
DAC table, because it originates inside the compensated VCO at a given control
voltage. Pressing the engine's momentary Tune action captures the current
thermal state; its bounded evolution is described in
[`VCO_DRIFT_MODEL.md`](VCO_DRIFT_MODEL.md).

Likewise, the ten offset, scale and curvature profiles are deterministic
component-tolerance fixtures. They make the calibration mechanism observable
and testable; they are not measurements from a specific vintage instrument.
When legally usable recordings or bench measurements become available, only
those profiles need replacement. No firmware or original program data is
shipped or loaded at runtime.
