# CEM3320 filter model

## Accepted hardware contract

Each Revision 3 voice contains one CEM3320 configured as four cascaded
low-pass cells. Cutoff follows the chip's exponential control law at one volt
per octave. Panel cutoff, the direct filter envelope, Wheel Mod, Poly Mod and
the keyboard switch meet at the filter control-voltage path. The service
procedure expects resonance to begin self-oscillating between panel positions
7 and 9.5.

## Active candidate

- Four topology-preserving trapezoidal-integrator one-pole cells are cascaded.
  Each numerical stage corresponds to exactly one CEM3320 variable-gain cell,
  pole capacitor and output buffer. The former extra input limiter has been
  removed: the signal now crosses four nonlinear buffers rather than five.
  The fourth-cell resonance return is closed inside the current internal
  sample with three bounded Newton steps and an analytic slope through every
  nonlinear cell. It no longer feeds back the previous digital state as though
  the analog path contained an extra sample delay.
- SD431's four 150 pF polystyrene pole capacitors are now admitted explicitly.
  Each 100 kohm feedback resistor sees the nominal 1 megohm buffer impedance
  in parallel, producing 90.909 kohm against the populated 91 kohm coupling
  resistor and therefore 0.999 nominal interstage passband gain.
- U464's oscillator-mixer output currents feed first-cell `IN A` directly.
  Common white noise arrives at the same node through each populated 100 kohm
  distribution resistor. Both paths now share the first cell's 90.909 kohm
  current-to-voltage conversion instead of independent normalized gains.
- Oscillators, audio mixer, audio-rate Poly Mod and all four filter cells run
  together at four times the host sample rate.
- Service trim 4-20 replaces the former 14 Hz intercept: 2.000 V of panel CV
  with keyboard tracking at A3/A4 must produce 440/880 Hz. Solving that anchor
  places the ten-octave panel sweep above 16.3516 Hz.
- Keyboard tracking is a physical on/off route and contributes exactly one
  octave of cutoff for every twelve semitones.
- Filter Cutoff and Filter Resonance use the documented 0-10 V common-CV
  domain instead of the former global 0-5 V approximation. R4414 converts the
  resonance span to 0-50 uA at the CEM3320 control-current input.
- Resonance now follows a strongly bending modified-linear Gm curve. A
  saturating rational law is fixed by the data sheet's typical 1 mmho at
  100 uA point and its 2.2 mmho maximum-Gm line, placing half-saturation at
  120 uA without a fitted parameter. It also follows normalized readings of
  Figure 6 at 50, 150 and 300 uA within a deliberately wider six-percent
  scan/digitization band. The former exponential and panel-8-normalized
  feedback constant have both been removed.
- The complete populated resonance return now runs inside the delay-free
  solver: OUT D crosses C4164's 2.2 uF/68 kohm high-pass, U474 applies its
  1+240k/100k gain, R4416 feeds Q IN through 51 kohm, and the pin sees both its
  published 2.7-4.5 kohm input impedance and R4415/C4145's 3 kohm/10 uF
  frequency-dependent shunt. Both capacitor memories advance even with the
  resonance control at zero. Their phase is included in the per-voice 440/880
  Hz service calibration rather than hidden in an empirical frequency offset.
  The same AC-coupled U474 output, rather than the raw OUT D voltage, now feeds
  the populated 20 kohm final-VCA input.
- U474 no longer behaves like an impossible unlimited 3.4 multiplier. Its
  effective audio load remains above 10 kohm, so five deterministic TL082
  swing limits stay between the published +/-12 V minimum and +/-13.5 V
  typical values. A 32nd-order late knee preserves the data sheet's less than
  0.02% distortion condition at 20 Vpp, while its analytic slope remains part
  of the instantaneous resonance solve.
- Each physical voice card now owns one deterministic CEM3320 profile. Its
  pole-control sensitivity remains inside 57.5-62.5 mV/decade, resonance-cell
  transconductance inside 0.8-1.2 times nominal, Q-input impedance inside
  2.7-4.5 kohm, and clipping span inside the published 10-14 Vpp range.
- Each profile's documented 57.5-62.5 mV/decade sensitivity is compensated by
  the populated per-voice scale trim. All five filters therefore meet at the
  serviced 440/880 Hz pair instead of diverging from an invented 1 kHz point.
- After calibration the five filters develop independent deterministic
  warm-up motion at a 10 Hz control rate. `VintageSpread` expands its hard
  magnitude boundary from the data-sheet typical 0.5% to the 1.5% maximum;
  it no longer applies five permanent cutoff offsets.
- Every cell output buffer uses the profile's clipping span. The bounded
  even-order term is distributed across the four cells so their complete
  passband cascade, rather than each cell independently, remains in the data
  sheet's published 0.1-0.3% predominantly second-harmonic range at the
  reference strong-signal condition.
- Invalid numerical state is rejected and reset rather than reaching the host.

## Bounded uncertainty

The topology, populated pole and resonance-return networks, exponential scale,
electrical ranges and resonance current domain, direct U464 current input and
white-noise distribution are source-backed. Filter signals are deviations around the data sheet's
nominal approximately 650 mV input summing node and 6.9 V buffer quiescent
level; source DC is retained as displacement from that serviced operating
point. The five deterministic points inside the published ranges are a
validation population, not measurements of five chips from one instrument.
The smooth Gm function is a replaceable rational reconstruction through the
published graph and typical point rather than a transistor-level model. The
five Gm/Q-input pairs are deterministic combinations inside published bounds
that also satisfy the service oscillation window; they are not measurements of
five selected chips. Filter input, cell state and output now use circuit volts
directly. U474's voltage bounds and 10 V linearity are
source-backed, while the differentiable transition between them is an explicit
late-knee hypothesis rather than a measured overload trace.
Warm-up magnitude is source-bounded, while its
210-390 second time constants, directions and correlation are explicit,
replaceable hypotheses because no admitted source publishes those trajectories.

## Acceptance tests

- the panel mapping spans exactly ten octaves;
- keyboard tracking doubles cutoff per octave and has no effect when disabled;
- a low cutoff rejects substantially more 6 kHz energy than a high cutoff;
- resonance extends an impulse tail and remains stable at supported rates;
- self-oscillation is absent below the service window and sustained inside it;
- a 1 kHz self-oscillation target remains within 0.1% from 44.1 through 192 kHz
  instead of rising from 966 Hz to 991 Hz with the former delayed return;
- self-oscillation reproduces the 440/880 Hz service-calibration pair within
  1 Hz at both 48 and 192 kHz;
- the nonlinear instantaneous-loop residual remains below 0.0004 circuit
  volts across cutoff, resonance, drive and clipping stress cases;
- full resonance CV produces 50 uA through the populated 200 kohm resistor;
- the resonance return reproduces C4164's approximately 1.064 Hz corner,
  U474's 3.4 gain, Q IN's DC/audio dividers and its approximately 2.501 Hz
  shunt transition for the typical 3.6 kohm input;
- a steady output is rejected by the AC-coupled return while audio is passed;
- the AC-coupled, 3.4-gain U474 node is shared by resonance and the final VCA;
- every U474 profile remains inside the published +/-12 to +/-13.5 V swing
  range under a reconstructed load above 10 kohm;
- U474 remains below 0.02% third harmonic at the published 10 V peak condition
  and is bounded under extreme drive;
- all five physical Gm/Q-input pairs cross four-pole loop unity between panel
  positions 6.5 and 9.5 without a normalized feedback constant;
- the Gm fit hits the published 100 uA point and has decreasing incremental
  slope;
- the no-fit-parameter rational Gm law places half-saturation at 120 uA and
  remains within six percent of three normalized Figure 6 landmarks;
- four 150 pF cells and their 100k/91k/1M network reproduce the populated
  near-unity interstage gain;
- the predicted and rendered paths contain exactly four nonlinear cells, with
  no separate fifth input limiter;
- the 100k feedback in parallel with the nominal 1M output impedance presents
  approximately 90.909 kohm to oscillator and noise injection currents;
- all five profiles remain inside every admitted data-sheet bound;
- all five cutoff curves reproduce the serviced 440/880 Hz calibration pair;
- five minutes produces distinct warm-up offsets without exceeding the
  published 0.5%/1.5% limits, independently of host sample rate;
- the complete four-cell passband retains finite profile-specific clipping
  and the published 0.1-0.3% predominantly second-harmonic asymmetry;
- Poly Mod reaches the filter at the internal audio rate.

Primary evidence: TM1000D.2 sections 2-6 and 4-10, voice schematic SD431 and
the original CEM3320 data sheet. Provenance is recorded in `SOURCE_LEDGER.md`.
