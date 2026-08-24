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

The default frequency-pot code is treated as the initial-frequency trim's
nominal reference. Normal mode spans four octaves around that calibrated
reference. LO FREQ spans nine octaves around the reference and then applies the
documented -7.5-octave offset. With B KEYBOARD disabled, the pitch remains tied
to the zero-volt end of the five-octave keyboard range instead of following the
played note.

## Explicit hypotheses

Three values cannot yet be proven from the admitted documents alone:

- MIDI note 36 is used for the keyboard's zero-volt, lowest-C reference;
- the initial-frequency trim places the default 7-bit pot code at musical
  unison;
- oscillator B fine currently spans approximately +/-50 cents.

These are isolated in `tuning.rs` and covered by tests, so later measurements
can replace them without changing oscillator topology or host parameter IDs.

## Automatic tune still open

The original tune routine measures all ten VCOs at octave points, stores a
200-byte bias table and interpolates a 14-bit correction while playing. RF-5
does not yet reproduce that routine. The current deterministic voice spread is
therefore a temporary residual-error model, not a reconstruction of the tune
table.
