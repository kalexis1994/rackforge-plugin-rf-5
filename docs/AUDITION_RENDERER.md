# Deterministic no-UI audition renderer

RF-5 can be evaluated without RackForge's graphical panel. The
`rf-5-audition` executable drives the real DSP engine with sample-accurate MIDI
events and writes twenty-eight unnormalized 48 kHz mono PCM WAV files:

1. baseline polyphonic chords;
2. strong dual-VCO filter drive;
3. the five filter profiles through the populated modified-linear resonance
   circuit near calibration;
4. Wheel Mod vibrato;
5. Wheel Mod pulse-width modulation;
6. Wheel Mod filter modulation.
7. repeated percussive notes through the ten CEM3310 profiles;
8. a slow chord with independent, populated CEM3310 timing trajectories.
9. strong five-voice chords through the complete CA3280/output population;
10. the common noise-level CA3280 feeding all five filter noise inputs.
11. oscillator-B audio-rate Poly Mod through its five amount VCAs;
12. descending filter-envelope Poly Mod through the five paired envelope VCAs;
13. the noise half of the Wheel Mod source CA3280 routed to filter cutoff.
14. the one-edge CEM3340 conventional hard-sync path with oscillator B removed
    from audio.
15. the documented first-five/earliest-used polyphonic assignment sequence.
16. low-note-priority Unison with legato retuning and envelope continuity.
17. the slow region of the common LFO's circuit-derived sweep.
18. a fast point on the same exponential common-LFO law.
19. medium Unison transitions through the populated Glide control network.
20. a C-major progression using a V8.1-quantized just-intonation Scale program.
21. maximum stored Release pots overridden by the programmable global RELEASE
    switch.
22. centred, sub-threshold and bipolar pitch-wheel moves through the SD334
    diode deadband.
23. matched oscillator-A/B coarse pitch with OSC B FINE at physical zero;
24. the same sustained note with OSC B FINE at its documented one-semitone
    endpoint.
25. oscillator A pulse at the documented 1% panel endpoint;
26. the nearest seven-bit code to a 50% square wave;
27. oscillator A pulse at the documented 99% panel endpoint.
28. oscillator B triangle in isolation across five octaves.

Run:

```bash
cargo run --release -p rf-5-audition
```

The default destination is `artifacts/auditions`. An alternative output
directory may be passed as the first argument. Alongside the WAV files, the
renderer writes `manifest.json` with peak, RMS, DC and clipped-sample counts.
It never peak-normalizes, applies loudness matching or post-processes the DSP
output, because those operations would hide gain-staging changes.

Short renders of all twenty-eight scenes are evaluated twice in the normal test
suite and must be sample-identical, finite, audible and bounded. The full
renderer additionally rejects silence, exhausted headroom, clipped samples or
excessive DC before writing a successful manifest. Release validation renders
the complete suite twice and compares file hashes. These files are listening
evidence, not measurements of an original instrument.
