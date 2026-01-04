
# Key Management (Ops)

This document specifies key management for M0Club (M0-CORE).
It covers how keys are generated, stored, rotated, and used across:
- Solana program authorities
- signer agents (oracle bundle signing)
- submitter identities (transaction fee payer)
- service-to-service authentication keys
- CI/CD signing and artifact integrity keys (optional)

This is a security-critical operational document.
All production deployments must treat private keys as high-risk assets.

---

## 1. Goals

- Minimize exposure of private keys and signing material.
- Enforce separation of duties (roles and scopes).
- Support rotation and emergency response.
- Provide auditable, reproducible configuration for signer sets.
- Provide secure local-dev workflow without leaking production secrets.

Non-goals:
- Publishing proprietary keys or encouraging unsafe key handling.
- Replacing formal security reviews and compliance requirements.

---

## 2. Key Inventory

M0Club uses multiple categories of keys.

### 2.1 Program authority keys
Used for:
- upgrade authority of Solana programs (if upgradeable)
- program configuration authorities (registry/oracle admin)
- emergency pause authority (guardian) if implemented

Risk:
- highest impact; compromise can rewrite program code or change critical config.

### 2.2 Oracle signer keys (signer set)
Used for:
- signing oracle bundle hashes off-chain
- threshold signatures validated on-chain
- slashing identity (if stake-based)

Risk:
- critical integrity; compromise can forge oracle data if threshold met.

### 2.3 Submitter keys (tx payer / publisher)
Used for:
- sending commit and reveal transactions
- paying fees and compute budget

Risk:
- moderate; compromise can cause spam or unauthorized submissions (program may still reject).

### 2.4 Service identity keys (mTLS / JWT)
Used for:
- service-to-service auth
- signing internal tokens and requests

Risk:
- varies; can enable lateral movement if compromised.

### 2.5 CI/CD signing keys (optional)
Used for:
- signing build artifacts or container provenance
- release integrity (SLSA/cosign)

Risk:
- supply-chain impact.

---

## 3. Security Principles

1) **Least privilege**
- each key has a single role and minimal scope

2) **Separation of duties**
- no single operator should control all critical keys

3) **Hardware-backed where possible**
- use HSM/KMS for production signing

4) **No raw private keys in containers**
- avoid shipping key material inside images

5) **Auditability**
- record key ids, public keys, and rotation events
- store fingerprints and metadata in registry/config

6) **Defense-in-depth**
- network isolation, RBAC, rate limits, monitoring

---

## 4. Recommended Key Storage

### 4.1 Production (recommended)
- Use a managed KMS/HSM:
  - AWS KMS + Nitro Enclaves (advanced)
  - GCP KMS / Cloud HSM
  - Azure Key Vault HSM
- Signer agent uses:
  - remote signing API to KMS
  - or enclave-protected signing process
- Program authority keys:
  - stored offline or in HSM with multi-party control

### 4.2 Staging
- KMS/HSM recommended but can be relaxed.
- Still avoid storing raw keys in plain Kubernetes secrets.

### 4.3 Local development
- Use local keypairs stored on disk under a dev folder.
- Use disposable keys.
- Never reuse production keys in local.

---

## 5. Key Generation

### 5.1 Solana keypairs
Generate with Solana CLI:
```bash
solana-keygen new --no-bip39-passphrase -o ./infrastructure/dev-keys/fee-payer.json
solana-keygen pubkey ./infrastructure/dev-keys/fee-payer.json
```

Signer keys:
```bash
solana-keygen new --no-bip39-passphrase -o ./infrastructure/dev-keys/signer-1.json
solana-keygen new --no-bip39-passphrase -o ./infrastructure/dev-keys/signer-2.json
solana-keygen new --no-bip39-passphrase -o ./infrastructure/dev-keys/signer-3.json
```

### 5.2 Ed25519 keys for signer agents
Solana keypairs are ed25519 and can be used for off-chain signatures if the system standardizes on ed25519.

If separate keys are required:
- generate via libsodium/openssl and store only public keys in config

### 5.3 Key naming and metadata
Each key must have metadata:
- key_id
- role
- environment
- creation timestamp
- operator/owner
- public key fingerprint

Store metadata in a secure inventory document or registry.

---

## 6. Signer Set Management

### 6.1 Signer set definition
A signer set is defined by:
- signer_set_id (u64 or 32-byte id)
- threshold `t`
- list of signer public keys
- activation epoch/time
- expiration epoch/time (optional)
- slashing policy references (optional)

Signer set data must be stored:
- on-chain in a registry program
- and off-chain in a config registry for operators

### 6.2 Threshold policy
Recommended:
- N=5, t=3 for small deployments
- N=7, t=5 for higher assurance

A threshold should tolerate:
- at least one signer outage
- and require multiple signers to collude for compromise

### 6.3 Rotation workflow
Rotation should be planned and rehearsed:
1) Generate new signer keys (or new KMS key ids).
2) Create new signer set on-chain with future activation.
3) Deploy signer agents with new keys in parallel (dual-run period).
4) Switch submitter to request signatures from new set at activation epoch.
5) Monitor for successful publishes.
6) Deactivate old signer set after safe period.
7) Securely revoke/disable old keys.

Rotation must be recorded in audit logs.

### 6.4 Emergency rotation
If compromise suspected:
- pause markets (guardian) if supported
- create new signer set with immediate activation
- disable compromised signer agents and keys
- rotate submitter payer key if needed
- initiate incident response and potentially slashing/dispute workflow

---

## 7. Program Authority Management

### 7.1 Upgrade authority
If programs are upgradeable:
- the upgrade authority must be protected by multi-party control.
Options:
- multisig (Squads or similar)
- offline hardware wallets
- HSM-backed workflows with approval gates

Recommended:
- store upgrade authority in a multisig with strict policies.

### 7.2 Admin and guardian authorities
Registry/oracle admin keys should be:
- separate from upgrade authority
- separate from signer keys

Pause authority should be:
- stored in a controlled multisig
- used only under guardrail incidents

### 7.3 Authority rotation
Authority rotation is a program-level operation:
- requires on-chain instruction to set new authority
- should be rehearsed on devnet/staging first

---

## 8. Submitter Key Management

Submitter keys are used for transactions.

Recommendations:
- use a dedicated fee payer per environment
- keep minimal SOL balance necessary for operations
- monitor balances and top up via secured processes
- rotate periodically or after suspicious activity

In production:
- consider using a custody service or HSM for the submitter payer key
- restrict submitter pod permissions and network access

---

## 9. Service-to-Service Authentication Keys

If services use mTLS:
- issue short-lived certs via cert-manager
- rotate automatically
- restrict SANs and namespaces

If services use JWT:
- sign tokens with rotating keys
- store signing keys in KMS
- issue short TTL tokens

---

## 10. Secrets Handling in Kubernetes

### 10.1 Never embed secrets in images
- do not commit keys
- do not bake keys into Docker images

### 10.2 Use external secret managers
- External Secrets Operator recommended

### 10.3 Mounting key material
If key files must be mounted:
- mount as read-only volume
- restrict file permissions
- run pods as non-root
- isolate on dedicated nodes for signer agents

---

## 11. Backup and Recovery

### 11.1 Key backups
Production signer keys should not require raw backups if using KMS.
If using raw keys:
- store encrypted backups offline
- require multi-party decryption (Shamir secret sharing recommended)

### 11.2 Disaster recovery
Maintain documented recovery steps:
- restore database state
- restore registry configs
- restore signer availability
- confirm publish pipeline integrity

Never store unencrypted private keys in shared backups.

---

## 12. Monitoring and Alerts

Monitor:
- signer availability and latency
- signature threshold misses
- unexpected signer pubkey changes
- failed signature verification on-chain
- unusual submitter tx patterns
- key usage logs from KMS (if available)

Alert on:
- repeated failed signature checks
- signer divergence anomalies
- sudden increase in publish failures
- unexpected admin authority changes

---

## 13. Incident Response Playbook (High Level)

If key compromise suspected:
1) Pause affected markets if supported.
2) Rotate signer set immediately.
3) Disable compromised keys in KMS or revoke access.
4) Rotate submitter payer key and drain funds.
5) Rotate service auth keys if lateral movement suspected.
6) Audit logs and publish incident report.
7) Consider dispute/correction workflow for impacted outputs.

---

## 14. Local Dev Safe Defaults

- keep dev keys under `infrastructure/dev-keys/`
- add dev key paths to `.gitignore`
- use `.env.local` for local paths
- regenerate keys often
- never copy production config or keys into local

---

## 15. Example Configuration Snippets

### 15.1 Signer set config (conceptual)
```toml
signer_set_id = 1
threshold = 3

[[signers]]
name = "signer-1"
pubkey = "..."

[[signers]]
name = "signer-2"
pubkey = "..."

[[signers]]
name = "signer-3"
pubkey = "..."
```

### 15.2 Signer agent env vars (conceptual)
```bash
M0_SIGNER_SET_ID=1
M0_SIGNER_KEY_SOURCE=kms
M0_KMS_KEY_ID=projects/.../locations/.../keyRings/.../cryptoKeys/...

# If file-based (dev only):
M0_SIGNER_KEYPAIR_PATH=/keys/signer-1.json
```

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
