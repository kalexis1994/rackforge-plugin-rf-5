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
- Oscillators, audio mixer, audio-rate Poly Mod and all four filter cells run
  together at four times the host sample rate.
- The panel sweep covers ten octaves above a 14 Hz candidate lower bound.
- Keyboard tracking is a physical on/off route and contributes exactly one
  octave of cutoff for every twelve semitones.
- Resonance uses a gently compressed modified-linear feedback curve and can
  sustain oscillation from its deterministic internal noise floor.
- Small smooth nonlinearities are present at the input and each cell; invalid
  numerical state is rejected and reset rather than reaching the host.

## Bounded uncertainty

The topology, exponential scale and qualitative resonance behaviour are
source-backed. The 14 Hz intercept, panel span, exact resonance VCA curve,
cell overload, second-harmonic content and voice-to-voice component spread are
calibration hypotheses. They are isolated constants and are not presented as
measurements of a particular instrument.

## Acceptance tests

- the panel mapping spans exactly ten octaves;
- keyboard tracking doubles cutoff per octave and has no effect when disabled;
- a low cutoff rejects substantially more 6 kHz energy than a high cutoff;
- resonance extends an impulse tail and remains stable at supported rates;
- self-oscillation is absent below the service window and sustained inside it;
- Poly Mod reaches the filter at the internal audio rate.

Primary evidence: TM1000D.2 sections 2-6 and 4-10, voice schematic SD431 and
the original CEM3320 data sheet. Provenance is recorded in `SOURCE_LEDGER.md`.
