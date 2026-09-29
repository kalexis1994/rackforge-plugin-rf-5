# Original factory programs

## Result

RF-5 carries the Rev 3's 120 factory programs, its three 40-program files, as
exact compact Rev 3 records. Each program is stored as the same 24 seven-bit
pot codes plus 22 switch bits used by the recovered V8.1 pack/unpack path.
Loading one therefore enters the normal program-recall and sample/hold
synchronization path; it is not a separate bank of hand-tuned RF-5
approximations. File 1 is the "RF-5 Original 40" bank; Files 2 and 3 follow
as their own banks.

The local V8.1 archive is not the program source. Its three 2708 images contain
three duplicated 1 KiB halves which concatenate exactly into the 3072-byte
operating image. It contains program-management code but neither the optional
`PROG5.5` factory-program PROM nor a cassette image.

## Source: the factory tapes

Sequential shipped the Rev 3's factory programs on cassette, and the owner's
manual (CM1000D, section 8) restores "the original condition" by loading them
through the cassette interface. Recordings of the three tapes are the program
source: every byte is what a Rev 3 held in memory.

- archive `Prophet_5_Factory_Patches_Rev_3.zip` (synthchaser) SHA-256:
  `c2bc0b117c5b76db512c720a742aa36eb0795e67823cb5f4e2091a597d5e48eb`;
- `FACT1R3.WAV` `e2e838624e3f548786f375cb2da1cffa809126a71263889c9d59bb7aede04266`,
  `FACT2R3.WAV` `071e790e867c09d4233d9307916d5f9190cb33dfa5a0732d452eddd0f5833564`,
  `FACT3R3.WAV` `0ef2884297965c344e72750e6b43b8b4b925635f974216c82ef72114f6bd1229`;
- 120 x 24-byte program matrix SHA-256:
  `d5cb0bed9f6d3a6cf8394c8e66e88525057d494d3e9d56b9f1bdddb10af280b3`.

The tape format was read off these recordings. Half-cycles count in minims
of 232.4 us; a bit is sixteen minims, a one twelve one-minim half-cycles and
a four-minim one, a zero four four-minim half-cycles. A leader of ones is
followed by a zero start bit, the 960 bytes of the file most significant bit
first, and a last byte holding their sum modulo 256. All three files pass
that checksum. RF-5's CONFIG page reads and writes the same format.

The names are the manual's File 1, 2 and 3 program maps (its section 8). Files
2 and 3 were checked against their maps by unmistakable programs: Bass Guitar
Unison is the only unison program of its bank; Snare, Tom-Toms, Gunshots and
Wind are noise with zero amplifier attack; Slow Strings and Slow Brass have
the slow attacks. File 1 keeps the names RF-5 has always used. The 1982 map
lists 5-7 and 5-8 as duplicates of 1-1 and 1-6; the factory tape, like the
Rev 4 recreation, carries Hollow Sound and Cat there.

### One slot from the Rev 4 recreation

On the File 1 recording, program 1-8 is a byte-exact copy of 1-1 (Brass),
recorded over the Percussive Organ. Program 1-8 is therefore kept from
Sequential's Rev 4 recreation of the original programs (below), and
Sequential's original patch sheet confirms it: oscillator A a pulse three
octaves and a fifth up, B an octave up, zero attacks. The importer refuses
the tape if that slot ever stops being the copy.

### The tape against the Rev 4 recreation

RF-5 formerly carried the forty programs from Sequential's Rev 4 factory
SysEx (Group 5). Program for program, 89 % of the pot codes are identical to
the tape's, so both descend from the same programs, but the Rev 4 set is not a
copy of Rev 3 memory. Its differences are the ones that make a Rev 4 sound
like a Rev 3, and they are wrong for a model of the Rev 3's circuit:

- the tape never stores a pot above code 121 (120 is the ADC's fully
  clockwise reading, and 154 of its 2880 pot values sit there), while the
  Rev 4 set uses 127;
- LFO FREQUENCY is two to five codes lower in 29 programs, a different LFO
  curve compensated;
- GLIDE is zeroed in ten programs where the tape has 53-78. The Rev 3's glide
  is the common unison glide on SD334 and none of those programs uses unison,
  so it was inaudible on the Rev 3; the Rev 4 glides polyphonically;
- filter CUTOFF and ENVELOPE AMOUNT are retouched in about ten programs each.

The largest single change is 2-1, Unison Glide With Resonance, whose Cutoff
is 14 on the tape and was 31 in the Rev 4 set. The patch sheet's knob reads
about 2, between the two, so it does not decide it; the tape does.

## Rev 4 source (program 1-8)

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

- The programs are the Rev 3's own memory, read off its factory tapes and
  verified by the tapes' checksums; no value is read from a knob drawing.
- In the Rev 4 source kept for 1-8, current-only parameters are fixed across the group: Rev selector, Vintage,
  velocity, aftertouch, voice count and unison detune do not contaminate the
  projected records.
- Original patch sheets independently agree with sampled digital routes. For
  example, Low Strings enables both pulse waves, both Wheel-Mod pulse-width
  destinations, filter keyboard and Release; Muted Clavinet disables Release
  and filter keyboard while enabling its documented wheel-filter route.
- A test decodes and re-encodes every imported record and requires all 2880
  bytes, including their 22 switch bits, to remain identical.

## Program 1-4 electrical and performance cross-check

Percussive Electric Piano stores direct filter-envelope amount `32/127` (34
in the Rev 4 set this cross-check was first made with), a
zero-sustain amplifier contour and RELEASE on. Sequential's original patch
sheet explicitly describes an octave overtone at the beginning of every note
that fades with the envelope, produced by Poly Mod and oscillator sync. The
Rev 3 hardware recording places the fundamental and first-octave bands within
about 0.1 dB during the first isolated attack; the earlier RF-5 transfer left
the octave about 30 dB below the fundamental.

RF-5 keeps the exact record and corrects the shared direct-envelope U422/U433
circuit instead of adding a program-specific EQ, a fitted gain or changing the
stored amount. R452 had been read as 475 kohm rather than its compactly
printed 47.5 kohm, and the CA3280 linearization did not follow the data
sheet's Figure 3A; with both corrected, amount 32 moves cutoff by
approximately 1.4-1.5 octaves.

This program is an open discrepancy. With the ROM-confirmed filter anchor,
U451's doubled triangle and the CEM3340 pulse-width law, RF-5 places the attack
octave about 33 dB below the fundamental, and still only about 19 dB below it
with the filter fully open (amount or cutoff at maximum). The remaining gap is
therefore in the synced oscillator A, its Poly Mod drive or the mixer balance
against oscillator B's pulse and triangle, not in the filter. The earlier pass
of the spectral test relied on the former 1-99% pulse-width law (46.5% at PW A
code 59 instead of about 51%) and a half-level triangle, both now corrected
from the schematic; the test is kept but ignored until the oscillator-section
cause is identified.

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

Unison Glide With Resonance decodes to saw only on both oscillators,
oscillator A at the concert position, oscillator B one octave below it,
filter Cutoff `14/127` (31 in the Rev 4 set), Resonance `71/127`, the full
direct filter-envelope amount (code 120, the panel ceiling), keyboard
tracking, Unison, Glide and Release enabled. Sequential's
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
the exact stored values instead of shortening this one program. Its stored codes
reach the CEM3310 through the SD430 time network, so Release 89 and Decay 75
follow the same exponential code law as every other program rather than a
per-program or panel-landmark calibration.

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
Sync-I-specific preset override. The remaining sweep was still longer and
brighter than the independently recorded hardware because the former U422
transfer multiplied transconductance by the diode impedance and programmed ID
across the full 30 V. The CA3280 data sheet's Figure 3A law,
`Iout = 0.776 Is IABC / ID`, and the ID pin's 28.8 V span replace that
product and the 0.84 reference gain once applied here, so no fitted gain
remains and the official amount byte is unchanged. A regression test holds the
program's code-78 peak Poly Mod bus (86 in the Rev 4 set) at 3.9-4.5 V, well below U431's 12 V
boundary, on every deterministic voice profile.

The same audit corrected a separate topology error: U446 receives oscillator
B's saw output on SD431, just as the CEM3340 Figure 5 circuit requires. Sync is
therefore clocked by B's saw reset and is independent of B pulse width. The
stored B audio-wave switches remain off. Sequential's note that a B waveform
may be added “for a fuller effect” describes an optional performer edit, not
license to bake oscillator B into the factory program.

## Reproduction

The programs are read again from the tape archive, which stays outside the
repository:

```powershell
python tools/import-factory-tapes.py C:\path\to\Prophet_5_Factory_Patches_Rev_3.zip
```

The importer rejects any archive or recording whose SHA-256 differs, any tape
bit that does not add up as the Prophet writes it, a failed checksum, a pot
above the panel ceiling or a set unused switch bit. It writes the engine's
program data and the preset catalog, keeping any bank the catalog declares
beyond the factory files. `tools/import-original-programs.py` still converts
the Rev 4 SysEx, the source of program 1-8. RF-5 has no runtime dependency on
the tapes, the SysEx, firmware or an external bank.
