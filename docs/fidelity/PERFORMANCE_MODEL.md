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
- Unison retriggers all five physical voices on the newest key.
- A fixed-capacity last-note stack returns to the previous held key without
  retriggering the envelopes when the newest key is released.
- Glide is a linear control-voltage slew. Its maximum setting moves at twelve
  semitones per second, satisfying five octaves in five seconds, while the
  remaining panel range changes the slew rate exponentially.
- Glide is exactly bypassed outside Unison.
- MIDI note-on velocity is accepted for protocol correctness but intentionally
  does not scale the sound.

## Bounded uncertainty

The feature routing and service limits are accepted. Exact pitch-wheel span,
Glide potentiometer taper, OTA current law, keyboard priority and envelope
retrigger behaviour need instrument measurements. The current candidate uses
last-note priority because it is deterministic and playable, not because the
manual fully specifies every overlapping-key sequence.

## Acceptance tests

- 14-bit pitch bend reaches exact negative, centre and positive endpoints;
- maximum Glide traverses sixty semitones in five seconds;
- Glide produces no offset when Unison is disabled;
- Unison allocates all five voices and falls back to the previous held note;
- different nonzero MIDI velocities render identical voice samples.
- sustain holds released keys and releases them when the pedal rises.

Primary evidence: TM1000D.2 common-circuit description and service tests 4-6
and 4-7. Provenance is recorded in `SOURCE_LEDGER.md`.
