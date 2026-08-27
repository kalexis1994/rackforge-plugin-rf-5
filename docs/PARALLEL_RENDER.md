# Parallel render contract

RF-5 implements RackForge's `parallel_render_v1` contract without changing the
instrument model. The host owns every worker thread. RF-5 exposes the five
physical voice cards as independent render units and keeps allocation, shared
control circuitry and the final output circuit on the coordinator.

```text
begin_block (coordinator, serial)
  MIDI + automation -> allocator -> shared LFO/noise/CV state
                           |
                           +-> frame-exact command journal
                           +-> common per-frame payload

render_unit 0..4 (isolated worker instances)
  physical voice state + commands + common payload -> mono voice slot

end_block (coordinator, serial)
  slots 0..4 in fixed order -> five-input sum -> master VCA -> output circuit
```

## State ownership

- The coordinator is the canonical owner of MIDI, automation, program/state
  changes, voice allocation, shared LFO and noise sources, the control scanner,
  sample/hold cells, automatic tuning, A-440 and the common output stages.
- Each worker owns exactly one persistent physical `Voice`, including both
  oscillators, both envelopes, filter, voice VCA and its service calibration.
- A bounded command journal carries reset, start, retune and release operations
  to the corresponding physical unit at the exact frame where the coordinator
  observed them.
- A monotonically increasing voice epoch invalidates worker state after a
  reset, program operation or topology rebuild without relying on shared Rust
  memory between WebAssembly instances.
- Free-running oscillators remain active after their envelope becomes idle.
  Once a physical card has been initialized, it continues to be scheduled.

The shared payload contains only the compact settings and modulation values
required by all voices. The per-unit payload contains calibration data and the
timed command journal. Payloads are fixed-capacity and allocation-free in the
audio callback.

## Determinism and fallback

`rackforge_process` remains mandatory and is generated from the same
three-phase implementation. With one worker, an unsupported host, or a graph
that selects the compatibility path, RackForge executes the units sequentially
and then calls the same final mixer. The five slots are always accumulated in
ascending physical-unit order, so worker completion order cannot change the
floating-point result.

The plugin regression suite renders chords, voice stealing, Unison, Glide,
pitch bend, Wheel Mod, sustain, note release, filter automation and all-notes-
off through both the ordinary `Engine::next_sample` path and the composed
parallel ABI. Every output sample must be bit-identical. The complete workspace
suite additionally verifies free-running state, sample-accurate control order,
program/state round trips and supported sample rates.

## Portability

The package still contains one platform-independent WebAssembly component.
There are no ARM, x86 or operating-system branches in RF-5. Worker count,
scheduling policy, fault containment and telemetry belong to RackForge; the
same `.rfplugin` uses the parallel extension where the host supports it and the
classic export elsewhere.
