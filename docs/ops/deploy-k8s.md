
# Kubernetes Deployment Guide

This guide explains how to deploy M0Club (M0-CORE + services) on Kubernetes.
It covers recommended cluster requirements, namespace layout, secrets, images, migrations, rollouts, observability, and operational runbooks.

This document assumes the repository includes Kubernetes manifests under:
- `infrastructure/k8s/`
or Helm charts under:
- `infrastructure/helm/`

If your repo uses a different structure, adapt paths accordingly.

---

## 1. Goals

- Provide a production-ready deployment blueprint for M0Club on Kubernetes.
- Support environment separation (dev/staging/prod).
- Ensure secure secret handling and signer boundary isolation.
- Enable horizontal scaling and safe rollouts.
- Provide observability and runbook guidance.

Non-goals:
- Providing cloud-vendor-specific Terraform in this doc.
- Replacing security reviews or compliance requirements.

---

## 2. Cluster Requirements

### 2.1 Baseline sizing (reference)
For 100 markets (NORMAL tier) and moderate throughput:
- 3+ worker nodes for engine workloads (CPU heavy)
- 2+ worker nodes for services and data stores
- 8 vCPU / 32GB RAM per node recommended
- SSD-backed storage for databases

### 2.2 Required Kubernetes components
- Ingress controller (NGINX or similar)
- Cert manager (TLS)
- Metrics server
- Prometheus + Grafana (recommended)
- Loki or ELK for logs (recommended)
- External Secrets Operator (recommended)
- NetworkPolicy support (Calico/Cilium recommended)

### 2.3 External dependencies
M0Club typically uses:
- Postgres (feature store + metadata)
- Redis (cache)
- ClickHouse (optional analytics)
- NATS or Kafka (optional event log)
- Object storage (S3/GCS) for artifacts (calibration, reports)
- RPC endpoints to Solana (public or private)

Production guidance:
- manage databases with managed services when possible
- keep event log durable (Kafka/NATS) in production

---

## 3. Namespace and Environment Layout

Recommended namespaces:
- `m0-system` (shared operators and platform components)
- `m0-dev`
- `m0-staging`
- `m0-prod`

Each environment namespace contains:
- engine deployments
- submitter + reconciler
- api-gateway + realtime
- optional backtest runner (usually separate)
- secrets and configmaps
- services and ingress

---

## 4. Service Topology on Kubernetes

Recommended deployments (can be merged for smaller setups):

Engine plane:
- `m0-ingestor` (Deployment, HPA)
- `m0-aggregator` (Deployment, HPA)
- `m0-quant` (Deployment, HPA)
- `m0-bundler` (Deployment or part of quant)
Publishing plane:
- `m0-submitter` (Deployment, low replicas, strict idempotency)
- `m0-reconciler` (Deployment)
Signer plane:
- `m0-signer-agent` (Deployment or StatefulSet, isolated namespace/policy)
API plane:
- `m0-api-gateway` (Deployment, HPA)
- `m0-realtime` (Deployment, HPA)

Batch plane (optional):
- `m0-backtest-runner` (CronJob)

Data plane:
- Postgres, Redis, ClickHouse, event log (prefer managed services)

---

## 5. Images and Versioning

### 5.1 Image tags
Use immutable image digests or pinned tags:
- `ghcr.io/m0club/m0-engine:<git_sha>`
- `ghcr.io/m0club/api-gateway:<git_sha>`
- `ghcr.io/m0club/realtime:<git_sha>`
- `ghcr.io/m0club/signer-agent:<git_sha>`

Avoid using `latest` in production.

### 5.2 Config versioning
Store config as:
- ConfigMaps per environment
- versioned config files baked into images (optional)
- environment variables for overrides

Always record:
- git SHA
- image digest
- config hash

---

## 6. Secrets Management

### 6.1 Secret sources
Use one of:
- External Secrets Operator with AWS Secrets Manager / GCP Secret Manager
- Sealed Secrets
- native Secrets (not recommended for high security without encryption controls)

### 6.2 Secrets required
Typical secrets:
- database URLs and passwords
- Redis auth
- event log credentials
- signer agent keys or KMS references
- TLS certs (managed by cert-manager)
- API keys for external data sources
- RPC keys (if private endpoints require auth)

### 6.3 Signer keys
Signer keys must be isolated:
- do not store raw private keys in plain Kubernetes secrets in production
- prefer KMS/HSM-backed signing or hardware-backed services
- if key files are used, mount them as read-only volumes with tight RBAC

---

## 7. Network Policies

Apply strict NetworkPolicies:
- engine pods can reach:
  - internal event log
  - feature store DB
  - signer agent (only signer client -> signer agent)
  - Solana RPC endpoints
- signer agent must accept inbound only from authorized pods
- API pods accept inbound from ingress only
- deny all by default, then allow explicit flows

---

## 8. Storage and Databases

### 8.1 Postgres
Used for:
- feature store (hot window)
- metadata and registry cache
- publish state store

Production guidance:
- use managed Postgres
- enable backups and PITR
- ensure read replicas if needed

### 8.2 Redis
Used for:
- cache latest features and outputs
- rate limiting (optional)

### 8.3 ClickHouse (optional)
Used for:
- long-range analytics
- backtest range scans

### 8.4 Event log (Kafka/NATS)
Production guidance:
- use durable event log with retention controls
- partition by market_id
- set adequate replication factor

---

## 9. Kubernetes Manifests (Recommended Structure)

Suggested repo structure:
- `infrastructure/k8s/base/`
  - `namespace.yaml`
  - `serviceaccounts.yaml`
  - `rbac.yaml`
  - `networkpolicies.yaml`
- `infrastructure/k8s/apps/`
  - `engine/`
  - `api/`
  - `signer/`
- `infrastructure/k8s/overlays/`
  - `dev/`
  - `staging/`
  - `prod/`

Use Kustomize or Helm. Both are acceptable.
Recommended v1:
- Kustomize with overlays per environment.

---

## 10. Deploy Steps (Kustomize)

### 10.1 Create namespaces
```bash
kubectl apply -f infrastructure/k8s/base/namespace.yaml
```

### 10.2 Apply base RBAC and policies
```bash
kubectl apply -k infrastructure/k8s/base
```

### 10.3 Deploy external dependencies (if self-hosted)
```bash
kubectl apply -k infrastructure/k8s/deps
```

### 10.4 Deploy apps for environment
Dev:
```bash
kubectl apply -k infrastructure/k8s/overlays/dev
```

Staging:
```bash
kubectl apply -k infrastructure/k8s/overlays/staging
```

Prod:
```bash
kubectl apply -k infrastructure/k8s/overlays/prod
```

### 10.5 Verify rollout
```bash
kubectl -n m0-prod get pods
kubectl -n m0-prod rollout status deploy/m0-api-gateway
kubectl -n m0-prod rollout status deploy/m0-ingestor
```

---

## 11. Migrations and Bootstrapping

### 11.1 Database migrations
Run migrations as a Kubernetes Job or initContainer.

Recommended:
- a `m0-migrations` Job that runs on deploy or manually.

Example command:
```bash
pnpm db:migrate
```

Or Rust migrations if used:
```bash
cargo run -p m0-migrations -- up
```

### 11.2 Registry seeding
Seed markets and signer sets:
- run a `m0-seed` Job or a CLI command with service account

Example:
```bash
cargo run -p m0-cli -- registry seed --cluster mainnet --config /config/markets.toml
```

Store seed configs in ConfigMaps, not in images, so they can be changed without rebuild.

---

## 12. Rollout Strategy

### 12.1 Canary deployments
For modeling components:
- deploy canary quant pods at small percentage
- compare outputs and drift metrics
- promote if stable

For API:
- standard rolling updates with readiness probes

### 12.2 Rollback
Rollback is done by:
- reverting image tag/digest
- reverting configmaps
- redeploying overlay

Record rollback events in incident logs.

---

## 13. Autoscaling

Use HPA based on:
- CPU utilization
- custom metrics (queue depth, ingestion lag)

Scaling guidance:
- ingestion and quant are primary scale targets
- submitter should not scale blindly; keep small replica count and rely on idempotency

---

## 14. Health Checks

Each service must expose:
- `/healthz` (liveness)
- `/readyz` (readiness)
- `/metrics` (prometheus)

Readiness should verify:
- DB connection reachable
- event log connection reachable
- signer agent reachable (for submitter)
- RPC reachable (optional)

---

## 15. Observability

### 15.1 Metrics
Prometheus scraping:
- engine stage metrics
- publish success rate
- signer latency
- RPC errors

### 15.2 Logs
Centralized logs via Loki/ELK.
Ensure sensitive fields are redacted.

### 15.3 Tracing
OpenTelemetry collector per cluster.
Ensure trace ids propagate across:
- ingestion -> aggregation -> quant -> bundler -> submitter -> api

---

## 16. Security Hardening

- use PodSecurity standards (restricted)
- run as non-root
- set read-only root filesystem where possible
- disallow privilege escalation
- limit capabilities
- mount secrets read-only
- set resource requests/limits
- use dedicated service accounts per component
- apply least privilege RBAC

Signer hardening:
- isolate signer in separate namespace
- deny all egress except required
- allow ingress only from signer client pods
- consider using node pools with taints/tolerations for signer pods

---

## 17. Runbooks

### 17.1 Engine not publishing
Check:
- ingestion lag and coverage metrics
- signer availability and thresholds
- RPC health and tx confirmations
- market paused flags

Actions:
- scale ingestion/quant if backlog
- switch RPC endpoint
- rotate signer set if signer outage
- pause affected markets if integrity issues

### 17.2 High divergence anomalies
Check:
- which sources diverged
- connector health and recent releases
- quarantine events

Actions:
- degrade publishing (risk bump) or block
- disable degraded connector
- investigate source manipulation

### 17.3 Database saturation
Check:
- DB CPU/IO
- slow queries
- missing indexes

Actions:
- add indexes for hot queries
- scale managed DB
- increase cache hit rate
- move long history to ClickHouse or object storage

---

## 18. Example Values (Environment Variables)

Typical env vars:
- `M0_ENV`
- `SOLANA_RPC_URL`
- `SOLANA_WS_URL`
- `M0_POSTGRES_URL`
- `M0_REDIS_URL`
- `M0_EVENTLOG_URL`
- `M0_SIGNER_ENDPOINT`
- `M0_SIGNER_SET_ID`
- `M0_PUBLISH_CONCURRENCY`
- `M0_MARKET_TIER_POLICY`

Prefer using ConfigMaps for non-secret values and Secrets for credentials.

---

## 19. Verification Checklist

Before enabling production publishing:
- programs deployed and verified
- registry seeded and signer set correct
- signer agent secured and reachable only by authorized pods
- commit/reveal tested in staging
- dashboards and alerts configured
- backups and disaster recovery configured
- incident runbooks accessible

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
