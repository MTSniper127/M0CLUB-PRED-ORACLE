
# Performance Specification (M0-CORE)

This document specifies performance targets, constraints, and optimization guidance for M0Club (M0-CORE).
It defines SLOs, throughput goals, latency budgets, resource models, and benchmarking requirements across ingestion, aggregation, modeling, bundling, signing, and publishing.

This spec is implementation-oriented and should be used to guide system design, CI performance gates, and production capacity planning.

---

## 1. Goals

- Sustain high-volume ingestion and low-latency end-to-end publishing.
- Provide measurable SLOs and budgets per pipeline stage.
- Define concurrency, partitioning, and backpressure strategies.
- Provide benchmarking and profiling guidance for Rust services and SDK verification.
- Ensure performance does not compromise determinism and integrity.

Non-goals:
- Optimizing for unrealistic microsecond latencies at the expense of safety.
- Providing proprietary infra numbers beyond high-level budgets.

---

## 2. Performance Model Overview

M0-CORE’s end-to-end path:
1) Ingest event
2) Normalize + dedup
3) Aggregate into window
4) Model probabilities + CIs + risk
5) Bundle and hash
6) Collect signatures
7) Commit and reveal on Solana
8) Confirm and reconcile

Performance must be evaluated both:
- per-stage latency
- and overall publish cadence (ticks per market)

---

## 3. SLOs and Targets

### 3.1 Throughput targets
- Daily updates: 10M+ (aggregate across markets)
- Peak events: burst handling at 10x steady-state for short periods
- Markets: 100+ elite markets (configurable)

### 3.2 Latency budgets (reference)
Budgets depend on market type. Define three tiers:

#### FAST tier (on-chain microstructure, high frequency)
- ingest -> bundle ready: p95 < 500 ms
- signature collection: p95 < 200 ms
- commit confirmation: p95 < 2 s
- reveal confirmation: p95 < 2 s
- end-to-end publish: p95 < 5 s

#### NORMAL tier (sports odds, standard markets)
- ingest -> bundle ready: p95 < 2 s
- signature collection: p95 < 500 ms
- end-to-end publish: p95 < 15 s

#### SLOW tier (politics/macro)
- ingest -> bundle ready: p95 < 30 s
- end-to-end publish: p95 < 60 s
Focus is on correctness, not speed.

### 3.3 Availability targets
- Engine availability: 99.9% (tiered by environment)
- Publish success rate: 99.5% per tick (excluding chain-wide incidents)

---

## 4. Stage Budgets and Constraints

### 4.1 Ingestion
Key constraints:
- network IO and parsing cost
- external API limits

Targets:
- connector worker p95 parse+normalize per event < 2 ms average
- dedup check p95 < 1 ms (in-memory), < 10 ms (durable)
- event-log publish p95 < 10 ms

Strategies:
- batch reads from APIs
- async IO
- pre-allocated buffers
- avoid allocations in hot path
- strict schema validation to reject early

### 4.2 Aggregation
Constraints:
- per-market window state size
- out-of-order handling

Targets:
- window update per event < 5 ms p95 for hot partitions
- window finalization per tick < 50 ms p95

Strategies:
- shard by market_id
- maintain compact rolling state
- use fixed-size ring buffers for time windows
- avoid copying feature arrays
- use incremental computations

### 4.3 Modeling (Quant)
Constraints:
- CPU-heavy math
- calibration and CI computations
- drift checks

Targets:
- per-market model compute p95:
  - FAST: < 50 ms
  - NORMAL: < 200 ms
  - SLOW: < 2 s

Strategies:
- precompute reusable transforms
- avoid expensive special functions in hot path
- cache calibration artifacts
- use integer math where possible
- batch multiple markets per worker for throughput

### 4.4 Bundling and hashing
Constraints:
- canonical serialization cost
- hashing cost
- bundle size limits

Targets:
- bundle construction + hash p95 < 20 ms for typical bundles
- merkle mode overhead acceptable (< 100 ms p95)

Strategies:
- streaming encoder (no intermediate allocations)
- pre-sort ids and use stable ordering
- use sha256 implementations with SIMD if available
- reuse buffers

### 4.5 Signature collection
Constraints:
- network round trip to signer agents
- threshold aggregation

Targets:
- p95 signature collection < 200-500 ms depending on tier
- retry budget bounded (avoid cascading latency)

Strategies:
- parallel requests to signers
- strict timeouts and early stop when threshold reached
- use persistent HTTP2/gRPC connections
- keep allocator local and fast

### 4.6 Publishing (commit/reveal)
Constraints:
- Solana RPC latency and reliability
- transaction size and compute units

Targets:
- commit tx build < 10 ms
- reveal tx build < 10 ms
- confirmation p95 depends on commitment level and chain conditions

Strategies:
- prebuild instruction templates
- use compute budget instructions as needed
- dynamic fee prioritization when congested
- idempotent retries with state checks

---

## 5. Concurrency and Partitioning

### 5.1 Partition strategy
Primary partition key: market_id.
For hot markets:
- use stable sharding: shard = first byte of event_id mod N
- keep shard count configurable

### 5.2 Worker sizing
Each stage should scale horizontally:
- ingestion connectors scale by source and partition
- aggregation scales by partition
- quant scales by partition
- submitter may be fewer but must handle concurrency safely

### 5.3 Backpressure
Backpressure is essential to avoid memory blow-ups.

Mechanisms:
- bounded queues between stages
- dropping or degrading non-critical signals when overloaded
- rate limiting per connector
- prioritize markets by tier (FAST > NORMAL > SLOW)

### 5.4 Scheduling policy
For high frequency:
- schedule modeling ticks per market on a fixed cadence
- skip ticks if prior tick not finished (avoid backlog)

---

## 6. Resource Models

### 6.1 CPU
Hot costs typically:
- parsing/normalization: low to moderate
- modeling and CI: moderate to high
- hashing: moderate
- JSON logging: can be significant

Guidance:
- keep log volume controlled
- prefer structured logs with sampling in hot paths

### 6.2 Memory
Main memory consumers:
- dedup caches
- aggregation window state
- feature buffers
- event log buffers

Guidance:
- enforce TTLs and max sizes
- use compact structs and arenas
- avoid storing raw payloads unless needed

### 6.3 Network
- ingestion external APIs can dominate
- signer requests and RPC calls are critical path

Guidance:
- use connection pooling
- keep RPC endpoints redundant
- implement retry with jitter

### 6.4 Storage
- event logs can grow rapidly
- feature store growth is a function of cadence and retention

Guidance:
- tiered retention policies
- compaction and downsampling for older features

---

## 7. Benchmarking

### 7.1 Microbenchmarks
For Rust, use criterion benchmarks on:
- canonical encoding + hashing
- fixed-point conversions
- CI quantile computation
- calibration transform application
- feature aggregation update

### 7.2 Load tests
Simulate:
- sustained event ingestion at target throughput
- burst events (10x for 1-5 minutes)
- multiple market partitions concurrently
- RPC timeout storms

Measure:
- p50/p95/p99 latencies per stage
- memory growth and GC/allocator behavior
- queue depths and drop rates

### 7.3 End-to-end benchmarks
Localnet and staging:
- time from event ingestion to on-chain reveal confirmation
- publish cadence adherence
- retry and idempotency behavior under failure

---

## 8. Performance Gates in CI

Add CI checks:
- run microbenchmarks and compare to baseline thresholds
- fail if regressions exceed allowed percent (e.g., 10%)
- include hash determinism tests (must always pass)
- run a small load test in CI (bounded)

Suggested gates:
- canonical hash compute < X ms for standard fixture
- model compute < Y ms for standard fixture
- submitter build tx < Z ms

---

## 9. Profiling Guidance

### 9.1 Rust profiling
- use `perf`/`flamegraph`
- instrument with `tracing` spans
- measure allocations with heap profilers

Focus areas:
- string allocations in hot paths
- JSON serialization overhead
- unnecessary copies in bundler and aggregator

### 9.2 RPC profiling
- measure RPC latencies and error codes
- evaluate fallback RPC endpoints
- monitor slot lag and blockhash refresh rates

---

## 10. Safe Optimizations

Only optimize in ways that preserve:
- determinism (especially hashing and encoding)
- correctness (schema validation)
- auditability (do not remove essential logs, but sample them)

Avoid:
- non-deterministic parallel reductions without stable ordering
- floating-point shortcuts that change results across platforms

---

## 11. Operational Sizing Example (Conceptual)

For 100 markets at NORMAL tier:
- ingestion: 4-8 connector workers
- aggregation: 4-8 workers
- quant: 4-16 workers depending on model complexity
- submitter: 2-4 workers (high reliability, idempotent)
- reconciler: 2 workers
- signer: N signers per set + load balancer

This is a conceptual starting point; actual sizing depends on workload and chain conditions.

---

## 12. Metrics for Performance Monitoring

Key dashboards:
- end-to-end latency distribution
- per-stage latency distribution
- queue depth over time
- event ingestion lag (watermark lag)
- publish success rate and retry counts
- signer latency and availability
- RPC latency and error rates
- CPU and memory utilization by service
- top allocations hot spots

Alerts:
- publish misses threshold for > N ticks
- watermark lag > max for > N minutes
- signer threshold misses
- RPC error rate spikes

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
