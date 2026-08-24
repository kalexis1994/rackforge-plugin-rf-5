# Source ledger

| ID | Source | Scope | Class | Status |
| --- | --- | --- | --- | --- |
| SRC-001 | Sequential, *Prophet-5 User's Guide 1.3* | Public control architecture, performance behaviour, Poly Mod routing | Manufacturer documentation | Accepted for topology; numerical mappings pending |
| SRC-002 | Sequential, current Prophet-5 product specifications | Oscillator, filter, envelope and performance inventory | Manufacturer documentation | Accepted for inventory only |
| SRC-003 | Original program patch sheets published by Sequential | Historical control positions | Manufacturer document | Reference only; programs will not be redistributed |
| SRC-004 | Sequential Circuits, *Prophet-5 Synthesizer Technical Manual*, TM1000D.2, Oct. 1981 ([archival scan](https://www.synfo.nl/servicemanuals/Sequential/SEQUENTIAL_PROPHET-5-REV3_SERVICE_MANUAL.pdf)) | Rev 3.0-3.2 topology, schematics, calibration, control scanning and selected IC data sheets | Manufacturer technical manual | Accepted; SHA-256 `6B8701C4F526AB415CBA8BE2CA5538BDE14B228A0948151EBA964DA06A97BD25` |
| SRC-005 | Bob Grieb, *Instructions for using the Prophet 5 Diagnostic Firmware* | Pot scan order, DAC exercise, oscillator tune counters and PCB 3 sample/hold diagnostics | Private diagnostic documentation | Accepted for factual control-system evidence only; not redistributable; SHA-256 `C3DD214FD60D9475C80F6BF74F81203F851D47DEB331170BFD032366219A3FEC` |

## Private local evidence inventory

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
