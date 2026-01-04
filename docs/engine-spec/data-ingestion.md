
# Data Ingestion Specification (M0-CORE)

This document specifies the data ingestion subsystem for M0Club (M0-CORE).
Ingestion is responsible for collecting multi-source signals (on-chain and real-world), validating them, normalizing them into deterministic envelopes, and emitting them into the engine pipeline.

This spec defines:
- connector model and interfaces
- normalized event envelope
- deduplication and idempotency
- partitioning and routing
- quality flags and anomaly checks
- storage/event-log backends
- operational and security requirements

---

## 1. Goals

- Ingest high-volume, multi-source signals in real time.
- Normalize events into a stable schema with strict versioning.
- Provide deterministic idempotency and replay support.
- Surface data quality metrics and anomaly flags.
- Support local, staging, and production backends.

Non-goals:
- Storing all raw proprietary data in public infra.
- Performing final model computation (handled by aggregation/modeling).

---

## 2. Ingestion Architecture

### 2.1 Logical pipeline
1) Source poll/stream
2) Parse and validate
3) Normalize into `NormalizedEvent`
4) Deduplicate/idempotency checks
5) Enrich with metadata (ingest_time, connector version, quality flags)
6) Publish to event log
7) Update watermarks and metrics

### 2.2 Worker topology
Recommended worker roles:
- connector workers (one per source type, horizontally scalable)
- router/partitioner (optional; can be in-connector)
- event-log writers
- backfill workers (batch mode)

Workers can be merged in local profile; in production they should be independent deployments.

---

## 3. Connector Model

### 3.1 Connector interface (conceptual)
Connectors implement a standardized interface:

- `id(): string`
- `version(): string`
- `capabilities(): ConnectorCaps`
- `subscribe(params) -> EventStream` OR `poll(params) -> Vec<RawEvent>`
- `normalize(raw) -> NormalizedEvent`
- `health() -> ConnectorHealth`

### 3.2 Connector capabilities
- streaming vs polling
- expected latency profile
- rate limits
- supported domains and market mapping

### 3.3 Connector types
Connectors should be grouped by source family:

On-chain:
- Solana program log connectors
- token price feeds (DEX, AMM, oracle feeds)
- governance proposal feeds
- on-chain volatility indicators

Off-chain:
- sports odds feeds
- election/forecast feeds
- macro economic releases
- news headline aggregation (optional)
- exchange rates and rates markets

Local/testing:
- fixture connector (replay from file)
- synthetic connector (generate deterministic events)

---

## 4. Normalized Event Envelope

Ingestion output is the canonical `NormalizedEvent` envelope. All downstream stages consume this envelope.

### 4.1 NormalizedEvent fields (logical)

Required:
- `schema_version: u16`
- `event_id: [u8; 32]`
- `source_id: string`
- `source_version: string`
- `connector_id: string`
- `connector_version: string`
- `market_id: string`
- `domain: enum`
- `event_time_ms: u64`
- `ingest_time_ms: u64`
- `payload_type: enum`
- `payload: bytes` (typed encoding, see 4.2)
- `quality_flags: u32`
- `partition_key: string` (optional but recommended)
- `trace_id: string` (optional)

Optional:
- `source_seq: string` (source-native sequence id)
- `source_uri: string` (reference link)
- `confidence: u16` (0..10000, input-level confidence score)

### 4.2 Payload encoding
Payload SHOULD be encoded deterministically.
Supported options:
- a fixed binary layout per payload_type
- protobuf with strict canonicalization
- JSON ONLY for debug/test connectors

Recommended v1:
- use a custom binary layout for each payload_type and version it.

### 4.3 Payload types (examples)
- `ODDS_SNAPSHOT_V1`
- `ODDS_TICK_V1`
- `ELECTION_POLL_V1`
- `MACRO_RELEASE_V1`
- `ONCHAIN_PRICE_TICK_V1`
- `ONCHAIN_ACTIVITY_V1`
- `NEWS_SIGNAL_V1` (optional)

---

## 5. Idempotency and Deduplication

### 5.1 event_id derivation
`event_id` must be stable for the same logical raw event.

Recommended:
`event_id = sha256(source_id || market_id || event_time_ms || canonical_raw_payload_bytes)`

If the source provides a stable unique id:
- include it in derivation to reduce collision risk:
`event_id = sha256(source_id || source_native_id || canonical_payload_bytes)`

### 5.2 Deduplication store
Ingestion must prevent duplicates from entering the log.
Options:
- in-memory LRU for short windows
- Redis / RocksDB for durable dedup
- Postgres unique constraints on event_id

Recommended:
- in-memory LRU + durable store in production.
- file-based store in local mode.

### 5.3 Idempotent publishing
Event-log writes should be idempotent:
- event_id is the primary key
- replays should not create duplicates

---

## 6. Partitioning and Routing

### 6.1 Partition key
Partition key is used to route events deterministically to workers.

Recommended:
- `partition_key = market_id` for most workloads
- for very hot markets, use `market_id + shard` where shard is stable from event_id prefix

### 6.2 Market mapping
Connectors must map raw signals into a MarketId.
Mapping is configured by:
- registry market definitions
- connector-specific mapping tables
- fallback rules for unknown markets (drop or route to quarantine)

### 6.3 Quarantine routing
If events cannot be mapped or validated:
- route to quarantine topic
- record reason code
- do not feed into modeling pipeline

---

## 7. Validation and Quality Flags

### 7.1 Validation stages
Each event passes:
1) schema validation (fields, types, ranges)
2) time validation (event_time bounds)
3) source sanity checks (odds sum, values, etc.)
4) anomaly detection (outliers vs rolling stats)
5) consistency checks across sources (divergence)

### 7.2 Quality flags bitmask (suggested)
- `0x0001` INVALID_SCHEMA
- `0x0002` STALE_EVENT_TIME
- `0x0004` FUTURE_EVENT_TIME
- `0x0008` DUPLICATE_EVENT
- `0x0010` SOURCE_DIVERGENCE
- `0x0020` OUTLIER_DETECTED
- `0x0040` PARTIAL_COVERAGE
- `0x0080` CONNECTOR_DEGRADED
- `0x0100` QUARANTINED

Flags can be combined. Downstream components must propagate and augment flags.

### 7.3 Time bounds
Registry should define:
- `max_event_time_skew_ms`
- `max_staleness_ms`

Default guidance:
- accept slightly future timestamps (e.g., 5s) for clock skew
- drop extremely stale events for real-time markets
- allow larger staleness for slow domains (politics/macro)

---

## 8. Watermarks and Backfill

### 8.1 Watermark tracking
Maintain per partition:
- `watermark_event_time_ms`
- `watermark_ingest_time_ms`
- `last_event_id` (optional)

Watermarks are used to:
- compute ingestion lag
- bound allowed lateness
- schedule backfill jobs

### 8.2 Backfill workflow
Backfill workers:
- fetch historical data for a time interval
- normalize into events using the same envelope
- publish into event log with stable event_id
- update watermarks only if backfill is allowed to advance them (policy)

Backfill must be labeled:
- set a flag or tag indicating backfill source
- ensure modeling can optionally treat backfill differently

---

## 9. Event Log Backends

Ingestion writes normalized events to an event log to enable replay and decoupling.

Supported backends:
- NATS JetStream
- Kafka
- File-based append log (local/dev)
- Postgres (append table with indexes)

Backend requirements:
- ordered consumption per partition
- at-least-once delivery (with dedup downstream)
- retention controls
- replay from offsets

---

## 10. Storage and Schema Registry

### 10.1 Schema registry
Maintain a schema registry for:
- NormalizedEvent schema versions
- payload_type layouts and versions
- connector versions

Registry can be:
- code-defined enums + versioned binary layouts
- or a runtime schema registry service

Recommended v1:
- code-defined layouts with strict tests and vectors

### 10.2 Persistent storage
Ingestion should persist:
- raw metadata needed for audits
- quarantine events
- connector health stats
- mapping tables and updates

ClickHouse and Postgres are common choices.

---

## 11. Security and Access Control

### 11.1 External sources
- do not embed API keys in binaries
- use secret managers
- rotate keys regularly

### 11.2 Network controls
- restrict outbound egress from ingestion workers
- allow only required endpoints
- enforce mTLS for internal event-log writers

### 11.3 Input sanitization
- treat all external data as untrusted
- enforce strict parsing and range checks
- do not log raw secrets or full payloads unless explicitly enabled

---

## 12. Observability

Ingestion must expose metrics:
- events_ingested_total (by connector, payload_type)
- events_quarantined_total (by reason)
- dedup_hits_total
- ingest_lag_ms histogram
- connector_errors_total
- connector_poll_latency_ms histogram
- watermark_event_time_ms gauge per partition
- source_divergence_count

Logs should be structured and include:
- connector_id/version
- market_id
- event_id
- flags and reason codes

Traces should link:
- connector request -> normalize -> publish

---

## 13. Determinism Requirements

- event_id derivation MUST be stable across re-ingestion.
- payload encoding MUST be deterministic for hashed fields.
- mapping tables MUST be versioned.
- late/out-of-order events MUST not reorder committed aggregates outside allowed policy.

---

## 14. Local Development Mode

Local ingestion should support:
- fixture replay from `fixtures/events/*.jsonl` or binary logs
- deterministic synthetic generator
- file-based append log

Local config should allow:
- turning on debug payload logging
- reduced rate limits
- deterministic clock offsets for testing

---

## 15. Test Plan

### 15.1 Unit tests
- event_id stability tests
- schema validation and flagging tests
- mapping rules tests
- dedup store correctness tests

### 15.2 Integration tests
- end-to-end: connector -> event log -> consumer
- backfill replay produces identical event ids
- quarantine routing correctness

### 15.3 Load tests
- sustained ingest at target throughput
- latency distribution under load
- dedup performance and memory growth controls

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
