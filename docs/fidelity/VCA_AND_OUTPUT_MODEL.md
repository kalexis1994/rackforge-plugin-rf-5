# CA3280 mixer, VCA and output model

## Accepted hardware contract

The Revision 3 signal path uses several RCA CA3280 operational
transconductance amplifiers rather than one generic gain stage. Each voice
routes the selected oscillator-A and oscillator-B waveforms through separate
CA3280 mixer VCAs with the linearizing-diode terminal cut off. CEM3320 OUT D is
AC-coupled by C4164, loaded by R4460 and amplified 3.4 times by U474 before the
filtered signal enters a CA3280 final VCA whose linearizing terminal is active
and whose bias current is controlled by the amplifier envelope through Q410.
Q410 is a grounded-base Fairchild 2N4250 PNP (TM1000D.2 Fig. 2-4 Detail B):
only its 3.3 kohm emitter resistor R4496 sets the current, while R4533 in the
collector merely isolates the IABC pin from a current source. Its emitter
junction therefore converts CEM3310 voltage to CA3280 IABC current rather
than passing a normalized envelope value directly.

Each final VCA drives its output current into R4529, the 25 kohm VOL
rheostat that service adjustment 4-22 leaves at or near maximum, so every card
reaches the common node as a Thevenin source behind 25 kohm plus its 39 kohm
R4565-R4569 summing resistor. The five cards and the A-440 network meet at the
high-impedance U480 follower; idle cards still load the node. A second
linearized CA3280 applies the physical volume control, followed by an NE5534
buffer and the back-panel output network. Balance trimmers remove OTA DC
offset and the service procedure separately calibrates final-VCA balance and
per-voice volume.

The same OTA family also controls modulation. U378 crossfades common LFO and
noise in opposite directions, while each voice has a linearized dual-envelope
amount device and an unlinearized oscillator-B Poly Mod amount device.

## Active candidate

- Each voice has one deterministic dual-OTA mixer profile. Its oscillator A
  and B halves retain close but non-identical transconductance and overload
  knees inside the CA3280 data-sheet output-current bounds.
- Each oscillator mix reconstructs the manual's approximately 100 kohm
  unlinearized input impedance against the selected 150 kohm saw/triangle and
  200 kohm pulse paths plus the populated 330 ohm input shunt. Parallel
  waveform selections produce the source-backed passive loading before the
  nonlinear current transfer. U464's output currents then enter the first
  CEM3320 cell directly and develop voltage through its 100k feedback in
  parallel with the nominal 1M output impedance.
- One separate unlinearized CA3280 sets common noise level before the result is
  developed across R4129's 10k, buffered by U474 and distributed through five
  100k paths to the CEM3320 inputs; noise does not pass through a fictitious
  third mixer OTA on every voice card.
- The oscillator A/B level cells (1/12 V per stored code, 10.000 V at the
  panel's code-120 ceiling) reach the paired voice-mixer VCAs
  through SD333 Q306/Q302 and 33k emitter resistors. The common noise cell
  reaches its OTA through Q305 and 75k. Each Q306/Q302 collector current is
  divided across the five parallel voice-card IABC inputs; Q305 instead drives
  the single common noise OTA. Their physical IABC currents now drive
  CA3280 current limits and the populated filter-input transimpedances; the
  intermediate settings follow the common 2N4250 junction equation instead of
  linear host multipliers.
- The five final VCAs use substantially wider diode-linearized transfers and
  are evaluated inside the same two-times-oversampled loop as the filters in
  the distributed profile (four times in the offline oracle).
  Their small-signal gain is equal after the documented per-voice service
  adjustment, while their strong-signal knees remain distinct.
- The final-VCA audio input is the same stateful C4164/U474 node that drives
  the CEM3320 resonance return. Its approximately 1.064 Hz DC-blocking corner
  and 3.4 non-inverting gain are applied once inside the filter model. U474's
  reconstructed load exceeds 10 kohm, and its profiled late knee is bounded by
  the TL082's published +/-12 V minimum and +/-13.5 V typical swing while
  retaining the published 20 Vpp linearity. Its profile-specific 8.4-12.9 V/us
  slew state is shared by resonance and final-VCA feed. No separate normalized
  pre-VCA gain or duplicate high-pass is added here.
- The final-VCA signal transfer is separate from the generic linearized
  modulation OTA but obeys the same Figure 3A law. Intersil's functional
  diagram shows the ID terminal feeding an internal current mirror referred to
  the V- pin, so every ID programming resistor from +15 V sees approximately
  28.8 V (`LINEARIZED_ID_PROGRAMMING_VOLTS`, two junctions above -15 V).
  U477's R4546 68 kohm therefore sets approximately 424 uA of diode current.
  Figure 3A (IABC 650 uA, ID 200 uA, 10 kohm + 10 kohm inputs) is linear until
  the input current reaches +/-ID, +/-4 V across its 20 kohm loop, so the
  linear range scales with ID times the populated input loop: SD431's
  R4548/R4547 20 kohm/20 kohm inputs give approximately +/-17 V. The filter and
  VCA exchange circuit volts directly; a sixth-order smooth norm keeps the
  graph's long linear centre and rounded knee, so the CEM3320 population's
  10-14 Vpp output passes essentially linearly and only far larger drive
  rounds.
- Toward the summer each card is a Thevenin source of `Iout * 25 kohm` behind
  64 kohm (`VOICE_SUMMER_SOURCE_OHMS`). With `Iout = 0.776 Is IABC / ID` at
  the nominal 1.325 mA Q410 current, the small-signal Thevenin gain is
  `0.776 * 1.325 mA / 424 uA * 25 kohm / 40 kohm`, approximately 1.52
  (`FINAL_VCA_THEVENIN_GAIN`), rather than an assumed unity gain.
- The nominal 0-5 V amplifier envelope is converted once per host sample by
  the populated R4496/Q410 network. A Fairchild 2N4250 junction fit uses the
  original approximately 0.56 V at 100 uA and 26 mV thermal slope, solving the
  implicit diode-plus-3.3-kohm equation directly. The nominal peak reaches
  approximately 1.33 mA IABC; the result then remains fixed across all four
  oversampled audio evaluations.
- The IABC result is normalized only after the physical conversion so a 5 V
  envelope preserves the serviced level anchor. The admitted 4.7-5.3 V CEM3310
  population can extend slightly above nominal instead of being digitally
  clamped at one.
- Each complete nonlinear voice path crosses the profile's anti-alias boundary
  before it enters the host-rate common summer. The distributed path uses the
  fixed two-to-one decimator; the complete four-times oracle uses the 127-tap
  low-pass. Both place reconstruction after filter resonance and VCA curvature.
- The five voice Thevenin sources meet at U480's input through their 64 kohm
  source resistances. The A-440 network hangs on the same node: TP401 (the
  0/5 V 8253 square passed by U459 while A-440 is selected, grounded by U460
  otherwise) feeds R4498 10 kohm into C4183 0.1 uF to ground, and R4519
  20 kohm joins C4183 to the node. C4183 is the node's single state;
  `summing_node` in `output.rs` eliminates the node voltage algebraically and
  advances the capacitor exactly for inputs held across each host sample.
  Above the shelf C4183 is a short, R4519 alone loads the node and each card
  contributes approximately 0.122 of its Thevenin voltage; at low frequency
  the 30 kohm branch loads it less (approximately 0.140). The result is an
  approximately +1.2 dB bass shelf, essentially complete below about 50 Hz
  and half-way near 180 Hz, plus the physical A-440 injection. Idle cards keep
  loading the node, so the one-voice level does not depend on how many cards
  are sounding.
- The master CA3280 is distinct from the per-voice VCAs and follows the
  physical master-volume control. PCB1's R113 is a 10 kohm linear pot (SD131,
  the panel board it is mounted on, notes "ALL POTS 10K, LIN"; SD334's
  cross-reference prints 100K). Its top is fed from the +5 V analog rail
  through R345 (100 ohm) and the normalled AMPLIFIER CV IN jack, and SD430
  R4535's 100 kohm and C4184's 0.22 uF load its wiper before the U480 buffer.
  Its loaded wiper voltage is smoothed with the position-dependent Thevenin
  resistance, rather than treating the control as an instantaneous digital
  multiplier.
- Q411 is a second grounded-base 2N4250 converter. As with Q410, only its
  4.7 kohm emitter resistor R4542 sets the current; R4541 in the collector
  merely isolates U479's IABC pin. The same room-temperature junction law used
  for Q410 reconstructs approximately 0.93 mA at the nominal five-volt
  endpoint. R345 leaves the fully open wiper at approximately 4.946 V, so Q411
  runs about 1.2% below that nominal current at full volume. The junction law
  turns the physical linear pot into a useful audio taper: the loaded midpoint
  produces approximately 2.414 V and 41.5% of the nominal control current.
- U479 follows the same Figure 3A law as U477. R4561's 68 kohm from +15 V
  programs its diodes at approximately 424 uA; the voice node enters through
  R4564 15 kohm with R4563 13 kohm on the other input, and the output current
  develops across R4562's 20 kohm in parallel with R4543's 100 kohm. At the
  nominal 0.93 mA Q411 current the full-volume voltage gain is
  `0.776 * 0.932 mA / 424 uA * 16.67 kohm / (15 kohm + 13 kohm)`,
  approximately 1.02 (`MASTER_VCA_VOLTAGE_GAIN`), and the linear range is
  +/-ID x 28 kohm, approximately +/-11.9 V: five ordinary voices pass
  proportionally and only a grossly overdriven node reaches the rounded
  sixth-order current limit.
- The master-VCA output is AC-coupled by the populated 2.2 uF C4189. R4562
  (20 kohm) loads its OTA side and R4543 (100 kohm) its U481 side. Because
  U479 is a current source, the midband load is their 16.67 kohm parallel
  value while the capacitor charges through their 120 kohm series sum: the
  first-order high-pass corner is `1 / (2 pi * 2.2 uF * 120 kohm)`,
  approximately 0.60 Hz (a 264 ms time constant), not the 4.34 Hz of the
  parallel pair. C4189 is represented
  by its stored physical capacitor voltage and advanced with the exact
  exponential RC solution, so its elapsed-time decay does not inherit a host
  sample-rate approximation.
- Five paired envelope-amount profiles, five oscillator-B Poly Mod profiles
  and the common dual Wheel Mod source profile preserve the modulation-side
  CA3280 boundaries and documented diode modes.
- The three stored amount controls reach their per-voice CA3280s through the
  separate grounded-base SD333 converters: Q301/5.1k for direct filter
  envelope, Q303/5.6k for oscillator-B Poly Mod and Q304/3k for envelope Poly
  Mod. Each converter establishes one total current shared by five parallel
  IABC inputs. Their 1/12 V-per-code held controls follow the same source-backed
  2N4250 junction equation and physical fanout as the audio VCAs instead of
  linear normalized gains.
- U422's direct filter-envelope half carries no fitted gain. Its 47.5 kohm
  R452 source (R462/R492 on voices 2/5) and 28.8 V-programmed ID feed follow
  the CA3280 data sheet's Figure 3A linearized transfer,
  `Iout = 0.776 Is IABC / ID` up to the 0.82 IABC peak, into U433's 100 kohm
  common-CV input; a 5 V envelope at code 120 moves cutoff by approximately
  6.3-6.6 octaves and factory 1-4's amount 34 by approximately 1.5-1.6
  octaves (see [`POLY_MOD_MODEL.md`](POLY_MOD_MODEL.md)).
- The two Poly Mod amount stages produce physical CA3280 output currents.
  U422's envelope half uses populated 22k signal/return and 120k diode-bias
  paths; U428's oscillator half retains the enabled 150k/200k waveform-source
  loading. Their currents meet at R4108's 30k load and the resulting voltage
  is bounded at the data sheet's guaranteed minimum +/-12 V output swing
  before U431's follower and the three destination networks.
- U481 is now an explicit NE5534 voltage follower on the populated +/-15 V
  rails. R4544 permanently loads its output with 1 kohm and R4545 contributes
  the measured 560 ohm jack source resistance. The accepted manufacturer
  boundaries are 24 Vpp guaranteed and 26 Vpp typical into at least 600 ohm,
  38 mA typical output current and 13 V/us typical slew rate. The high-
  impedance RackForge input leaves R4545 unloaded; a finite external-load
  fixture verifies the divider without silently assuming a particular mixer.
- All modeled audio stages exchange circuit volts through the jack. One
  explicit candidate conversion maps approximately 2.96 jack volts to one host
  unit only after U481 (`CANDIDATE_CIRCUIT_VOLTS_PER_HOST_UNIT`). It is derived,
  not chosen: the audited circuit runs about 2 dB hotter than earlier releases
  (1/12 V DAC steps, U451's triangle, the C4183 shelf), so the constant places
  one voice above the shelf 2 dB below the earlier releases' per-voice host
  level (0.2 summer x 0.80 master / 2 V per unit), restoring their headroom
  for the strongest resonant programs. The mapping is strictly linear and
  replaces the former host `tanh`, which compressed strong chords before any
  physical stage reached its own overload boundary.
- Master volume is direct, is not delayed by the CPU control scheduler and is
  preserved when programs change.

## Bounded uncertainty

The device modes, routing, approximate input impedances, waveform-source
resistors, final-VCA 20 kohm input network, Q410/Q411 identities and bias
networks, R113/R345/R4535/C4184 master-volume network, the 2N4250
room-temperature base-emitter curve, the R4529/R4565-R4569 voice summer and
R4498/C4183/R4519 A-440 branch, linear gain-versus-bias law, AC-coupling
values and output topology are source-backed.
The deterministic population is bounded by the published 0.70-1.30
peak-output-current ratio and kept deliberately narrower. Figure 3A is a
printed bitmap, so its 0.776 linearized transfer, the +/-ID linear limit and
the sixth-order knee are explicit bounded interpretations rather than
digitized device measurements. The ID terminal's two-junction reference to
V- is read from Intersil's functional diagram, not a tabulated terminal
voltage. R4529 is taken at the maximum that trim 4-22 normally leaves; a
serviced card set slightly below it would lower that voice's Thevenin gain and
source resistance together. The oscillator and noise current networks no
longer require a one-saw loading normalization. Q410 temperature, overload-knee
spread, populated-device matching, external output load and the final
approximately 2.96-volts-per-host-unit conversion remain hypotheses. Neither
that conversion nor a digital full-scale limiter is present inside the analog
path.
The five-volt volume reference follows the documented analog control domain,
but its populated rail and R113 end-to-end tolerance are unmeasured, and
SD131's 10K note is preferred over SD334's 100K cross-reference because SD131
is the board R113 is mounted on. Exact THD, gain law and overload require
recorded sweeps from a serviced reference instrument.

## Acceptance tests

- zero bias current closes every physical VCA boundary exactly;
- gain rises monotonically with control current;
- Q306/Q302 each produce approximately 280 uA total at the code-120 panel
  ceiling,
  or about 56 uA per parallel voice-card input, while Q305 reaches approximately
  125 uA at the one common noise OTA; all three preserve their transistor knees
  and calibrated endpoints;
- the 5 V envelope reconstructs 1.30-1.35 mA IABC through R4496 alone, has a
  silicon-junction knee, and accepts the complete bounded 4.7-5.3 V CEM3310
  population;
- the linearized transfer retains more strong-signal range than the mixer VCA;
- both 68 kohm linearizing feeds program 420-428 uA against the negative
  rail; Figure 3A's +/-ID x 20 kohm reproduces its +/-4 V limit, the final
  VCA's linear range is 16.8-17.1 V with a 1.50-1.54 Thevenin gain, and it
  retains more than 99.5% of its small-signal gain at 14 Vpp while rounding
  below 90% only near 40 Vpp;
- all mixer profiles remain inside published output bounds, paired halves stay
  close but distinct, and serviced final-voice small-signal gains match;
- a single 150 kohm mixer path develops approximately 10.9 mV at U464's input,
  the first filter cell presents approximately 90.909 kohm transimpedance, and
  one full saw reaches 4.0-4.8 circuit volts across all five profiles;
- parallel paths and the 200 kohm pulse path follow the finite-input loading
  law, while the complete common-noise path reaches 0.84-1.0 circuit volts;
- all transfers are finite and odd-symmetric and reject non-finite input;
- the two Poly Mod currents share one populated 30k current-to-voltage load,
  preserve waveform-dependent input loading and remain within guaranteed
  CA3280 voltage compliance;
- the master stage remains bounded while retaining multi-voice headroom;
- the loaded linear volume pot reaches zero and, through R345, approximately
  4.946 V with approximately 98.9 ohm source resistance; its midpoint is
  approximately 2.414 V behind approximately 2.46 kohm; Q411 reaches
  0.92-0.94 mA through R4542 alone at the nominal 5 V; full volume settles at
  98-100% of that nominal control and the midpoint at 40-44%;
- C4184 smoothing is monotonic and produces the same elapsed-time response at
  48 and 96 kHz;
- the R4519/C4183/R4498 branch lifts the per-voice node gain to 1.14-1.16
  times its midband value at 5 Hz and leaves 10 kHz within 0.1%;
- U479's full-volume small-signal voltage gain lies between 1.0 and 1.04 and
  its linear range between 11.8 and 12.0 V, so a five-voice node reaches the
  jack within 1% of five times one voice while a grossly overdriven node is
  held below half its linear value;
- the 0.60 Hz coupling network rejects steady DC at every supported sample
  rate, its 100 ms capacitor decay matches the analytic 264 ms time constant
  at 44.1/48/96/192 kHz, and a 20 Hz tone matches the analytic product of the
  coupling network (approximately 99.95%), the C4183 shelf and the full-volume
  control ratio;
- the loaded NE5534 remains exactly linear through its guaranteed +/-12 V
  span at 44.1/48/96/192 kHz, is bounded by its typical +/-13 V swing, draws
  less than the published current capability through R4544 and preserves the
  documented 560 ohm source resistance under a finite-load fixture;
- the final circuit-volts-to-host conversion is linear above unity and cannot
  masquerade as analog saturation;
- program changes preserve the physical master volume.
- the decimator preserves DC gain, produces exactly one output per four
  internal samples, passes 10 kHz essentially flat and rejects a 60 kHz
  internal tone below -80 dB relative to its input RMS.

Primary evidence: TM1000D.2 sections 2-5 and 2-8, schematics SD431-SD435 and
SD430, Fig. 2-4 Detail B, service adjustments 4-21 and 4-22, the CA3280
(including Figure 3A) and NE5534 data sheets and Fairchild's
2N4248/2N4249/2N4250 data. Provenance is recorded in
`SOURCE_LEDGER.md`.
