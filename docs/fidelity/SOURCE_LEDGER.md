# Source ledger

| ID | Source | Scope | Class | Status |
| --- | --- | --- | --- | --- |
| SRC-001 | Sequential, *Prophet-5 User's Guide 1.3* | Public control architecture, performance behaviour, Poly Mod routing | Manufacturer documentation | Accepted for topology; numerical mappings pending |
| SRC-002 | Sequential, current Prophet-5 product specifications | Oscillator, filter, envelope and performance inventory | Manufacturer documentation | Accepted for inventory only |
| SRC-003 | Original program patch sheets published by Sequential | Historical control positions | Manufacturer document | Reference only; programs will not be redistributed |

## Private local evidence inventory

The development environment contains three archives relevant to the reference
family: a Rev 2 OS set, a Rev 3 V8.1 OS/diagnostic set and a Rev 3.3 diagnostic
image. They are evidence inputs only and are intentionally excluded from Git,
CI and the `.rfplugin`. Four neighboring archives target later sampler/vector
instruments and are out of scope.

Firmware may help establish scanning, allocation, calibration and program-memory
behaviour, but it does not replace modelling the analog signal path. RF-5 will
implement admitted behaviour in Rust and will never require or load these files
at runtime.

## Required next sources

- Service manual and board schematics for the frozen hardware revision.
- Primary datasheets for every oscillator, filter, envelope and amplifier IC.
- Calibration and trim procedures.
- Legally redistributable measurements with instrument, load, sample rate and
  uncertainty recorded.

URLs and immutable local hashes will be added when each source is admitted into
an implementation block.
