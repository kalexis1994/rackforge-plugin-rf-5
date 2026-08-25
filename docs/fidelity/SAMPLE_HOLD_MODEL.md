# Sample/hold and CV distribution model

## Accepted hardware structure

Schematics SD333 and SD430 divide the DAC destinations into two boards:

- 23 common and patch sample/holds on PCB3;
- 10 individual oscillator pitch sample/holds on PCB4;
- 5 individual filter keyboard-CV sample/holds on PCB4.

The PCB3 cells hold filter and amplifier ADSR values, filter cutoff and
envelope amount, oscillator and noise mixer levels, both pulse widths,
resonance, Glide, LFO frequency, Wheel Mod source mix, both Poly Mod source
amounts, Unison CV and the sequencer output. The PCB4 cells are one A pitch,
one B pitch and one filter value for each of the five physical voices.

The technical manual states that all destinations are refreshed sequentially.
A low-leakage capacitor and BIFET follower retain each voltage while its 4051
is inhibited. Up to 0.5 mV droop over 7 ms is a service limit, not a nominal
modulation amount. The normal CPU loop is approximately 6 ms and can extend to
about 11 ms after a change.

SD332 populates 5 kohm at R354 between the LF356 DAC buffer and the CV
distribution bus. SD333 and SD430 populate 0.01 uF at every hold cell. Their
first-order source time constant is therefore 50 us before the comparatively
small 4051 on-resistance is considered. In the admitted V8.1 output loop, each
address-enable write is followed by two 23-T-state `EX (SP),IX` instructions,
a 7-T-state load and the 11-T-state inhibit write. At the documented 2.5 MHz
CPU clock this is a 64-T-state, 25.6 us acquisition window. An ideal populated
cell consequently closes

`1 - exp(-25.6 us / 50 us) = 0.400704`

of its remaining voltage error on each scheduled visit.

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
ten oscillator cells hold absolute per-voice pitch CV after automatic-tune
bias. The five filter cells hold per-voice keyboard tracking. A newly assigned
voice reacquires its A, B and filter CV before its first rendered sample;
subsequent refreshes occur through the shared sequential scheduler. Pitch and
modulation wheels remain outside these stored program CVs.

Scheduled visits now begin at the cell's present voltage, including accumulated
droop, and apply the recovered 40.0704% RC acquisition. Repeated scans therefore
converge monotonically rather than teleporting the capacitor to the new DAC
voltage. Startup, state/program synchronization and newly assigned voice pitch
remain explicit immediate boundaries. The latter represents the firmware/gate
ordering needed to avoid an unsupported audible sweep from the previous note;
it is intentionally isolated from normal panel-CV settling until gate-relative
bench measurements are available.

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

The 5 kohm resistor, 0.01 uF capacitor and 64-T-state firmware dwell are
accepted for the Rev 3 V8.1 target. Remaining uncertainty is the populated
4051 resistance, capacitor tolerance, buffer loading, per-cell leakage and
the exact gate-relative acquisition sequence used when a physical voice is
reassigned.
