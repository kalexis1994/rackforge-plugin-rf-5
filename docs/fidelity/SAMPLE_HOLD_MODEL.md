# Sample/hold and CV distribution model

## Accepted hardware structure

Schematics SD333 and SD430 divide the DAC destinations into two boards:

- 23 common and patch sample/holds on PCB3;
- 10 individual oscillator pitch sample/holds on PCB4;
- 5 individual filter keyboard-CV sample/holds on PCB4.

The PCB3 cells hold filter and amplifier ADSR values, filter cutoff and
envelope amount, oscillator and noise mixer levels, both pulse widths,
resonance, Glide, LFO frequency, Wheel Mod source mix, both Poly Mod source
amounts, Unison keyboard CV and the sequencer output. The PCB4 cells are one A pitch,
one B pitch and one filter value for each of the five physical voices.

The technical manual states that all destinations are refreshed sequentially.
A low-leakage capacitor and BIFET follower retain each voltage while its 4051
is inhibited. Up to 0.5 mV droop over 7 ms is a service limit, not a nominal
modulation amount. The normal CPU loop is approximately 6 ms and can extend to
about 11 ms after a change.

SD332 shows the LF356 DAC buffer driving `Vdac` directly. R354's 5 kohm branch
instead feeds the separate ADC-gain stage and is not in series with the CV
distribution bus. SD333 and SD430 populate 0.01 uF at every hold cell, so the
acquisition resistance is principally the selected 4051 channel. A conservative
modern 4051-class bound is 175 ohm at a 15 V span; the original circuit's larger
bipolar supply can only reduce that resistance. The resulting upper-bound time
constant is 1.75 us. In the admitted V8.1 output loop, each
address-enable write is followed by two 23-T-state `EX (SP),IX` instructions,
a 7-T-state load and the 11-T-state inhibit write. At the documented 2.5 MHz
CPU clock this is a 64-T-state, 25.6 us acquisition window. An ideal populated
cell therefore closes at least

`1 - exp(-25.6 us / 1.75 us) = 0.99999956`

of its remaining voltage error on each scheduled visit. Even a full 10 V jump
retains less than 5 uV of acquisition error under this conservative bound.

## Active reconstruction

RF-5 now represents all 38 cells independently. One CPU control cycle contains
64 logical service positions: the documented 24 panel-pot reads followed by
the V8.1 routine's five complete banks of eight DAC strobe addresses. SD333
shows U355 X7 unconnected after the 23 common cells; SD430 shows U405 X7
unconnected after the 15 individual cells. The two empty slots acquire no
voltage, but retaining them keeps every real destination at its recovered
firmware phase. The complete sequence fits inside one 6 or 11 ms cycle, so
panel scan and CV distribution do not incorrectly add two full control-loop
delays.

The 23 common cells feed the held panel values used by the audio engine. The
ten oscillator cells hold per-voice pitch CV after automatic-tune bias. The
five filter cells hold per-voice keyboard tracking. Pitch and modulation wheels
remain outside these stored program CVs.

Scheduled visits now begin at the cell's present voltage, including accumulated
droop, and apply the conservative greater-than-99.9999% acquisition bound. This
preserves the physical capacitor boundary without creating a false multi-scan
glissando. Startup and state/program synchronization remain explicit immediate
boundaries.

V8.1 fixes the gate-relative ordering that was previously open. The main loop
outputs the existing CV table at `0x0268`, writes the gate latch at
`0x026C-0x026F`, and only then scans the keyboard at `0x029E`. A new-key path at
`0x0657-0x0673` asserts its gate and immediately strobes address `0x1B`, which
SD333 identifies as the external sequencer-output cell. It does not strobe the
new voice's A, B or filter cells. Those three destinations receive the new note
from the next complete `0x0583-0x05C4` CV pass. RF-5 therefore starts the
envelopes at the performance-event boundary while retaining the previous held
pitch/filter values until the next CPU sweep, in physical destination order.

The common cell at RAM/output index `0x15` is also not a Boolean Unison switch.
V8.1 `0x0336-0x0358` writes the lowest active key there. In Unison,
`0x04D1-0x04F6` removes keyboard pitch from the ten individual oscillator cells
and `0x0503-0x051F` suppresses individual filter keyboard CV. The held common
keyboard voltage instead feeds the Q309/CA3280/C376 Glide path. RF-5 now keeps
the digital Unison latch separate from that analog cell and follows the same
routing.

The sequencer-output cell exists in the topology but currently has no public
source because RF-5 does not yet implement the external analog sequencer
interface.

## Droop hypothesis and bound

Each virtual cell has a deterministic signed buffer-bias current corresponding
to 0.003-0.0126 V/s. This produces 0.021-0.0882 mV movement over 7 ms, safely
below the documented 0.5 mV service ceiling. Accumulation uses double
precision so sub-float increments are not silently lost at high held voltages.

These rates and polarities are a conservative component population for
testing, not measurements from an original unit. Common normalized controls
provisionally use a 5 V span. `ControlVoltageDestination` remains the stable
38-cell vocabulary, while `CONTROL_VOLTAGE_STROBE_ORDER` separately records the
five physical eight-address groups recovered from V8.1. This separation keeps
the two unconnected addresses out of the audio state without erasing their
timing.

The direct LF356-to-`Vdac` path, 0.01 uF capacitor, 64-T-state firmware dwell,
gate/CV ordering and Unison keyboard-cell semantics are accepted for the Rev 3
V8.1 target. The 175 ohm switch value is deliberately a conservative modern
upper bound, not a populated-unit measurement. Remaining uncertainty is the
historical 4051 population, capacitor tolerance, buffer loading and per-cell
leakage.
