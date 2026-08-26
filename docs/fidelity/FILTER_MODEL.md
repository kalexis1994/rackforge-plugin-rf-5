# CEM3320 filter model

## Accepted hardware contract

Each Revision 3 voice contains one CEM3320 configured as four cascaded
low-pass cells. Cutoff follows the chip's exponential control law at one volt
per octave. Panel cutoff, the direct filter envelope, Wheel Mod, Poly Mod and
the keyboard switch meet at the filter control-voltage path. The service
procedure expects resonance to begin self-oscillating between panel positions
7 and 9.5.

## Active candidate

- Four topology-preserving trapezoidal-integrator one-pole cells are cascaded.
  The fourth-cell resonance return is closed inside the current internal
  sample with two bounded Newton steps and an analytic slope through every
  nonlinear cell. It no longer feeds back the previous digital state as though
  the analog path contained an extra sample delay.
- SD431's four 150 pF polystyrene pole capacitors are now admitted explicitly.
  Each 100 kohm feedback resistor sees the nominal 1 megohm buffer impedance
  in parallel, producing 90.909 kohm against the populated 91 kohm coupling
  resistor and therefore 0.999 nominal interstage passband gain.
- U464's oscillator-mixer output currents feed first-cell `IN A` directly.
  Common white noise arrives at the same node through each populated 100 kohm
  distribution resistor. Both paths now share the first cell's 90.909 kohm
  current-to-voltage conversion instead of independent normalized gains.
- Oscillators, audio mixer, audio-rate Poly Mod and all four filter cells run
  together at four times the host sample rate.
- Service trim 4-20 replaces the former 14 Hz intercept: 2.000 V of panel CV
  with keyboard tracking at A3/A4 must produce 440/880 Hz. Solving that anchor
  places the ten-octave panel sweep above 16.3516 Hz.
- Keyboard tracking is a physical on/off route and contributes exactly one
  octave of cutoff for every twelve semitones.
- Filter Cutoff and Filter Resonance use the documented 0-10 V common-CV
  domain instead of the former global 0-5 V approximation. R4144 converts the
  resonance span to 0-50 uA at the CEM3320 control-current input.
- Resonance now follows a strongly bending modified-linear Gm curve. A
  saturating fit is fixed by the data sheet's typical 1 mmho at 100 uA point
  and its 2.2 mmho maximum-Gm line; the four-pole loop crosses unity at the
  nominal panel-8 service anchor and can sustain oscillation from its
  deterministic internal noise floor.
- Each physical voice card now owns one deterministic CEM3320 profile. Its
  pole-control sensitivity remains inside 57.5-62.5 mV/decade, resonance-cell
  transconductance inside 0.8-1.2 times nominal, and clipping span inside the
  published 10-14 Vpp range.
- Each profile's documented 57.5-62.5 mV/decade sensitivity is compensated by
  the populated per-voice scale trim. All five filters therefore meet at the
  serviced 440/880 Hz pair instead of diverging from an invented 1 kHz point.
- After calibration the five filters develop independent deterministic
  warm-up motion at a 10 Hz control rate. `VintageSpread` expands its hard
  magnitude boundary from the data-sheet typical 0.5% to the 1.5% maximum;
  it no longer applies five permanent cutoff offsets.
- Input and cell output limiting now use the profile's clipping span and a
  bounded even-order term. The latter preserves the data sheet's statement
  that passband distortion is predominantly second harmonic and remains in
  its published 0.1-0.3% range at the reference strong-signal condition.
- Invalid numerical state is rejected and reset rather than reaching the host.

## Bounded uncertainty

The topology, populated pole network, exponential scale, electrical ranges and
resonance current domain, direct U464 current input and white-noise distribution
are source-backed. The five deterministic points inside those ranges are a
validation population, not measurements of five chips from one instrument.
The smooth Gm function is a replaceable fit through published graph/typical
points rather than a transistor-level model. Filter input, cell state and
output now use circuit volts directly; the exact populated Gm curve remains a
calibration hypothesis.
Warm-up magnitude is source-bounded, while its
210-390 second time constants, directions and correlation are explicit,
replaceable hypotheses because no admitted source publishes those trajectories.

## Acceptance tests

- the panel mapping spans exactly ten octaves;
- keyboard tracking doubles cutoff per octave and has no effect when disabled;
- a low cutoff rejects substantially more 6 kHz energy than a high cutoff;
- resonance extends an impulse tail and remains stable at supported rates;
- self-oscillation is absent below the service window and sustained inside it;
- a 1 kHz self-oscillation target remains within 0.1% from 44.1 through 192 kHz
  instead of rising from 966 Hz to 991 Hz with the former delayed return;
- self-oscillation reproduces the 440/880 Hz service-calibration pair within
  1 Hz at both 48 and 192 kHz;
- the nonlinear instantaneous-loop residual remains below 0.0002 internal
  units across cutoff, resonance, drive and clipping stress cases;
- full resonance CV produces 50 uA through the populated 200 kohm resistor;
- the Gm fit hits the published 100 uA point and has decreasing incremental
  slope;
- four 150 pF cells and their 100k/91k/1M network reproduce the populated
  near-unity interstage gain;
- the 100k feedback in parallel with the nominal 1M output impedance presents
  approximately 90.909 kohm to oscillator and noise injection currents;
- all five profiles remain inside every admitted data-sheet bound;
- all five cutoff curves reproduce the serviced 440/880 Hz calibration pair;
- five minutes produces distinct warm-up offsets without exceeding the
  published 0.5%/1.5% limits, independently of host sample rate;
- strong signals retain finite profile-specific clipping and even-order
  asymmetry;
- Poly Mod reaches the filter at the internal audio rate.

Primary evidence: TM1000D.2 sections 2-6 and 4-10, voice schematic SD431 and
the original CEM3320 data sheet. Provenance is recorded in `SOURCE_LEDGER.md`.
