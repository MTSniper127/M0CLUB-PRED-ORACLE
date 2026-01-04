![1500x500](https://github.com/user-attachments/assets/46aba31e-c954-475f-acc1-ba13180962b8)

<div align="center">

# M0Club — Omni-Domain Predictive Oracle on Solana

**Solana prediction oracle via a proprietary engine.**  
**Real-time analytics for on-chain & real-world events.**

[![Website](https://img.shields.io/badge/Website-m0club.com-000000?style=for-the-badge)](https://m0club.com/)
[![X](https://img.shields.io/badge/X-@M0Clubonx-000000?style=for-the-badge&logo=x)](https://x.com/M0Clubonx)
[![Docs](https://img.shields.io/badge/Docs-Overview-000000?style=for-the-badge)](./docs/overview.md)
[![Architecture](https://img.shields.io/badge/Docs-Architecture-000000?style=for-the-badge)](./docs/architecture.md)
[![Security](https://img.shields.io/badge/Security-Policy-000000?style=for-the-badge)](./SECURITY.md)

</div>

---

## Token + Trading Links

- **Token CA (Solana):** `TBA`
- **Pump.fun Link:** `TBA`
- **DEXScreener Link:** `TBA`

---

## Table of Contents

- [What is M0Club?](#what-is-m0club)
- [Core Principles](#core-principles)
- [What M0Club Produces](#what-m0club-produces)
- [System Overview](#system-overview)
  - [Repository Layout](#repository-layout)
  - [Data-to-Oracle Pipeline](#data-to-oracle-pipeline)
  - [Commit-Reveal Oracle Flow](#commit-reveal-oracle-flow)
  - [Signer Sets and Rotation](#signer-sets-and-rotation)
  - [Replay Protection](#replay-protection)
  - [Disputes and Slashing](#disputes-and-slashing)
- [Components](#components)
  - [On-Chain Programs](#on-chain-programs)
  - [Core Engine](#core-engine)
  - [Services](#services)
  - [SDKs](#sdks)
  - [Infrastructure](#infrastructure)
- [Quickstart](#quickstart)
  - [Prerequisites](#prerequisites)
  - [Start Local Stack](#start-local-stack)
  - [Verify the Stack](#verify-the-stack)
  - [Run Tests](#run-tests)
- [Build and Development](#build-and-development)
  - [Build All](#build-all)
  - [Lint and Format](#lint-and-format)
  - [Generate IDLs and Types](#generate-idls-and-types)
  - [Configuration](#configuration)
- [API](#api)
  - [Authentication](#authentication)
  - [Endpoints](#endpoints)
  - [OpenAPI](#openapi)
- [SDK Usage](#sdk-usage)
  - [TypeScript](#typescript)
  - [Rust](#rust)
  - [Python](#python)
- [Operations](#operations)
  - [Key Management](#key-management)
  - [Signer Rotation Runbook](#signer-rotation-runbook)
  - [Incident Response](#incident-response)
  - [Migrations](#migrations)
- [Observability](#observability)
- [Security](#security)
- [Contributing](#contributing)
- [License](#license)
- [Official Links](#official-links)
- [Publishing Checklist](#publishing-checklist)

---

## What is M0Club?

M0Club is a next-generation **omni-domain predictive oracle** built on Solana.

**Core idea:** *The world doesn't just happen. It's calculated.*

M0Club goes beyond price feeds. It models high-signal events across multiple domains and publishes:
- probability distributions
- confidence intervals
- integrity proofs (commit-reveal + bundle hashing)
- on-chain verifiable output bundles

Domains include:
- **POLITICS** — elections, geopolitical risk, governance outcomes
- **SPORTS** — elite leagues, odds dynamics, win-rate distributions
- **MARKETS** — macro indicators, cross-market regimes, crypto + tradfi interactions

---

## Core Principles

- **Integrity by Design**: commit-reveal + signer sets + replay protection.
- **Omni-Domain**: same oracle interface for on-chain markets and real-world events.
- **Quantified Uncertainty**: confidence intervals and calibrated probabilities, not vague signals.
- **Deterministic Outputs**: canonical normalization + bundle hashing for reproducibility.
- **Operational Readiness**: deployable with docker-compose for local dev and Kubernetes for production.

---

## What M0Club Produces

M0Club produces **Oracle Output Bundles** that are:
- content-addressed (hash)
- optionally Merkleized (for partial verification)
- signed by the active signer set
- posted on-chain with epoch metadata and anti-replay constraints

See: `docs/protocol-spec/oracle-output-format.md`

Example output shape (conceptual):
```json
{
  "market_id": "SPORTS_NBA_LAL_BOS",
  "epoch_id": 1842,
  "timestamp": 1730000000,
  "outcomes": {
    "LAL": { "p": 0.51, "ci": [0.48, 0.54] },
    "BOS": { "p": 0.49, "ci": [0.46, 0.52] }
  },
  "bundle_hash": "0x...",
  "signatures": ["..."]
}
```

---

## System Overview

### Repository Layout

```text
m0club/
  .github/
  config/
  scripts/
  programs/          # On-chain Anchor programs
  core-engine/       # Engine monorepo (ingest/normalize/quant/bundle/signer/runtime)
  services/          # api-gateway, indexer, realtime, dashboard, jobs
  sdk/               # TS/Rust/Python SDKs + shared types
  infra/             # docker, k8s/helm, terraform, monitoring
  docs/              # specs, ops, architecture
  tests/             # integration, load (k6), fuzz
```

### Data-to-Oracle Pipeline

High-level pipeline:

```text
[ Connectors ] -> [ Normalization ] -> [ Feature Store ] -> [ Models ] -> [ Calibration ]
        |                 |                 |                |               |
        v                 v                 v                v               v
   raw events        canonical events   feature vectors   distributions   confidence intervals
        \___________________________________________________________________________________/
                                         |
                                         v
                                  [ Bundle + Hash ]
                                         |
                                         v
                                  [ Sign + Submit ]
                                         |
                                         v
                              [ On-chain Oracle Program ]
```

Engine spec references:
- `docs/engine-spec/data-ingestion.md`
- `docs/engine-spec/normalization.md`
- `docs/engine-spec/feature-store.md`
- `docs/engine-spec/models-bayes.md`
- `docs/engine-spec/calibration.md`
- `docs/engine-spec/confidence-intervals.md`
- `docs/engine-spec/bundle-hashing.md`
- `docs/engine-spec/performance.md`

### Commit-Reveal Oracle Flow

The on-chain oracle supports a commit-reveal publication pattern:
- **Commit**: signers commit to a hash of payload + salt for an epoch
- **Reveal**: signers reveal payload + salt, validated against commits
- **Finalize**: protocol aggregates revealed payloads into a final bundle

Spec: `docs/protocol-spec/commit-reveal.md`

### Signer Sets and Rotation

Signer sets provide the authority layer for publishing bundles and managing updates.
Rotation is designed to be auditable and operationally safe.

Spec: `docs/protocol-spec/signer-set.md`  
Ops: `docs/ops/signer-rotation.md`

### Replay Protection

Signed payloads and bundle submissions include replay protection to prevent reuse across epochs or markets.

Spec: `docs/protocol-spec/replay-protection.md`

### Disputes and Slashing

Dispute resolution and slashing parameters are intended to deter manipulation and enforce signer behavior.

Specs:
- `docs/protocol-spec/dispute-resolution.md`
- `docs/protocol-spec/slashing.md`

---

## Components

### On-Chain Programs

Primary programs (Anchor):
- `programs/m0-oracle` — core oracle protocol (epochs, commit-reveal, publish)
- `programs/m0-registry` — market registry + metadata
- `programs/m0-fee-router` — fee routing primitives
- `programs/m0-governance` — timelock/governor scaffolding (optional)

Developer focus areas:
- deterministic account layouts
- explicit invariants
- strict input validation
- anti-front-running patterns

### Core Engine

`core-engine/` is a Rust workspace containing independent crates:
- `m0-ingestor` connectors (solana, sports, politics, macro, webhooks)
- `m0-normalizer` canonicalization and validation
- `m0-feature-store` feature transforms + storage adapters
- `m0-quant` models (elo/poisson/garch/hmm/ensemble) + bayes inference + scoring
- `m0-anomaly` drift/outlier/feed-integrity detection
- `m0-bundle` bundle format + hashing + Merkle
- `m0-signer` keyring + commit/reveal + tx submission + replay protection
- `m0-core` pipeline runtime and scheduling

Binaries:
- `m0d` main orchestrator daemon
- `m0-ingestd` dedicated ingestion worker
- `m0-backtestd` backtest runner
- `m0-signer-agent` signer agent for commits/reveals

### Services

`services/` provides:
- `api-gateway` REST endpoints + OpenAPI + auth
- `realtime` websocket realtime feeds
- `indexer` reads program events and persists materialized views
- `jobs` maintenance tasks (backfill/recompute/reconcile/cleanup)
- `dashboard` Next.js interface

### SDKs

`sdk/` includes multi-language client libraries:
- `sdk/ts` TypeScript SDK
- `sdk/rust` Rust SDK
- `sdk/python` Python SDK
- `sdk/types` shared types across languages

### Infrastructure

`infra/` includes:
- docker-compose dev/staging/prod
- Kubernetes manifests + Helm chart scaffold
- Terraform scaffolding
- monitoring bootstrap (Prometheus/Grafana/Loki/alerts)

---

## Quickstart

### Prerequisites

Minimum for local stack:
- Docker + Docker Compose
- Rust (pinned by `rust-toolchain.toml`)
- Node.js 20+ (dashboard + tests)
- Python 3.10+ (optional)

If building programs:
- Solana + Anchor toolchains (installed separately)

### Start Local Stack

```bash
cd infra/docker
docker compose -f compose.dev.yml up --build
```

### Verify the Stack

Health:
```bash
curl -s http://localhost:8080/health
```

Markets:
```bash
curl -s http://localhost:8080/markets | head
```

### Run Tests

Integration tests:
```bash
cd tests
npm install
npm test
```

Optional k6 smoke:
```bash
cd tests
npm run k6:smoke
```

---

## Build and Development

### Build All

If available:
```bash
./scripts/build_all.sh
```

Manual builds:

Engine:
```bash
cd core-engine
cargo build
```

Services:
```bash
cd services
cargo build
```

Rust SDK:
```bash
cd sdk/rust
cargo build
```

TypeScript SDK:
```bash
cd sdk/ts
npm install
npm run build
```

### Lint and Format

Rust:
```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

TypeScript:
```bash
cd sdk/ts && npm run lint || true
cd tests && npm run lint || true
cd services/dashboard && npm run lint || true
```

### Generate IDLs and Types

Anchor IDLs are generated per-program:
```bash
./scripts/gen_idl.sh
```

Shared types are maintained in `sdk/types/` and should remain stable.

### Configuration

Config directory:
- `config/dev.toml`
- `config/staging.toml`
- `config/prod.toml`

Domain catalogs:
- `config/markets/politics.toml`
- `config/markets/sports.toml`
- `config/markets/macro.toml`
- `config/markets/crypto.toml`

Risk + guardrails:
- `config/risk/thresholds.toml`
- `config/risk/anomaly-rules.toml`
- `config/risk/slashing-params.toml`

Telemetry:
- `config/telemetry/otel.toml`
- `config/telemetry/prometheus.yml`

---

## API

### Authentication

The gateway supports API key and JWT auth scaffolds (implementation may be configured per deployment):
- API keys: `M0_API_KEYS` env var (comma-separated)
- JWT: `M0_JWT_SECRET` env var (optional)

### Endpoints

Expected baseline endpoints (local dev):
- `GET /health`
- `GET /markets`
- `GET /epochs`
- `GET /predictions/:market_id/latest`
- `GET /metrics` (optional Prometheus)

### OpenAPI

The OpenAPI spec is generated from `services/api-gateway/src/openapi/spec.rs`.
When enabled, it is typically exposed at:
- `/openapi.json` (implementation-dependent)

---

## SDK Usage

### TypeScript

```bash
cd sdk/ts
npm install
npm run build
```

Example (conceptual):
```ts
import { M0Client } from "@m0club/sdk";

const client = new M0Client({ baseUrl: "http://localhost:8080" });
const markets = await client.markets.list();
const latest = await client.predictions.latest(markets[0].market_id);
console.log(latest);
```

### Rust

```bash
cd sdk/rust
cargo test
```

### Python

```bash
cd sdk/python
python -m pip install -e .
pytest -q
```

---

## Operations

### Key Management

Read: `docs/ops/key-management.md`

Principles:
- separate keys per environment
- never commit key material
- prefer KMS/HSM for production
- enforce rotation and audit trails

### Signer Rotation Runbook

Read: `docs/ops/signer-rotation.md`

### Incident Response

Read: `docs/ops/incident-response.md`

### Migrations

Read: `docs/ops/migrations.md`

---

## Observability

The stack supports:
- logs (structured via `tracing`)
- metrics (Prometheus)
- traces (OpenTelemetry, optional)
- dashboards (Grafana)
- alerts (Prometheus/Grafana rules)

See: `infra/monitoring/` and `config/telemetry/`

---

## Security

- Report vulnerabilities to: **security@m0club.com**
- Policy: `SECURITY.md`
- Prefer coordinated disclosure.
- Never disclose sensitive details in public issues.

---

## Contributing

See: `CONTRIBUTING.md`  
Code of Conduct: `CODE_OF_CONDUCT.md`

---

## License

Apache 2.0 — see `LICENSE.md`.

---

## Official Links

- Website: https://m0club.com/
- X: https://x.com/M0Clubonx

---


