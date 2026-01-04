
# M0Club Core Engine (m0-core)

This folder contains the off-chain engine that powers:
- data ingestion
- normalization
- feature generation
- probabilistic modeling
- calibration and confidence intervals
- anomaly guardrails
- bundle formatting and hashing
- commit/reveal signing and transaction submission

The code is designed as a Rust workspace with multiple crates and runnable binaries.

Quick start (local build):
```bash
cd core-engine
cargo build --workspace
cargo test --workspace
```

Run the daemons (in separate terminals):
```bash
cargo run -p m0-ingestd -- --config ../../config/dev.toml
cargo run -p m0d -- --config ../../config/dev.toml
cargo run -p m0-signer-agent -- --config ../../config/dev.toml
```
