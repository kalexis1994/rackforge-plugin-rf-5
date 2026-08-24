# Contributing

Changes to the audio path must identify the evidence they implement, add a
repeatable test or measurement and update the fidelity gap matrix. A visually
or subjectively plausible result is not enough to become the reference model.

Run the following before submitting a change:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --release --workspace
```

Do not contribute proprietary firmware, ROM images, factory sound banks,
trademarked artwork or measurements that cannot legally be redistributed.
