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

The V8.1 operating ROM resolves the lower-octave arithmetic omitted by the
manual. Offsets `0x0101-0x0125` form the signed 16-bit difference between the
measured C4 and C3 entries, then subtract that same difference successively to
write C2, C1 and C0. RF-5 now reproduces this first-order extrapolation instead
of fitting an unrelated quadratic through C3-C9. The ROM is neither distributed
nor loaded at runtime; its admitted hash and evidence boundary are recorded in
`SOURCE_LEDGER.md`.

At render time, oscillator A and B each receive the residual error produced by
their own calibrated table. Bias is linearly interpolated between adjacent
octave points, applied at DAC-code resolution and retained by the corresponding
physical sample/hold cell. This keeps performance
wheel and audio-rate modulation outside the digital tune table, matching their
separate paths.

The acceptance gate evaluates every semitone from C0 through C9 on all ten
VCOs. The deterministic validation population must remain below 0.75 cent mean
absolute error and 4 cents worst-case error. The latter admits the slightly
larger low-octave residual produced by the authentic first-order extrapolation
instead of tuning the test around the former quadratic fit. These are gates for
this reconstruction, not claimed factory specifications.

## Isolated uncertainty

The C0-C2 arithmetic is no longer uncertain. The exact lower-octave
extrapolation is independently tied to the manual's tune architecture and the
admitted V8.1 ROM. Remaining uncertainty begins at the physical VCO population,
the still-candidate runtime interpolation details and any revision-specific
difference outside this Rev 3 target.

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
