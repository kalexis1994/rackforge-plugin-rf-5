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

## Program 1-4 electrical and performance cross-check

Percussive Electric Piano stores direct filter-envelope amount `34/127`, a
zero-sustain amplifier contour and RELEASE on. Sequential's original patch
sheet explicitly describes an octave overtone at the beginning of every note
that fades with the envelope, produced by Poly Mod and oscillator sync. The
Rev 3 hardware recording places the fundamental and first-octave bands within
about 0.1 dB during the first isolated attack; the earlier RF-5 transfer left
the octave about 30 dB below the fundamental.

RF-5 keeps the exact record and corrects the shared direct-envelope U422/U433
device boundary instead of adding a program-specific EQ or changing the
stored amount. The corrected factory render places the same bands within
about 1.2 dB, and a spectral regression test protects that documented attack.

The short articulation in the reference performance is a separate front-panel
choice. The patch sheet recommends switching RELEASE off and using the
footswitch to engage the programmed release as a piano sustain pedal. RF-5
therefore preserves the official RELEASE-on bit; selecting RELEASE off produces
the short demo-style tail without falsifying the factory dump. Because the
official sheet identifies that switch state and the recording contains an
isolated onset/cutoff, those observations now bound only the fast Attack and
RELEASE-off region of the global CEM3310 panel law. The programmed RELEASE-on
value and the manual/service medium and slow landmarks remain independent.

## Program 2-1 internal-rate cross-check

Unison Glide With Resonance decodes to the exact official record: saw only on
both oscillators, oscillator A at the concert position, oscillator B one octave
below it, filter Cutoff `31/127`, Resonance `71/127`, full direct filter-envelope
amount, keyboard tracking, Unison, Glide and Release enabled. Sequential's
original patch sheet independently agrees with those routes and values. RF-5
therefore does not brighten this program by changing a pot byte or adding EQ.

The Synthmania hardware performance exposed a numerical boundary instead. On
the first stable approximately-B2 region, the former host-rate filter path was
about 6.2 dB below the complete four-times oracle in the 3-8 kHz band. Keeping
the oscillators/mixer at four times and the held/interpolated filter/final VCA
at two times matches all five broad oracle bands within 0.03 dB. This path is now the
portable default and restores the upper saw/resonance content globally rather
than special-casing program 2-1.

A second Rev. 3.2 factory-program performance independently retained a finer
saw edge than the former fixed eight-internal-sample PolyBLEP candidate. Saw
now selects the same profile-scaled harmonic tables as static pulse, retaining
all partials below 90% of host Nyquist while leaving the official Cutoff,
Resonance and mixer bytes untouched. Moving PWM keeps its time-domain
PolyBLEP; this change is confined to the periodic CEM3340 saw boundary.

## Program 2-4 performance interpretation

Toy Piano stores amplifier `A/D/S/R = 7/75/0/89`: its Release control is
slower than its Decay control. Releasing a key very early therefore changes a
fast zero-sustain decay into a longer release tail. The capacitor trajectory
remains strictly descending; momentary loudness increases come from the
deliberately detuned oscillators beating through the moving resonant filter.
Sequential's patch sheet explicitly identifies that detuning as part of the
sound and recommends detached playing. The Synthmania hardware performance
lets most isolated notes decay under the held-key trajectory before key-up,
so it is not a direct measurement of the programmed Release time. RF-5 keeps
the exact stored values instead of shortening this one program. The shared
CEM3310 charge and discharge laws are calibrated globally from the documented
positions 5, 6 and 10. Interpreting the service manual's audible one-second
Release test as a discharge observation rather than an attack threshold moves
the Toy Piano -50 dB point under a 50 ms diagnostic gate from about 4.34 to
1.89 seconds without removing its detuned resonant motion.

## Program 1-7 electrical cross-check

Sync I's exact record enables oscillator-A saw, hard sync and the filter-envelope
source routed to oscillator-A frequency. Oscillator B remains the sync master
even though none of its audio waveforms is selected. Sequential's patch sheet
independently describes oscillator A one octave plus a minor third above the
keyboard pitch, oscillator B one octave above, zero direct filter-envelope
amount and the filter envelope retained as the Poly Mod source. It also states
that oscillator-A pitch changes the animation at the beginning of the sound.

The first reconstruction applied Q304's complete collector current separately
to all five U422s. That held U431 close to its compliance limit through much of
the decay and changed the documented descending sync animation into a sustained
upper-harmonic whistle. SD333 instead shows one Q304 collector bus connected to
the five parallel voice-card IABC inputs. The same fanout is present for Q301,
Q303, Q302 and Q306. Dividing each total current at that physical boundary
removes the plateau without changing the official 24-byte program or adding a
Sync-I-specific preset override. The remaining typical-equation sweep was
still longer and brighter than the independently recorded hardware. Because
the CA3280 sheet does not specify the exact populated U422 transconductance,
RF-5 applies a documented 0.84 reference gain at that physical device
boundary rather than rewriting the official amount byte. A regression test
holds the program's 86/127 amount below U431's 12 V boundary on every
deterministic voice profile.

The same audit corrected a separate topology error: U446 receives oscillator
B's saw output on SD431, just as the CEM3340 Figure 5 circuit requires. Sync is
therefore clocked by B's saw reset and is independent of B pulse width. The
stored B audio-wave switches remain off. Sequential's note that a B waveform
may be added “for a fuller effect” describes an optional performer edit, not
license to bake oscillator B into the factory program.

## Reproduction

The official ZIP or its SysEx can be converted again without retaining the
third-party container in the repository:

```powershell
python tools/import-original-programs.py C:\path\to\Prophet-510-Factory-Programs-ReadMe1.02.zip
```

The importer rejects any source whose SHA-256, message count, addressing,
packing or legacy switch domains differ. RF-5 has no runtime dependency on the
SysEx, firmware or an external bank.
