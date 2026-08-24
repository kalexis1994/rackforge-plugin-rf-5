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

## Active candidate

- MIDI pitch bend consumes the complete 14-bit message, has an exact centre
  and applies the documented approximately plus or minus seven semitones to
  both VCOs.
- CC1 remains the live modulation-wheel amount and is not stored in programs.
- MIDI CC64 defers key releases until the sustain pedal rises, in both
  polyphonic and Unison allocation.
- Polyphonic assignment gives the first five distinct notes to physical voices
  1 through 5. Later notes steal the earliest-used voice, while a repeated
  pitch reuses its current physical voice and refreshes its queue age.
- Unison derives the pitch of all five physical voices from the lowest held
  key. All five gates occur together.
- The first held key triggers both envelopes; overlapping Unison notes only
  retune the voices, preserving the envelope capacitor trajectories.
- Glide is a linear control-voltage slew through the populated SD334 path. The
  0-5 V held control crosses R3124/R3125's 100 kohm/2.7 kohm divider and steers
  the matched Q309 differential pair. Its collector fraction controls the
  CA3280 current that charges the 0.1 uF C376 timing capacitor.
- All 128 panel positions therefore follow one bounded transistor law. Panel 6
  traverses five octaves in approximately 0.68 seconds; panel 10 uses the
  fastest service-compliant absolute anchor of twelve semitones per second,
  or five seconds across five octaves. Panel 0 remains a very fast analog slew
  rather than an invented digital bypass.
- Glide is exactly bypassed outside Unison.
- MIDI note-on velocity is accepted for protocol correctness but intentionally
  does not scale the sound.

## Bounded uncertainty

The feature routing, pitch-wheel span, polyphonic assignment, Unison
priority/retrigger rules, Glide topology, populated divider/timing components
and service limits are accepted. Q309 temperature and device population, the
absolute CA3280 current, capacitor tolerance and the serviced unit's time at
panel 10 remain unmeasured. The five-second boundary is deliberately isolated
as one replaceable absolute anchor; the relative panel curve is circuit-derived.

## Acceptance tests

- 14-bit pitch bend reaches exact negative, centre and positive endpoints;
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
- different nonzero MIDI velocities render identical voice samples.
- sustain holds released keys and releases them when the pedal rises.

Primary evidence: original owner's manual section 1-4; TM1000D.2 common-circuit
description, SD333/SD334 and service tests 4-4 and 4-11. Provenance is recorded
in `SOURCE_LEDGER.md`.
