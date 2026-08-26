# Oversampling and decimation

## Processing boundary

Every voice evaluates both CEM3340 candidates, hard sync, audio-rate Poly Mod,
the dual CA3280 mixer, the nonlinear four-pole CEM3320 candidate and the final
CA3280 VCA at four times the host sample rate. The common five-input summer,
master VCA and physical output coupling remain at host rate.

Saw and pulse steps use a two-host-sample PolyBLEP correction so the 1%/99%
pulse-width endpoints remain controlled when both edges occupy one short
window. Oscillator B's continuous asymmetric triangle instead uses a periodic
PolyBLAMP over one internal sample at each slope transition. The correction
changes only the corner neighborhoods and precedes both its audio and Poly Mod
routes.

Placing reconstruction after the final per-voice nonlinearity is deliberate:
hard-sync discontinuities, resonance and both OTA transfer curves can all create new
content above host Nyquist. Filtering only the oscillators would leave those
later products free to fold into the audible band.

## Reconstruction filter

The previous implementation averaged four consecutive internal samples. That
box filter had unity DC gain but a shallow stopband, so it was not a sufficient
anti-alias boundary for a resonant nonlinear path.

The active candidate uses a 127-tap, linear-phase, Kaiser-windowed sinc at the
four-to-one boundary. Its cutoff is the host Nyquist frequency and its taps are
normalized to unity DC gain. The fixed filter adds 63 internal samples of group
delay: 15.75 host samples, approximately 0.357 ms at 44.1 kHz or 0.328 ms at
48 kHz. It adds no public parameter and no serialized state.

Numerical regressions verify:

- exactly one output sample for every four internal samples;
- unity steady-state DC gain;
- less than 0.001 relative gain error at 10 kHz/44.1 kHz;
- more than 0.93 relative gain at 20 kHz/44.1 kHz;
- less than 0.0001 relative RMS at 60 kHz with a 48 kHz host rate;
- deterministic reset and containment of non-finite input.

An in-crate radix-2 spectral harness additionally evaluates complete
oscillator-plus-decimator renders at 44.1, 48, 96 and 192 kHz. Four pitch
regions cover saw, square, triangle and the 1%/99% pulse endpoints; periodic
fractional hard sync is covered at three pitch regions. Energy outside the
mathematically valid harmonic bins must remain below -40 dB relative to total
AC energy.

The passband/stopband figures describe this digital reconstruction filter, not
a claimed analog bandwidth for the original instrument. Populated-unit output
bandwidth and spectral measurements remain separate evidence-gated questions.
