
# Signer Set and Key Policy (Protocol Spec)

This document specifies the signer set model used by M0Club for authenticating oracle updates.
Signer sets define which keys may sign oracle bundle commitments, the threshold required, replay protection rules, and rotation procedures.

This spec is intended to map directly to:
- on-chain registry/oracle program verification logic
- off-chain signer agent policy enforcement
- SDK verification routines
- operational runbooks for rotation and incident response

---

## 1. Goals

- Authenticate oracle outputs with an auditable signer policy.
- Reduce single-key compromise risk via multi-signer thresholds.
- Enable safe, controlled signer set rotation.
- Provide replay protection to prevent signature reuse.
- Support production-grade key storage (KMS/HSM).

Non-goals:
- Implementing threshold cryptography (MPC) in v1.
- Exposing private key material beyond signer boundary.

---

## 2. Signer Set Model

### 2.1 Definitions
- **Signer**: A public key authorized to sign oracle bundle hashes.
- **Signer Set**: A collection of signers plus policy metadata.
- **Threshold**: Minimum number of distinct signers required for acceptance.
- **Sequence**: A replay protection counter included in the signature message.
- **Signer Agent**: Off-chain service responsible for key usage, signing, and policy enforcement.

### 2.2 Required properties
A signer set MUST define:
- `signer_set_id` (u32)
- `pubkeys[]` (ed25519, 32 bytes each)
- `min_signatures` (u8)
- `sig_scheme` (enum, default ed25519)
- `sequence_policy` (GLOBAL or PER_MARKET recommended)
- `created_at_slot` / `updated_at_slot` (optional metadata)
- `status` (ACTIVE/DEPRECATED recommended)

### 2.3 Allowed signature scheme
v1 supports:
- ed25519 (`sig_scheme = 0`)

Future options may include:
- secp256k1
- BLS/threshold schemes
- post-quantum schemes (research)

---

## 3. Signature Message Format

### 3.1 Domain separation
Every signature message MUST include a fixed ASCII prefix to prevent cross-protocol replay.

Recommended prefix:
- `M0CLUB_ORACLE_V1`

### 3.2 Required bindings
Signatures MUST bind at least:
- `bundle_hash` (32 bytes)
- `signer_set_id` (u32 LE)
- `sequence` (u64 LE)

Signatures SHOULD ALSO bind:
- `market_id` bytes
- `epoch_id` bytes

Recommended message construction:
`msg = sha256(prefix || market_id || epoch_id || bundle_hash || signer_set_id_le || sequence_le)`

Notes:
- `market_id` and `epoch_id` bindings prevent cross-market replay if the same signer set is reused.
- The exact byte layout MUST be stable across SDKs and implementations.

### 3.3 Signature validation requirements
On-chain and off-chain validators MUST check:
- signature is valid for `msg`
- signer pubkey exists in signer set
- signer pubkeys are distinct (no double counting)
- signature count >= min_signatures

---

## 4. Replay Protection (Sequence Policy)

Replay protection ensures an attacker cannot reuse old signatures or re-submit old bundles as new updates.

### 4.1 Sequence field
A `sequence` is a u64 value included in the signature message.
The sequence must be unique according to the sequence policy.

### 4.2 Policies
Define one of:

- **GLOBAL**
  - One global monotonically increasing sequence per signer_set_id.
  - Strongest protection and simplest reasoning.
  - Requires a shared counter across signers/signer agent instances.

- **PER_MARKET**
  - Sequence increases monotonically per (signer_set_id, market_id).
  - Lower coordination scope.
  - Strong protection if market_id is included in signature message.

- **PER_EPOCH**
  - Sequence scoped to epoch. Generally discouraged.
  - More vulnerable to replay across epochs unless additional bindings exist.

Recommended default:
- GLOBAL, or PER_MARKET if scaling requires it.

### 4.3 On-chain enforcement strategies
On-chain replay tracking options:

1) **Monotonic counter**
- Store `last_sequence` and require `sequence > last_sequence`.
- Simple but requires ordered updates.

2) **Windowed bitmap**
- Track a sliding window of sequence values to allow out-of-order updates.
- More complex but more tolerant to network variance.

3) **Commitment keyed by sequence**
- Derive commitment account PDA including `sequence`.
- Prevent duplicates naturally through account uniqueness.
- Still need policy to bound acceptable sequences.

Recommended v1:
- Monotonic counter when updates are ordered.
- PDA uniqueness plus staleness bounds for safety.

### 4.4 Off-chain enforcement
Signer agent MUST ensure:
- the sequence is allocated once and never reused
- allocation is durable (persisted) before signing
- multi-instance deployments coordinate sequence allocation

Suggested approach:
- database-backed allocator (Postgres/Redis) with atomic increments
- per signer set table keyed by signer_set_id and optionally market_id

---

## 5. Signer Agent Requirements

### 5.1 Isolation boundary
Signer agent should be deployed as a separate service with:
- strict network policies
- minimal outbound access
- separate credentials and secrets scopes

### 5.2 Key storage
Recommended options (in order):
- HSM-backed KMS (cloud or dedicated HSM)
- Cloud KMS keys with signing API
- Encrypted disk keys (development only)

Never:
- store plaintext private keys in repository
- log private keys
- expose key material via API

### 5.3 Signing API (conceptual)
Signer agent can expose:
- `POST /v1/sign`
  - input: `bundle_hash`, `market_id`, `epoch_id`, `signer_set_id`, `sequence`
  - output: `pubkey`, `sig`, `sig_scheme`, `sequence`

The API must:
- authenticate callers (mTLS/service identity)
- authorize which caller may sign for which market
- apply rate limiting
- record audit logs for every signing action

### 5.4 Audit log requirements
Every signing event MUST record:
- timestamp
- request id
- market_id, epoch_id
- bundle_hash
- signer_set_id, sequence
- signer pubkey used
- caller identity
- result (success/failure)

Audit logs should be immutable and searchable.

---

## 6. Rotation Procedures

Signer rotation is necessary for:
- scheduled key hygiene
- compromise response
- policy changes (threshold, pubkeys)

### 6.1 Rotation types
- **Additive rotation**: add new keys, increase threshold gradually
- **Replace rotation**: replace entire signer set id
- **Emergency rotation**: immediate replacement due to compromise

### 6.2 Recommended rotation process (non-emergency)
1) Create new signer set with new keys (signer_set_id_new).
2) Publish signer set on-chain via registry authority.
3) Configure signer agent to support both old and new sets.
4) Update market to reference new signer_set_id with timelock.
5) During transition window:
   - accept both signer sets (optional) or only new after activation slot.
6) Deprecate old signer set and remove keys from signer agent.
7) Verify system health, sequence tracking, and consumer compatibility.

### 6.3 Emergency rotation
1) Pause affected markets (guardian action).
2) Rotate signer set id immediately.
3) Invalidate old signer set and revoke keys in KMS/HSM.
4) Resume markets after verification and monitoring.

### 6.4 Compatibility notes
Consumers MUST fetch signer sets from registry and verify against the correct signer_set_id.
SDK caches must be invalidated on signer rotation.

---

## 7. Threshold Policy

### 7.1 min_signatures
`min_signatures` defines how many distinct signer pubkeys must be present.

Recommended:
- At least 2-of-N for production.
- 1-of-N acceptable for early development but not recommended for mainnet.

### 7.2 Duplicate suppression
Validators MUST ensure:
- the same pubkey does not count twice toward threshold
- signatures list is de-duplicated

### 7.3 Ordering
Signatures can be in any order but SHOULD be sorted by pubkey bytes to improve determinism in storage and caching.

---

## 8. On-chain Verification Notes (Solana)

Solana programs typically verify ed25519 signatures via:
- the ed25519 program instruction included in the transaction
- sysvar instructions to introspect and validate the signature instruction

On-chain checks:
- confirm ed25519 instruction matches expected pubkey, message, and signature bytes
- confirm pubkey belongs to signer set account
- enforce threshold and replay policy

Storage recommendation:
- store only minimal signature metadata on-chain (or hash of signatures)
- rely on off-chain availability for full audit when necessary

---

## 9. Failure Modes

### 9.1 Key compromise
- Emergency rotate signer set
- Pause markets if uncertain
- Invalidate compromised keys at KMS/HSM
- Review audit logs and sequence usage

### 9.2 Sequence collision
- If sequence is reused, reveals may be rejected or result in replay vulnerability.
- Fix allocator and enforce monotonic constraints.
- Consider switching to GLOBAL monotonic allocator backed by durable storage.

### 9.3 Partial signer outage
- If min_signatures cannot be met, publishing stalls.
- Mitigate by:
  - larger signer sets
  - lower threshold temporarily (governance) with timelock and monitoring
  - hot standby signers

---

## 10. Implementation Guidance

- Prefer stable on-chain accounts for signer sets:
  - store pubkeys and default threshold
  - include status (ACTIVE/DEPRECATED)
- Ensure signer sets are referenced by id, not by raw pubkeys.
- Include signer_set_id in all bundles and signatures.
- Provide SDK helper:
  - fetch signer set
  - compute signature message
  - verify signatures and threshold
- Provide test vectors:
  - known bundle hash
  - known msg hash
  - known signature(s)

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
