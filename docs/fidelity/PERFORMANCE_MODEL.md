# Performance, Unison and Glide model

## Accepted hardware contract

The original performance path provides a bipolar pitch wheel, a unipolar
modulation wheel, five-voice Unison and Glide. The service procedure requires
the pitch wheel to bend at least a fifth, all five voices to sound in Unison,
and maximum Glide to take at least five seconds across five octaves. Glide is
part of the Unison keyboard-control path and must not detune ordinary
polyphonic playing. The original keyboard has neither velocity nor aftertouch.
The original owner's manual further identifies the pitch-wheel excursion as
approximately a fifth in both directions.

SD334 places anti-parallel 1N914 diodes D315/D316 between the 100 kohm R1
wheel wiper and the tune path, and labels the pair "PW DEADBAND". The service
procedure centres P301-7 to 0 V, allows only +/-0.05 V while mechanically
detented and then trims the residual summer contribution back to 0.000 V.
After the diodes conduct, R3102's 100 kohm 1% input and R378's 100 kohm
feedback pass the wheel voltage at unity gain through the MASTER TUNE/PITCH
summer U367 to both oscillator master sums. The 1 Mohm R377 belongs to
MASTER TUNE, not to the wheel.

## Active candidate

- MIDI pitch bend consumes the complete 14-bit message and maps it to R1's
  physical track. R3106's 4.7k positive feed and the serviced R3129 trim place
  the nominal 100k track endpoints at approximately +/-13.711 V. The
  documented approximately-seven-semitone excursion implies 4.834% track
  travel to either side of the mechanical centre rather than an invented
  electronic gain.
- R1's position-dependent Thevenin resistance, R3100's 100k wiper shunt, the
  anti-parallel D315/D316 pair and R3102's 100k summer input are solved as one
  nonlinear network for every MIDI pitch-bend event. The previous hard 0.6 V
  threshold is gone. A bounded 25 C 1N914 curve fit gives the centre a smooth
  silicon knee while R378's 100k feedback converts the resulting branch
  current directly to both oscillators' pitch voltage. Through the unity
  summer the diode drop is a large part of the small wiper voltage, which is
  the designed deadband: a fifth of the MIDI travel moves pitch by only about
  0.16 semitone, and half of it by about 2.2 semitones.
- The summed tune voltage crosses R378/C368 (100 kohm/0.1 uF), a 10 ms lag,
  and then the 1 ms A SUM/B SUM output stages described in
  [`LFO_AND_WHEEL_MOD_MODEL.md`](LFO_AND_WHEEL_MOD_MODEL.md).
- CC1 remains the live modulation-wheel amount and is not stored in programs.
  It moves R2's wiper across the same wheel-assembly travel as the PITCH
  wheel: 9.67% of the track from the grounded stop at full CC1 (see
  [`LFO_AND_WHEEL_MOD_MODEL.md`](LFO_AND_WHEEL_MOD_MODEL.md)).
- MIDI CC64 defers key releases until the sustain pedal rises, in both
  polyphonic and Unison allocation.
- Polyphonic assignment gives the first five distinct notes to physical voices
  1 through 5. Later notes steal the earliest-used voice, while a repeated
  pitch reuses its current physical voice and refreshes its queue age.
- Unison derives the pitch of all five physical voices from the lowest held
  key. All five gates occur together. The lowest-key voltage occupies common
  sample/hold destination 21; it is not the digital Unison switch state.
- The first held key triggers both envelopes; overlapping Unison notes only
  retune the voices, preserving the envelope capacitor trajectories.
- Glide is a linear control-voltage slew through the populated SD334 path,
  with every constant a populated part and no fitted panel-time anchor. The
  held lowest-key CV reaches U381, an unlinearized CA3280 whose ID terminal is
  tied to -15 V, and its output current charges C376 (0.1 uF). D318/D319 bound
  the differential input, so any audible interval slews at U381's peak output
  current, 0.82 IABC, and only the final few tens of millivolts follow the
  OTA's `tanh(dV / 2 V_T)` approach without overshoot.
- Q309, an AD820 matched PNP pair, sets IABC. R3126 (30 kohm from +15 V) feeds
  the common emitters; the right base and collector are grounded; the left
  base receives GLIDE CV, which Fig. 2-4 labels 0-10 V, through the R3124
  100 kohm/R3125 2.7 kohm divider; and the left collector reaches U381's IABC
  through R3123 100 kohm. Q309 saturating against R3123 caps IABC at about
  0.14 mA. The slew rate is `0.82 IABC / C376`.
- Five octaves therefore take about 4 ms at code 0, about 0.6 s at code 72
  (panel 6) and about 30 s at the code-120 panel ceiling, comfortably inside
  the service test's at-least-five-seconds limit. Panel 0 remains a very fast
  analog slew rather than an invented digital bypass. The C376 voltage is kept
  in double precision so a slow glide can close its last cents, and the Glide
  output passes through the same 1 ms A SUM/B SUM lags as the other pitch
  sums.
- Glide is exactly bypassed outside Unison.
- In Unison, V8.1 removes keyboard pitch from the ten individual oscillator
  S/H cells and zeros the five individual filter-keyboard cells. The common
  lowest-key S/H and Glide path restore keyboard pitch to both oscillators.
- MIDI note-on velocity is accepted for protocol correctness but intentionally
  does not scale the sound.

## Bounded uncertainty

The feature routing, pitch-wheel topology/span, polyphonic assignment, Unison
priority/retrigger rules, Glide topology, populated divider/timing components
and service limits are accepted. The D315/D316 fit follows a modern 1N914
typical curve and therefore bounds, but does not identify, the historical
diodes. Their temperature, matching and leakage remain unmeasured. The exact
mechanical wheel-to-pot travel is inferred from the owner's-manual fifth. The
MOD wheel shares it, because R1/R2 and their wheel and bracket are the same
parts. A measured endpoint can replace it for both wheels without changing
the circuit solver.
Q309 temperature and device population, U381's populated peak-current
ratio, the exact Q309 saturation margin, capacitor tolerance and the serviced
unit's time at panel 10 also remain unmeasured. No absolute Glide time is
fitted: both the relative panel curve and the absolute rate follow the
populated circuit, and the service test's five-second minimum is a check that
the approximately 30 s panel-10 result passes.

## Acceptance tests

- 14-bit pitch bend reaches exact negative, centre and positive endpoints;
- the serviced supply/trim network centres R1 and places its nominal track
  endpoints inside the +/-15 V rails;
- the anti-parallel pair follows the admitted microamp 1N914 curve, and the
  complete loaded transfer is smooth, symmetric, bounded and monotonic;
- the solved 4.834% travel reaches exactly seven semitones through the unity
  R3102/R378 summer, a fifth of the MIDI travel stays below 0.2 semitone and
  half of it lands at approximately 1.9-2.5 semitones;
- the +/-0.05 V service-centre tolerance contributes less than half a cent
  before the documented residual-offset trim;
- every one of the 128 Glide positions is finite and monotonically slower;
- the populated divider places 0.2629 V on Q309's left base at the code-120
  panel ceiling;
- Q309 saturation holds the fastest IABC at 135-150 uA, and code 0 crosses
  five octaves in under 10 ms;
- code 72 (panel 6) crosses five octaves in 0.3-0.9 s and code 120 in
  20-45 s, longer than the service test's five seconds;
- the final tanh approach reaches the target without overshoot;
- Glide produces no offset when Unison is disabled;
- the first five notes map to voices 1-5, the sixth steals voice 1, and a
  repeated pitch keeps its physical voice;
- Unison allocates all five voices, follows the lowest held key and preserves
  envelope state across legato pitch changes;
- Unison's common cell holds the lowest-key voltage while all ten oscillator
  cells omit that same keyboard component;
- different nonzero MIDI velocities render identical voice samples.
- sustain holds released keys and releases them when the pedal rises.

Primary evidence: original owner's manual section 1-4; TM1000D.2 common-circuit
description, Fig. 2-4, SD333/SD334 and service tests 4-4 and 4-11; Vishay
1N914 typical forward-current curve; the Intersil CA3280 data sheet's peak
output current. Provenance is recorded in `SOURCE_LEDGER.md`.
