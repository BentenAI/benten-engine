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
- `crates/benten-core/tests/canonical_bytes_fastpath_stable.rs` + `crates/benten-core/tests/node_cid.rs` — sentinel-CID + canonical-bytes round-trip pins.
- `crates/benten-graph/tests/redb_schema_version_envelope_pin.rs` + `crates/benten-graph/tests/in_memory_backend_equiv_to_redb.rs` + `crates/benten-graph/tests/kvbackend_conformance.rs` — exercise the redb on-disk format (which embeds the Node/Edge canonical bytes) and conformance equivalence.

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
- AAD binds `(plaintext_cid: &[u8], chunk_index: u64, total_chunks: u32)` per `crates/benten-crypto-suite/src/aead.rs::aad_per_chunk` (4-segment layout: `b"benten-aead:chunk:" || plaintext_cid || chunk_index.to_le_bytes() || total_chunks.to_le_bytes()`). The `total_chunks` segment closes the cross-chunk-truncation attack (an attacker truncating a 10-chunk ciphertext to 5 chunks cannot fabricate per-chunk AAD-matching tags). **R6 R1 fix-pass:** the prior G-CORE-9 R1 Fork-1 2-tuple disposition was RETRACTED; code revised to match the spec text. Pinned at `crates/benten-crypto-suite/tests/canonical_bytes_v1_codepoints_and_aad.rs::aad_per_chunk_canonical_layout_pinned` + behavioral pins at `crates/benten-graph/src/aead_wrap.rs::tests::{cross_chunk_truncation_fails, cross_chunk_inflation_fails}`.
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

**Scope clarification (L11-MIN-1 close at R6-FP-D 2026-05-24):** this row scopes the **header bytes** (Varsig codepoint + signature bytes + multibase prefix). The **UCAN BODY bytes that the header signs over** (the UCAN claim CBOR envelope) are covered structurally by §6 (AuthorizationGrant CBOR) for the AuthorizationGrant-borne UCAN and behaviorally by the chain-validation surface (which exercises the body bytes end-to-end via verify-and-walk pinned tests at `crates/benten-caps/tests/`). An explicit body-bytes byte-pin row is intentionally NOT in this v1-beta inventory — the body's canonical-bytes contract follows DAG-CBOR canonicalization (covered by §1 Phase-1 baseline) + the freeze contract is that the UCAN body shape stays additive (`#[non_exhaustive]` per item 11). A future G-COMP-1 row may add explicit body byte-pins; the current behavioral coverage is per Row D-9 wire-format-deferred posture.

**Wire format:**
- Multiformats Varsig v1 — signature suite codepoint + signature bytes prefixed with the multibase header.
- Sig codepoint table (V1-FROZEN-INTERFACE item 6.2):
  - `HYBRID_ED25519_MLDSA65 = 0x0001` (default; concat/committing/strip-resistant per NF-4).
  - `CLASSICAL_ED25519 = 0x0002` (downgrade).
  - `HYBRID_MLDSA65_SLHDSA = 0x0003` (swap-matrix arm).

**Format version:** Codepoint dispatch (item 6).

**Byte-pin test coverage:**
- `crates/benten-crypto-suite/tests/tf3a_ucan_varsig_v1_header_carries_hybrid_signature.rs` + `crates/benten-crypto-suite/tests/tf4_gcore3c_swap_matrix_conformance_additional.rs` — envelope encode/decode + sig-codepoint dispatch.
- Full hex-pinned-bytes UCAN-Varsig header pin DEFERRED to G-COMP-1 per V1-FROZEN-INTERFACE-DEFERRED.md Row D-9.

**FREEZE-WAVE status:** ✅ COVERED (roundtrip + codepoint-dispatch; hex-pin deferred per Row D-9).

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
- `crates/benten-drop/tests/tf3f_drop_bundle_offline_consume.rs` + `crates/benten-drop/src/bundle.rs` (DropBundleVersion roundtrip + version-mismatch arms).
- Full hex-pinned-bytes Drop bundle pin DEFERRED to G-COMP-1 per V1-FROZEN-INTERFACE-DEFERRED.md Row D-9.

**FREEZE-WAVE status:** ✅ COVERED (roundtrip + version-mismatch; hex-pin deferred per Row D-9).

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
- `crates/benten-crypto-suite/tests/canonical_bytes_v1_codepoints_and_aad.rs` (`codepoint_table_integer_values_pinned` + `reserved_codepoints_stay_typed_rejected_at_v1_beta`) — asserts every codepoint integer matches the V1-FROZEN-INTERFACE.md item 6.2 table + pins typed-reject for 0x0003 / 0x647b / 0x647c.
- `crates/benten-crypto-suite/tests/tf4_gcore3c_swap_matrix_conformance*.rs` — exercises every swap-matrix arm.

**FREEZE-WAVE status:** ✅ COVERED.

---

## 11. ExecutionStateEnvelope (redb-persisted resume state)

**Surface:** `benten_eval::ExecutionStateEnvelope` — carries `schema_version: u8 = 1` discriminator + the persisted resume state; cross-process round-trip mandatory.

**Wire format:** DAG-CBOR over the envelope shape; redb-persisted in the `execution_state` table.

**Format version:** `schema_version: u8 = 1` (additive-via-discriminator; v2 path is the explicit re-open mechanism).

**Byte-pin test coverage:**
- `crates/benten-eval/tests/exec_state_envelope_shape.rs` round-trip pins.
- Cross-process resume coverage at the engine-eval boundary tests.

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level (round-trip + schema-version discriminator); hex-byte regression-pin deferred to G-COMP-1 inventory walk per Row D-9 widening.

---

## 12. RedbBackend whole-file schema-version envelope

**Surface:** `benten_graph::redb_backend` `GRAPH_SCHEMA_VERSION: u32 = 1` + the `__benten_schema_version` reserved table key (per #992).

**Wire format:** redb single-byte / single-u32 record at a reserved table key; gates open of any subsequent table.

**Format version:** `GRAPH_SCHEMA_VERSION: u32 = 1`.

**Byte-pin test coverage:**
- `crates/benten-graph/tests/redb_schema_version_envelope_pin.rs` — 3 arms (open-at-current / refuse-at-older / refuse-at-newer).

**FREEZE-WAVE status:** ✅ COVERED.

---

## 13. DeviceAttestationEnvelope (on-the-wire Atrium handshake)

**Surface:** `benten_engine::DeviceAttestationEnvelope` + `MAX_WIRE_VERSION: u8 = 2`; consumed by `apply_atrium_merge`.

**Wire format:** CBOR with explicit version-byte + Ed25519 attestation signature + nonce; per Phase-3 G16-D wave-6b.

**Format version:** `MAX_WIRE_VERSION: u8 = 2`.

**Byte-pin test coverage:**
- `crates/benten-engine/tests/device_attestation_envelope_direct.rs` device-attestation pins (post-COLLAPSE-P4 retense; the historical `g16_d_*.rs` family was consolidated).
- Cross-wire-version negotiation pins inside `apply_atrium_merge`.

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level; G-COMP-1 byte-walk deferred per Row D-9 widening.

---

## 14. DeviceAttestation (signed inner record, distinct from envelope)

**Surface:** `benten_id::DeviceAttestation` — signed Ed25519 inner record carried by the envelope above.

**Wire format:** CBOR-canonical; CanonicalBytes-trait byte-identity contract.

**Format version:** the parent envelope's `MAX_WIRE_VERSION` discriminates.

**Byte-pin test coverage:**
- `crates/benten-id/tests/device_attestation.rs` + `crates/benten-id/tests/canonical_bytes_trait.rs` round-trip + signature-verification pins.

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level.

---

## 15. RotationLog + RotationAttestation (durable signed DID-rotation chain)

**Surface:** `benten_id::RotationLog` + `benten_id::RotationAttestation` (a chain of signed DID-key rotation records; durable per-DID).

**Wire format:** CBOR-canonical sequence; each entry carries the prior-key Ed25519 signature; CanonicalBytes-trait identity contract.

**Format version:** N/A baseline; additive-via-record-append.

**Byte-pin test coverage:**
- `crates/benten-id/tests/rotation_*.rs` chain-extension + signature-verification + replay-prevention pins.

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level.

---

## 16. UCAN body canonical bytes (distinct from UCAN-Varsig v1 HEADER at item 5)

**Surface:** `benten_id::Ucan` canonical bytes (the UCAN body — claims/audience/issuer/expiry — that the Varsig header signs over).

**Wire format:** CBOR-canonical UCAN body shape per ssi-ucan upstream.

**Format version:** UCAN spec version; v1 frozen per the Varsig-header coupling at item 5.

**Byte-pin test coverage:**
- `crates/benten-id/tests/ucan.rs` + `crates/benten-id/tests/prop_ucan_attenuation.rs` round-trip + canonical-bytes pins.
- `crates/benten-caps/tests/prop_ucan_window.rs` proptests over nbf/exp time-window properties.

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level; the Varsig HEADER is item 5 (separately frozen); this is the SIGNED-OVER BODY.

---

## 17. ModuleManifest (Phase-2b SANDBOX-module envelope)

**Surface:** `benten_engine::ModuleManifest` — persisted into the `system:ModuleManifest` zone; the manifest schema the SANDBOX runtime walks at execute time.

**Wire format:** CBOR-canonical; declares host-fn `requires` + module-identity bytes.

**Format version:** the manifest's own `schema_version` field.

**Byte-pin test coverage:**
- `crates/benten-engine/tests/module_manifest_canonical.rs` + `crates/benten-engine/tests/engine_open_rebuilds_module_manifest_active_set_from_persisted_zone.rs` + `crates/benten-engine/tests/sandbox_execute_via_engine_dispatch_invokes_executor.rs` round-trip pins + module-store reopen pins.

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level.

---

## 18. PluginManifest (shareable + signing-payload — two distinct canonical-bytes surfaces)

**Surface:** `benten_platform_foundation::PluginManifest` — the load-bearing CLAUDE.md #18 plugin-trust-model substrate; surfaces TWO distinct CBOR shapes:
- (a) shareable form (with `peer_signature` field populated; the wire-distributed form).
- (b) signing-payload form (without `peer_signature`; the form the peer signs).

**Wire format:** CBOR-canonical per each shape; (b) is a strict prefix of (a) by serde shape.

**Format version:** the manifest carries no own version; CID identity (per CLAUDE.md #18 — CID covers shape).

**Byte-pin test coverage:**
- `crates/benten-platform-foundation/tests/plugin_manifest_*.rs` round-trip + signing-payload-shape pins.
- `crates/benten-platform-foundation/tests/admin_ui_v0_install_rejects_substituted_bundle_via_peer_did_signature.rs` end-to-end signature-verification pin.

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level. Note: `peer_signature` is classical-only Ed25519 at v1-beta per the R6 R1 L2-R6-MAJOR-2 fork (path-(b) defer; see `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-15e + sister-rows pending orchestrator selection).

---

## 19. ManifestStore records (persisted PluginManifestRecord)

**Surface:** `benten_platform_foundation::manifest_store::PluginManifestRecord` (persisted to redb; carries the full PluginManifest + install-time metadata).

**Wire format:** CBOR-canonical; redb-persisted at the manifest-store table.

**Format version:** record-level `schema_version` field.

**Byte-pin test coverage:**
- `crates/benten-platform-foundation/tests/tf7_*.rs` redb-reopen + cross-process resume pins.

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level.

---

## 20. Atrium HandshakeFrame + HandshakePayload + RevocationEntry

**Surface:** `benten_sync::HandshakeFrame` + `HandshakePayload` + `RevocationEntry` — Phase-3 Atrium handshake wire protocol.

**Wire format:** CBOR-canonical; on-the-wire ALPN handshake; carries device-attestation + revocation deltas.

**Format version:** the frame carries an explicit `wire_version: u8` field.

**Byte-pin test coverage:**
- `crates/benten-sync/tests/handshake.rs` round-trip + wire-version-negotiation pins.

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level; hex-byte regression-pin deferred to G-COMP-1 inventory walk per Row D-9 widening.

---

## 21. MST proto messages + MST canonical encoding

**Surface:** `benten_sync` Merkle Search Tree diff protocol messages + the MST canonical encoding (Phase-3 sync diff protocol).

**Wire format:** CBOR-canonical per the diff-message shape + MST-internal canonical encoding for the merkle-prefix nodes.

**Format version:** message-tagged; per-message-type discriminator.

**Byte-pin test coverage:**
- `crates/benten-sync/tests/mst_*.rs` round-trip + diff-application pins.

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level.

---

## 22. Atrium PeerId

**Surface:** `benten_sync::PeerId` — Phase-3 sync identity primitive; per Spike A2 ratification IS byte-identical to `ed25519_dalek::VerifyingKey` (iroh EndpointId zero-conversion contract).

**Wire format:** 32-byte fixed-width Ed25519 public-key bytes.

**Format version:** N/A baseline (32-byte width is the contract).

**Byte-pin test coverage:**
- `crates/benten-sync/tests/tf3e_zero_conversion_endpoint_id_is_verifying_key.rs` byte-identity contract pin (host crate is `benten-sync`, NOT `benten-id`).
- `crates/benten-sync/tests/peer_id.rs` round-trip pins.

**FREEZE-WAVE status:** ✅ COVERED.

---

## 23. LoroDoc canonical export + StampedValue codec (CRDT format)

**Surface:** `benten_sync` LoroDoc canonical export + `StampedValue` codec (the CRDT-internal format the Loro upstream dependency encodes).

**Wire format:** Loro's own canonical-export shape (binary-canonical per the Loro upstream); StampedValue is a codec wrapper for the per-update timestamp + author.

**Format version:** Loro upstream version is the discriminator; pinned via Cargo.toml.

**Byte-pin test coverage:**
- `crates/benten-sync/tests/loro_*.rs` round-trip pins.
- `crates/benten-sync/tests/loro_lww.rs` + `crates/benten-sync/tests/loro_rich_type.rs` codec-roundtrip pins (StampedValue is defined inside `crates/benten-sync/src/crdt.rs::StampedValue`; coverage lives in the Loro integration tests, not a dedicated `stamped_value_*.rs` file).

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level. Note: upstream Loro is a dependency-pinned wire-format; mutating it requires a Loro upstream-version bump which couples to the sync wire-protocol freeze.

---

## 24. suspension_store on-disk records (crate-private wire-format-bearing)

**Surface:** `benten_engine::suspension_store` persisted records — `PersistedCursorMeta` + `PersistedRetentionWindow` + `SerializableCapSnapshot` + `SerializableWaitMetadata`.

**Wire format:** CBOR-canonical; redb-persisted; `#[serde(default)]` forward-compat discipline per per-field defaults.

**Format version:** record-level discriminator on each record-type.

**Byte-pin test coverage:**
- `crates/benten-engine/tests/g12_e_suspension_store_round_trips.rs` + `crates/benten-engine/tests/redb_suspension_in_process.rs` round-trip + cross-process resume + forward-compat pins.

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level. Note: these are CRATE-PRIVATE wire-format-bearing surfaces (not part of the public freeze contract — listed here for completeness per the L11 phase-wide sweep; the freeze-contract-public scope covers items 1-23).

---

## Summary

| # | Surface | Format-version discriminator | Byte-pin test | Status |
|---|---|---|---|---|
| 1 | Node/Edge canonical CBOR + sentinel CID | N/A (Phase-1 baseline) | canonical_bytes_fastpath_stable.rs + node_cid.rs (benten-core) | ✅ COVERED |
| 2 | SnapshotBlob v2 | `SNAPSHOT_BLOB_SCHEMA_VERSION = 2` | snapshot_blob_backend.rs + tf11_*.rs | ✅ COVERED |
| 3 | MerkleRangeProof v2 | TBD per Option (b) | — | ⚠️ DEFERRED to G-COMP-1 |
| 4 | Per-chunk AEAD (4-segment AAD per F3 R6 R1 fix-pass) | Cipher codepoint | tf3a_*.rs + tf4_*.rs + canonical_bytes_v1_codepoints_and_aad.rs (aad_per_chunk_canonical_layout_pinned) | ✅ COVERED |
| 5 | UCAN-Varsig v1 header | Sig codepoint | tf3a_ucan_varsig_v1_header_carries_hybrid_signature.rs + tf4_gcore3c_swap_matrix_conformance*.rs | ✅ COVERED |
| 6 | AuthorizationGrant CBOR (scope-binding-message at v2 per L3-r1-1 R6 R1 fix-pass) | #[non_exhaustive] + BINDING_SIG_DOMAIN v2 | tf3b_authorization_grant_*.rs + tf3b_scope_substitution_post_sign_rejected.rs | ✅ COVERED |
| 7 | Drop bundle CBOR | `DropBundleVersion` enum | benten-drop/tests/ | ✅ COVERED |
| 8 | TwoCidStore mapping | redb schema-version | tf3e_*.rs | ✅ COVERED |
| 9 | EncryptionClass codepoint (NEW G-CORE-9) | #[non_exhaustive] + codepoint table | encryption_class.rs unit tests | ✅ COVERED |
| 10 | Crypto-suite codepoint table | V1-FROZEN §6 integers | canonical_bytes_v1_codepoints_and_aad.rs + tf4_gcore3c_swap_matrix_*.rs | ✅ COVERED |
| 11 | ExecutionStateEnvelope (redb-persisted resume) | `schema_version: u8 = 1` | crates/benten-eval/tests/exec_state_envelope_shape.rs | ✅ COVERED (substrate-level) |
| 12 | RedbBackend whole-file schema-version | `GRAPH_SCHEMA_VERSION: u32 = 1` | crates/benten-graph/tests/redb_schema_version_envelope_pin.rs (3 arms) | ✅ COVERED |
| 13 | DeviceAttestationEnvelope (Atrium handshake) | `MAX_WIRE_VERSION: u8 = 2` | crates/benten-engine/tests/device_attestation_envelope_direct.rs + apply_atrium_merge pins | ✅ COVERED (substrate-level) |
| 14 | DeviceAttestation (signed inner record) | parent envelope version | crates/benten-id/tests/device_attestation.rs + canonical_bytes_trait.rs | ✅ COVERED (substrate-level) |
| 15 | RotationLog + RotationAttestation (DID-rotation chain) | additive-via-record | crates/benten-id/tests/rotation_log_rehydrated_at_engine_open.rs + sibling rotation tests | ✅ COVERED (substrate-level) |
| 16 | UCAN body canonical bytes (distinct from Varsig header) | UCAN spec v1 | crates/benten-id/tests/ucan.rs + prop_ucan_attenuation.rs | ✅ COVERED (substrate-level) |
| 17 | ModuleManifest (SANDBOX-module envelope) | manifest schema_version | crates/benten-engine/tests/module_manifest_canonical.rs | ✅ COVERED (substrate-level) |
| 18 | PluginManifest (2 shapes: shareable + signing-payload) | CID identity (#18) | crates/benten-platform-foundation/tests/plugin_manifest_full_round_trip.rs + crates/benten-engine/tests/admin_ui_v0_install_rejects_substituted_bundle_via_peer_did_signature.rs | ✅ COVERED (substrate-level; PQ-hybrid app-layer sig pending per L2-R6-MAJOR-2 fork) |
| 19 | ManifestStore records (PluginManifestRecord) | record schema_version | crates/benten-platform-foundation/tests/ (manifest-store reopen pins) | ✅ COVERED (substrate-level) |
| 20 | HandshakeFrame + HandshakePayload + RevocationEntry | `wire_version: u8` | crates/benten-sync/tests/handshake.rs | ✅ COVERED (substrate-level) |
| 21 | MST proto messages + canonical encoding | message-tagged discriminator | crates/benten-sync/tests/mst_diff.rs + mst_revocation_priority.rs | ✅ COVERED (substrate-level) |
| 22 | Atrium PeerId (== iroh EndpointId byte-identical) | 32-byte width contract | crates/benten-sync/tests/tf3e_zero_conversion_endpoint_id_is_verifying_key.rs + peer_id.rs | ✅ COVERED |
| 23 | LoroDoc canonical export + StampedValue codec | Loro upstream version | crates/benten-sync/tests/loro_lww.rs + loro_rich_type.rs (StampedValue defined in `crates/benten-sync/src/crdt.rs::StampedValue`) | ✅ COVERED (upstream-pinned) |
| 24 | suspension_store on-disk records (crate-private) | per-record discriminator + #[serde(default)] | crates/benten-engine/tests/g12_e_suspension_store_round_trips.rs + redb_suspension_in_process.rs | ✅ COVERED (substrate-level; crate-private — not in public freeze scope) |

**Outcome (R6 R1 L11 expansion, 2026-05-24):** 23 of 24 surfaces have byte-pin / round-trip coverage at v1-beta substrate-level (items 11-24 added at R6 R1 L11 closure per the lens's phase-wide sweep finding L11-R6-R1-MAJOR-1). The one DEFERRED public surface (MerkleRangeProof, item 3) is genuinely-not-built (no phantom freeze). The G-COMP-1 wave consumes this expanded inventory for the hex-byte regression-pin sweep per Row D-9 widening. Item 24 is crate-private + retained for completeness; it is NOT in the public freeze contract scope.

¹ **Format note (L11-MIN-2 close at R6-FP-D 2026-05-24):** the glob-form `tf3a_*.rs` / `tf3b_authorization_grant_*.rs` / `tf4_*.rs` cites resolve at wave-time to multiple discrete test files under `crates/benten-crypto-suite/tests/` + `crates/benten-caps/tests/`. The glob-form is intentional for items where the byte-pin coverage spans a test-file family (multiple swap-matrix arms × wire directions); items 1, 2, 7, 8, 9, 10 reference single test files because their byte-pin coverage IS in one file. A future CI inventory-walk lane that resolves these cites should expand the glob via `git ls-files` rather than treating it as a literal path.

---

## P-III Ben decision-point (per V1-FROZEN-INTERFACE.md item 4)

This inventory is the wave-time enumeration; Ben signs the freeze decision separately at the V1-FROZEN-INTERFACE.md item 4 P-III decision-point sweep. The decision-point question Ben answers:

> "Are the 23 covered wire-format surfaces + the deferred MerkleRangeProof surface the COMPLETE v1-beta wire-format inventory (with item 24's suspension_store records noted as crate-private)? Is there any surface NOT listed above whose bytes the v1-beta lock-in needs to bind?"

A "yes, complete" answer locks the inventory; a "no, add X" answer adds the missing surface inline + extends the byte-pin coverage at the same wave.

**R6 R1 expansion provenance (2026-05-24):** items 11-24 were added at R6 R1 phase-close council per L11 lens finding `L11-R6-R1-MAJOR-1` (phase-wide canonical-bytes sweep). The 9-of-10 prior framing was scoped to the G-CORE-9 R4 FREEZE subset; R6 R1 widened to phase-wide which surfaced 14 additional wire-format-bearing surfaces. Per L11 lens recommendation path-(1): expand inventory items 11-24 for the 12 publicly-observable surfaces + retain item 24 (crate-private suspension_store) for completeness.

---

**End of inventory.**
