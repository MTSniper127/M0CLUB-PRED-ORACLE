
# Bundle Hashing Specification (M0-CORE)

This document specifies canonical bundle hashing for M0Club (M0-CORE).
Bundle hashing is the integrity root for commit-reveal, signature verification, replay protection, and consumer validation.

This spec defines:
- canonical bundle serialization
- hashing algorithm and domain separation
- signature message construction
- merkle mode for large bundles
- determinism constraints and test vectors
- implementation guidance for Rust and SDKs

---

## 1. Goals

- Provide a single canonical byte representation for an OracleBundle.
- Ensure identical bundle_hash across all implementations for the same logical bundle.
- Prevent ambiguity, malleability, and replay across markets/epochs.
- Support chunked or merkle-based reveals when bundles exceed limits.
- Enable consumers to verify integrity and signatures deterministically.

Non-goals:
- Providing confidentiality for proprietary features (use features_hash commitments instead).
- Publishing raw proprietary datasets.

---

## 2. Definitions

- **OracleBundle**: the complete payload published via commit-reveal (see oracle-output-format.md).
- **Canonical bytes**: deterministic serialization bytes used for hashing and signing.
- **bundle_hash**: sha256 hash of canonical bytes (with hash field zeroed).
- **signer_set_id**: identifier of active signer set used for signing.
- **sequence**: replay-protection sequence included in signature message.

---

## 3. Canonical Serialization

### 3.1 Canonicalization principles
Canonical serialization MUST:
- have a single valid encoding for any logical bundle
- define stable field ordering
- avoid platform-dependent types (float, map iteration order)
- use fixed-endian integers
- use explicit length prefixes
- use ASCII for ids where required

Recommended encoding:
- custom binary encoding (“M0BUNDLE”) versioned by schema_version.

### 3.2 Schema version
The first bytes of canonical bytes must include:
- magic prefix: `M0BUNDLE`
- schema_version: u16 little-endian

Example header:
- bytes: `b"M0BUNDLE" || u16le(schema_version)`

### 3.3 Field ordering
Canonical bytes include fields in a fixed order:
1) header (magic + schema_version)
2) bundle metadata fields:
   - bundle_id
   - created_at_ms
   - signer_set_id
   - publish_epoch_id
   - commit_reveal_mode
3) markets list (sorted)
4) signatures list (sorted)
5) integrity footer (optional)

### 3.4 Sorting rules
To avoid malleability:
- markets must be sorted by `market_id` ascending ASCII
- within a market:
  - outcomes sorted by `outcome_id` ascending ASCII
- signatures sorted by signer pubkey bytes ascending
- if merkle mode uses chunks, chunks are ordered by chunk_index

### 3.5 Integer encoding
- u16/u32/u64 little-endian
- signed integers use i64 little-endian two’s complement
- fixed-point values are integers with declared scale

### 3.6 String encoding
- ASCII only for protocol identifiers
- encode as: `u16 length` + bytes
- length limits enforced (e.g., 0..256)

---

## 4. Hashing Algorithm

### 4.1 bundle_hash
`bundle_hash = sha256(canonical_bundle_bytes_for_hashing)`

### 4.2 Hash field zeroing
If the bundle includes an integrity section that contains `bundle_hash`:
- that field must be treated as zero bytes during hashing
This prevents self-referential mismatch.

Recommended:
- do not include bundle_hash inside hashed bytes at all; store it as derived.

If stored, define:
- in canonical bytes for hashing, the bundle_hash field is 32 zero bytes.

### 4.3 Domain separation
All hashes must use domain separation prefixes to avoid cross-context collisions.

Recommended:
- `bundle_hash = sha256("M0_BUNDLE_HASH_V1" || canonical_bundle_bytes_without_signatures)`

However, if signatures must be included or excluded, it must be explicit.
Recommended v1:
- hash the bundle content WITHOUT signatures
- signatures attest to the hash

Thus:
- `bundle_content_hash = sha256("M0_BUNDLE_CONTENT_V1" || canonical_content_bytes)`
- bundle_hash = bundle_content_hash

### 4.4 Content vs full hash
Define:
- **content bytes**: canonical bundle data excluding signatures
- **content hash**: sha256 of content bytes with domain separation

Signatures should always sign the content hash, not the full bytes including signatures.

---

## 5. Signature Message Construction

Signers must sign a deterministic message that binds:
- market scope (market_id list or publish_epoch scope)
- epoch_id
- signer_set_id
- sequence
- bundle_content_hash

Recommended message hash:
`sig_msg = sha256("M0_ORACLE_SIG_V1" || signer_set_id_le || publish_epoch_id_le || sequence_le || bundle_content_hash)`

Notes:
- signer_set_id should be a u64 or fixed 32-byte id; choose one and make it stable.
- publish_epoch_id binds the signed hash to a specific epoch.
- sequence provides replay protection (see replay-protection.md).

Signature format:
- ed25519 signatures over `sig_msg` bytes (or over the hash depending on library)
- must store signer pubkey and signature bytes in bundle.

Verification:
- validate signer pubkeys in active signer set
- require distinct pubkeys
- require threshold `t` signatures

---

## 6. Merkle Mode for Large Bundles

When a bundle exceeds on-chain reveal size limits, M0-CORE can use merkle mode.

### 6.1 Chunking
Split canonical content bytes into chunks:
- fixed chunk size (e.g., 900 bytes) or dynamic based on tx limits
- each chunk has:
  - chunk_index (u32)
  - chunk_bytes
  - chunk_hash = sha256("M0_CHUNK_V1" || u32le(chunk_index) || chunk_bytes)

### 6.2 Merkle root
Compute merkle root over ordered chunk_hashes:
- leaves: chunk_hashes in ascending chunk_index
- inner nodes: sha256("M0_MERKLE_NODE_V1" || left || right)
- if odd number of leaves, duplicate last leaf (or define padding rule)

The bundle_content_hash becomes:
- `bundle_content_hash = merkle_root` in merkle mode
- signatures sign the merkle root

### 6.3 Reveal process
Reveal can publish:
- root + chunk count
- then publish chunks across multiple transactions
- on-chain program verifies chunk hashes and reconstructs root or verifies membership

### 6.4 Consumer verification
Consumers can:
- reconstruct canonical content bytes from chunks
- verify merkle root equals signed root
- parse content bytes to extract market outputs

---

## 7. Determinism Requirements

To guarantee stable hashes:
- canonical ordering is mandatory
- no maps without sorting
- fixed-point scaling constants are protocol constants
- string casing rules for ids are enforced
- use explicit endianness
- exclude non-deterministic metadata from hashed content

Do not include:
- local timestamps unrelated to created_at_ms
- random ids unless derived deterministically
- unordered JSON fields

---

## 8. Test Vectors

Provide test vectors for:
- canonical encoding of a small bundle
- expected content hash
- expected signature message hash
- merkle mode chunk hashes and root

Store under:
- `core-engine/m0-common/test-vectors/bundle-hashing/`
- replicated in SDKs:
  - `sdk/ts/test-vectors/bundle-hashing/`
  - `sdk/rust/test-vectors/bundle-hashing/`
  - `sdk/python/test-vectors/bundle-hashing/`

CI should validate:
- recomputed hashes match expected
- signature verification passes
- canonical encoding stable across platforms

---

## 9. Implementation Guidance

### 9.1 Rust
- implement canonical encoding in a dedicated crate (m0-common)
- avoid serde JSON for hashed bytes
- implement explicit writer that writes little-endian primitives
- implement stable sorting by ASCII bytes

### 9.2 TypeScript
- implement a byte writer using Uint8Array buffers
- avoid object iteration without sorting keys
- match Rust encoding exactly and test vectors

### 9.3 Python
- implement byte writer using `struct.pack("<Q")` etc.
- ensure stable string encoding and length prefixes
- validate with test vectors

---

## 10. Security Notes

- Never sign unhashed large payload bytes directly if library ambiguity exists; sign the message hash or canonical message bytes.
- Ensure signer keys never leave signer boundary.
- Ensure hash domain separation strings are constant and versioned.
- Reject bundles with unknown schema_version unless explicitly supported.

---

## Links

- Website: https://m0club.com/
- X (Twitter): https://x.com/M0Clubonx
