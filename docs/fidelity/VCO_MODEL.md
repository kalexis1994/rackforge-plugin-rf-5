# Dual VCO model

## Source-backed topology

TM1000D.2 section 2-4 and its CEM3340 appendix establish the Revision 3 voice
oscillator boundary:

- two CEM3340 oscillators per voice and one common CEM3340 LFO;
- a nominal 1 V/octave frequency scale;
- full 14-bit CV resolution for individual oscillator pitch;
- saw and pulse outputs for oscillator A;
- saw, triangle and pulse outputs for oscillator B;
- oscillator B can conventionally hard-synchronize oscillator A through the panel SYNC switch;
- selected waveforms are additive, not mutually exclusive.

The automatic tune system and low-frequency/keyboard modes are active parts of
this candidate.

## Active numerical model

Each physical voice owns two independent phase accumulators. They are seeded at
different phases, are not reset by note-on, and continue advancing after the
amplifier envelope becomes inactive. This preserves the free-running behaviour
of analog VCOs and avoids deterministic, phase-locked attacks.

Saw uses a five-internal-sample PolyBLEP reset in the distributed four-times
path. It is narrower than the former eight-sample candidate while retaining the
accepted alias bound. Static pulse is reconstructed from build-time band-limited saw
tables. The tables use dense harmonic counts in the exposed register, retain every partial
below the active profile's 90%-of-Nyquist boundary and crossfade the next table
only after that partial lies below true Nyquist. This avoids the former
host-rate bug that treated pulse as though every profile were four-times
oversampled and removed three quarters of its valid high-register spectrum.
For pulse this also handles 1%/99% and narrower widths without overlapping
polynomial edges. When continuous modulation moves far enough to replace the
safe harmonic basis, RF-5 crossfades the two complete periodic
reconstructions for 0.5 ms; it does not carry their first-sample difference as
a decaying DC offset. A keyboard pitch admission explicitly discards this
numerical transition state while preserving VCO phase and comparator/PWM
state, so reassigned cards cannot emit one previous-register reconstruction
transient before settling. No table is derived from firmware or audio ROM.

The distributed portable path runs the dual-VCO and mixer topology at four
times the host rate, then feeds a held/interpolated two-times nonlinear filter
and final-VCA domain. The retained complete four-times reference uses a unity-DC,
127-tap low-pass to reconstruct one host-rate sample instead of the former
four-sample box average. Triangle is generated
directly from the profiled 45-55% phase geometry, but each of its two slope
changes receives a one-internal-sample periodic PolyBLAMP correction. This
keeps the continuous CEM3340 shape and its asymmetry while preventing the
formerly sharp numerical corners from folding above the internal Nyquist
limit. The reconstruction boundary and its measured numerical response are isolated in
[`OVERSAMPLING_AND_DECIMATION.md`](OVERSAMPLING_AND_DECIMATION.md).

Each seven-bit PULSE WIDTH pot follows the populated circuit rather than a
normalized duty span. The held 1/12 V-per-code CV crosses U432 (52.3k/100k,
no offset) into CEM3340 pin 5, whose 0-5 V input spans 0-100% duty, so
`duty = 0.523 * (code / 12 V) / 5 V`. Code 0 is DC, about code 57 is the
square the owner's manual places at approximately 5, and codes 115 and above
are DC again, as the manual describes for both ends of the control. Each
oscillator keeps its own data-sheet PWM full scale inside 4.6-5.4 V. Wheel Mod
and Poly Mod are summed at the same pin, and positive Poly Mod narrows
oscillator A's pulse because the PMOD path crosses one fewer inversion than
the panel CV (see [`POLY_MOD_MODEL.md`](POLY_MOD_MODEL.md)). Either can drive
a pulse to exactly 0% or 100%, where the CEM3340 pulse output becomes steady
DC. Hard sync remains
active because its clock comes from oscillator B's saw reset, not this PWM
edge. Static pulse widths use the mipmapped reconstruction. When Wheel Mod or
audio-rate Poly Mod moves the comparator threshold, a two-host-sample
velocity-aware PolyBLEP follows that moving edge. Coincident edges cancel
exactly at static 0/100% DC.

SYNC is resolved at the same four-times internal rate but does not use the
CEM3340's bidirectional hard-sync pin 6, which SD431 leaves unconnected.
Oscillator B's saw output instead crosses U446 and the populated
C4107/R4296/R4297/Q401 version of the manufacturer's Figure 5 conventional
hard-sync circuit. RF-5 retains the saw reset's fractional position inside the internal sample,
advances A exactly to that instant, starts A at the lower endpoint of a new
cycle and then advances the remainder. The physical discontinuity is not
wrapped in a synthetic multi-sample residual; doing so audibly doubled attack
transients in high factory Sync I notes. Existing band-limited waveform
reconstruction remains active on both oscillators.
Sync remains active even when no B waveform is sent to the audio mixer.
Detailed acceptance is documented in
[`HARD_SYNC_MODEL.md`](HARD_SYNC_MODEL.md).

Waveforms sum before their oscillator level. Enabling a second waveform can
therefore raise level and drive later blocks harder; RF-5 does not normalize
the selection count. Saw, triangle and pulse now retain their data-sheet
voltage relationships and the populated board's 150/200 kohm input weighting.
Oscillator B exposes separate physical mixer and Poly Mod voltages. U451
doubles the raw triangle and subtracts the 4.57 V TRI REF, and that
approximately +/-5 V wave feeds both the audio mixer and Poly Mod; saw and
pulse retain their electrical bias in both paths.

Selecting oscillator B triangle also reproduces the CEM3340's load-dependent
frequency pull. U451's 500 kohm input bias (R4253/R4252, 1M/1M) loads the
finite 65-150 ohm triangle buffer, which also drives the internal comparator,
lowering B by approximately 0.2-0.5 cents according to its physical output
profile. Saw's
buffer isolation and pulse's open-emitter output prevent the same pitch shift
when those waveforms are selected.

## Exposed controls in this block

- oscillator A and B levels;
- oscillator B fine detune;
- oscillator A and B frequency;
- oscillator A saw/pulse and pulse width;
- oscillator B saw/triangle/pulse and pulse width;
- oscillator B low-frequency and keyboard tracking switches;
- oscillator sync.

No public parameter was added in this block. The existing normalized PULSE
WIDTH values are the stored seven-bit codes; the circuit law above, not a
duty-cycle clamp, decides their duty.

## Residual uncertainty

This candidate does not yet claim final CEM3340 waveform equivalence. Open
items are:

- waveform curvature and high-frequency rounding at the actual board nodes;
- exact populated-chip PWM threshold and control-current loading;
- analog bandwidth of hard-sync discontinuities after their now-fractional
  placement;
- final calibration of the frequency-knob and B fine-control laws;
- measured component populations, exact drift time evolution and exact
  low-octave tune extrapolation arithmetic;
- populated-unit bandwidth of sync discontinuities under extreme modulation.

The active CV-to-frequency mapping and its explicit hypotheses are maintained
separately in [`TUNING_MODEL.md`](TUNING_MODEL.md).
The ten-channel calibration pipeline is documented in
[`AUTOTUNE_MODEL.md`](AUTOTUNE_MODEL.md).
The ten independent post-tune trajectories and their data-sheet magnitude
limits are documented in [`VCO_DRIFT_MODEL.md`](VCO_DRIFT_MODEL.md).
The electrical output limits, board resistor weighting and separate audio/Poly
Mod polarities are documented in
[`VCO_OUTPUT_MODEL.md`](VCO_OUTPUT_MODEL.md).

Numerical spectral sweeps at 44.1, 48, 96 and 192 kHz verify that the corrected
triangle produces less non-harmonic energy than the uncorrected phase geometry
at every accepted symmetry, rate and pitch probe. It and the saw, square,
1%/99%, 5%/95%, original Harpsichord-width pulse and periodic hard-sync
conditions all pass the -40 dB alias threshold. Periodic audio-rate PWM at
moderate and near-full depth also remains inside that boundary. A dedicated
program-level regression requires the 1-6 Harpsichord's top-note strike to
retain at least 45% of its bottom-note RMS level in the portable profile. The
candidate still requires legally usable hardware measurements
to bound the remaining waveform curvature and analog sync-transient
hypotheses.
