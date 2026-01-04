
# Confidence Intervals Specification (M0-CORE)

This document specifies how M0Club (M0-CORE) computes, represents, and publishes confidence/credible intervals for probabilistic outputs.
Because M0Club publishes probability distributions, uncertainty must be expressed in a way that is:
- statistically meaningful
- deterministic and reproducible
- compatible with fixed-point encoding and on-chain bundling constraints
- aligned with risk scoring and guardrails

This spec focuses on discrete outcome probabilities, which are the primary oracle outputs.
Where applicable, it also discusses intervals for derived quantities (e.g., expected value) but these are optional.

---

## 1. Goals

- Provide uncertainty bounds for each outcome probability.
- Maintain determinism and stable rounding for encoded outputs.
- Ensure bounds are well-formed and interpretable by consumers.
- Integrate interval width with risk scoring and anomaly guardrails.
- Support multiple interval methods with explicit versioning.

Non-goals:
- Providing a full Bayesian posterior distribution on-chain.
- Publishing proprietary model internals or priors publicly.

---

## 2. Definitions

- **Probability**: the point estimate for an outcome, `p`.
- **Credible interval**: Bayesian interval for posterior probability, e.g., 95% credible interval.
- **Confidence interval**: frequentist interval; in this system, we generally use Bayesian credible intervals but the output format supports both.
- **CI level**: interval probability mass, typically 0.90, 0.95, 0.99 (encoded in basis points).
- **Fixed-point scale**: probability scale `P = 1_000_000_000` (1e9).

This document uses “CI” as a generic term for an interval bound.

---

## 3. Output Representation

Intervals are included per outcome in the oracle output format.

Per outcome fields (logical):
- `p_scaled` (u32/u64 depending on encoding) scaled by P
- `ci_low_scaled`
- `ci_high_scaled`
- `ci_level_bps` (u16)

Constraints:
- `0 <= ci_low_scaled <= p_scaled <= ci_high_scaled <= P`
- `ci_level_bps` must be one of supported levels for the model/domain
- All values must be deterministic and stable for given inputs

If a model does not support intervals:
- set `ci_low_scaled = p_scaled`
- set `ci_high_scaled = p_scaled`
- set a flag indicating `CI_NOT_AVAILABLE`

---

## 4. Interval Methods

M0-CORE supports multiple methods. Each ModelOutput must declare which method was used.

### 4.1 Beta posterior intervals (binary)
For binary outcomes, probabilities can be modeled with a Beta posterior:
- posterior ~ Beta(a, b)
- p = a / (a+b)

Compute credible interval via Beta quantiles:
- low = Q(alpha/2)
- high = Q(1 - alpha/2)

Where alpha = 1 - level.

Notes:
- This is appropriate if the model’s posterior can be expressed or approximated as Beta.
- Quantile computation must be deterministic; see section 6.

### 4.2 Dirichlet posterior marginal intervals (multi-class)
For categorical outcomes, posterior can be Dirichlet:
- posterior ~ Dirichlet(alpha_1..alpha_K)
Marginal distribution for each p_i is Beta(alpha_i, sum(alpha)-alpha_i).

Compute intervals for each outcome i using Beta quantiles.

Notes:
- marginal intervals do not guarantee joint coverage; they provide per-outcome uncertainty bounds.
- ensure intervals do not violate sum constraints; see section 7.

### 4.3 Normal approximation (discouraged for tails)
For large-sample posteriors:
- approximate p with Normal(mean, var) and derive interval
This is less reliable near 0 and 1 and should be used only as fallback.

### 4.4 Bootstrap-based intervals (offline only)
Bootstrapping can be used in backtests or research, but is not recommended for real-time production unless deterministically seeded and bounded.

---

## 5. CI Level Policy

Supported levels must be explicit per domain/market.

Recommended defaults:
- Sports and on-chain fast markets: 90% or 95%
- Politics and macro: 95% or 99%

`ci_level_bps` must be one of:
- 9000
- 9500
- 9900

Markets can override levels in registry.

---

## 6. Deterministic Quantile Computation

Quantile functions can introduce non-determinism if different math libraries are used.
M0-CORE must ensure deterministic quantiles across deployments.

### 6.1 Options
A) Use a deterministic numeric library with fixed versions and test vectors.
B) Use precomputed lookup tables for common parameter ranges (advanced).
C) Use rational approximations with explicit iteration counts and rounding.

Recommended v1:
- Use a deterministic library for incomplete beta function / inverse CDF
- Pin versions and include test vectors for known inputs
- Clamp iteration limits and rounding mode

### 6.2 Deterministic iteration
If numerical methods are used:
- fixed max iterations
- fixed tolerance thresholds
- deterministic branching and rounding

### 6.3 Fixed-point conversion
After computing low/high as floating values:
- convert to scaled integers with explicit rounding (nearest-even)
- clamp to [0, P]

---

## 7. Multi-Class Sum Consistency

Publishing per-outcome intervals can lead to inconsistencies where:
- sum of lower bounds > 1
- sum of upper bounds < 1

M0-CORE must handle this for consumer usability.

### 7.1 Bounds normalization policy (recommended)
Do not force sums of bounds to equal 1. Instead:
- ensure each bound is individually valid
- include a metadata flag if bounds are not jointly consistent (expected)
- consumers should treat bounds as marginal intervals

### 7.2 Optional tightening (advanced)
If you want stricter bounds:
- compute simplex-consistent bounds via optimization under constraints
This is complex and not recommended for v1.

### 7.3 Probability normalization
Regardless of intervals, the point probabilities must sum to exactly P after fixed-point rounding.
If rounding causes mismatch:
- adjust last outcome by residual to enforce exact sum

---

## 8. Interval Width and Risk Score

Interval width is a key uncertainty measure.

Define per outcome width:
- `w_i = ci_high - ci_low` (scaled)

Aggregate uncertainty measures:
- max width across outcomes
- average width
- entropy of distribution

Risk score should increase with wider intervals and higher entropy, modulated by data quality flags.

Guardrail rules may trigger when:
- width is too narrow given low evidence (overconfidence)
- width is too wide indicating unstable model or low information

---

## 9. Guardrails for CI Values

Guardrails must validate CI outputs:
- bounds within [0, P]
- low <= p <= high
- level supported
- no NaNs or invalid conversions

If invalid:
- block publish or degrade by collapsing intervals to p and raising risk_score
- set flag `CI_INVALID`

Overconfidence check:
- if coverage low or divergence high but CI width very small:
  - widen intervals to minimum width or increase risk score
  - optionally block

---

## 10. Bundle Encoding Notes

CI values must be encoded in the oracle bundle in a deterministic order.
For each outcome, include:
- outcome_id (or hashed id)
- p_scaled
- ci_low_scaled
- ci_high_scaled
- ci_level_bps

Ensure canonical ordering by:
- market_id
- epoch_id
- tick_index
- outcome_id ascending ASCII

---

## 11. Test Vectors

To guarantee determinism:
- store test vectors of (alpha params, level) -> expected low/high scaled values
- store vectors for both binary and multi-class marginal cases

Location:
- `core-engine/m0-common/test-vectors/ci/`
- replicated in SDKs under `sdk/*/test-vectors/ci/`

CI should validate:
- quantile computation stable
- fixed-point rounding stable
- output constraints satisfied

---

## 12. Operational Guidance

- Prefer Dirichlet/Beta marginal intervals for categorical distributions.
- Use 95% as default unless market requires higher.
- Monitor interval widths over time:
  - sudden width collapse can indicate a bug
  - sudden width explosion can indicate data outages or drift
- Include interval metadata in dashboards for transparency.

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
