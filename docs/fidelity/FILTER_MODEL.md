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
- SD431's four 150 pF polystyrene pole capacitors are now admitted explicitly.
  Each 100 kohm feedback resistor sees the nominal 1 megohm buffer impedance
  in parallel, producing 90.909 kohm against the populated 91 kohm coupling
  resistor and therefore 0.999 nominal interstage passband gain.
- Oscillators, audio mixer, audio-rate Poly Mod and all four filter cells run
  together at four times the host sample rate.
- The panel sweep covers ten octaves above a 14 Hz candidate lower bound.
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
- The five profiles meet at a 1 kHz calibration reference and diverge smoothly
  above and below it according to their control sensitivity. This replaces the
  former assumption that all five ICs share an exact exponential scale.
- Input and cell output limiting now use the profile's clipping span and a
  bounded even-order term. The latter preserves the data sheet's statement
  that passband distortion is predominantly second harmonic and remains in
  its published 0.1-0.3% range at the reference strong-signal condition.
- Invalid numerical state is rejected and reset rather than reaching the host.

## Bounded uncertainty

The topology, populated pole network, exponential scale, electrical ranges and
resonance current domain are source-backed. The five deterministic points
inside those ranges are a validation population, not measurements of five
chips from one instrument. The smooth Gm function is a replaceable fit through
published graph/typical points rather than a transistor-level model. The 14 Hz
intercept, circuit-volts-to-normalized-audio conversion, exact populated Gm
curve and dynamic warm-up remain calibration hypotheses.

## Acceptance tests

- the panel mapping spans exactly ten octaves;
- keyboard tracking doubles cutoff per octave and has no effect when disabled;
- a low cutoff rejects substantially more 6 kHz energy than a high cutoff;
- resonance extends an impulse tail and remains stable at supported rates;
- self-oscillation is absent below the service window and sustained inside it;
- full resonance CV produces 50 uA through the populated 200 kohm resistor;
- the Gm fit hits the published 100 uA point and has decreasing incremental
  slope;
- four 150 pF cells and their 100k/91k/1M network reproduce the populated
  near-unity interstage gain;
- all five profiles remain inside every admitted data-sheet bound;
- profile cutoff curves intersect at the explicit calibration reference;
- strong signals retain finite profile-specific clipping and even-order
  asymmetry;
- Poly Mod reaches the filter at the internal audio rate.

Primary evidence: TM1000D.2 sections 2-6 and 4-10, voice schematic SD431 and
the original CEM3320 data sheet. Provenance is recorded in `SOURCE_LEDGER.md`.
