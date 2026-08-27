# Original 40 program import

## Result

RF-5 carries the forty original programs as exact compact Rev 3 records. Each
program is stored as the same 24 seven-bit pot codes plus 22 switch bits used
by the recovered V8.1 pack/unpack path. Loading one therefore enters the normal
program-recall and sample/hold synchronization path; it is not a separate bank
of hand-tuned RF-5 approximations.

The local V8.1 archive is not the program source. Its three 2708 images contain
three duplicated 1 KiB halves which concatenate exactly into the 3072-byte
operating image. It contains program-management code but neither the optional
`PROG5.5` factory-program PROM nor a cassette image.

## Digital source

Sequential's official `Prophet-510-Factory-Programs-ReadMe1.02.zip` contains
200 current-instrument SysEx records. Sequential's original-patch publication
identifies Group 5, programs 511 through 558, as the original forty-program
set. The admitted source identities are:

- download ZIP SHA-256:
  `6EE129B4ED2422B7B9758ED270A6934DCBA3CFDAA18F02E9E10596139F2B3261`;
- `P5_Factory_Programs_v1.02.syx` SHA-256:
  `0050B8ED0021CB21262E96B0513068C5D36524E04DC45A1095BB73CCAF3FC8F1`;
- converted 40 x 24-byte V8.1 matrix SHA-256:
  `EB91F6EBE84C14450E7490D775A7438BC7F77D98A6C15DB85EAB31331964D089`.

The official MIDI implementation defines each 159-byte message as its six-byte
address header, 152 bytes of packed-MSB data representing 128 parameters, and
the terminating `F7`. The 128 unpacked bytes follow NRPN order.

## Rev4 to V8.1 projection

Only controls present in the vintage 24-byte record are projected. The order
below is the exact V8.1 storage order already used by `encode_program` and
`decode_program`.

| V8.1 analog pot | Rev4 NRPN |
| --- | ---: |
| Filter Attack, Decay, Sustain, Release | 43, 45, 47, 49 |
| Amplifier Attack, Decay, Sustain, Release | 44, 46, 48, 50 |
| Filter Cutoff, Envelope Amount | 17, 40 |
| Oscillator B Level, Pulse Width | 15, 9 |
| Oscillator A Level, Pulse Width | 14, 8 |
| Noise, Filter Resonance | 16, 18 |
| Glide, LFO Frequency | 13, 21 |
| Wheel Source Mix | 26 |
| Poly Mod Oscillator B, Filter Envelope | 33, 32 |
| Oscillator A Frequency, B Frequency, B Fine | 0, 1, 2 |

| V8.1 switch group | Rev4 NRPN order |
| --- | --- |
| Oscillator and Unison | 4, 3, 10, 5, 6, 7, 12, 52 |
| Poly Mod, LFO, Filter Keyboard, Release | 34, 35, 36, 23, 24, 25, 19, 51 |
| Wheel Mod and Oscillator B Low Frequency | 27, 28, 29, 30, 31, 11 |

Rev4 represents the original FILTER KEYBOARD on-state as `FULL=2`; RF-5
projects `0` to off and `2` to the original on state. All other stored switches
are already zero or one.

## Evidence that this is not a visual approximation

- Group 5 contains legacy pot codes up to 127 even where the current MIDI
  document publishes a 0-120 panel range. Scaling modern knob positions would
  not produce those preserved endpoints.
- Current-only parameters are fixed across the group: Rev selector, Vintage,
  velocity, aftertouch, voice count and unison detune do not contaminate the
  projected records.
- Original patch sheets independently agree with sampled digital routes. For
  example, Low Strings enables both pulse waves, both Wheel-Mod pulse-width
  destinations, filter keyboard and Release; Muted Clavinet disables Release
  and filter keyboard while enabling its documented wheel-filter route.
- A test decodes and re-encodes every imported record and requires all 960
  bytes, including their 22 switch bits, to remain identical.

## Reproduction

The official ZIP or its SysEx can be converted again without retaining the
third-party container in the repository:

```powershell
python tools/import-original-programs.py C:\path\to\Prophet-510-Factory-Programs-ReadMe1.02.zip
```

The importer rejects any source whose SHA-256, message count, addressing,
packing or legacy switch domains differ. RF-5 has no runtime dependency on the
SysEx, firmware or an external bank.
