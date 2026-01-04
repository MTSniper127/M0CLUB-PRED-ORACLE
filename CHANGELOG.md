
# Changelog

All notable changes to this project will be documented in this file.

This project adheres to **Keep a Changelog** and **Semantic Versioning**.

- Keep a Changelog: https://keepachangelog.com/en/1.1.0/
- SemVer: https://semver.org/spec/v2.0.0.html

## Conventions

### Versioning
- **MAJOR**: incompatible changes to public APIs, protocol state layouts, or wire formats.
- **MINOR**: backward-compatible features and improvements.
- **PATCH**: backward-compatible bug fixes and security patches.
- **PRE-RELEASE** (`-alpha.N`, `-beta.N`, `-rc.N`): unstable versions for testnets/staging.

### Component tags
Entries are grouped by component where useful:
- **programs**: on-chain Anchor programs (m0-oracle, m0-registry, m0-fee-router, m0-governance)
- **engine**: core-engine monorepo (ingestion, normalization, feature-store, quant, anomaly, bundle, signer, runtime)
- **services**: api-gateway, realtime, indexer, dashboard, jobs
- **sdk**: TypeScript, Rust, Python SDKs + shared types
- **infra**: docker, k8s/helm, terraform scaffolds, monitoring
- **docs**: specifications, runbooks, integration guides
- **security**: audits, hardening, dependency updates
- **ci**: workflows, release automation

### Change types
- **Added** for new features.
- **Changed** for changes in existing functionality.
- **Deprecated** for soon-to-be removed features.
- **Removed** for now removed features.
- **Fixed** for any bug fixes.
- **Security** in case of vulnerabilities.

---

## [Unreleased]

### Added
- programs: commit-reveal oracle flow with epoch lifecycle (open → commit → reveal → finalize).
- programs: signer set rotation and governance hooks for admin/timelock actions.
- engine: modular pipeline stages (ingest → normalize → feature → model → calibrate → backtest → bundle).
- services: API Gateway endpoints for markets/epochs/predictions plus OpenAPI generation.
- services: websocket realtime fanout layer with throttling + pubsub abstraction.
- sdk: shared type definitions (JSON schema + TS/Rust/Python) and clients.
- infra: docker-compose dev/staging/prod, K8s manifests, Helm chart scaffold.
- tests: integration tests (E2E read paths), k6 smoke, fuzz scaffolds.

### Changed
- docs: consolidated protocol specification for oracle outputs, market registry, replay protection, dispute resolution.
- infra: standardized image naming and chart values across environments.
- ci: separated workflows by component (programs, engine, services, sdk, docker, security, release).

### Fixed
- services: request id propagation and structured error envelopes for gateway routes.
- engine: backoff/retry defaults for ingestion connectors (scaffold).
- sdk: consistent base URL + bearer token handling.

### Security
- security: dependency pinning for Rust toolchain, CI hardened permissions (scaffold).
- security: placeholder policies for secret management and signer key rotation.

---

## [0.1.0] - 2026-01-04

### Added
- Repository bootstrap with monorepo layout:
  - `programs/` Anchor programs
  - `core-engine/` high-throughput modeling engine
  - `services/` API Gateway, realtime, indexer, dashboard, jobs
  - `sdk/` multi-language SDKs + shared types
  - `infra/` docker/k8s/terraform/monitoring scaffolds
  - `tests/` integration/load/fuzz suites
  - `docs/` specs, ops, and architecture documentation
- Initial developer experience:
  - docker-compose local environment
  - scripts for build/lint/test/deploy (where applicable)
  - baseline docs for local dev, k8s deployment, and incident response

### Notes
- This release is intended as a runnable baseline for local development and CI validation.
- Production-grade configuration (cloud resources, secret management, full observability) is expected to be finalized in subsequent MINOR releases.

---

## Links
- Website: https://m0club.com/
- X: https://x.com/M0Clubonx
