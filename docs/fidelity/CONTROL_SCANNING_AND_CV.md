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

The DAC/comparator path is a window ADC with 34 mV of electrical hysteresis.
One 7-bit code across the panel's 5 V span is approximately 39.37 mV, so even
the smallest representable movement exceeds that window. V8.1 nevertheless
qualifies direction in software: a pot must compare above or below its accepted
value on two consecutive scans before the new value enters the active table.
RF-5 now reproduces that rule for panel/host control motion. A reversal restarts
the confirmation, a stable one-code movement is accepted on its second scan,
and returning to the accepted code clears the pending direction.

Program and serialized-state recalls do not simulate physical pot motion. They
synchronize the scanner and active table immediately, matching the firmware's
separate program-unpack path while avoiding a stale physical comparison after
recall. Master Volume remains outside both paths because it is an unscanned
analog control on the output board.

## DAC and CV distribution

The physical DAC is 16-bit, with 14 writable bits. Most processed control
voltages use the seven most-significant bits. Individual oscillator pitch CVs
use all 14 writable bits so automatic tuning can correct much finer than one
semitone. Full scale is approximately 10.67 V, while software normally limits
most CVs to 10 V.

The active circuit boundary no longer forces one voltage span onto every
common destination. Filter Cutoff is fixed by service trim 4-14 at 10 V for
panel maximum; Filter Resonance shares that DAC domain and reaches its
populated 200 kohm current-input resistor. Glide retains its separately
admitted 0-5 V control span. Other destinations remain isolated behind the
existing candidate mapping until an equally specific electrical anchor is
available.

CV distribution is sequential. The DAC services 38 connected sample-and-hold
destinations: 23 common/patch destinations on the computer board and 15
individual oscillator/filter destinations on the voice board. V8.1 nevertheless
walks five complete groups of eight strobe addresses. U355 X7 at the end of the
PCB3 group and U405 X7 at the end of the PCB4 group are unconnected, so the
firmware executes 40 address slots while only 38 cells acquire a voltage. A
normal control loop is approximately 6 ms and extends to roughly 11 ms when
state changes. The manual treats 0.5 mV of droop over 7 ms as the upper service
expectation, not as a nominal modulation source.

The DAC buffer's populated 5 kohm output resistor and each cell's 0.01 uF hold
capacitor establish a 50 us first-order time constant. V8.1 leaves each strobe
active for 64 CPU T-states, or 25.6 us at 2.5 MHz, so a normal visit acquires
40.0704% of the remaining difference from the capacitor's current voltage.
RF-5 applies this finite settling on scheduled CV visits and retains leakage
as the starting point of the next acquisition.

## Program memory

The documented machine holds 40 programs of 24 bytes. Each byte combines the
corresponding 7-bit pot value in bits 0-6 with one switch state in bit 7. V8.1
offsets `0x07c8-0x0813` recover the high bits into three switch-latch bytes on
load and fold them back into the same 24 pot bytes on write.

RF-5 now models the complete format rather than merely the individual byte.
The pot half follows `AnalogPot` order. The switch half follows SD333 and the
three V8.1 output bytes exactly:

1. oscillator A pulse/saw/sync, oscillator B saw/triangle/pulse, oscillator B
   keyboard and Unison;
2. Poly Mod oscillator-A frequency/pulse-width/filter, LFO saw/triangle/square,
   filter keyboard and Release;
3. Wheel Mod oscillator-A frequency, oscillator-B frequency, oscillator-A
   pulse width, oscillator-B pulse width, filter and oscillator-B low
   frequency; the last two bits are unused.

`encode_program` and `decode_program` perform this physical packing. Factory
program loads pass through that codec, so hardware pots are quantized to 128
positions and only actually stored controls replace the current patch.
Master Volume, RF-5's machine-character control and Scale Mode remain outside
the patch, as they do not occupy those bytes.

RF-5 factory content remains original. The raw layout is an engineering fact;
the original program data is not part of the product.

## Consequences for the engine

- Public controls can remain normalized, but their hardware quantization and
  mapping must be explicit at the circuit boundary.
- Per-destination S/H spans must remain explicit; a single global normalized
  voltage is not a valid hardware model.
- Oscillator pitch cannot share the coarse seven-bit path used by general CVs.
- Control-rate stepping and sample/hold behaviour now live in a dedicated
  scheduler, separate from audio-rate modulation. One 6 ms unchanged or 11 ms
  changed cycle contains the 24 documented pot reads followed by the exact 40
  V8.1 strobe-address slots; 38 slots refresh real cells and two preserve the
  timing of unconnected hardware addresses. Connected visits use the populated
  50 us RC and recovered 25.6 us strobe dwell rather than instantaneous capture.
- Panel-pot changes cross the documented 34 mV comparator window and the V8.1
  two-scan same-direction qualifier before entering held state. Program and
  state recalls synchronize held and scanner state directly.
- Automatic tune corrections belong to each physical oscillator, not to a
  single global detune control.
- Program decoding remains independent from current UI parameter indices and
  preserves the two unused high bits as non-controls.

## Active scheduler boundary

MIDI notes, wheels and sustain remain sample accurate because they belong to
the performance path. Stored panel controls enter held state through the CPU
cycle. Master volume bypasses the scheduler because SD430 shows it as a direct
analog path to the master CA3280, and program changes preserve its value.

The physical machine refreshes 38 DAC destinations, including individual
oscillator and filter sample-and-holds. The active candidate now models all 38
cells, their exact PCB3-then-PCB4 V8.1 strobe order, both unconnected address
slots, finite acquisition and bounded leakage. The ten oscillator cells receive independent
automatic-tune corrections at fourteen-bit resolution; the five filter cells
retain per-voice keyboard CV. Measured leakage populations remain open and are
isolated in [`SAMPLE_HOLD_MODEL.md`](SAMPLE_HOLD_MODEL.md).
