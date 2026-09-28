# CEM3340 output and board-level waveform model

## Accepted electrical boundary

The CEM3340 data sheet specifies the following output geometry at its stated
electrical-characteristic conditions:

- sawtooth upper level: 9.4-10.6 V, with a lower endpoint within +/-25 mV;
- triangle upper level: 4.85-5.15 V, with a lower endpoint within +/-15 mV;
- triangle symmetry: 45-55%;
- pulse upper and lower levels depend on supply and pull-down current.

The Revision 3 voice schematic adds the board-level boundary. Saw and
triangle enter the audio and Poly Mod summing paths through 150 kohm
resistors, while pulse uses 200 kohm. Each open-emitter pulse output is pulled
to ground through 10 kohm (R4295/R4187 on SD431) before its 4016 selector, so
the low state is 0 V. The high state is not unloaded: its 10 kohm pull-down
draws about 1.30 mA, above the data sheet's 0.6 mA breakpoint. Solving the
published `Vhigh = V+ - 0.3 V - 1.3 kohm * Ipull-down` relation together with
that populated resistor gives approximately +13.009 V. The accepted
first-order board range is therefore 0 to +13.009 V, the same loaded level as
the LFO's grounded pull-down.

The technical manual also states that all CEM3340 outputs are positive-going,
but oscillator B's triangle is DC level-shifted to become symmetric about
ground. SD431's U451 (R4254/R4255 100k) produces `2 * Vtri - TRI REF`. SD430
derives TRI REF from +15 V through R4279 10k/R4290 4.99k; loaded by the five
voice cards it is the annotated 4.57 V. The circled 2.27 V on the schematic is
U451's inverting-input node, not the reference. The resulting approximately
+/-5 V wave feeds both R4285 150k into the U464 mixer and R4280 150k into the
U428 Poly Mod amount OTA. Saw and pulse retain their one-sided bias on both
paths.

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
  200 kohm and U451's level-shifted oscillator-B triangle through R4285
  150 kohm reach the negative input. Both 330 ohm shunts and both approximately 100 kohm OTA inputs are
  loaded independently. The raw waveform DC is retained because SD431 has no
  coupling capacitor before U464;
- `poly_mod_source_volts` preserves the board-level polarity entering
  oscillator-B Poly Mod. Saw and pulse arrive directly, while the triangle is
  the same U451 output that reaches the mixer, through R4280. Its
  separate conductance field describes the common U428 input where all three
  sources meet.

Both are generated from the same phase, pulse width, waveform switches and
band-limited edges. They cannot drift apart temporally. The resulting nominal
relationships are:

- saw reaches its profiled approximately 0-10 V data-sheet endpoints;
- raw triangle reaches its profiled approximately 0-5 V endpoints; after
  U451 it spans approximately -4.57 to +5.43 V, nearly the saw's excursion;
- pulse reaches approximately 0 to +9.757 equivalent source volts after its
  loaded voltage excursion and 150/200 conductance ratio are combined;
- U464 subtracts pulse and triangle input-node voltages from saw rather than
  treating selected waveforms as same-polarity host samples;
- saw and pulse preserve those same one-sided electrical levels in Poly Mod;
- triangle audio and triangle Poly Mod carry the identical U451 voltage;
  neither path sees the raw 0-5 V pin voltage or a half-depth `tri - 2.27 V`
  source.

The triangle output is the one waveform buffer that also drives the internal
oscillator comparator. The CEM3340 data sheet therefore specifies that its
finite output impedance pulls frequency downward by `Rout / Rload`; it gives a
150 ohm / 100 kohm = 0.15% worst-case example. Selecting oscillator B triangle
on SD431 connects only the 4016 and U451's non-inverting input, biased by
R4253/R4252 (1M/1M): 500 kohm. The 150 kohm mixer and Poly Mod resistors hang
on U451's output, not on the CEM3340. Each RF-5 output profile therefore incurs
its own 0.013-0.030% downward shift (approximately 0.2-0.5 cents). Saw remains
buffer-isolated from oscillator performance, and pulse remains an open-emitter
comparator output, so selecting either does not add this pull.

The public pulse-width pot is first quantized to the physical 128 codes and
held as the common DAC's 1/12 V per code. It crosses SD334's unity PW MSUM and
the voice card's U432/U433-class inverting summer (R4163 100k in, R4162 52.3k
feedback; R351/R350 for oscillator B) with no offset resistor, so CEM3340 pin
5 receives 0.523 times the held CV. The data sheet's PWM input spans 0% duty at
0 V to 100% at 5.0 V typical (4.6-5.4 V across devices), giving

`duty = 0.523 * (code / 12 V) / 5 V`.

Code 0 is therefore DC, code 57 is the square wave the owner's manual finds
"at approximately 5", and codes 115 and above are DC again, matching the
manual's description of pulses thinning out until they degenerate to DC at
both ends of the control. Each oscillator additionally carries its own PWM
full scale (`PULSE_WIDTH_FULL_SCALE_VOLTS`, 4.64-5.30 V, profile 4 at the
typical 5.0 V) inside that data-sheet range, so the same code lands at a
slightly different duty on each device. Modulation sums at the same pin. At
either limit the numerical pulse is stable DC and produces no false
hard-sync transitions; the modeled output coupling rejects that DC at the host
boundary. The pulse comparator additionally tracks the previous internal
threshold position. Its antialias width follows phase velocity minus threshold
velocity, so Poly Mod can move the edge in either direction instead of being
treated as a sequence of unrelated static pulse widths. Static endpoint edges
remain coincident and cancel exactly.

Selected waveforms still meet before both oscillator amount VCAs, but U464's
positive and negative input sums remain separate until its differential pair.
Each approximately 100 kohm input and 330 ohm shunt therefore loads only its
own selected resistors. Oscillator B's Poly Mod sum and conductance feed U428
independently of its audio mixer level, exactly as the separate board routing
requires.

## Numerical treatment

Saw retains a short internal-rate PolyBLEP reset, static pulse retains the
profile-scaled mipmapped Fourier reconstruction, and the full oscillator/mixer path remains four-times
oversampled. Triangle uses each VCO profile's 45-55% rise/fall
symmetry instead of assuming a perfect 50% shape. A periodic PolyBLAMP rounds
only its two slope transitions over one internal sample, preserves zero DC and
stays inside the profiled electrical endpoints. U464 and U428 consume the
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
  filter before the modeled final 0.60 Hz output coupling network;
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
  triangle against U451's 500 kohm input lowers pitch by 0.013-0.030%
  (profile 7: approximately 0.52 cent), while saw and pulse selection do not
  pull oscillator frequency;
- the mixer and Poly Mod triangle both equal `2 * Vtri - 4.57 V`, and U451's
  gain of two brings the triangle within two percent of the saw excursion;
- the loaded pulse high level is 13.009 V against the grounded 10 kohm
  pull-down;
- the loaded pulse and saw excursions remain within five percent after the
  board resistor ratio;
- every waveform selection reports its exact populated relative conductance;
- equal positive/negative source nodes cancel and unequal 150/200 kohm paths
  retain their independent loading;
- saw/pulse Poly Mod retain their electrical bias while triangle crosses zero;
- every waveform combination stays finite at the oversampled rate;
- the triangle correction is continuous at both asymmetric corners, changes
  only their local windows, preserves zero DC and reduces non-harmonic energy
  against the uncorrected geometry at every accepted symmetry, rate and pitch
  probe while remaining below the common -40 dB bound;
- its A4 peak displacement remains below 0.4% at every supported host rate;
- panel codes 0-114 are strictly monotonic, code 57 is within 0.5% of the
  square, codes 115-127 are DC, and modulation can reach stable 0/100% DC
  without emitting sync edges;
- all ten PWM full-scale voltages remain inside the data sheet's 4.6-5.4 V;
- a moving PWM threshold is smooth in either direction even at zero oscillator
  frequency, while a stationary threshold retains the exact comparator state;
- periodic audio-rate PWM at moderate and near-full depth improves over the
  former static-threshold correction at all supported rates and remains below
  the common -40 dB non-harmonic bound;
- complete engine renders remain finite at 44.1, 48, 96 and 192 kHz.
