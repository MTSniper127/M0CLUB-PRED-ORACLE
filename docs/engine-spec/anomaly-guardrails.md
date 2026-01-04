
# Anomaly Guardrails Specification (M0-CORE)

This document specifies anomaly detection and guardrails for M0Club (M0-CORE).
Guardrails protect the oracle publishing pipeline from corrupted inputs, manipulated sources, unstable model behavior, and unsafe publishing conditions.

This spec defines:
- anomaly categories and detection signals
- guardrail policies and thresholds
- actions (degrade, quarantine, pause, retraction triggers)
- integration points (ingestion, aggregation, modeling, bundling, publishing)
- deterministic requirements and auditability

---

## 1. Goals

- Detect and mitigate anomalies before they become published oracle outputs.
- Preserve liveness while preventing unsafe publishes.
- Provide deterministic, explainable guardrail decisions.
- Emit auditable flags and metrics for operators and consumers.
- Integrate with dispute/correction and slashing modules.

Non-goals:
- Guaranteeing perfect detection of all adversarial behavior.
- Replacing manual incident response.

---

## 2. Anomaly Categories

### 2.1 Input anomalies (source-level)
- malformed payloads or schema violations
- timestamp anomalies (future, stale, non-monotonic)
- duplicate floods
- missing coverage or partial feeds
- sudden discontinuities (spikes/drops) beyond plausible bounds
- divergence between independent sources

### 2.2 Aggregation anomalies (feature-level)
- NaNs or invalid numeric ranges (should be impossible with fixed-point)
- feature explosion or collapse (e.g., volume becomes zero unexpectedly)
- window incompleteness beyond tolerance
- watermark lag exceeding thresholds

### 2.3 Modeling anomalies (output-level)
- probabilities outside [0,1] or sums not equal to 1 (post-normalization)
- extreme confidence with low coverage (overconfident outputs)
- sudden probability jumps beyond configured bounds
- risk score paradox (low risk despite high uncertainty signals)
- drift signals exceeding thresholds

### 2.4 Publishing anomalies (chain-level)
- commit succeeds but reveal fails repeatedly
- sequence monotonicity violations
- signer threshold not met or signer agent degraded
- market paused but submit attempted
- RPC instability or chain congestion beyond limits

---

## 3. Guardrail Design Principles

1) Deterministic decisions
- Given the same inputs, the guardrail decision must be identical.
- Use fixed thresholds and stable rounding.

2) Multi-signal scoring
- Combine multiple signals into a final decision.
- Avoid relying on a single fragile metric.

3) Fail-safe defaults
- If critical integrity checks fail, block publishing.
- If non-critical checks fail, degrade output and raise risk.

4) Auditability
- Every decision produces:
  - reason codes
  - signal values
  - guardrail action taken
  - trace_id correlation
  - evidence hashes where applicable

---

## 4. Guardrail Actions

Guardrails can trigger one or more actions:

### 4.1 SOFT actions (degrade)
- increase risk_score
- set quality_flags (LOW_COVERAGE, DIVERGENCE, STALE)
- reduce publish cadence (skip ticks)
- reduce feature set or model complexity
- publish with degraded confidence intervals

### 4.2 HARD actions (block/quarantine)
- quarantine the event/feature/model output (do not publish)
- block commit/reveal for the tick/epoch
- require manual approval (if enabled)
- pause market via guardian (if configured)

### 4.3 Emergency actions (incident)
- trigger correction workflow (supersede/retract)
- rotate signer set recommendation
- open slashing case recommendation (if offense provable)

---

## 5. Guardrail Signals

This section defines primary signals and how to compute them.

### 5.1 Coverage ratio
Coverage ratio measures source completeness:
- `coverage_ratio = sources_seen / sources_expected`

Thresholds:
- `coverage_warn` (e.g., 0.8)
- `coverage_block` (e.g., 0.5)

### 5.2 Source divergence
Compute divergence between independent source probability estimates or derived implied probabilities.

Example metrics:
- max absolute difference across sources for key values
- Jensen-Shannon divergence between distributions

Thresholds:
- `divergence_warn`
- `divergence_block`

### 5.3 Timestamp skew and staleness
Signals:
- `event_future_skew_ms`
- `event_staleness_ms`
- `watermark_lag_ms`

Thresholds per market:
- `max_future_skew_ms`
- `max_staleness_ms`
- `max_watermark_lag_ms`

### 5.4 Outlier detection
Outlier detection on key numeric fields:
- z-score relative to rolling mean/std
- robust MAD-based z-score
- percentile bounds

Outlier triggers:
- `outlier_warn_count`
- `outlier_block_count`

### 5.5 Probability jump constraints
Compare current probability distribution to previous tick:
- `delta = max_i |p_i(t) - p_i(t-1)|`

Thresholds:
- `jump_warn` (e.g., 0.10)
- `jump_block` (e.g., 0.30)

If jump exceeds warn:
- increase risk_score
If exceeds block:
- block publish or require cooldown window

### 5.6 Confidence vs evidence check
If model outputs high confidence but inputs indicate low evidence:
- high confidence defined by low entropy or narrow CI
- low evidence defined by low coverage or high divergence

Rule:
- if confidence_high AND evidence_low -> force risk_score high and widen CI or block

### 5.7 Drift score
Use drift signals from backtesting or online feature drift detection.
If drift_score exceeds threshold:
- set DRIFT flag
- increase risk_score
- optionally block for severe drift

### 5.8 Signer availability
Signals:
- signer agent latency and error rate
- signatures collected vs required threshold

If threshold cannot be met:
- block publish

### 5.9 RPC health
Signals:
- RPC latency
- error rate
- slot lag

If unhealthy:
- reduce cadence
- retry with backoff
- block if cannot confirm commits reliably

---

## 6. Guardrail Policy Engine

### 6.1 Policy levels
Define policy levels per market:
- STRICT (politics/macro high impact)
- NORMAL (sports/markets standard)
- FAST (high frequency, more tolerant to noise)

Each policy level sets different thresholds.

### 6.2 Decision function (deterministic)
Compute a final decision from signals.

Example deterministic decision:
- if integrity_failure -> BLOCK
- else if signatures_unavailable -> BLOCK
- else if coverage_ratio < coverage_block -> BLOCK
- else if divergence > divergence_block -> BLOCK
- else if drift_score > drift_block -> BLOCK
- else if any warn thresholds triggered -> DEGRADE
- else -> PASS

DEGRADE action includes:
- set flags
- risk_score bump based on severity

### 6.3 Risk score bump (example)
Compute bump as sum of weighted severities:
- coverage severity: `max(0, coverage_warn - coverage_ratio) / coverage_warn`
- divergence severity: `divergence / divergence_warn`
- jump severity: `delta / jump_warn`
Clamp bump and map to bps:
- `risk_bump_bps = clamp(round(severity * 3000), 0, 6000)`
Final risk_score = base_risk + bump, clamped to 10000.

All rounding must be explicit and deterministic.

---

## 7. Integration Points

### 7.1 Ingestion
- schema validation and quarantine
- dedup flood detection
- timestamp validation

### 7.2 Aggregation
- window completeness checks
- watermark lag checks
- feature range validation

### 7.3 Modeling
- output validity checks
- confidence vs evidence checks
- jump constraints vs previous tick
- drift checks

### 7.4 Bundling
- schema enforcement and bundle size checks
- ensure canonical ordering
- attach guardrail metadata to bundle (optional)

### 7.5 Publishing
- check market pause state
- check signer set status
- check replay protection and sequence allocation
- enforce commit/reveal deadlines

---

## 8. Guardrail Metadata in Outputs

Oracle outputs should include guardrail flags and metadata:
- `quality_flags` bitmask
- `risk_score`
- `guardrail_reason_codes[]`
- optionally `guardrail_signals_hash` commitment to signal values

Reason codes examples:
- LOW_COVERAGE
- HIGH_DIVERGENCE
- STALE_INPUTS
- OUTLIER_SPIKE
- JUMP_EXCEEDED
- DRIFT_DETECTED
- SIGNER_UNAVAILABLE
- RPC_UNHEALTHY

---

## 9. Operational Playbooks (High Level)

When guardrails trigger BLOCK frequently:
1) Inspect coverage metrics and connector health.
2) Inspect source divergence and outlier logs.
3) Inspect signer agent health and signature collection.
4) Inspect RPC health and chain congestion.
5) If integrity issues, pause market and open incident report.
6) Decide on correction (supersede/retract) if a bad output was published.

---

## 10. Testing

### 10.1 Unit tests
- each signal computation
- threshold comparisons and rounding
- decision function determinism

### 10.2 Integration tests (localnet)
- simulate missing sources -> DEGRADE/BLOCK
- simulate divergence -> DEGRADE/BLOCK
- simulate signer outage -> BLOCK
- simulate probability jumps -> DEGRADE/BLOCK
- ensure blocked ticks do not publish commits

### 10.3 Chaos tests
- random connector failures and recoveries
- delayed inputs and watermark lag
- RPC timeout storms

---

## 11. Implementation Guidance

- Keep guardrail logic in a single module with versioned config.
- Log all decision inputs and outputs with trace_id.
- Provide dashboards for:
  - coverage by market
  - divergence distribution
  - jump deltas
  - drift scores
  - block/degrade counts
- Allow per-market overrides without redeploying code when possible.

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
