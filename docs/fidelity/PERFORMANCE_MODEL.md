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
wheel wiper and the tune path. The service procedure centres P301-7 to 0 V,
allows only +/-0.05 V while mechanically detented and then trims the residual
summer contribution back to 0.000 V. After the diodes conduct, R3177's 1 Mohm
input and the 100 kohm master-summer feedback attenuate the wheel voltage by
ten before it reaches both oscillator master sums.

## Active candidate

- MIDI pitch bend consumes the complete 14-bit message and maps it to R1's
  physical track. R3106's 4.7k positive feed and the serviced R3129 trim place
  the nominal 100k track endpoints at approximately +/-13.711 V. The
  documented approximately-seven-semitone excursion implies 26.978% track
  travel to either side of the mechanical centre rather than an invented
  electronic gain.
- R1's position-dependent Thevenin resistance, R3100's 100k wiper shunt, the
  anti-parallel D315/D316 pair and R3177's 1M master-summer input are solved as
  one nonlinear network for every MIDI pitch-bend event. The previous hard
  0.6 V threshold is gone. A bounded 25 C 1N914 curve fit gives the centre a
  smooth silicon knee while the 100k master-summer feedback converts the
  resulting branch current directly to both oscillators' pitch voltage.
- CC1 remains the live modulation-wheel amount and is not stored in programs.
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
- Glide is a linear control-voltage slew through the populated SD334 path. The
  held lowest-key CV crosses R3124/R3125's 100 kohm/2.7 kohm divider and steers
  the matched Q309 differential pair. Its collector fraction controls the
  CA3280 current that charges the 0.1 uF C376 timing capacitor.
- All 128 panel positions therefore follow one bounded transistor law. Panel 6
  traverses five octaves in approximately 0.68 seconds; panel 10 uses the
  fastest service-compliant absolute anchor of twelve semitones per second,
  or five seconds across five octaves. Panel 0 remains a very fast analog slew
  rather than an invented digital bypass.
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
mechanical wheel-to-pot travel is inferred from the owner's-manual fifth and
can be replaced by a measured endpoint without changing the circuit solver.
Q309 temperature and device population, the absolute CA3280 current,
capacitor tolerance and the serviced unit's time at panel 10 also remain
unmeasured. The five-second Glide boundary is deliberately isolated as one
replaceable absolute anchor; the relative panel curve is circuit-derived.

## Acceptance tests

- 14-bit pitch bend reaches exact negative, centre and positive endpoints;
- the serviced supply/trim network centres R1 and places its nominal track
  endpoints inside the +/-15 V rails;
- the anti-parallel pair follows the admitted microamp 1N914 curve, and the
  complete loaded transfer is smooth, symmetric, bounded and monotonic;
- the +/-0.05 V service-centre tolerance contributes less than half a cent
  before the documented residual-offset trim;
- every one of the 128 Glide positions is finite and monotonically slower;
- the populated divider produces 0.13145 V across Q309's control span and its
  matched-pair law produces the bounded full-range rate ratio;
- panel 6 and panel 10 reproduce the medium and service-limit observations;
- panel 0 remains continuous and non-instantaneous;
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
description, SD333/SD334 and service tests 4-4 and 4-11; Vishay 1N914 typical
forward-current curve. Provenance is recorded in `SOURCE_LEDGER.md`.
