# Source ledger

| ID | Source | Scope | Class | Status |
| --- | --- | --- | --- | --- |
| SRC-001 | Sequential, original *Prophet-5 Owner's Manual* ([archival scan](https://device.report/m/53bd1bb5dfcacc44b25c01ad486bc93a9adc1fdfc4fdb0340c51b239ee1c578d_pdf)) | Public control architecture, performance behaviour, non-programmable MASTER TUNE classification, oscillator-B FINE direction/span, 1-99% pulse-width range and modulation overtravel, pitch-wheel span, Poly Mod routing and Scale Mode operation | Manufacturer documentation | Accepted for topology, MASTER TUNE's machine-state boundary, B FINE's zero-to-one-semitone law, panel pulse-width endpoints and DC degeneration under modulation, approximately +/-one-fifth pitch-wheel span and twelve-note Scale Mode operator behavior; SHA-256 `4C5D84BBB5A0FE8D1687A66DB67292259BFF160D7738F0366DB8DE39026FB85C` |
| SRC-002 | Sequential, current Prophet-5 product specifications | Oscillator, filter, envelope and performance inventory | Manufacturer documentation | Accepted for inventory only |
| SRC-003 | Original program patch sheets published by Sequential | Historical control positions | Manufacturer document | Reference only; programs will not be redistributed |
| SRC-004 | Sequential Circuits, *Prophet-5 Synthesizer Technical Manual*, TM1000D.2, Oct. 1981 ([archival scan](https://www.synfo.nl/servicemanuals/Sequential/SEQUENTIAL_PROPHET-5-REV3_SERVICE_MANUAL.pdf)) | Rev 3.0-3.2 topology, schematics, calibration, control scanning, MASTER TUNE/master-summer path, pitch-wheel deadband, SD334 LFO timing/conditioning/pulse-load population, SD431's CEM3340 pulse pull-down/mixer switching and complete CEM3320 resonance-return population, plus selected IC data sheets | Manufacturer technical manual | Accepted; SHA-256 `6B8701C4F526AB415CBA8BE2CA5538BDE14B228A0948151EBA964DA06A97BD25` |
| SRC-005 | Bob Grieb, *Instructions for using the Prophet 5 Diagnostic Firmware* | Pot scan order, DAC exercise, oscillator tune counters and PCB 3 sample/hold diagnostics | Private diagnostic documentation | Accepted for factual control-system evidence only; not redistributable; SHA-256 `C3DD214FD60D9475C80F6BF74F81203F851D47DEB331170BFD032366219A3FEC` |
| SRC-006 | National Semiconductor, *MM5837 Digital Noise Source* ([archival scan](https://radio-hobby.org/uploads/datasheet/39/mm58/mm5837.pdf)) | 17-bit feedback topology, self-clocked operation, cycle time and half-power range | Manufacturer data sheet | Accepted for topology and both published ranges; RF-5's geometric-centre cycle candidate and held-bit half-power validation are explicit reconstruction rules, not a claimed device typical; SHA-256 `C8E8D2D8E7B03D3F5C8E9E5AF653BF750FBFA27E6757660BD48ED2022A708793` |
| SRC-007 | Curtis Electromusic, *CEM3320 Voltage Controlled Filter* ([original data sheet scan](https://akizukidenshi.com/goodsaffix/CEM3320.pdf)) | Four independent filter cells, exponential cutoff law, resonance-cell Gm including Figure 6, 2.7-4.5 kohm Q-input impedance, AC-coupling requirement, oscillation and distortion behaviour | Manufacturer data sheet | Accepted; the rational Gm reconstruction is fixed by the tabulated 1 mmho at 100 uA typical and 2.2 mmho maximum, then checked within a six-percent reading band at three normalized Figure 6 landmarks; SHA-256 `EFA10895A3D2D432124681F135CF7CA01FB46BC65E9971D80C99A2BEF5C92B67` |
| SRC-008 | Curtis Electromusic, *CEM3310 Voltage Controlled Envelope Generator* ([original data sheet scan](https://sandsoftwaresound.net/wp-content/uploads/2021/03/CES_CEM3310_VCEG.pdf)) | RC envelope equations, time range, attack asymptote, sustain and retrigger behaviour | Manufacturer data sheet | Accepted; SHA-256 `D4136F8DA288892E38CE174EA0A1196747CD87CF84A52A8AE8D41716CF901A17` |
| SRC-009 | Intersil, *CA3280/CA3280A Dual 9 MHz Operational Transconductance Amplifier* ([manufacturer data sheet mirror](https://www.rxelectronics.com.ua/datasheet/db/ca3280e.pdf)) | OTA transfer, linearizing diodes, gain control, current output, distortion and operating range | Manufacturer data sheet | Accepted; SHA-256 `05E6CFF9EAE9AB8203E2FA5A75AF03E4BC5FAA1DDEE93AE351EB74190E038B55` |
| SRC-010 | Curtis Electromusic, *CEM3340/3345 Voltage Controlled Oscillator* ([original data sheet scan](https://electricdruid.net/wp-content/uploads/2020/02/CEM33403345-VCO.pdf)) | Temperature compensation, oscillator drift limits, exponential-scale error, 0-5 V PWM control law, waveform/output behaviour, loaded pulse-high equation and triangle-buffer load-dependent frequency pull | Manufacturer data sheet | Accepted, including the published pulse-output 0.6 mA breakpoint/1.3 kohm slope, 65-150 ohm triangle-output-impedance range and `Rout / Rload` triangle-frequency reduction; SHA-256 `9D23F54FE97114C45BCE7B1B74BABE7FF4EC05AFF6CF5D17206C1EE6649EC82A` |
| SRC-011 | Vishay Semiconductors, *1N914 Small Signal Fast Switching Diodes*, document 85622, rev. 2.2, Nov. 2024 ([manufacturer data sheet](https://www.vishay.com/docs/85622/1n914.pdf)) | Forward-voltage/current curve and temperature dependence for the SD334 diode-deadband candidate | Manufacturer data sheet | Accepted only to bound the silicon-diode candidate, not as a measurement of the historical D315/D316 pair; SHA-256 `D6BA700D86C6776DE162E065EA8E38380DA27E877F14831B406CE6167616940F` |
| SRC-012 | Nexperia, *HEF4051B 8-channel analog multiplexer/demultiplexer*, rev. 14, July 2024 ([manufacturer data sheet](https://assets.nexperia.com/documents/data-sheet/HEF4051B.pdf)) | Conservative 4051-class on-resistance bound for CV sample/hold acquisition | Manufacturer data sheet | Accepted only as a conservative modern upper bound: 175 ohm maximum peak at 15 V, not as identification of the historical populated part; SHA-256 `7BEF8EC5FB05C9FEA387E5C8453F41CD78B0242B84725BD394D4A141B3055887` |
| SRC-013 | Fairchild Semiconductor, *Switching, General Purpose, and RF Transistors*, 1969 catalog section 10, pp. 10-120 through 10-123 ([manufacturer catalog scan](https://archive.decromancer.ca/bitsavers.org/components/fairchild/_dataBooks/1969_Fairchild_Semiconductor_Integrated_Circuit_Data_Catalog/10.pdf)) | Original 2N4250 PNP identity, current gain and room-temperature base-emitter voltage versus collector current | Manufacturer data sheet | Accepted for the SD431 Q410 voltage-to-current candidate at 25 C; the fit uses the 100 uA approximately 0.56 V point and silicon thermal slope, not an invented fixed threshold; SHA-256 `CB6249267862AC157BA998BAE1669441D8FE8B192B54AEBF9F84DE79E98BE7D3` |
| SRC-014 | Texas Instruments, *TL082 Wide Bandwidth Dual JFET Input Operational Amplifier*, SNOSBW5C, Apr. 1998, rev. Apr. 2013 ([manufacturer data sheet](https://www.ti.com/lit/ds/symlink/tl082-n.pdf)) | U474 output swing, slew rate, gain bandwidth and large-signal distortion under the Rev 3 +/-15 V supply | Manufacturer data sheet | Accepted for the load-qualified +/-12 V minimum and +/-13.5 V typical output swing, 8/13 V/us minimum/typical slew rate and less than 0.02% THD at 20 Vpp into 10 kohm; the exact overload-knee curve remains unmeasured |

## Private local evidence inventory

Non-redistributable manuals, diagnostic notes and firmware archives belong in
`references-local/`. That directory is deliberately ignored by Git and is the
only persistent location inside the checkout for private evidence. Rendered PDF
pages, package-installation stores and repeat auditions are temporary data and
must remain under `tmp/`; `tools/clean-generated.ps1` removes them safely.

The development environment contains three archives relevant to the reference
family: a Rev 2 OS set, a Rev 3 V8.1 OS/diagnostic set and a Rev 3.3 diagnostic
image. They are evidence inputs only and are intentionally excluded from Git,
CI and the `.rfplugin`. Four neighboring archives target later sampler/vector
instruments and are out of scope.

The admitted Rev 3 V8.1 archive has SHA-256
`3D2BF29CD5EC55D1938CEF3A6A5D237D3C738DBEA6D2C4219338D21571D893A2`;
its operating image `v81_2732.bin` has SHA-256
`7990D7667B3B08A06755BA6CE98B57CC8E20AFCD162C096BB1BCFB9DC6707EB5`.
Operating-image offsets `0x0101-0x0125` are admitted specifically for the
signed C4-C3 difference repeatedly subtracted to create the C2-C0 automatic
tune entries. This conclusion is independently tied to the technical manual's
C3-C9 measurement and C0-C2 extrapolation description. Offsets
`0x0235-0x023D` are admitted for the equal-tempered code-64 power-up state.
Offsets `0x02C9-0x0333` are admitted for the 24-pot window-ADC scan, its
increment/decrement comparison and its same-direction change qualification.
The technical manual independently fixes the 34 mV comparator hysteresis and
the requirement for two movements in one direction.
Offsets `0x0383-0x03E9` are admitted for normal/LO FREQ coarse-code assembly, keyboard
inclusion and the 108-semitone cap. Offsets `0x03EE-0x0483`, the multiply data
they address at `0x0A00-0x0BDF`, and the DAC
write paths at `0x0155-0x01DB` are admitted for the signed runtime
interpolation, its lookup-rounding behavior and its 128-writable-code semitone
scale. The complete output loop at `0x0583-0x05C4` is additionally admitted for
the five banks of eight S/H strobe addresses and their two unconnected terminal
slots. Its two `EX (SP),IX` delays, intervening load and inhibit write establish
a 64-T-state active dwell. SD332 shows that `Vdac` leaves the LF356 directly;
R354's 5 kohm branch feeds the ADC-gain stage instead. SD333 and SD430 anchor
the 0.01 uF hold capacitors, while SRC-012 bounds the intervening 4051 at 175
ohm for a conservative 1.75 us acquisition constant.
The main-loop order at `0x025D-0x02A4`, new-key path at `0x0630-0x0674` and CV
pass at `0x0583-0x05C4` are admitted for gate-before-next-CV sequencing; the
special immediate `0x1B` strobe is independently identified by SD333 as
sequencer output. Offsets `0x0336-0x0358`, `0x04D1-0x04F6` and
`0x0503-0x051F` are admitted for the lowest-key common Unison CV, removal of
keyboard pitch from individual oscillator cells and suppression of individual
filter-keyboard CV in Unison.
Offsets `0x0484-0x04BF` and lookup bytes `0x0BE7-0x0BF2` are
admitted for the active twelve-note Scale Mode pot map, signed half-semitone
arithmetic and post-interpolation application. Offsets `0x07C8-0x0813` are
admitted for the 24-byte program pack/unpack loops, `0x0563-0x056E` for their
three switch-latch output bytes, and `0x0524-0x0558` for the stored RELEASE
branch and its fixed `0x64` disabled value. SD333 independently fixes the
latch-bit destinations, while the owner's manual fixes the audible meaning of
RELEASE on and off. Those conclusions are independently corroborated by the
manuals' keyboard/frequency/LO FREQ CV sums, ten-octave bias-table,
between-octave calculation, 14-bit writable-DAC, 83 mV/651 uV descriptions and
Scale Mode operator behavior. No other binary region is admitted as behavioural
truth without the same corroboration.

Firmware may help establish scanning, allocation, calibration and program-memory
behaviour, but it does not replace modelling the analog signal path. RF-5 will
implement admitted behaviour in Rust and will never require or load these files
at runtime.

## Required next sources

- Independent measurements of oscillator waveforms and sync transitions.
- Filter sweeps at controlled input level and resonance settings.
- Envelope time/shape measurements across the 128 hardware steps.
- Legally redistributable measurements with instrument, load, sample rate and
  uncertainty recorded.
