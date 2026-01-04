
# Signer Rotation (Ops)

This document defines the signer rotation procedure for M0Club (M0-CORE).
Signer rotation updates the active signer set used to sign oracle bundle content hashes.
Rotation can be planned (routine) or emergency (compromise/outage).

This procedure assumes:
- an on-chain registry that stores signer sets and activation rules
- signer agents that hold signing keys (preferably in KMS/HSM)
- submitter services that request signatures and publish commit-reveal
- monitoring for signer health and signature verification

---

## 1. Goals

- Rotate signer keys safely without interrupting oracle publishing.
- Maintain threshold signing integrity across the rotation window.
- Provide deterministic activation and rollback paths.
- Preserve auditability (who rotated, what changed, when).
- Support emergency rotation within minutes when required.

Non-goals:
- Defining organization-specific approvals (adapt to your process).
- Replacing formal security incident processes.

---

## 2. Concepts

### 2.1 Signer set
A signer set is defined by:
- `signer_set_id`
- `threshold`
- `pubkeys[]`
- `activation_epoch` or `activation_time_ms`
- optional `expiration_epoch/time`
- policy metadata (slashing references, environment)

### 2.2 Dual-run rotation
During rotation, the system can run both:
- old signer agents (S_old)
- new signer agents (S_new)

The submitter selects which set to use based on activation rules.

### 2.3 Safety window
A safety window is a period where:
- both sets are available
- the system can roll back quickly if issues appear

---

## 3. Pre-Rotation Checklist (Planned)

Before rotating, verify:
- registry and CLI tooling available
- on-chain programs healthy
- current signer set stable (threshold success rate normal)
- monitoring dashboards accessible
- operator has correct permissions (multisig approvals if needed)
- new signer keys generated and stored securely (KMS/HSM)
- new signer agent images built and tested in staging
- rollback plan prepared

---

## 4. Key Generation and Storage

### 4.1 Production guidance
- Prefer KMS/HSM-backed keys.
- Do not export private key material.
- Ensure each signer agent has least-privilege access to its key.

### 4.2 If file-based keys are required (not recommended for production)
- generate using Solana CLI (ed25519):
```bash
solana-keygen new --no-bip39-passphrase -o signer-new-1.json
```
- store encrypted at rest
- mount as read-only volume
- restrict namespace and node pool

---

## 5. Planned Rotation Procedure

### 5.1 Step 1: Create new signer set (registry)
Create signer set S_new with a future activation epoch/time.

Example (conceptual CLI):
```bash
cargo run -p m0-cli -- signer-set create   --signer-set-id 2   --threshold 3   --pubkey <PUBKEY1>   --pubkey <PUBKEY2>   --pubkey <PUBKEY3>   --activate-epoch 123456
```

If using a multisig for admin:
- submit transaction to multisig and wait for approvals
- record transaction signature for audit

Verify on-chain:
```bash
cargo run -p m0-cli -- signer-set get --signer-set-id 2
```

### 5.2 Step 2: Deploy signer agents with new keys
Deploy signer agents that serve S_new signatures.

Kubernetes example:
- deploy `m0-signer-agent` instances configured with:
  - `M0_SIGNER_SET_ID=2`
  - key source config (KMS key id or key file path)
  - strict network policies allowing inbound only from submitter

Verify signer agent health:
- `/healthz`
- `/metrics`
- signature test endpoint (if provided)

Example (conceptual):
```bash
curl http://m0-signer-agent-2:9000/healthz
```

### 5.3 Step 3: Enable dual-run in submitter (optional)
If submitter supports dual-run:
- it can request signatures from both sets during a shadow period
- it only publishes using S_old until activation

Shadow mode checks:
- verify S_new returns valid signatures for the same bundle hash
- compare signature verification results offline
- ensure signer latency within thresholds

### 5.4 Step 4: Activation
At activation epoch/time:
- submitter automatically switches to S_new
- engine includes `signer_set_id=2` in bundle metadata
- on-chain verification uses S_new pubkeys

Monitor:
- signature threshold success rate
- invalid signature errors (should be zero)
- publish success rate and latency

### 5.5 Step 5: Observe stability window
For N epochs/hours after activation:
- keep old signer agents running but unused
- be ready to roll back

Recommended window:
- staging: 1-2 hours
- production: 6-24 hours depending on risk

### 5.6 Step 6: Deactivate old signer set
After stability window:
- mark S_old as inactive/expired on-chain (if supported)
- scale down old signer agents
- revoke old key access in KMS or destroy keys per policy

Record:
- deactivation tx id
- date/time

---

## 6. Rollback Procedure (Planned)

Rollback should be possible quickly if issues arise after activation.

Conditions:
- invalid signatures
- threshold cannot be met
- publish failures correlated with signer set switch

Steps:
1) Set submitter to use S_old explicitly:
   - config override `M0_ACTIVE_SIGNER_SET_ID=<old>`
2) Confirm publishes succeed with S_old.
3) Investigate S_new issues:
   - key permissions
   - wrong pubkeys in registry
   - mismatched hashing schema versions
4) Fix and reattempt rotation with a new signer set version if necessary.

If the registry allows changing activation rules:
- avoid mutating existing signer sets; create a new one instead for auditability.

---

## 7. Emergency Rotation Procedure

Emergency rotation is required when:
- key compromise suspected/confirmed
- multiple signer agents down and threshold cannot be met
- security policy requires immediate rotation

### 7.1 Step 0: Safety first
Immediately:
- pause affected markets or disable publishing (kill switch)
- escalate to security lead (org dependent)
- preserve evidence (logs, hashes, tx ids)

### 7.2 Step 1: Create new signer set with immediate activation
```bash
cargo run -p m0-cli -- signer-set create   --signer-set-id 99   --threshold 3   --pubkey <NEW1>   --pubkey <NEW2>   --pubkey <NEW3>   --activate-epoch now
```

### 7.3 Step 2: Deploy new signer agents
- deploy signer agents for set 99
- verify health and signature functionality

### 7.4 Step 3: Force submitter to new set
- set `M0_ACTIVE_SIGNER_SET_ID=99`
- restart submitter

### 7.5 Step 4: Resume publishing carefully
- start with a subset of markets
- monitor signature verification and publish success
- expand gradually

### 7.6 Step 5: Revoke compromised keys
- disable KMS key versions
- revoke IAM permissions
- destroy key material if required

---

## 8. Validation Checklist

After switching to a new signer set:
- bundle_content_hash computed correctly
- signatures verify locally and on-chain
- bundle includes correct signer_set_id
- threshold met across markets
- no replay protection errors
- latency within SLO

If any failures:
- rollback or pause until resolved

---

## 9. Common Failure Modes

### 9.1 Pubkeys mismatch
- registry pubkeys do not match signer agent keys
Fix:
- recreate signer set with correct pubkeys

### 9.2 Schema version mismatch
- signer agent signs a different hash than submitter expects
Fix:
- ensure canonical bundle hashing version pinned and consistent

### 9.3 Network policy blocks submitter
Fix:
- update NetworkPolicy to allow submitter -> signer agent traffic

### 9.4 KMS permissions
Fix:
- grant signer agent service account access to correct key version

### 9.5 Threshold too high
Fix:
- lower threshold or increase N
- ensure redundancy

---

## 10. Observability and Alerts

Dashboards:
- signature collection p95 latency
- threshold misses over time
- signer agent health
- invalid signature count
- publish success rate during rotation

Alerts:
- invalid signatures > 0
- threshold miss rate above baseline
- signer agent error rate spikes

---

## 11. Audit Logging

Record:
- who initiated rotation
- signer_set_id old and new
- pubkey list (public)
- threshold change
- activation epoch/time
- deployment changes (image digests)
- config overrides used
- rollback events if any

Store in:
- incident system or rotation log
- optionally, append-only audit store

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
