
# Incident Response (Ops)

This document defines the incident response process for M0Club (M0-CORE).
It covers detection, classification, triage, mitigation, communication, recovery, and post-incident actions.
It is designed for on-call operators and engineering teams running the oracle pipeline in production.

This guide assumes the system includes:
- ingestion connectors
- aggregation + modeling engine
- bundler + signer + submitter (commit-reveal)
- API gateway + realtime
- monitoring (metrics/logs/traces)
- market registry and pause/guardian controls (if enabled)

---

## 1. Goals

- Restore safe oracle operation as quickly as possible.
- Prevent publishing corrupted or unsafe outputs.
- Minimize customer impact and preserve integrity.
- Provide clear, auditable decision-making and communications.
- Capture learnings to prevent recurrence.

Non-goals:
- Replacing formal organizational security incident processes.
- Providing legal advice.

---

## 2. Incident Severity Levels

Use four levels. Default conservatively.

### SEV0 (Critical)
Impact:
- unsafe/incorrect oracle output published broadly
- key compromise suspected/confirmed
- signer threshold compromised or forged signatures
- program authority compromise
- widespread publish halt on production

Response:
- immediate paging of core team
- consider pausing markets
- executive/security escalation (org dependent)

### SEV1 (High)
Impact:
- significant publish degradation (many markets blocked/degraded)
- repeated invalid bundles or commit/reveal failures
- major data source corruption affecting core markets
- database outage affecting serving

Response:
- on-call engages immediately
- mitigation within minutes-hours
- frequent status updates

### SEV2 (Medium)
Impact:
- partial market impact
- elevated latency and intermittent failures
- individual connector outage
- drift/correlation anomalies without confirmed bad output

Response:
- on-call engages during business hours or as per rotation
- mitigation within hours

### SEV3 (Low)
Impact:
- minor issues, warnings, non-customer-facing incidents
- near-misses (guardrails blocked bad output successfully)

Response:
- ticket and follow-up, no urgent paging

---

## 3. Incident Detection Signals

Incidents are detected by:
- guardrail BLOCK/DEGRADE spikes
- publish success rate drop
- signer threshold misses
- signature verification failures
- replay protection violations
- divergence/outlier alarms
- RPC error bursts
- API error rate spikes
- data store saturation

Primary alerts:
- publish_miss_rate > threshold
- signer_threshold_miss_count > threshold
- invalid_bundle_count > threshold
- api_5xx_rate > threshold
- watermark_lag_ms > threshold
- divergence_block_events > threshold

---

## 4. Roles and Responsibilities

### 4.1 Incident Commander (IC)
- owns coordination and decisions
- sets severity and timeline
- ensures communication cadence

### 4.2 On-Call Engineer
- executes triage and mitigations
- coordinates with domain experts

### 4.3 Scribe
- records timeline, actions, and decisions
- captures metrics and references

### 4.4 Domain Experts
- Solana publishing expert
- Data/connector expert
- Quant/modeling expert
- Infra/DB expert
- Security expert (for key issues)

For small teams, one person may fill multiple roles.

---

## 5. Triage Checklist (First 10 Minutes)

1) Confirm severity level.
2) Identify scope:
   - which markets affected
   - which pipeline stage failing
3) Determine safety:
   - are incorrect outputs published?
   - are signatures valid?
   - are guardrails blocking?
4) If unsafe output suspected:
   - pause affected markets (guardian) if available
   - stop submitter publishing (kill switch)
5) Capture snapshots:
   - last good bundle hash and epoch
   - error logs and traces
   - metrics dashboards screenshot references
6) Start incident timeline document.

---

## 6. Immediate Mitigation Controls

Use safe, reversible mitigations first.

### 6.1 Kill switch (publishing)
Disable submitter:
- scale submitter deployment to 0
- or set `M0_PUBLISH_ENABLED=false` via config map
- or block publishing at guardrail policy (BLOCK all)

Use when:
- unsafe output suspected
- signer integrity compromised
- replay protection issues

### 6.2 Pause affected markets
If registry supports pausing:
- pause only impacted markets if possible
- pause all markets if unsure

Use when:
- a specific domain feed corrupted
- a modeling bug affects subset of markets

### 6.3 Degrade policy
Set guardrail policy to DEGRADE:
- publish less frequently
- increase risk score
- widen confidence intervals
- require more evidence thresholds

Use when:
- partial outage but safety can be maintained

### 6.4 Switch RPC endpoints
If commit/reveal failures due to RPC:
- failover to backup RPC
- reduce concurrency
- adjust fee prioritization

### 6.5 Disable connector(s)
If divergence/outliers due to a connector:
- disable the connector
- adjust expected coverage thresholds temporarily

### 6.6 Rollback releases
- rollback engine or submitter images to last known good digest
- rollback configs
- rollback calibration artifact versions if needed

---

## 7. Incident Playbooks by Category

### 7.1 Data Source Outage / Degradation
Symptoms:
- coverage_ratio drops
- staleness flags
- quarantine spikes

Actions:
1) Confirm which source is failing.
2) Reduce reliance:
   - disable connector
   - lower expected_sources if safe
3) Increase risk score and set flags.
4) If market cannot be served safely:
   - pause market or block publishing
5) Start provider escalation.

Recovery:
- re-enable connector after stability window
- run backtest/verification to ensure no drift

### 7.2 Source Manipulation / Divergence
Symptoms:
- divergence_block triggers
- outlier spikes correlated to a single source
- sudden odds probability drift

Actions:
1) Identify divergent source(s).
2) Quarantine that source feed.
3) Publish with reduced source set if safe.
4) Increase risk score.
5) If manipulation affects many markets:
   - pause impacted markets
6) Capture evidence hashes and raw metadata for audit.

Recovery:
- keep source disabled until validated
- update mapping tables and anomaly thresholds

### 7.3 Modeling Bug / Overconfidence
Symptoms:
- probabilities invalid
- CI collapse or explosion
- jump constraints exceeded systematically

Actions:
1) Stop publishing (kill switch) if unsafe outputs possible.
2) Rollback model version or quant service image.
3) Verify against test vectors and last good outputs.
4) Increase guardrail constraints until fixed.

Recovery:
- deploy patch with canary
- run backtest comparison before re-enabling full publish

### 7.4 Calibration Regression
Symptoms:
- ECE/Brier worsens after calibration update
- probabilities shift systematically without data changes

Actions:
1) Rollback calibration artifact to prior version.
2) Validate artifact hash and registry selection.
3) Investigate training window and dataset issues.

Recovery:
- retrain calibration with corrected dataset
- add gates to require improvement thresholds

### 7.5 Signer Threshold Miss / Signer Outage
Symptoms:
- threshold not met
- signer agent timeouts
- signature collection p95 spikes

Actions:
1) Check signer agent health and logs.
2) Failover to backup signers if configured.
3) Reduce publish cadence to reduce load.
4) If threshold cannot be met:
   - block publishing or pause markets

Recovery:
- restore signer nodes
- rotate signer set if persistent

### 7.6 Signature Verification Failure
Symptoms:
- on-chain signature verification fails
- submitter reports signature invalid
- consumer reports invalid bundles

Actions (treat as SEV0 until proven otherwise):
1) Immediately stop publishing.
2) Confirm active signer set id and pubkeys.
3) Confirm bundle hashing canonicalization and schema version.
4) Check for mismatch between signer_set registry and signer agents.
5) If compromise suspected:
   - rotate signer set
   - disable suspect keys

Recovery:
- restore consistent signer set and bundle hashing
- publish correction if bad bundles were accepted

### 7.7 Replay Protection / Sequence Issues
Symptoms:
- sequence mismatch errors
- duplicate commits rejected
- reveals not matching commits

Actions:
1) Stop concurrent submitters (ensure single leader).
2) Inspect publish state store.
3) Reconcile last committed epoch/sequence.
4) Fix idempotency and leader election issues.

Recovery:
- deploy fix and run localnet reproduction
- add CI regression tests

### 7.8 Solana RPC / Chain Congestion
Symptoms:
- tx timeouts
- blockhash not found
- high confirmation latency

Actions:
1) Switch to multiple RPC endpoints.
2) Increase priority fees (within policy).
3) Reduce publish concurrency and cadence.
4) Retry with idempotent checks.

Recovery:
- restore normal cadence after stability window

### 7.9 Database Outage (Feature Store / Metadata)
Symptoms:
- engine cannot read/write features
- API errors spike
- query latency spikes

Actions:
1) Failover to read replica (if available).
2) Reduce write volume or disable non-critical writes.
3) Enable cache-only serving for latest values if safe.
4) Pause publishing if state cannot be verified.

Recovery:
- restore DB health, run migrations if needed
- backfill missed data from event logs

---

## 8. Communication

### 8.1 Internal updates
For SEV0/SEV1:
- update every 15-30 minutes
For SEV2:
- update every 60 minutes or as needed

Include:
- current status
- impacted markets
- mitigation actions
- next steps and ETA (avoid promises)

### 8.2 External updates
If public consumers are impacted:
- post status updates via official channels
- include which markets and what type of degradation (paused, delayed, degraded confidence)
- avoid exposing sensitive details during active security incidents

---

## 9. Recovery and Validation

Before resuming full publishing:
1) Verify guardrails stable (no excessive blocks).
2) Verify signer threshold success rate.
3) Verify bundle hashes and signature verification end-to-end.
4) Run a short backtest or replay on recent data to validate outputs.
5) Canary enable publishing for a subset of markets.
6) Gradually scale back to full cadence.

---

## 10. Post-Incident Review

Within 24-72 hours (depending on severity):
- write postmortem with timeline and root cause
- list contributing factors
- define action items with owners and deadlines
- add tests and guardrails to prevent recurrence
- update runbooks and alerts

Postmortem must include:
- incident summary
- customer impact
- detection and response timeline
- root cause analysis (technical + process)
- what worked / what didn’t
- prevention plan

---

## 11. Evidence Collection

During incident, preserve:
- bundle hashes (content hash) and epoch ids
- signer_set_id and pubkeys
- logs and trace ids
- guardrail decision reasons and signal snapshots
- connector raw metadata (without secrets)
- config versions and image digests

Store evidence in an incident folder with access controls.

---

## 12. Quick Commands (Examples)

These are examples; adapt to your deployment tooling.

Scale submitter down:
```bash
kubectl -n m0-prod scale deploy/m0-submitter --replicas=0
```

Pause publishing via config:
```bash
kubectl -n m0-prod patch configmap m0-engine-config --type merge -p '{"data":{"M0_PUBLISH_ENABLED":"false"}}'
```

Switch RPC:
```bash
kubectl -n m0-prod set env deploy/m0-submitter SOLANA_RPC_URL=https://backup-rpc.example.com
```

Restart a deployment:
```bash
kubectl -n m0-prod rollout restart deploy/m0-quant
```

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
