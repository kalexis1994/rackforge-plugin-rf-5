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

At render time, oscillator A and B each receive the residual error produced by
their own calibrated table. Bias is linearly interpolated between adjacent
octave points, then applied at DAC-code resolution. This keeps performance
wheel and audio-rate modulation outside the digital tune table, matching their
separate paths.

The acceptance gate evaluates every semitone from C0 through C9 on all ten
VCOs. The deterministic validation population must remain below 0.75 cent mean
absolute error and 2.5 cents worst-case error. Those are engineering gates for
this reconstruction, not claimed factory specifications.

## Isolated uncertainty

The documents prove that C0-C2 are extrapolated from the C3-C9 curve but do not
publish the operating ROM's exact arithmetic. RF-5 currently fits a quadratic
to all seven measured octave biases and evaluates its lower three points. That
choice is isolated in `extrapolate_lower_octaves`.

Likewise, the ten offset, scale and curvature profiles are deterministic
component-tolerance fixtures. They make the calibration mechanism observable
and testable; they are not measurements from a specific vintage instrument.
When legally usable recordings or bench measurements become available, only
those profiles and the extrapolation hypothesis need replacement. No firmware
or original program data is shipped or loaded at runtime.
