
# Bayesian Modeling Specification (M0-CORE)

This document specifies the Bayesian modeling layer used by M0Club (M0-CORE).
It defines how Bayesian components consume FeatureVectors, produce probability distributions and uncertainty metadata, and integrate with calibration, backtesting, and oracle bundling.

This spec describes interfaces and deterministic requirements. It does not reveal proprietary model parameters. Instead, it defines how models must behave to be pluggable and verifiable within M0-CORE.

---

## 1. Goals

- Provide rigorous probability distributions for outcomes, not point estimates.
- Represent uncertainty via credible intervals and risk scores.
- Support multi-source fusion (Bayesian updating and hierarchical priors).
- Provide deterministic and reproducible model outputs given fixed inputs.
- Integrate with feature schema versioning, calibration, and drift checks.

Non-goals:
- Providing a single universal model for all domains.
- Exposing proprietary priors and weights publicly.

---

## 2. Modeling Interfaces

### 2.1 Model contract
Models implement a stable interface:

Inputs:
- `FeatureVector`
- `ModelContext` (market config, priors, calibration state)
- `ModelRunConfig` (determinism flags, debug options)

Outputs:
- `ModelOutput`

### 2.2 Deterministic execution
Given identical inputs and the same model version:
- output distribution must be identical
- risk score must be identical
- credible interval bounds must be identical

Determinism constraints:
- no stochastic sampling in production path unless seeded deterministically
- use fixed-point math or stable floating-point with standardized rounding
- for any approximations (e.g., log/exp), use deterministic libraries and tests

Recommended:
- implement core math in Rust with careful rounding and test vectors

---

## 3. Domain Modeling Templates

M0Club supports multiple domains; Bayesian modeling is used differently per domain.

### 3.1 SPORTS
- outcomes: binary or trinary (home/away/draw) and occasionally multi
- signals: odds, injuries, schedule, form, on-chain sentiment proxies (optional)
- priors: team strength priors, league priors
- updates: frequent, tick-based

### 3.2 POLITICS
- outcomes: multi candidate or binary proposition
- signals: poll averages, forecast models, macro signals
- priors: hierarchical region priors, incumbency priors
- updates: slower, event-driven

### 3.3 MACRO
- outcomes: range buckets for releases (e.g., CPI yoy buckets)
- signals: prior releases, nowcasts, rates markets
- priors: hierarchical and seasonal priors
- updates: schedule-based

### 3.4 MARKETS / ONCHAIN
- outcomes: regime classification, volatility buckets, directional probability
- signals: price, volume, order flow proxies, funding rates
- priors: regime transition priors (HMM-like), volatility priors
- updates: high frequency

---

## 4. Bayesian Components

### 4.1 Priors
A prior is a distribution over outcomes before considering new evidence.

Priors can be:
- categorical Dirichlet prior for discrete outcomes
- Beta prior for binary outcomes
- hierarchical Dirichlet/Beta for grouped markets
- Gaussian priors for continuous latent variables

The model must store:
- `prior_id` and `prior_version`
- `prior_hash` (sha256 of canonical prior config) for audit integrity (optional)

Priors should be market-specific but share templates where possible.

### 4.2 Likelihood
Likelihood models how evidence affects outcomes.

Likelihood sources:
- odds implied probabilities
- statistical features (form, momentum)
- structured signals (injury count)
- market microstructure features

Likelihood must map feature values into likelihood contributions.

### 4.3 Posterior update
Posterior updates combine priors and likelihood.

For categorical outcomes:
- Dirichlet-multinomial update or general Bayesian update
For binary:
- Beta-Binomial or logistic update

Posterior should yield:
- posterior probability distribution
- credible intervals per outcome (if desired)
- uncertainty metrics used in risk_score

### 4.4 Evidence fusion
When multiple sources provide evidence:
- weight sources by quality and coverage
- discount stale or divergent sources
- use robust fusion (e.g., mixture models) where needed

Fusion should respect quality_flags from FeatureVector.

---

## 5. Output Requirements

### 5.1 Probability distribution
For each outcome:
- `p_scaled` as fixed-point (scale = 1e9)
- sum of p_scaled across outcomes must equal scale (within rounding tolerance)

Normalization rules:
- compute raw scores
- normalize to sum to 1.0
- convert to fixed-point with deterministic rounding
- adjust last outcome to ensure exact sum equals scale if required

### 5.2 Credible intervals
Represent uncertainty via credible intervals:

Fields:
- `ci_low_scaled`
- `ci_high_scaled`
- `ci_level_bps` (e.g., 9500 for 95%)

Intervals must satisfy:
- 0 <= low <= p <= high <= 1
- determinism and stable rounding

### 5.3 Risk score
risk_score is an integer 0..10000 (bps):
- higher = more risk / more uncertainty
- derived from:
  - posterior entropy
  - source divergence
  - coverage gaps
  - staleness
  - model drift flags

risk_score should be monotonic with uncertainty measures.

### 5.4 Quality flags
Model output must propagate and add flags:
- propagate ingestion and aggregation flags
- add model-specific flags (DRIFT, LOW_COVERAGE, HIGH_DIVERGENCE)

---

## 6. Calibration

### 6.1 Why calibration
Raw probabilities must be calibrated to align predicted probability with observed frequencies.

### 6.2 Calibration strategies
- Platt scaling for binary cases
- Isotonic regression for non-parametric calibration
- Temperature scaling for categorical distributions
- Domain-specific calibration curves

Calibration must be:
- versioned
- deterministic
- applied only when sufficient data exists
- monitored for drift

### 6.3 Calibration state storage
Calibration state can be stored in:
- feature store metadata
- model registry
- dedicated calibration store

Store:
- `calibration_version`
- `calibration_hash`
- `trained_on_range` metadata

---

## 7. Drift Detection

Drift detection monitors changes that degrade model validity.

Signals:
- feature distribution shifts
- error rate shifts in backtests
- increased entropy or variance
- source divergence increases

Actions:
- raise risk_score
- set DRIFT flags
- trigger retraining or operator alert

---

## 8. Model Registry

### 8.1 Model identification
Models are referenced by:
- `model_id` (string)
- `model_version` (u32)
- `feature_schema_version` required
- `config_hash` (sha256)

### 8.2 Deployment
Model artifacts are distributed via:
- container images
- build artifacts in monorepo
- registry entries mapping market_id -> model_id/version

Production supports:
- canary deployments
- rollback of model versions
- per-market pinning

---

## 9. Deterministic Math Notes

### 9.1 Fixed-point recommendations
Where possible:
- represent probabilities and rates as integers with scale
- use integer math for normalization steps
- avoid platform-dependent floating conversions

Where floating is required:
- use a standardized math library and deterministic rounding
- add test vectors to assert identical outputs across platforms

### 9.2 Stable rounding
Use explicit rounding mode:
- round to nearest, ties to even

Ensure consistent conversions:
- from f64 to scaled integers
- from logits to probabilities

---

## 10. Example (Conceptual)

Example binary market with Beta prior:

- prior: Beta(a, b)
- evidence: implied probability p0 from odds + feature adjustments
- update: posterior Beta(a', b') with pseudo-counts derived from evidence weight

Outputs:
- p = a' / (a'+b')
- ci bounds from Beta quantiles
- risk score from variance and entropy

Note:
- Specific parameterization is proprietary; this illustrates structure only.

---

## 11. Testing and Validation

### 11.1 Unit tests
- distribution normalization equals scale exactly
- credible intervals bounds valid
- deterministic output given fixed inputs
- calibration transforms deterministic and bounded

### 11.2 Integration tests
- run model on stored FeatureVectors
- compare outputs to expected fixtures
- ensure bundler computes stable bundle hashes

### 11.3 Backtests
Backtesting pipeline should:
- fetch features from feature store over a range
- run model deterministically
- compare predicted probabilities to observed outcomes
- generate calibration curves and metrics

Key metrics:
- Brier score
- log loss
- calibration error
- sharpness
- coverage of intervals

---

## 12. Operational Guidance

- For high-frequency markets, keep Bayesian updates lightweight.
- For slow markets, incorporate hierarchical priors and scheduled releases.
- Always propagate quality flags and increase risk score when sources degrade.
- Maintain a changelog for model versions and calibration updates.

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
