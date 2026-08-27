# RF-5 reference hardware

## Frozen target

RF-5 targets the Revision 3.2 member of the original five-voice architecture.
For the audio path this means the Revision 3 voice design documented in
TM1000D.2: two CEM3340 oscillators per voice, CA3280 mixer and amount VCAs, a
CEM3320 four-pole low-pass filter, two CEM3310 envelope generators and five
functionally identical voices.

Revision 3.2 is selected because its manufacturer technical manual is complete,
its control system aligns with the admitted local diagnostic material, and its
changes from 3.0/3.1 are explicitly documented. The later revision expands
memory and interfaces and adds external pitch/modulation CV summing; it does not
replace the Revision 3 voice topology.

This is one reference, not an averaged family model. RF-5 therefore excludes:

- the SSM-based Revision 1/2 filter and voice topology;
- later instruments, reissues and switchable multi-revision behaviour;
- any runtime dependency on original EPROM or program memory;
- trade dress and manufacturer artwork.

The original forty factory-program values are admitted as data, not as a
change to the frozen hardware target. They are projected from an official
digital Group 5 dump into the already reconstructed V8.1 record and documented
in [`ORIGINAL_FACTORY_PROGRAMS.md`](ORIGINAL_FACTORY_PROGRAMS.md).

## Board and signal boundary

The implementation boundary follows the physical system:

1. control panel and keyboard scanning;
2. CPU-side program state, voice assignment, tuning and CV scheduling;
3. common analog summing, wheel modulation and glide;
4. five independent voice paths;
5. final voice summing, volume VCA and audio output.

Firmware evidence is authoritative only for digital behaviour that can be tied
to this boundary. It cannot establish oscillator, filter, envelope or VCA sound
by itself.

## Admission rule

Every circuit block must record the schematic nodes it represents, its control
units, nonlinear assumptions, sample-rate strategy and an acceptance test.
Where measurements do not exist, RF-5 will expose the uncertainty in its
fidelity notes instead of silently treating an inferred constant as fact.
