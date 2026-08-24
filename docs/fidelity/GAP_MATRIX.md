# Fidelity gap matrix

| Block | Active implementation | Evidence state | Release status |
| --- | --- | --- | --- |
| Voice allocation | Deterministic five-voice round robin | Architecture inventory | Baseline |
| VCO A/B | Independent, free-running, 4x-oversampled saw/pulse A and saw/triangle/pulse B with PolyBLEP edges; ten deterministic post-tune drift paths bounded to CEM3340 data-sheet limits | Rev 3 topology, CV scale and 50/200 ppm drift limits accepted; output geometry and thermal time evolution unmeasured | Candidate |
| Hard sync | Oscillator B wrap resets oscillator A at the internal oversampling rate | Rev 3 routing accepted; exact reset edge and transient unmeasured | Candidate |
| LFO | One common free-running saw/triangle/square source with additive switches and 50% square | Rev 3 topology and service behavior accepted; absolute frequency range unmeasured | Candidate |
| Wheel Mod | MIDI CC1 routes a complementary LFO/pink-noise source mix to A/B frequency, A/B pulse width and filter through original destination switches | Rev 3 routing accepted; modulation depths unmeasured | Candidate |
| Mixer/noise | Separate unlinearized CA3280 transfers for A, B and shared MM5837-class pink noise before each filter | CA3280 mode, MM5837 sequence and SD334 topology accepted; normalized drive and overload unmeasured | Candidate |
| Poly Mod | Per-voice negative filter-envelope and audio-rate oscillator-B sources, two amount controls and independent oscillator-A frequency, oscillator-A pulse-width and filter destinations | Rev 3 sources, polarity, destinations and summing paths accepted; depths and OTA response unmeasured | Candidate |
| Filter | Four cascaded nonlinear TPT cells at 4x rate, 1 V/oct keyboard tracking, audio-rate modulation and self-oscillation | CEM3320 topology, control law and service oscillation window accepted; calibration and overload unmeasured | Candidate |
| Filter envelope | Independent true-RC CEM3310 candidate driving cutoff and inverted Poly Mod | Data-sheet shape, endpoints and Rev 3 polarity accepted; panel taper unmeasured | Candidate |
| Amplifier envelope | Independent true-RC CEM3310 candidate driving voice level | Data-sheet shape and endpoints accepted; panel taper and VCA interaction unmeasured | Candidate |
| VCA/output | Linearized CA3280 per voice at 4x, equal five-input summer, master CA3280 and smooth host output boundary | SD430/SD431 topology and CA3280 transfer modes accepted; gain staging and THD unmeasured | Candidate |
| Unison/glide | Five-voice last-note Unison with held-note fallback and Unison-only linear CV Glide | Rev 3 routing and service limits accepted; priority and panel taper unmeasured | Candidate |
| Control scanning/CV | Unified 62-position CPU cycle with 24 pot reads, 38 independent S/H strobes, bounded droop and 6 ms idle/11 ms changed timing | Topology, pot order, resolution, loop timing and service droop limit accepted; exact ROM strobe order and component leakage population unmeasured | Candidate |
| Automatic tune | Ten-VCO tune mux, 2.5 MHz period counter, 14-bit successive approximation, 200-byte C0-C9 bias table, runtime interpolation and non-serialized retune/reconditioning action | C3-C9 measurement and C0-C2 extrapolation accepted; exact ROM extrapolation arithmetic and measured component population unavailable | Candidate |
| Performance controls | Note on/off, all notes off, CC1, CC64 sustain and 14-bit pitch bend; velocity correctly ignored | MIDI contract plus original service behaviour | Candidate |
| Factory programs | Four original baseline programs | Internal | Expand |
