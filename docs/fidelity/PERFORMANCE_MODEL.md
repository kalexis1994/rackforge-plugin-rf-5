# Performance, Unison and Glide model

## Accepted hardware contract

The original performance path provides a bipolar pitch wheel, a unipolar
modulation wheel, five-voice Unison and Glide. The service procedure requires
the pitch wheel to bend at least a fifth, all five voices to sound in Unison,
and maximum Glide to take at least five seconds across five octaves. Glide is
part of the Unison keyboard-control path and must not detune ordinary
polyphonic playing. The original keyboard has neither velocity nor aftertouch.

## Active candidate

- MIDI pitch bend consumes the complete 14-bit message, has an exact centre
  and applies a candidate range of plus or minus seven semitones to both VCOs.
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
- Glide is a linear control-voltage slew. Its maximum setting moves at twelve
  semitones per second, satisfying five octaves in five seconds, while the
  remaining panel range changes the slew rate exponentially.
- Glide is exactly bypassed outside Unison.
- MIDI note-on velocity is accepted for protocol correctness but intentionally
  does not scale the sound.

## Bounded uncertainty

The feature routing, polyphonic assignment, Unison priority/retrigger rules and
service limits are accepted. Exact pitch-wheel span, Glide potentiometer taper
and OTA current law still need instrument measurements.

## Acceptance tests

- 14-bit pitch bend reaches exact negative, centre and positive endpoints;
- maximum Glide traverses sixty semitones in five seconds;
- Glide produces no offset when Unison is disabled;
- the first five notes map to voices 1-5, the sixth steals voice 1, and a
  repeated pitch keeps its physical voice;
- Unison allocates all five voices, follows the lowest held key and preserves
  envelope state across legato pitch changes;
- different nonzero MIDI velocities render identical voice samples.
- sustain holds released keys and releases them when the pedal rises.

Primary evidence: TM1000D.2 common-circuit description and service tests 4-6
and 4-7. Provenance is recorded in `SOURCE_LEDGER.md`.
