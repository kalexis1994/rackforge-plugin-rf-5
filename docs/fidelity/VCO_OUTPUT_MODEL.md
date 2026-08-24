# CEM3340 output and board-level waveform model

## Accepted electrical boundary

The CEM3340 data sheet specifies the following output geometry at its stated
electrical-characteristic conditions:

- sawtooth upper level: 9.4-10.6 V, with a lower endpoint within +/-25 mV;
- triangle upper level: 4.85-5.15 V, with a lower endpoint within +/-15 mV;
- triangle symmetry: 45-55%;
- pulse upper and lower levels depend on supply and pull-down current.

The Revision 3 voice schematic adds the board-level boundary. Saw and triangle
enter the audio and Poly Mod summing paths through 150 kohm resistors, while
pulse uses 200 kohm. Pulse is pulled toward -5 V through 10 kohm, and the 4016
switch network limits the negative excursion to approximately one diode drop.
With the data-sheet high-level equation and the board's +15 V rail, this gives
an accepted first-order pulse range of approximately -0.6 to +14.1 V after
selection.

The technical manual also states that all CEM3340 outputs are positive-going,
but oscillator B's triangle is DC level-shifted to become symmetric about
ground. This is required for smooth bipolar modulation. Saw and pulse retain
their one-sided bias when used as Poly Mod sources.

## Active reconstruction

Every one of the ten audio VCOs owns an independent deterministic output
profile. Its saw upper/lower endpoints, triangle upper/lower endpoints and
triangle symmetry all remain inside the published CEM3340 ranges. These are a
validation population, not measurements from one particular instrument.

One oscillator evaluation now produces two related signals:

- `audio` is a host-safe AC representation for the oscillator mixer;
- `modulation` preserves the board-level polarity entering oscillator-B Poly
  Mod.

Both are generated from the same phase, pulse width, waveform switches and
band-limited edges. They cannot drift apart temporally. The resulting nominal
relationships are:

- saw audio excursion is the 1.0 reference;
- triangle is approximately 0.5 because its voltage excursion and input
  resistor differ from neither side of that ratio;
- pulse is approximately 1.1025 times saw after its larger voltage excursion
  and 200/150 kohm resistor ratio are combined;
- saw modulation is positive-going;
- pulse modulation spans approximately -0.09 to +2.115 normalized units;
- triangle modulation is bipolar and equals its audio-domain waveform.

Selected waveforms still add before the oscillator amount VCA. Oscillator B's
`modulation` sum feeds Poly Mod independently of its audio mixer level, exactly
as the separate board routing requires.

## Numerical treatment

Saw and pulse retain PolyBLEP edge correction, and the full signal path remains
four-times oversampled. Triangle now uses each VCO profile's 45-55% rise/fall
symmetry instead of assuming a perfect 50% shape. No state or public parameter
was added.

## Bounded uncertainty

The sources bound component outputs but do not publish the ten chips fitted to
any particular unit. The deterministic profile order is therefore a
hypothesis. The following remain open:

- exact pulse clamp voltage, rise/fall asymmetry and loading through the 4016;
- high-frequency rounding and output-buffer impedance at the populated board;
- static DC propagation through the mixer, filter and final output chain;
- correlations between amplitude, symmetry, scale error and temperature;
- waveform captures from a calibrated Revision 3 instrument.

The audio representation removes each waveform's static midpoint because the
current host boundary cannot safely carry an unknown board DC rail. The
modulation representation retains the source polarity where it changes musical
behaviour. This split is explicit and replaceable when full-path measurements
become available.

## Acceptance tests

- all ten profiles remain inside every published endpoint and symmetry limit;
- triangle is bipolar and approximately half the saw excursion;
- pulse is slightly hotter than saw after the board resistor ratio;
- saw/pulse Poly Mod remain one-sided while triangle remains bipolar;
- every waveform combination stays finite at the oversampled rate;
- complete engine renders remain finite at 44.1, 48, 96 and 192 kHz.
