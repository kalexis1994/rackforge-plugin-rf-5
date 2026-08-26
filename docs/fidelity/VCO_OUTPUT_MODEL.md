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
pulse uses 200 kohm. Pulse is pulled toward -5 V through 10 kohm before a
ground-referenced 4016 switch, whose input clamp limits the negative excursion
to approximately one diode drop. The high state is not unloaded: its 10 kohm
pull-down draws about 1.74 mA, above the data sheet's 0.6 mA breakpoint. Solving
the published `Vhigh = V+ - 0.3 V - 1.3 kohm * Ipull-down` relation together
with that populated resistor gives approximately +12.434 V. The accepted
first-order board range is therefore approximately -0.6 to +12.434 V.

The technical manual also states that all CEM3340 outputs are positive-going,
but oscillator B's triangle is DC level-shifted to become symmetric about
ground. This is required for smooth bipolar modulation. Saw and pulse retain
their one-sided bias when used as Poly Mod sources.

## Active reconstruction

Every one of the ten audio VCOs owns an independent deterministic output
profile. Its saw upper/lower endpoints, triangle upper/lower endpoints and
triangle symmetry and 65-150 ohm triangle-output impedance all remain inside
the published CEM3340 ranges. These are a validation population, not
measurements from one particular instrument.

One oscillator evaluation now produces two related electrical signals:

- the paired `mixer_positive_*` and `mixer_negative_*` values carry independent
  conductance-weighted source volts and conductances for U464's two inputs.
  SD431 routes saw through 150 kohm to the positive input, while pulse through
  200 kohm and oscillator-B triangle through 150 kohm reach the negative
  input. Both 330 ohm shunts and both approximately 100 kohm OTA inputs are
  loaded independently. The raw waveform DC is retained because SD431 has no
  coupling capacitor before U464;
- `poly_mod_source_volts` preserves the board-level polarity entering
  oscillator-B Poly Mod. Saw and pulse arrive directly, while U451 subtracts
  the populated 2.27 V TRI REF from the raw oscillator-B triangle. Its
  separate conductance field describes the common U428 input where all three
  sources meet.

Both are generated from the same phase, pulse width, waveform switches and
band-limited edges. They cannot drift apart temporally. The resulting nominal
relationships are:

- saw reaches its profiled approximately 0-10 V data-sheet endpoints;
- raw triangle reaches its profiled approximately 0-5 V endpoints and has
  approximately half the saw excursion;
- pulse reaches approximately -0.45 to +9.325 equivalent source volts after
  its loaded voltage excursion and 150/200 conductance ratio are combined;
- U464 subtracts pulse and triangle input-node voltages from saw rather than
  treating selected waveforms as same-polarity host samples;
- saw and pulse preserve those same one-sided electrical levels in Poly Mod;
- triangle Poly Mod spans approximately -2.27 to +2.73 V after U451, instead
  of reusing the raw positive-going audio path.

The triangle output is the one waveform buffer that also drives the internal
oscillator comparator. The CEM3340 data sheet therefore specifies that its
finite output impedance pulls frequency downward by `Rout / Rload`; it gives a
150 ohm / 100 kohm = 0.15% worst-case example. Selecting oscillator B triangle
on SD431 connects the populated 150 kohm mixer path, so each RF-5 output profile
now incurs its own 0.043-0.100% downward shift (approximately 0.75-1.73 cents).
Saw remains buffer-isolated from oscillator performance, and pulse remains an
open-emitter comparator output, so selecting either does not add this pull.
U451 buffers the separate triangle Poly Mod route and does not double the raw
triangle-output load.

The public pulse-width pot is first quantized to the physical 128 codes and
then mapped to approximately 1-99% duty cycle. Modulation is added at the
common CV node afterwards and may reach the data sheet's 0% and 100% limits.
At either limit the numerical pulse is stable DC and produces no false
hard-sync transitions; the modeled output coupling rejects that DC at the host
boundary.

Selected waveforms still meet before both oscillator amount VCAs, but U464's
positive and negative input sums remain separate until its differential pair.
Each approximately 100 kohm input and 330 ohm shunt therefore loads only its
own selected resistors. Oscillator B's Poly Mod sum and conductance feed U428
independently of its audio mixer level, exactly as the separate board routing
requires.

## Numerical treatment

Saw and pulse retain PolyBLEP edge correction, and the full signal path remains
four-times oversampled. Triangle uses each VCO profile's 45-55% rise/fall
symmetry instead of assuming a perfect 50% shape. U464 and U428 consume the
equivalent source volts directly, so there is no hidden five-volts-per-unit
conversion at either CA3280 boundary. No state or public parameter was added.

## Bounded uncertainty

The sources bound component outputs but do not publish the ten chips fitted to
any particular unit. The deterministic profile order is therefore a
hypothesis. The following remain open:

- exact pulse clamp voltage, selector on-resistance and rise/fall asymmetry;
- exact populated PWM threshold and transient behavior at modulation
  overtravel;
- high-frequency rounding and output-buffer impedance at the populated board;
- exact populated DC operating-point displacement through the mixer and
  filter before the modeled final 4.34 Hz output coupling network;
- correlations between amplitude, symmetry, scale error and temperature;
- correlation between populated triangle-output impedance and the other nine
  profile dimensions;
- waveform captures from a calibrated Revision 3 instrument.

The candidate now preserves every schematic-visible DC component through the
unlinearized mixer and nonlinear filter. That operating point can change
saturation and therefore audible harmonics even though the modeled final
coupling network rejects the remaining steady DC before the host boundary.
The exact populated displacement is still a bounded hypothesis until a full
voice-card waveform capture becomes available.

## Acceptance tests

- all ten profiles remain inside every published endpoint and symmetry limit;
- all ten triangle-output impedances remain within 65-150 ohms; selecting the
  populated 150 kohm triangle path lowers pitch by 0.043-0.100%, while saw and
  pulse selection do not pull oscillator frequency;
- raw audio triangle is positive-going and approximately half the saw
  excursion, while its U451 Poly Mod path is offset by exactly 2.27 V;
- the loaded pulse and saw excursions remain within five percent after the
  board resistor ratio;
- every waveform selection reports its exact populated relative conductance;
- equal positive/negative source nodes cancel and unequal 150/200 kohm paths
  retain their independent loading;
- saw/pulse Poly Mod retain their electrical bias while triangle crosses zero;
- every waveform combination stays finite at the oversampled rate;
- all 128 panel codes are monotonic from 1% to 99%, while modulation can reach
  stable 0/100% DC without emitting sync edges;
- complete engine renders remain finite at 44.1, 48, 96 and 192 kHz.
