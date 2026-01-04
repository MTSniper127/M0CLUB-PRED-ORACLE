
# Backtesting Specification (M0-CORE)

This document specifies the backtesting subsystem for M0Club (M0-CORE).
Backtesting evaluates model performance over historical periods using archived FeatureVectors and resolved outcomes, producing metrics, calibration artifacts, and drift signals.

This spec defines:
- dataset contracts (features + labels)
- backtest execution flow and determinism rules
- metrics and reporting outputs
- calibration training integration
- drift detection integration
- reproducibility, storage, and operational guidance

---

## 1. Goals

- Evaluate probability forecasts with rigorous metrics (Brier, logloss, ECE).
- Compare model versions and calibration artifacts across markets.
- Generate calibration artifacts in an offline training loop.
- Provide reproducible runs with stable configs and hashes.
- Produce drift signals used by online engine risk scoring.

Non-goals:
- Predicting future performance guarantees.
- Providing proprietary dataset distribution.

---

## 2. Backtest Inputs

Backtesting consumes:

1) **FeatureVectors**
- from the feature store (online/offline)
- keyed by (market_id, window_end_ms) aligned to publish cadence

2) **Outcome labels**
- resolved outcomes for the prediction targets
- stored in an outcomes store or derived from domain resolution feeds

3) **Model artifacts**
- specific model_id and model_version binaries/configs
- optional calibration artifacts to apply

4) **Backtest configuration**
- time range
- cadence and sampling policy
- market selection and inclusion filters
- evaluation windows and cutoffs

---

## 3. Labeling and Outcome Store

### 3.1 Outcome representation
Backtests require the “truth label” for each prediction event.

For discrete outcomes:
- `outcome_id` (string)
- optional multi-label support for ranges/buckets
- resolution timestamp

For binary:
- label y in {0,1}

For multi-class:
- one-hot or index of correct outcome

### 3.2 Outcome store schema (logical)
- `market_id`
- `event_id` or `epoch_id` depending on domain
- `resolution_time_ms`
- `resolved_outcome_id`
- `resolution_source`
- `quality_flags`

Important:
- label must correspond to the same market definition used by features/model.
- resolution source must be auditable and versioned.

### 3.3 Alignment with features
Backtest must align each FeatureVector with the correct label.
Alignment strategies:
- by epoch_id and tick_index for oracle style
- by event_id for event-based markets
- by time window (the last features before resolution cutoff)

Recommended:
- define a per-market alignment policy in registry.

---

## 4. Deterministic Backtest Execution

Backtests must be reproducible.

### 4.1 Run identity
Every backtest run should produce a deterministic run_id:
`run_id = sha256(config_bytes || model_artifact_hash || calibration_hash || dataset_manifest_hash)`

### 4.2 Dataset manifest
A dataset manifest is a deterministic list of samples:
- market_id
- window_end_ms
- feature_schema_version
- features_hash
- label reference and resolution info

Manifest hash:
`dataset_manifest_hash = sha256(canonical_manifest_bytes)`

### 4.3 Sampling rules
To avoid bias:
- sample at fixed cadence (e.g., every tick)
- or sample at specific decision points (e.g., pre-game cutoff)
Sampling must be deterministic:
- any downsampling must use stable rules (e.g., take every Nth tick)
- no random sampling unless seeded and recorded

### 4.4 Cutoffs (no lookahead)
Ensure no lookahead bias:
- features used must have window_end_ms <= decision_cutoff_ms
- labels must be from resolution_time_ms > decision_cutoff_ms
- enforce strict constraints in dataset builder

---

## 5. Backtest Pipeline

### 5.1 Steps
1) Build dataset manifest for specified markets and time range.
2) Load FeatureVectors and labels.
3) Run model to produce raw probabilities.
4) Optionally apply calibration artifact(s).
5) Compute metrics per sample and aggregate.
6) Produce reports and artifacts:
   - metrics summary
   - reliability diagrams (data points)
   - drift signals
   - calibration training candidates
7) Store results in backtest store and publish metadata to registry (optional).

### 5.2 Batch vs streaming
Backtests can be:
- batch (offline) for long ranges
- streaming replay mode for continuous evaluation

---

## 6. Metrics

Backtests must compute standard probabilistic forecasting metrics.

### 6.1 Brier score
For binary:
- `(p - y)^2` averaged

For multi-class:
- sum of squared differences between predicted vector and one-hot vector

### 6.2 Log loss / NLL
Binary:
- `- [y * log(p) + (1-y)*log(1-p)]`

Multi-class:
- `- log(p_true)`

### 6.3 Calibration metrics
- ECE (Expected Calibration Error)
- MCE (Maximum Calibration Error)
- reliability curve points

ECE computation:
- bin p into K bins
- compute avg predicted probability and empirical frequency
- weighted average absolute difference

### 6.4 Sharpness and entropy
- entropy of predicted distributions
- variance of predicted probabilities
These measure how confident predictions are independent of correctness.

### 6.5 Coverage of credible intervals (if produced)
- % of outcomes falling within predicted intervals (where meaningful)

### 6.6 Risk score evaluation
Compare risk_score to realized error:
- correlation between risk and error
- lift metrics: high-risk bucket should have worse performance

---

## 7. Reports and Outputs

Backtest outputs should be stored as structured artifacts.

### 7.1 Run summary (JSON)
Fields:
- run_id
- model_id/version
- calibration_id/version
- time range
- markets included
- sample counts
- aggregated metrics
- dataset_manifest_hash
- config_hash

### 7.2 Per-market metrics
For each market:
- brier, logloss, ece
- sample counts
- performance over time (rolling window)
- drift indicators

### 7.3 Reliability diagram data
Store:
- bin_edges
- bin_avg_pred
- bin_emp_freq
- bin_counts

### 7.4 Drift signals
Store:
- feature drift statistics (PSI, KS, etc.)
- performance drift (metrics over time)
- alert thresholds exceeded flags

---

## 8. Calibration Training Integration

Backtesting is the primary driver for calibration artifact training.

Workflow:
1) Run backtest and collect p_raw and labels.
2) Fit calibration method (Platt, temperature, isotonic).
3) Evaluate calibration improvement.
4) If approved, publish a new calibration artifact with version and hash.
5) Deploy artifact to model registry and online engine.

Artifacts should record:
- training window
- eval window
- baseline vs calibrated metrics

---

## 9. Drift Detection Integration

Backtesting produces drift signals for online risk scoring.

Drift categories:
- feature distribution drift
- label drift (outcome frequencies shift)
- calibration drift (ECE increases)

Signals are stored and can be used by online engine to:
- increase risk_score
- set DRIFT quality flags
- trigger retraining/recalibration jobs

---

## 10. Backtest Store

Results should be stored in a backtest store:
- Postgres for metadata
- object storage for large per-sample outputs

Store:
- run metadata keyed by run_id
- per-market summaries
- manifest references and hashes
- artifact links (calibration artifacts generated)

Retention:
- keep metadata indefinitely
- keep large per-sample files per policy tier

---

## 11. Configuration

Backtest config should be a versioned file format (YAML/TOML).

Fields:
- markets list or filters
- start_ms, end_ms
- sample cadence policy
- decision cutoff rules per market
- model_id/version
- calibration selection
- metrics bins K
- output paths

Config must produce:
- config_hash (sha256 of canonical config bytes)

---

## 12. Reproducibility and Auditing

To reproduce a run, you need:
- config file (hash)
- model artifact hash and version
- calibration artifact hash and version
- dataset manifest hash
- feature store snapshot or deterministic query ranges

Backtest runner should record:
- git commit hash
- container image digest
- dependency versions

---

## 13. Operational Guidance

- Run continuous backtests on production-like data nightly.
- Run full-range backtests before deploying new model versions.
- Canary models and compare live performance to backtest expectations.
- Use alerts when metrics degrade beyond thresholds.
- Segment metrics by domain and market tier.

---

## 14. Test Plan

Unit tests:
- dataset alignment and cutoffs prevent lookahead
- metric computations against known small examples
- deterministic run_id and manifest hash

Integration tests:
- feature store query -> dataset manifest -> model run -> report
- calibration training pipeline generates deterministic artifact hashes

Load tests:
- large market sets and long time ranges
- parallel execution scaling

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
