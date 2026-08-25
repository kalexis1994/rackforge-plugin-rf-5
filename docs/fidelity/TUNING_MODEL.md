# VCO tuning and control-voltage model

## Accepted hardware facts

The Revision 3 technical manual establishes the following boundary:

- CEM3340 oscillators use a nominal 1 V/octave law;
- the keyboard contributes a 0-5 V component across five octaves;
- each oscillator frequency pot contributes a 0-4 V component;
- individual oscillator pitch uses the full 14 writable DAC bits;
- B LO FREQ inserts a -7.5 V offset and expands its initial-frequency control
  range to 9 V;
- B KEYBOARD controls whether the keyboard CV enters oscillator B's sum.

## Active mapping

RF-5 converts pitch CV to frequency with an exponential base-2 law. Panel
frequency and fine controls are quantized to the documented 128 pot positions
before conversion. Automation is evaluated for every rendered sample, so a
sustained note responds without being retriggered.

The V8.1 operating ROM fixes the previously inferred coarse mapping. A normal
oscillator-frequency pot is read as a 7-bit code, divided by two with integer
truncation, and capped at 48 semitones. This produces 49 reachable semitone
positions rather than a continuously centred four-octave span. Raw code 48
therefore contributes 24 semitones and is RF-5's concert/unison default.

Oscillator B LO FREQ follows the separate ROM and analog paths. Its undivided
7-bit code is capped at 108 semitones for the digital tune-DAC coordinate; the
hardware then inserts the documented -7.5 V (-90 semitone) analog offset. This
preserves the full nine-octave control range without pretending that the analog
offset belongs to the automatic-tune table. B FINE is likewise a separate
common analog sum and does not move the table coordinate. The original owner's
manual fixes its physical law: code 0 adds no detuning and the 127-step endpoint
raises oscillator B by one semitone. With B KEYBOARD
disabled, keyboard CV is omitted while the coarse, fine and tune-bias sources
remain active.

The original Scale Mode is now active as a distinct facility. Its twelve
note-class offsets reuse the documented panel knobs around centre code 64 and
are added after automatic-tune interpolation. Patch loads preserve the active
scale. Exact arithmetic, physical-pot mapping and state semantics are recorded
in [`SCALE_MODE.md`](SCALE_MODE.md).

## Explicit hypotheses

One value cannot yet be proven from the admitted documents alone:

- MIDI note 36 is used for the keyboard's zero-volt, lowest-C reference.

This is isolated in `tuning.rs` and covered by tests, so later measurements can
replace it without changing oscillator topology or host parameter IDs.

## Automatic tune

RF-5 now reconstructs the ten-channel tune multiplexer, 2.5 MHz period
measurement, fourteen-step successive approximation, 200-byte octave-bias
table and per-semitone runtime interpolation. C3-C9 are measured directly and
C0-C2 are extrapolated, as described by the technical manual.

The detailed acceptance bounds and the remaining physical-population
uncertainty are recorded in
[`AUTOTUNE_MODEL.md`](AUTOTUNE_MODEL.md). Post-tune temperature motion and the
new bounded meaning of `VintageSpread` are documented independently in
[`VCO_DRIFT_MODEL.md`](VCO_DRIFT_MODEL.md).
