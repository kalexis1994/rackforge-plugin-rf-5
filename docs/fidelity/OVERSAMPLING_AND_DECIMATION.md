# Oversampling and decimation

## Processing boundary

Every voice evaluates both CEM3340 candidates, hard sync, audio-rate Poly Mod,
the dual CA3280 mixer, the nonlinear four-pole CEM3320 candidate and the final
CA3280 VCA at four times the host sample rate. The common five-input summer,
master VCA and physical output coupling remain at host rate.

Placing reconstruction after the final per-voice nonlinearity is deliberate:
hard-sync edges, resonance and both OTA transfer curves can all create new
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

The passband/stopband figures describe this digital reconstruction filter, not
a claimed analog bandwidth for the original instrument. Populated-unit output
bandwidth and spectral measurements remain separate evidence-gated questions.
