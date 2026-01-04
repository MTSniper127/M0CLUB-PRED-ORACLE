
# Operational Runbooks (Ops)

This document contains operational runbooks for M0Club (M0-CORE).
Runbooks are step-by-step procedures for common operational events, failures, and maintenance tasks.

This document assumes:
- Kubernetes deployment (see deploy-k8s.md)
- monitoring (Prometheus/Grafana, logs, tracing)
- Solana publish pipeline using commit-reveal
- signer agents providing threshold signatures
- Postgres for feature store/metadata, optional event log

All steps are written to be safe and reversible when possible.

---

## 0. Quick Reference

Primary services (typical):
- m0-ingestor
- m0-aggregator
- m0-quant
- m0-submitter
- m0-reconciler
- m0-signer-agent
- m0-api-gateway
- m0-realtime

Core control toggles (typical env vars / config keys):
- `M0_PUBLISH_ENABLED` (true/false)
- `M0_ACTIVE_SIGNER_SET_ID`
- `SOLANA_RPC_URL`
- `M0_MARKET_PAUSE_*` (registry-level pause mechanism)
- `M0_GUARDRAIL_POLICY_LEVEL` (STRICT/NORMAL/FAST)
- `M0_PUBLISH_CONCURRENCY`
- `M0_EVENTLOG_URL`
- `M0_POSTGRES_URL`

---

## 1. Runbook: Publishing Stops (No New On-Chain Updates)

### Symptoms
- API shows stale outputs
- publish success rate drops
- commit/reveal tx count flatlines

### Triage
1) Check submitter pods:
```bash
kubectl -n m0-prod get pods | grep submitter
kubectl -n m0-prod logs deploy/m0-submitter --tail=200
```
2) Check signer threshold:
- metrics: threshold misses, signer latency
3) Check RPC health:
- RPC error rate
- confirmation latency
4) Check guardrail blocks:
- guardrail BLOCK rate spike

### Mitigation
A) RPC issue
- switch RPC endpoint:
```bash
kubectl -n m0-prod set env deploy/m0-submitter SOLANA_RPC_URL=https://backup-rpc.example.com
kubectl -n m0-prod rollout status deploy/m0-submitter
```

B) Signer issue
- restart signer agents:
```bash
kubectl -n m0-prod rollout restart deploy/m0-signer-agent
```

C) Guardrails blocking
- identify cause in logs (coverage/divergence/drift)
- if safe, temporarily relax policy (avoid unsafe publishing):
```bash
kubectl -n m0-prod set env deploy/m0-quant M0_GUARDRAIL_POLICY_LEVEL=NORMAL
```

### Validation
- commit tx count increases
- API shows new epoch/tick
- no invalid signature errors

---

## 2. Runbook: Invalid Signature Errors

### Symptoms
- on-chain program rejects reveal due to signature verification
- submitter logs show invalid signature errors
- consumers report bundle verification failures

### Severity
Treat as SEV0 until proven otherwise.

### Immediate Action
1) Disable publishing:
```bash
kubectl -n m0-prod scale deploy/m0-submitter --replicas=0
```

2) Capture evidence:
- last bundle_content_hash
- signer_set_id
- list of signer pubkeys included
- failing tx signatures
- logs and trace ids

### Root Cause Checks
- signer_set registry pubkeys mismatch with signer agents
- canonical hashing schema mismatch between submitter and signer agents
- signer agent returning signatures for wrong message format
- replay protection/sequence mismatch included in signed message

### Mitigation
- rollback submitter and signer agent to last known good images
- verify active signer_set_id and pubkeys on-chain
- if compromise suspected, rotate signer set (see signer-rotation.md)

### Validation
- run a local verification step using SDK verifier
- re-enable publishing with canary markets only

---

## 3. Runbook: Signer Threshold Misses

### Symptoms
- submitter cannot collect enough signatures
- signer latency spikes
- threshold misses increase

### Triage
1) Check signer pod health:
```bash
kubectl -n m0-prod get pods | grep signer
kubectl -n m0-prod logs deploy/m0-signer-agent --tail=200
```

2) Check network policies:
- submitter -> signer connectivity

3) Check KMS permissions (if used):
- KMS access errors

### Mitigation
- restart signer agents
- scale signer agents if they are stateless and allowed
- reduce publish cadence or concurrency:
```bash
kubectl -n m0-prod set env deploy/m0-submitter M0_PUBLISH_CONCURRENCY=2
```

- if persistent outage, rotate signer set to a healthy set

### Validation
- threshold miss rate returns to baseline
- signature collection p95 normal

---

## 4. Runbook: Commit Succeeds, Reveal Fails

### Symptoms
- commit tx confirmed
- reveal tx fails or times out repeatedly

### Likely Causes
- bundle bytes too large (tx size/compute)
- inconsistent bundle_hash
- blockhash expiration due to latency
- RPC instability

### Mitigation
- enable merkle/chunk reveal mode (if supported)
- reduce bundle size by lowering market count per bundle
- increase priority fees or compute budget
- shorten commit->reveal interval and retry with fresh blockhash

### Validation
- reveals confirm after commits
- no replay protection violations

---

## 5. Runbook: Replay Protection Errors

### Symptoms
- program rejects due to sequence mismatch
- duplicate sequence used
- submitter logs show out-of-order publish

### Likely Causes
- multiple submitters publishing concurrently without leader election
- state store out of sync
- idempotency bug after restart

### Mitigation
1) Ensure single submitter leader:
```bash
kubectl -n m0-prod scale deploy/m0-submitter --replicas=1
```

2) Reconcile publish state:
- run reconciler job or CLI to determine last committed epoch/sequence
- update state store carefully (prefer automated reconciliation)

3) Restart submitter after reconciliation:
```bash
kubectl -n m0-prod rollout restart deploy/m0-submitter
```

### Validation
- new publishes accepted without sequence errors
- no duplicate commits

---

## 6. Runbook: Data Ingestion Lag / Watermark Lag

### Symptoms
- watermark_lag_ms exceeds threshold
- coverage ratio drops
- outputs become stale or degraded

### Triage
- identify connector causing lag
- check external API rate limits
- check event log backlog
- check ingestion worker saturation

### Mitigation
- scale ingestor workers:
```bash
kubectl -n m0-prod scale deploy/m0-ingestor --replicas=8
```

- apply rate limits or reduce polling frequency
- disable failing connector temporarily
- increase cache TTL if API rate limits

### Validation
- watermark lag returns to baseline
- coverage ratio stable

---

## 7. Runbook: Divergence/Outlier Alerts

### Symptoms
- divergence_warn/block spikes
- outlier events increasing
- guardrails degrade/block

### Triage
- identify which source(s) diverged
- compare implied probabilities from sources
- check for schema changes by provider

### Mitigation
- quarantine/disable offending connector
- adjust expected sources temporarily
- pause impacted markets if unsafe

### Validation
- divergence metrics normalize
- guardrails no longer block

---

## 8. Runbook: Model Drift Alerts

### Symptoms
- drift_score crosses threshold
- performance metrics regress in live evaluation
- increased risk scores

### Triage
- confirm drift is real vs data outage
- inspect feature distribution shifts
- inspect calibration health (ECE)

### Mitigation
- increase risk score and widen CIs (degrade)
- trigger recalibration job
- rollback model version if regression confirmed
- pause high-impact markets if uncertain

### Validation
- drift_score returns below threshold after mitigation
- calibration metrics improve after retrain

---

## 9. Runbook: API Errors (5xx) / Serving Outage

### Symptoms
- api_5xx_rate spike
- clients fail to fetch latest outputs

### Triage
- check api-gateway pods and logs:
```bash
kubectl -n m0-prod get pods | grep api-gateway
kubectl -n m0-prod logs deploy/m0-api-gateway --tail=200
```

- verify DB connectivity
- check cache layer health

### Mitigation
- restart api-gateway
- scale api-gateway replicas:
```bash
kubectl -n m0-prod scale deploy/m0-api-gateway --replicas=6
```

- enable cache-only mode if supported
- failover DB if down

### Validation
- 5xx returns to baseline
- latency normal

---

## 10. Runbook: Postgres Saturation / Slow Queries

### Symptoms
- DB CPU high
- query latency spikes
- timeouts in services

### Triage
- identify top queries (pg_stat_statements)
- check missing indexes
- check vacuum/analyze status

### Mitigation
- scale DB instance or add read replicas
- add indexes (prefer concurrent index creation)
- reduce write volume temporarily (decrease publish cadence)
- move heavy analytics queries to ClickHouse or offline

### Validation
- query latency improves
- service error rates drop

---

## 11. Runbook: Rolling Back a Release

### When to rollback
- new release correlates with failures
- invalid bundles or signature errors after deploy
- performance regression causing lag

### Steps
1) Identify last known good image digest.
2) Update deployment image:
```bash
kubectl -n m0-prod set image deploy/m0-quant m0-quant=ghcr.io/m0club/m0-quant@sha256:<GOOD_DIGEST>
kubectl -n m0-prod rollout status deploy/m0-quant
```

3) Roll back configs if changed.
4) Monitor metrics for recovery.

---

## 12. Runbook: Pausing Markets

### Use cases
- unsafe data source
- model bug affecting subset
- suspected manipulation

### Steps (conceptual)
Use CLI or admin endpoint:
```bash
cargo run -p m0-cli -- market pause --market-id NBA_LAL_BOS --cluster mainnet
```

Verify paused:
```bash
cargo run -p m0-cli -- market get --market-id NBA_LAL_BOS --cluster mainnet
```

Unpause after resolution:
```bash
cargo run -p m0-cli -- market unpause --market-id NBA_LAL_BOS --cluster mainnet
```

---

## 13. Runbook: Emergency Disable Publishing

### Use cases
- key compromise suspected
- invalid signatures
- unsafe outputs published

### Steps
1) scale submitter to 0:
```bash
kubectl -n m0-prod scale deploy/m0-submitter --replicas=0
```

2) optionally block at guardrail:
```bash
kubectl -n m0-prod set env deploy/m0-quant M0_GUARDRAIL_POLICY_LEVEL=STRICT
```

3) communicate internally and begin incident response.

---

## 14. Runbook: Backup and Restore (High Level)

### Backup verification
- ensure PITR enabled
- verify last backup timestamp
- test restore in staging periodically

### Restore procedure (conceptual)
1) disable writers (engine/submitter)
2) restore DB to point-in-time
3) re-enable services gradually
4) run reconciler to align on-chain state and DB

---

## 15. Runbook: Routine Maintenance

### Weekly
- review signer latency and threshold misses
- review drift and calibration health
- verify backups and restore drills
- review RPC health and costs
- rotate non-critical API keys if required

### Monthly
- evaluate signer rotation policy
- verify program authority access and multisig health
- performance review and capacity planning

---

## 16. Checklists

### 16.1 Deploy checklist
- migrations applied
- configs validated
- canary enabled
- dashboards monitored
- rollback plan ready

### 16.2 Incident checklist
- severity assigned
- publish safety ensured
- evidence captured
- communication started
- mitigation executed

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
