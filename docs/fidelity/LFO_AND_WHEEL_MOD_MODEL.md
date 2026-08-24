# LFO and Wheel Mod model

## Accepted hardware contract

The Rev 3 technical manual identifies one common CEM3340 LFO in addition to
the ten audio oscillators. It is not keyboard controlled and is not restarted
by note events. Saw, triangle and square outputs pass through independent
switches to the common modulation bus, so the selected shapes are additive.
The square wave has a fixed 50% duty cycle.

The Wheel Mod service test verifies five switch destinations: oscillator A
frequency, oscillator B frequency, oscillator A pulse width, oscillator B
pulse width and filter cutoff. MIDI CC1 represents the physical modulation
wheel in RF-5.

## Active candidate

- One engine-owned, free-running phase is evaluated once per output sample and
  distributed to every active and inactive voice.
- Frequency follows an exponential mapping from the scanned 7-bit panel value.
- Saw, triangle and square are bipolar and independently summable.
- Wheel Mod amount is a live performance value and is not stored in a program.
- Enabled frequency destinations currently span a candidate maximum of one
  octave at full wheel and unit waveform amplitude.
- Enabled pulse-width destinations span a candidate normalized depth of 0.48.
- The filter destination spans a candidate normalized depth of 0.45 until the
  CEM3320 control-voltage model replaces the baseline filter.

## Bounded uncertainty

The manual provides qualitative slow/faster-ramp checks but no absolute LFO
frequency endpoints. RF-5 therefore isolates a 0.08-20 Hz candidate range in
the LFO module rather than treating it as measured hardware fact. The three
Wheel Mod depth constants are similarly isolated calibration hypotheses.

The original Wheel Mod source-mix control blends LFO and noise. This block
implements the documented LFO side only. Source mix remains unavailable until
the shared analog noise generator and its spectrum are modeled; selecting a
placeholder digital-noise source would create false fidelity.

## Acceptance tests

- the frequency mapping is monotonic and reaches only its candidate endpoints;
- square-wave positive and negative intervals are equal within one sample;
- simultaneously selected waveforms sum on one shared bus;
- silence and note events do not stop or retrigger the LFO;
- CC1 changes the render when a documented destination is enabled;
- all supported sample rates remain finite and bounded.

Primary evidence: Sequential Circuits technical manual TM1000D.2, sections
2-2, 2-4 and service test 4-7. Provenance and hash are recorded in
`SOURCE_LEDGER.md`.
