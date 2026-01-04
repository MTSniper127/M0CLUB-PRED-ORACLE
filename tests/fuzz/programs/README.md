
# Program Fuzzing (cargo-fuzz)

This is a scaffold for fuzzing on-chain instruction decoding and state transitions.
Requires nightly Rust and cargo-fuzz.

Quick start:
```bash
cd programs/m0-oracle
rustup toolchain install nightly
cargo install cargo-fuzz
cargo fuzz list
cargo fuzz run m0_oracle_fuzz
```
