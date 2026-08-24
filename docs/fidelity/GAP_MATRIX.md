# Fidelity gap matrix

| Block | Active implementation | Evidence state | Release status |
| --- | --- | --- | --- |
| Voice allocation | Deterministic five-voice round robin | Architecture inventory | Baseline |
| VCO A/B | Independent, free-running, 4x-oversampled saw/pulse A and saw/triangle/pulse B with PolyBLEP edges | Rev 3 CEM3340 topology and CV scale accepted; output geometry unmeasured | Candidate |
| Hard sync | Oscillator B wrap resets oscillator A at the internal oversampling rate | Rev 3 routing accepted; exact reset edge and transient unmeasured | Candidate |
| LFO | One common free-running saw/triangle/square source with additive switches and 50% square | Rev 3 topology and service behavior accepted; absolute frequency range unmeasured | Candidate |
| Wheel Mod | MIDI CC1 routes a complementary LFO/pink-noise source mix to A/B frequency, A/B pulse width and filter through original destination switches | Rev 3 routing accepted; modulation depths unmeasured | Candidate |
| Mixer/noise | Separate additive A/B levels plus one shared MM5837-class pink source feeding each voice | MM5837 sequence and SD334 filter topology accepted; chip clock, gain and CA3280 overload unmeasured | Candidate |
| Poly Mod | Per-voice negative filter-envelope and audio-rate oscillator-B sources, two amount controls and independent oscillator-A frequency, oscillator-A pulse-width and filter destinations | Rev 3 sources, polarity, destinations and summing paths accepted; depths and OTA response unmeasured | Candidate |
| Filter | Four cascaded nonlinear TPT cells at 4x rate, 1 V/oct keyboard tracking, audio-rate modulation and self-oscillation | CEM3320 topology, control law and service oscillation window accepted; calibration and overload unmeasured | Candidate |
| Filter envelope | Independent true-RC CEM3310 candidate driving cutoff and inverted Poly Mod | Data-sheet shape, endpoints and Rev 3 polarity accepted; panel taper unmeasured | Candidate |
| Amplifier envelope | Independent true-RC CEM3310 candidate driving voice level | Data-sheet shape and endpoints accepted; panel taper and VCA interaction unmeasured | Candidate |
| VCA/output | Fixed scaling | CA3280 topology accepted; gain staging unmeasured | Replace |
| Unison/glide | Five-voice last-note Unison with held-note fallback and Unison-only linear CV Glide | Rev 3 routing and service limits accepted; priority and panel taper unmeasured | Candidate |
| Control scanning/CV | VCO level, pulse-width, coarse and fine controls quantize to 128 positions; other host values remain direct | 24-pot order, 7/14-bit resolution, loop timing and 38 S/H destinations accepted | Partial |
| Performance controls | Note on/off, all notes off, CC1, CC64 sustain and 14-bit pitch bend; velocity correctly ignored | MIDI contract plus original service behaviour | Candidate |
| Factory programs | Four original baseline programs | Internal | Expand |
