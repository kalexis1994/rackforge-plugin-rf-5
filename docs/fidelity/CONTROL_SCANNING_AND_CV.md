# Control scanning, program bytes and control voltages

This document records the first source-backed hardware contract. It describes
the control plane, not the final DSP model.

## Panel scanning

The CPU scans 24 analog potentiometers through three 4051 multiplexers. Each
pot is represented by a 7-bit value, giving 128 positions. The diagnostic
material fixes the scan order now encoded by
`rf_5_contract::hardware::AnalogPot`:

1. filter ADSR;
2. amplifier ADSR;
3. filter cutoff and envelope amount;
4. oscillator B mix and pulse width;
5. oscillator A mix and pulse width;
6. noise and filter resonance;
7. glide, LFO frequency and wheel-mod source mix;
8. Poly Mod oscillator B and filter-envelope amounts;
9. oscillator A frequency, oscillator B frequency and oscillator B fine.

The comparator path includes electrical hysteresis, and the operating software
requires movement in the same direction across two steps before accepting a
pot change. This is relevant to authentic panel response, but it must not be
applied blindly to host automation until its interaction with RackForge is
tested.

## DAC and CV distribution

The physical DAC is 16-bit, with 14 writable bits. Most processed control
voltages use the seven most-significant bits. Individual oscillator pitch CVs
use all 14 writable bits so automatic tuning can correct much finer than one
semitone. Full scale is approximately 10.67 V, while software normally limits
most CVs to 10 V.

CV distribution is sequential. The DAC services 38 sample-and-hold
destinations: 23 common/patch destinations on the computer board and 15
individual oscillator/filter destinations on the voice board. A normal control
loop is approximately 6 ms and extends to roughly 11 ms when state changes.
The manual treats 0.5 mV of droop over 7 ms as the upper service expectation,
not as a nominal modulation source.

## Program memory

The documented machine holds 40 programs of 24 bytes. Each byte combines a
7-bit pot value in bits 0-6 and one switch state in bit 7. RF-5 models that raw
layout with `ProgramByte` so private research dumps can be checked without
shipping, loading or depending on them.

RF-5 factory content remains original. The raw layout is an engineering fact;
the original program data is not part of the product.

## Consequences for the engine

- Public controls can remain normalized, but their hardware quantization and
  mapping must be explicit at the circuit boundary.
- Oscillator pitch cannot share the coarse seven-bit path used by general CVs.
- Control-rate stepping and sample/hold behaviour now live in a dedicated
  scheduler, separate from audio-rate modulation. It refreshes the 24 scanned
  pots in documented order across a 6 ms unchanged cycle or an 11 ms changed
  cycle, then updates the switch latches.
- Automatic tune corrections belong to each physical oscillator, not to a
  single global detune control.
- Program decoding must remain independent from current UI parameter indices.

## Active scheduler boundary

MIDI notes, wheels and sustain remain sample accurate because they belong to
the performance path. Stored panel controls enter held state through the CPU
cycle. Master volume bypasses the scheduler because SD430 shows it as a direct
analog path to the master CA3280, and program changes preserve its value.

The physical machine refreshes 38 DAC destinations, including individual
oscillator and filter sample-and-holds. The active candidate schedules the 24
source controls and latches the resulting switch state at cycle completion.
The ten individual oscillator destinations now receive their own automatic
tune correction at fourteen-bit resolution. Exact interleaving of those writes
with the other 28 destinations and the five per-voice filter correction values
remains a lower-level timing refinement.
