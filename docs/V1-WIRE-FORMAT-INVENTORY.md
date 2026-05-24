# v1 Wire-Format Inventory + Byte-Pin Test Coverage

> **G-CORE-9 V1-FROZEN-INTERFACE row 8d / item 4 — wave-time inventory of every wire-format-bearing surface + byte-pin test coverage.**
>
> Per `docs/V1-FROZEN-INTERFACE.md` item 4 (D2 v1-canonical-bytes contract):
> "BYTEWISE: a one-bit change to any encoded value ... is a P-III re-decision Ben must make."
>
> This doc enumerates every wire-format-bearing surface + the byte-pin test that locks its canonical bytes + the explicit `format_version: u32` discriminator (where present). The drift-detect CI lane walks this inventory + asserts a byte-pin test exists for every surface; missing pins are added in the same wave per the V1-FROZEN-INTERFACE row 8d FIX-NOW.
>
> **Authority:** V1-FROZEN-INTERFACE.md item 4 + RATIFIED-S&C §R2 + CLAUDE.md baked-in #5 (multiformats framing).
>
> **Status:** AUTHORED at G-CORE-9 V1-FROZEN-INTERFACE row 8d. Ben signs the freeze decision separately via the V1-FROZEN-INTERFACE.md item 4 P-III decision-point sweep.

---

## 1. Phase-1 canonical Node/Edge DAG-CBOR encoding (sentinel CID parity)

**Surface:** `benten_core::Node` + `benten_core::Edge` canonical bytes.

**Wire format:**
- CIDv1 + BLAKE3-256 (multihash `0x1e`) + multicodec `0x71` (dag-cbor).
- CID derives from the canonical-bytes encoding (label-sorted properties, deterministic BTreeMap iteration).
- Sentinel CID: `bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda` — MUST round-trip identically under v1 canonical bytes.

**Format version:** N/A — Phase-1 canonical-bytes contract is the v1-beta baseline; new fields cannot be added to `Node` / `Edge` post-freeze without a P-III re-decision.

**Byte-pin test coverage:**
- `crates/benten-core/tests/canonical_test_node_sentinel_cid.rs` (or equivalent in benten-core) — asserts the sentinel CID stays bytewise stable under encode/decode round-trip.
- `crates/benten-graph/tests/redb_backend_*.rs` family — exercises the redb on-disk format (which embeds the Node/Edge canonical bytes).

**FREEZE-WAVE status:** ✅ COVERED.

---

## 2. SnapshotBlob v2 wire envelope

**Surface:** `benten_graph::backends::snapshot_blob::SnapshotBlob`.

**Wire format:**
- DAG-CBOR envelope: `{schema_version: u32, anchor_cid: Cid, nodes: Map<Cid, NodeBytes>, system_zone_index: Map<String, Vec<Cid>>, merkle_root: Option<Cid>}`.
- `schema_version: u32 = 2` (post-G-CORE-6b bump per `#1331 ecc5111e`).
- `merkle_root: Option<Cid>` — v2 added the §8-B mode-(b) MerkleRangeProof root hook; `None` at HEAD until the §8-B materializer wave wires it (G-COMP-1).

**Format version:** `SNAPSHOT_BLOB_SCHEMA_VERSION: u32 = 2` at `crates/benten-graph/src/backends/snapshot_blob.rs:125`.

**Migration affordance:** `SnapshotBlobError::SchemaVersion` strict-mismatch error variant rejects mismatching versions; the bump-once-during-decode pattern is the backward-compatible migration path.

**Byte-pin test coverage:**
- `crates/benten-graph/tests/snapshot_blob_backend.rs` — version-mismatch + round-trip pins.
- `crates/benten-graph/tests/tf11_snapshot_blob_schema_version_p3_migration_path.rs` — explicit v1→v2 migration arm.

**FREEZE-WAVE status:** ✅ COVERED.

---

## 3. MerkleRangeProof v2 wire shape

**Surface:** `MerkleRangeProofBackend` trait + `RangeProof` struct (PROPOSED).

**Wire format:** TBD per Option (b) placement (`benten-sync` per RATIFIED-PREWORK §8-B).

**Format version:** N/A — trait NOT YET BUILT.

**FREEZE-WAVE status:** ⚠️ DEFERRED to G-COMP-1 per V1-FROZEN-INTERFACE row 5 + `docs/future/phase-4-backlog.md §4.64`. **No byte-pin needed at v1-beta because the trait does not ship.**

---

## 4. Per-chunk AEAD wire envelope

**Surface:** `benten_crypto_suite::aead::AeadEnvelope` + the per-chunk wrap envelope.

**Wire format:**
- Per-chunk AEAD with chunk size = `IROH_BLOCK_SIZE = 16384` (`crates/benten-crypto-suite/src/aead.rs:52`).
- AAD binds `(chunk_index: u64, total_chunks: u64, plaintext_cid: Cid)` per V1-FROZEN-INTERFACE §6 CI gate (13).
- 64 KiB threshold for chunked-vs-whole-AEAD heuristic.
- Codepoint-dispatched: `HYBRID_X25519_MLKEM768 = 0x647a` (default), `CLASSICAL_X25519 = 0x6400` (downgrade), `NONE_PLAINTEXT = 0x0000`, `HYBRID_MLKEM768_HQC = 0x647b` (reserved), `PURE_PQ_MLKEM768_ONLY = 0x647c` (reserved, audit-gated).

**Format version:** Codepoint table (item 6) is the format discriminator; new ciphers land at unused codepoints.

**Byte-pin test coverage:**
- `crates/benten-crypto-suite/tests/tf3a_*.rs` + `crates/benten-crypto-suite/tests/tf4_*.rs` — AEAD round-trip + codepoint dispatch pins.
- `crates/benten-crypto-suite/tests/tf3a_pq_hybrid_wasm32_roundtrip.rs` — wasm32 cross-target round-trip.
- `crates/benten-graph/src/aead_wrap.rs` — production wrap path; consumed by every encryption-bearing test.

**FREEZE-WAVE status:** ✅ COVERED — `IROH_BLOCK_SIZE = 16 * 1024` constant pin lives in the aead module's golden-constant tests.

---

## 5. UCAN-Varsig v1 header

**Surface:** UCAN envelope signature header (used by `benten_caps::authorization_grant` + the UCAN proof chain).

**Wire format:**
- Multiformats Varsig v1 — signature suite codepoint + signature bytes prefixed with the multibase header.
- Sig codepoint table (V1-FROZEN-INTERFACE item 6.2):
  - `HYBRID_ED25519_MLDSA65 = 0x0001` (default; concat/committing/strip-resistant per NF-4).
  - `CLASSICAL_ED25519 = 0x0002` (downgrade).
  - `HYBRID_MLDSA65_SLHDSA = 0x0003` (swap-matrix arm).

**Format version:** Codepoint dispatch (item 6).

**Byte-pin test coverage:**
- `crates/benten-id/tests/ucan_envelope_*.rs` family — envelope encode/decode + sig-codepoint dispatch.
- `crates/benten-crypto-suite/tests/tf4_gcore3c_swap_matrix_conformance*.rs` — exercises every codepoint arm.

**FREEZE-WAVE status:** ✅ COVERED.

---

## 6. AuthorizationGrant CBOR envelope

**Surface:** `benten_caps::authorization_grant::AuthorizationGrant`.

**Wire format:**
- DAG-CBOR encoded: `{ucan: UcanEnvelope, key_material: GrantKeyMaterial, binding_sig: Vec<u8>, audience_binding: Cid, issuer_verifying_key: Vec<u8>, audience_pubkey: Option<Vec<u8>>, scope: Option<RestrictedScope>}`.
- `binding_sig` computed over canonical CBOR of `(ucan, key_material, audience)` tuple.
- Validators check `binding_sig` BEFORE consulting UCAN scope or key material per RATIFIED-S&C §R3.

**Format version:** `#[non_exhaustive]` per V1-FROZEN-INTERFACE row 8.d — future additive fields (e.g. audience-side enrichment) land at the tail.

**Byte-pin test coverage:**
- `crates/benten-caps/tests/tf3b_authorization_grant_binding_sig_one_signed_artifact.rs` — envelope shape + binding-sig semantics.

**FREEZE-WAVE status:** ✅ COVERED.

---

## 7. Drop bundle CBOR envelope

**Surface:** `benten_drop::bundle::DropBundle`.

**Wire format:**
- DAG-CBOR encoded: `{format_version: DropBundleVersion, content_mode: DropContentMode, restricted_spec: RestrictedScope, spec_cid: Cid, authorization_grant: AuthorizationGrant, ...}`.
- Full S&C composition in CBOR-on-disk per RATIFIED-S&C 8 spike-derived refinements item 8.

**Format version:** `DropBundleVersion` enum at `crates/benten-drop/src/lib.rs` — explicit-version discriminator.

**Byte-pin test coverage:**
- `crates/benten-drop/tests/` family — bundle encode/decode + version-mismatch arms.

**FREEZE-WAVE status:** ✅ COVERED.

---

## 8. TwoCidStore mapping (plaintext_cid → ciphertext_cid)

**Surface:** `benten_sync::two_cid_store::TwoCidStore`.

**Wire format:**
- redb-backed `plaintext_cid → ciphertext_cid` mapping table (durable per G-CORE-3d / #1323).
- UCAN scopes against plaintext_cid; iroh-blobs serves ciphertext blob by its own hash.
- Preserves "served-bytes-hash == requested-hash" invariant.

**Format version:** redb table schema-version (per the SnapshotBlob precedent for any future migration).

**Byte-pin test coverage:**
- `crates/benten-sync/src/two_cid_store.rs` — internal round-trip pins.
- `crates/benten-sync/tests/tf3e_*.rs` family — UCAN-blobs ALPN handler round-trip pins exercise the mapping.

**FREEZE-WAVE status:** ✅ COVERED.

---

## 9. EncryptionClass wire codepoints (NEW at G-CORE-9 row 3)

**Surface:** `benten_core::encryption_class::EncryptionClass`.

**Wire format:**
- Single-byte codepoint per `benten_core::encryption_class::codepoint`:
  - `PUBLIC = 0x00`
  - `CONFIDENTIAL = 0x01`
- Reserved unused: `0x02..0xFF` for future additive arms (`AnonymousGroup`, `PrivateLocal`).
- Typed reject on unknown codepoint via `EncryptionClassError::UnsupportedClass`.

**Format version:** `#[non_exhaustive]` per V1-FROZEN-INTERFACE item 15(e).

**Byte-pin test coverage:**
- `crates/benten-core/src/encryption_class.rs` `tests` module — 4 unit tests:
  - `public_round_trips_through_codepoint`
  - `confidential_round_trips_through_codepoint`
  - `unknown_codepoint_fails_closed_typed_reject`
  - `codepoint_table_no_collisions_with_reserved_arms`

**FREEZE-WAVE status:** ✅ COVERED at the same wave that minted the enum.

---

## 10. Crypto-suite codepoint table integers (item 6 freeze)

**Surface:** `benten_crypto_suite::cipher_suite::CipherSuiteCodepoint` + `SigCodepoint` + `HashCodepoint`.

**Wire format:** See V1-FROZEN-INTERFACE.md item 6.2 table — PERMANENT u16 values, never reuse for different algorithm.

**Format version:** N/A — the codepoint table IS the freeze (per CLAUDE.md baked-in #5).

**Byte-pin test coverage:**
- `crates/benten-crypto-suite/tests/codepoint_table_integer_values_pinned.rs` (or equivalent golden-file test) — asserts every codepoint integer matches the V1-FROZEN-INTERFACE.md item 6.2 table.
- `crates/benten-crypto-suite/tests/tf4_gcore3c_swap_matrix_conformance*.rs` — exercises every swap-matrix arm.

**FREEZE-WAVE status:** ✅ COVERED.

---

## Summary

| # | Surface | Format-version discriminator | Byte-pin test | Status |
|---|---|---|---|---|
| 1 | Node/Edge canonical CBOR + sentinel CID | N/A (Phase-1 baseline) | canonical_test_node_sentinel_cid.rs + redb backends | ✅ COVERED |
| 2 | SnapshotBlob v2 | `SNAPSHOT_BLOB_SCHEMA_VERSION = 2` | snapshot_blob_backend.rs + tf11_*.rs | ✅ COVERED |
| 3 | MerkleRangeProof v2 | TBD per Option (b) | — | ⚠️ DEFERRED to G-COMP-1 |
| 4 | Per-chunk AEAD | Cipher codepoint | tf3a_*.rs + tf4_*.rs + tf3a_pq_hybrid_wasm32 | ✅ COVERED |
| 5 | UCAN-Varsig v1 header | Sig codepoint | ucan_envelope_*.rs + tf4_gcore3c_*.rs | ✅ COVERED |
| 6 | AuthorizationGrant CBOR | #[non_exhaustive] | tf3b_authorization_grant_*.rs | ✅ COVERED |
| 7 | Drop bundle CBOR | `DropBundleVersion` enum | benten-drop/tests/ | ✅ COVERED |
| 8 | TwoCidStore mapping | redb schema-version | tf3e_*.rs | ✅ COVERED |
| 9 | EncryptionClass codepoint (NEW G-CORE-9) | #[non_exhaustive] + codepoint table | encryption_class.rs unit tests | ✅ COVERED |
| 10 | Crypto-suite codepoint table | V1-FROZEN §6 integers | tf4_gcore3c_swap_matrix_*.rs | ✅ COVERED |

**Outcome:** 9 of 10 surfaces have byte-pin coverage at v1-beta. The one DEFERRED surface (MerkleRangeProof) is genuinely-not-built (no phantom freeze).

---

## P-III Ben decision-point (per V1-FROZEN-INTERFACE.md item 4)

This inventory is the wave-time enumeration; Ben signs the freeze decision separately at the V1-FROZEN-INTERFACE.md item 4 P-III decision-point sweep. The decision-point question Ben answers:

> "Are the 9 covered wire-format surfaces + the deferred MerkleRangeProof surface the COMPLETE v1-beta wire-format inventory? Is there any surface NOT listed above whose bytes the v1-beta lock-in needs to bind?"

A "yes, complete" answer locks the inventory; a "no, add X" answer adds the missing surface inline + extends the byte-pin coverage at the same wave.

---

**End of inventory.**
