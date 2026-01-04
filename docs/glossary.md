
# M0Club Glossary

This glossary defines terms used across the M0Club codebase and documentation.
Definitions are written to be operationally useful for developers, operators, and integrators.

---

## A

### Anchor
A Solana development framework used to build and test on-chain programs. Anchor provides IDL generation, account validation, and client tooling.

### Anomaly Detection
A set of statistical or model-based checks that detect abnormal data behavior (outliers, distribution shifts, divergence between sources) indicating data quality issues or adversarial manipulation.

### API Gateway
A service that exposes HTTP endpoints for querying market metadata, latest analytics, historical results, and system status. Often includes auth, rate limiting, and caching.

### Authority
An on-chain role or key that is permitted to perform privileged actions (e.g., configure markets, rotate signer sets, upgrade programs). Authority is typically guarded by governance or multisig.

---

## B

### Backfill
A batch process that populates missing historical data (events, model outputs, on-chain records) after downtime or to initialize a new deployment.

### Backtesting
A process of evaluating models by running them over historical data to measure calibration, accuracy, drift, and robustness.

### Bayesian Update
A statistical method used to update probability distributions as new evidence arrives. M0Club uses Bayesian-style updates to keep forecasts responsive and calibrated.

### Bundle
A deterministic payload produced by the engine containing analytics for one or more markets and epochs. Bundles are hashed and signed before being published on-chain.

### Bundle Hash
A stable hash of a bundle computed over canonical bytes. Used as the commitment in commit-reveal and as the verification target for signatures.

---

## C

### Calibration
Techniques that align predicted probabilities with observed outcomes over time. A well-calibrated model output of 0.70 should occur roughly 70% of the time in the long run.

### Canonical Serialization
A deterministic encoding method that ensures the same logical bundle produces the same bytes and hash across implementations and platforms.

### CI (Continuous Integration)
Automated workflows that run checks (formatting, linting, tests, security scanning) on every push and pull request.

### ClickHouse
A column-oriented database often used for high-throughput analytics storage and queries.

### Commit-Reveal
A two-phase on-chain publication flow:
1) Commit publishes only a hash (commitment) to the payload.
2) Reveal later publishes the payload (or proof) that matches the commitment.
This reduces copying/front-running of full payloads.

### Confidence Interval
A range around an estimate describing uncertainty. M0Club attaches confidence metadata to probability outputs to represent model uncertainty and data variability.

### Connector
A module that ingests signals from a specific source (on-chain, sports odds feeds, election feeds, macro data). Connectors normalize and validate source data.

---

## D

### Data Ingestion
The process of collecting raw data from sources, validating it, normalizing it, and emitting standardized events into the engine pipeline.

### Deduplication
Logic that removes repeated events using stable identifiers or idempotency keys to prevent double counting.

### Drift
A change in the statistical properties of data or model performance over time. Drift can indicate regime changes or manipulation.

---

## E

### Epoch
A time-bounded window for a market during which updates are produced and eventually finalized on-chain. Epochs can be defined by fixed intervals or event-driven windows.

### Event Log
An append-only store of normalized events (e.g., Kafka, NATS JetStream) used for replay, recovery, and consistent processing.

---

## F

### Feature Store
A storage layer that holds derived features used for modeling (aggregations, rolling windows, normalized indicators). Can be implemented using Postgres, ClickHouse, or specialized stores.

### Finalization
The act of selecting and recording the canonical oracle output for an epoch on-chain. After finalization, consumers should treat the finalized output as authoritative.

### Front-Running
A class of attacks where an adversary observes pending transactions or known payloads and submits transactions to gain advantage before the original update is finalized.

---

## G

### Governance
A set of processes and on-chain/off-chain controls for approving changes (program upgrades, parameter changes, signer rotation). Often implemented via timelocks or multisigs.

### Guardian
A privileged role that can trigger emergency actions (pause, rollback, disable markets) under predefined conditions.

---

## H

### Hash Commitment
A cryptographic commitment to a payload using a hash function. The commitment can be verified later by revealing the payload and recomputing the hash.

### HSM (Hardware Security Module)
A hardware device that securely stores and uses cryptographic keys. Recommended for production signer keys.

---

## I

### Idempotency Key
A stable key associated with an action (event ingest, tx submission) such that retries do not produce duplicates.

### Indexer
A service that reads on-chain logs/events and writes them into a queryable store. The indexer reconciles commits, reveals, and finalizations with off-chain bundles.

### Integrity Metadata
Information that allows verification of correctness and authenticity, such as bundle hashes, signatures, signer set identifiers, and replay protection counters.

---

## K

### KMS (Key Management Service)
A cloud service that stores cryptographic keys and performs signing without exposing private key material to the application process.

### Kubernetes (K8s)
A container orchestration platform used to deploy and scale engine and service components.

---

## L

### Latency
The time it takes for data to move through the system (ingestion → modeling → signing → commit/reveal → availability to consumers). Latency targets differ by domain.

### Localnet
A local Solana test environment created with `solana-test-validator` used for development and integration tests.

---

## M

### Market
A prediction target with defined outcomes, e.g., a sports match or election result. Each market has a stable identifier and an outcome schema.

### Market Registry
An on-chain/off-chain registry that stores market metadata such as domain, symbols, outcome schema, update cadence, and policy settings.

### Merkle Root
A root hash of a Merkle tree built over bundle items. Enables efficient proofs for subsets of a large bundle.

### M0-CORE
The proprietary engine layer responsible for modeling, calibration, bundling, and integrity primitives.

### Monorepo
A single repository containing multiple components (programs, engine, services, SDKs, infra) managed together.

---

## O

### Observability
The set of practices and tools used to understand system behavior, including logs, metrics, and traces.

### Outcome
A discrete result option within a market (team A wins, team B wins, draw, candidate wins, etc.). Each outcome has an identifier used in bundles and on-chain storage.

### Oracle
A system that delivers off-chain data or computed analytics to a blockchain in a verifiable way.

---

## P

### Probability Distribution
A mapping from outcomes to probabilities that sum to 1. M0Club publishes distributions rather than single point estimates.

### Program (Solana)
An on-chain executable deployed to Solana. In this repo, programs are typically built with Anchor under `programs/`.

### Proof
A cryptographic artifact that allows verification of a claim. In M0Club, proofs can include Merkle proofs that a bundle item belongs to a committed root.

---

## R

### Reconciliation
A process that compares expected state with observed state and repairs discrepancies (e.g., bundle committed but reveal missing, indexer lag, missed epochs).

### Replay Protection
Mechanisms that prevent reusing old payloads or signatures, such as nonces, sequence numbers, or epoch gating.

### Realtime Service
A websocket or streaming service that delivers push updates to consumers as new analytics are produced or finalized.

### RPC
Remote Procedure Call. In Solana, RPC nodes provide access to blockchain state and transaction submission.

---

## S

### Schema Version
A version identifier for data formats (events, bundles, IDLs). Schema versioning enables backward compatibility and controlled migrations.

### Signer Set
A set of public keys permitted to sign oracle bundles. The signer set can be rotated by authority/governance.

### Signer Agent
An off-chain component that signs bundle hashes and enforces key policies. In production, it should be isolated and backed by KMS/HSM.

### SLA
Service Level Agreement. Targets for uptime, latency, and correctness.

### Solana Test Validator
A local validator binary used for localnet development and tests.

### Supply Chain Security
Controls that reduce risk from dependencies, build tooling, and CI artifacts (audits, deny lists, secret scanning).

---

## T

### Timelock
A governance mechanism that enforces a delay between scheduling and executing privileged changes, improving safety and transparency.

### Trace
A distributed trace that spans multiple services and components, typically implemented using OpenTelemetry.

---

## U

### Uncertainty
A measure of how confident the system is in its predictions. Represented with confidence intervals, risk scores, and drift flags.

---

## V

### Verifiability
The ability for any party to independently verify that published oracle outputs match committed hashes and valid signatures.

---

## W

### Watermark
A tracking value representing the latest processed timestamp or event id, used to resume ingestion and guarantee ordering.

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
