# Programmable Scale Mode

## Accepted Rev 3 V8.1 behavior

The original owner's manual and admitted operating ROM jointly establish the
following boundary:

- power-up tuning is twelve-tone equal temperament;
- Scale Mode reuses twelve physical panel pots as C through B pitch offsets;
- the physical order is LFO Frequency, oscillator B Frequency, oscillator B
  Fine, oscillator B Pulse Width, filter ADSR and amplifier ADSR;
- raw pot code 64 is the equal-tempered centre;
- each offset repeats by chromatic note class in every octave;
- the selected scale remains active globally when returning to Patch Mode;
- patch selection does not replace the active scale;
- scale programs occupied the same physical memory locations as patch programs.

The target V8.1 arithmetic doubles each raw 7-bit code and subtracts `0x80` in
an internal pitch word with 256 units per semitone. One pot step is therefore
exactly 1/128 semitone, or 0.78125 cent. The reachable interval is asymmetric:
code 0 is -50 cents, code 64 is zero and code 127 is +49.21875 cents.

The later manual addendum describes a second range reaching approximately +94
cents. The admitted V8.1 runtime has no corresponding range branch or flag, so
RF-5 deliberately targets the original approximately half-semitone range. That
later extension will not be mixed into the Rev 3.0/V8.1 candidate.

## Runtime position

The ROM applies the note-class offset after coarse pitch and automatic-tune
bias interpolation. RF-5 follows that ordering: the active scale does not move
the automatic-tune lookup coordinate, but its signed offset reaches both audio
oscillators' held CVs. The independent filter keyboard-CV path is unchanged.

Scale changes use the same contextual panel-pot scan positions and settle
through the normal 6/11 ms control cycle. The RackForge parameter vocabulary
shows the twelve note names on a dedicated Scale Mode page because a host
cannot expose two semantic identities for one physical vintage knob. This is a
presentation adaptation; the raw code, timing and audible arithmetic remain
the V8.1 behavior.

## Program and state semantics

RF-5 keeps the active twelve-note scale separate from its 47 patch parameters.
Loading any factory sound therefore preserves the current temperament, as the
hardware did after returning to Patch Mode. Host state serializes both the
patch and active scale. Version-7 patch-only state remains loadable and receives
the original equal-tempered power-up scale.

The destructive limitation in which a physical memory slot could contain
either a sound or a scale is not imposed on RackForge's preset library. It is a
storage constraint with no effect on synthesis; reproducing it would make host
state less reliable without improving audible fidelity.

The deterministic audition suite includes a C-centred just-intonation example
quantized to the exact 128 scale positions. It is owned validation content, not
an original factory scale program.
