
# Replay Protection (Protocol Spec)

This document specifies replay protection mechanisms used by M0Club to prevent reusing old signatures, bundles, or commit/reveal submissions as valid new oracle updates.

Replay protection is required across:
- off-chain signer agents
- commit/reveal submitters
- on-chain oracle programs
- SDK validators and consumer integrations

This spec defines:
- sequence and nonce models
- on-chain enforcement strategies
- off-chain allocation and durability requirements
- acceptance bounds and staleness constraints
- recommended defaults and test plans

---

## 1. Threat Model

Replay attacks include:

1) **Signature replay**
- Reuse a valid signature from a prior update to authorize a different update.
- Re-submit the same signed bundle_hash as if it were new.

2) **Transaction replay**
- Re-submit the same commit or reveal transaction multiple times to confuse indexing or trigger unintended effects.

3) **Cross-market replay**
- Use a signature for one market to authorize another market if the signature message is not bound to market identity.

4) **Cross-epoch replay**
- Use a signature from an older epoch to authorize a newer epoch if not bound to epoch id/window.

5) **Out-of-order and delayed replay**
- Re-submit an old commit within staleness windows to overwrite or conflict with current state.

Replay protection must make these attacks infeasible under the protocol’s acceptance rules.

---

## 2. Core Mechanisms

M0Club uses layered replay protections:

- **Signature binding** to market_id, epoch_id, bundle_hash, signer_set_id, and sequence.
- **Sequence policies** (GLOBAL or PER_MARKET) with monotonic constraints.
- **On-chain uniqueness** via PDA derivations for commitments and reveals.
- **Staleness bounds** to reject commits and reveals outside allowed time windows.
- **Idempotent submission** with deterministic keys and retry-safe behavior.

---

## 3. Sequence Model

### 3.1 Sequence definition
A `sequence` is a u64 replay context value included in the signed message.

It MUST be:
- allocated by the signer agent (or a dedicated allocator)
- durable (persisted) before signatures are produced
- unique per sequence policy

### 3.2 Sequence policies
Choose one of:

#### GLOBAL
- Unique per signer_set_id.
- Strongest default.
- Requires a shared allocator across signer instances.

#### PER_MARKET
- Unique per (signer_set_id, market_id).
- Lower coordination scope and good scalability.
- Requires market_id binding in signature message.

#### PER_EPOCH (discouraged)
- Unique per (signer_set_id, market_id, epoch_id).
- Weaker protection and should only be used for special cases.

Recommended:
- GLOBAL or PER_MARKET.

---

## 4. Signature Binding Rules

To prevent cross-market and cross-epoch replay, signers MUST sign a message that includes:

Required:
- domain separation prefix
- market_id bytes
- epoch_id bytes
- bundle_hash
- signer_set_id
- sequence

Recommended message:
`msg = sha256("M0CLUB_ORACLE_V1" || market_id || epoch_id || bundle_hash || signer_set_id_le || sequence_le)`

Validators MUST reject signatures where:
- market_id or epoch_id mismatch the targeted commit/reveal
- signer_set_id mismatch
- bundle_hash mismatch
- sequence is reused or violates monotonic constraints

---

## 5. On-chain Enforcement Strategies

On-chain programs must enforce replay safety without relying solely on off-chain correctness.

### 5.1 PDA uniqueness
Use PDAs derived from stable identifiers to prevent duplicates:

- Commitment PDA:
  - `commit_pda = PDA("commit", epoch_pda)` for single commit per epoch, or
  - `commit_pda = PDA("commit", epoch_pda, sequence)` if multi-commit policy enabled

- Reveal PDA:
  - `reveal_pda = PDA("reveal", epoch_pda, bundle_hash)`

These ensure:
- repeated submissions fail or are idempotent
- reveal cannot be duplicated for the same hash

### 5.2 Monotonic sequence tracking
Store a `last_sequence` and require:
- `sequence > last_sequence`

Where the scope depends on policy:
- GLOBAL: stored in signer set account
- PER_MARKET: stored in market account

Pros:
- Simple and efficient

Cons:
- Requires in-order sequences (or use a more complex window)

### 5.3 Sliding window bitmap (optional)
To allow out-of-order sequences within a range:
- track a base sequence and bitmap of seen sequences in `[base, base+W)`
- accept sequences in window if unseen
- advance base when contiguous sequences consumed

Pros:
- tolerant to network ordering variance

Cons:
- more complex on-chain state and logic

Recommended v1:
- monotonic tracking unless out-of-order submissions are expected.

### 5.4 Reject stale and future updates
Staleness bounds prevent old commits from being accepted as new.

Recommended per market:
- `max_commit_staleness_ms`
- `max_reveal_delay_ms`
- `future_skew_ms`

On-chain checks should approximate time using:
- Clock sysvar unix_timestamp (seconds) converted to ms
- or slot-based heuristics

Do not require exact ms equality; accept minor drift.

---

## 6. Off-chain Allocation Requirements

Signer agent must allocate sequences safely.

### 6.1 Durability
Sequence allocation MUST be durable before signatures are returned.
If a crash occurs after signing but before persistence, a sequence might be reused.

Recommended:
- allocate and persist sequence first (atomic transaction)
- then sign and record signature event linked to that sequence

### 6.2 Atomic allocation
Use a single-writer or atomic increment approach:
- Postgres row lock
- Redis INCR with persistence guarantees
- dedicated allocator service

### 6.3 Multi-instance coordination
If multiple signer agents exist:
- they MUST share the allocator
- they MUST not allocate sequences independently

### 6.4 Audit log
Every signature event must record:
- signer_set_id
- market_id
- epoch_id
- sequence
- bundle_hash
- signer pubkey used
- timestamp and request id
- caller identity

This supports incident response and replay debugging.

---

## 7. Idempotent Submission and Retries

Transaction submission is unreliable; retries must not create duplicates.

### 7.1 Commit idempotency
Commit should be idempotent by:
- using the same commitment PDA for the same target
- rejecting commits if a commitment already exists (unless overwrite policy is enabled)

### 7.2 Reveal idempotency
Reveal should be idempotent by:
- using reveal PDA derived from (epoch, bundle_hash)
- allowing retries that write the same data

### 7.3 Client retry strategy
Submitter should:
- use blockhash refresh
- backoff retries
- check for existing on-chain state before retrying
- avoid creating new sequences for the same bundle unless necessary

---

## 8. Acceptance Bounds (Recommended Defaults)

These values should be configurable per market in registry.

- `future_skew_ms`: 15_000
- `max_commit_staleness_ms`: 600_000 (10 minutes) for fast markets, larger for slow markets
- `max_reveal_delay_ms`: 600_000 (10 minutes)
- `finalization_delay_ms`: 120_000 (2 minutes)

For slower domains (politics/macro):
- staleness and reveal windows can be larger (hours)

---

## 9. Failure Modes

### 9.1 Sequence reuse
Symptoms:
- on-chain rejects reveal/commit due to replay violation
- signatures appear valid but rejected due to sequence

Mitigation:
- fix allocator
- rotate signer set if necessary
- add monitoring on sequence monotonicity

### 9.2 Cross-market replay attempts
Symptoms:
- signature verifies if message not bound to market_id
- consumer sees unexpected acceptance

Mitigation:
- enforce market_id binding in signature message
- add tests ensuring verification fails across markets

### 9.3 Out-of-order acceptance issues
If monotonic enforcement is strict:
- delayed commits may be rejected

Mitigation:
- ensure submitter ordering
- increase staleness bounds
- consider sliding window tracking if necessary

---

## 10. Implementation Checklist

On-chain program:
- [ ] include market_id + epoch_id in signature message verification
- [ ] verify signer pubkeys belong to signer set
- [ ] enforce min signature threshold with distinct pubkeys
- [ ] store and enforce last_sequence according to policy
- [ ] reject stale/future commits
- [ ] use PDA uniqueness for reveals

Signer agent:
- [ ] durable atomic sequence allocation
- [ ] record audit logs for every signing
- [ ] authenticate and authorize signing requests
- [ ] expose health metrics and alerts

SDK:
- [ ] compute signature message deterministically
- [ ] verify signatures and threshold
- [ ] verify bundle hash matches payload
- [ ] surface replay context in API responses

---

## 11. Test Plan

### 11.1 Unit tests
- signature message construction test vectors
- monotonic sequence enforcement
- duplicate pubkey suppression
- PDA derivation stability

### 11.2 Integration tests (localnet)
- commit + reveal success path
- duplicate commit rejection or idempotent behavior
- duplicate reveal idempotency
- replay attempt with same sequence rejected
- cross-market replay attempt rejected

### 11.3 Chaos tests (optional)
- simulate submitter retry storms
- simulate signer agent failover
- simulate delayed network confirmations

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
