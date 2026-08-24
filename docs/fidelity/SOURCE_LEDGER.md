# Source ledger

| ID | Source | Scope | Class | Status |
| --- | --- | --- | --- | --- |
| SRC-001 | Sequential, original *Prophet-5 Owner's Manual* | Public control architecture, performance behaviour, pitch-wheel span and Poly Mod routing | Manufacturer documentation | Accepted for topology and approximately +/-one-fifth pitch-wheel span; other numerical mappings pending |
| SRC-002 | Sequential, current Prophet-5 product specifications | Oscillator, filter, envelope and performance inventory | Manufacturer documentation | Accepted for inventory only |
| SRC-003 | Original program patch sheets published by Sequential | Historical control positions | Manufacturer document | Reference only; programs will not be redistributed |
| SRC-004 | Sequential Circuits, *Prophet-5 Synthesizer Technical Manual*, TM1000D.2, Oct. 1981 ([archival scan](https://www.synfo.nl/servicemanuals/Sequential/SEQUENTIAL_PROPHET-5-REV3_SERVICE_MANUAL.pdf)) | Rev 3.0-3.2 topology, schematics, calibration, control scanning and selected IC data sheets | Manufacturer technical manual | Accepted; SHA-256 `6B8701C4F526AB415CBA8BE2CA5538BDE14B228A0948151EBA964DA06A97BD25` |
| SRC-005 | Bob Grieb, *Instructions for using the Prophet 5 Diagnostic Firmware* | Pot scan order, DAC exercise, oscillator tune counters and PCB 3 sample/hold diagnostics | Private diagnostic documentation | Accepted for factual control-system evidence only; not redistributable; SHA-256 `C3DD214FD60D9475C80F6BF74F81203F851D47DEB331170BFD032366219A3FEC` |
| SRC-006 | National Semiconductor, *MM5837 Digital Noise Source* ([archival scan](https://radio-hobby.org/uploads/datasheet/39/mm58/mm5837.pdf)) | 17-bit feedback topology, self-clocked operation, cycle time and half-power range | Manufacturer data sheet | Accepted; SHA-256 `C8E8D2D8E7B03D3F5C8E9E5AF653BF750FBFA27E6757660BD48ED2022A708793` |
| SRC-007 | Curtis Electromusic, *CEM3320 Voltage Controlled Filter* ([original data sheet scan](https://akizukidenshi.com/goodsaffix/CEM3320.pdf)) | Four independent filter cells, exponential cutoff law, resonance cell, oscillation and distortion behaviour | Manufacturer data sheet | Accepted; SHA-256 `EFA10895A3D2D432124681F135CF7CA01FB46BC65E9971D80C99A2BEF5C92B67` |
| SRC-008 | Curtis Electromusic, *CEM3310 Voltage Controlled Envelope Generator* ([original data sheet scan](https://sandsoftwaresound.net/wp-content/uploads/2021/03/CES_CEM3310_VCEG.pdf)) | RC envelope equations, time range, attack asymptote, sustain and retrigger behaviour | Manufacturer data sheet | Accepted; SHA-256 `D4136F8DA288892E38CE174EA0A1196747CD87CF84A52A8AE8D41716CF901A17` |
| SRC-009 | Intersil, *CA3280/CA3280A Dual 9 MHz Operational Transconductance Amplifier* ([manufacturer data sheet mirror](https://www.rxelectronics.com.ua/datasheet/db/ca3280e.pdf)) | OTA transfer, linearizing diodes, gain control, current output, distortion and operating range | Manufacturer data sheet | Accepted; SHA-256 `05E6CFF9EAE9AB8203E2FA5A75AF03E4BC5FAA1DDEE93AE351EB74190E038B55` |
| SRC-010 | Curtis Electromusic, *CEM3340/3345 Voltage Controlled Oscillator* ([original data sheet scan](https://electricdruid.net/wp-content/uploads/2020/02/CEM33403345-VCO.pdf)) | Temperature compensation, oscillator drift limits, exponential-scale error and waveform/output behaviour | Manufacturer data sheet | Accepted; SHA-256 `9D23F54FE97114C45BCE7B1B74BABE7FF4EC05AFF6CF5D17206C1EE6649EC82A` |

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
`3D2BF29CD5EC55D1938CEF3A6A5D237D3C738DBEA6D2C4219338D21571D893A2`.
Its binary contents have not yet been admitted as behavioural truth. Before any
firmware-derived conclusion enters the engine it must be independently tied to
the documented hardware or a reproducible observation.

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
