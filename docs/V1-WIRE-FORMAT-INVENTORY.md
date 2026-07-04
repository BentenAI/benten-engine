# v1 Wire-Format Inventory + Byte-Pin Test Coverage

> **G-CORE-9 V1-FROZEN-INTERFACE row 8d / item 4 — wave-time inventory of every wire-format-bearing surface + byte-pin test coverage.**
>
> Per `docs/V1-FROZEN-INTERFACE.md` item 4 (D2 v1-canonical-bytes contract):
> "BYTEWISE: a one-bit change to any encoded value ... is a P-III re-decision Ben must make."
>
> This doc enumerates every wire-format-bearing surface + the byte-pin test that locks its canonical bytes + the explicit `format_version: u32` discriminator (where present). At v1-beta each surface is covered by roundtrip + constant-position + format-version-byte-position pins; the full HEX-PINNED byte-pin sweep (a drift-detect CI lane that walks this inventory and asserts a hex byte-pin test exists for every surface) is a **DEFERRED** lane — 6 of 8 hex byte-pins ship at the **G-COMP-1** wave per `docs/V1-FROZEN-INTERFACE-DEFERRED.md` **Row D-9**, NOT a live lane at v1-beta. See V1-FROZEN-INTERFACE row 8d.
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
- AAD binds `(plaintext_cid: &[u8], chunk_index: u64, total_chunks: u32)` per `crates/benten-crypto-suite/src/aead.rs::aad_per_chunk` (4-segment layout: `b"benten-aead:chunk:" || plaintext_cid || chunk_index.to_be_bytes() || total_chunks.to_be_bytes()`; **big-endian per M-19**, migrated from LE at F-full Wave-0). The `total_chunks` segment closes the cross-chunk-truncation attack (an attacker truncating a 10-chunk ciphertext to 5 chunks cannot fabricate per-chunk AAD-matching tags). **R6 R1 fix-pass:** the prior G-CORE-9 R1 Fork-1 2-tuple disposition was RETRACTED; code revised to match the spec text. Pinned at `crates/benten-crypto-suite/tests/canonical_bytes_v1_codepoints_and_aad.rs::aad_per_chunk_canonical_layout_pinned` + behavioral pins at `crates/benten-graph/src/aead_wrap.rs::tests::{cross_chunk_truncation_fails, cross_chunk_inflation_fails}`. The M-19 endianness gate is golden-pinned with an **explicit anti-LE differential guard**: `aad_per_chunk_canonical_layout_pinned` asserts the `chunk_index`/`total_chunks` tail equals `to_be_bytes()` AND asserts it is NOT `to_le_bytes()` (a `to_le_bytes()` regression on `aead.rs` would-FAIL the pin), so a silent LE re-introduction is caught at test-time, not at a future wire-incompat.
- 64 KiB threshold for chunked-vs-whole-AEAD heuristic.
- Codepoint-dispatched: `HYBRID_X25519_MLKEM768 = 0x647a` (default), `CLASSICAL_X25519 = 0x6400` (downgrade), `NONE_PLAINTEXT = 0x0000`, `HYBRID_MLKEM768_HQC = 0x647b` (reserved), `PURE_PQ_MLKEM768_ONLY = 0x647c` (reserved, audit-gated).

**Format version:** Codepoint table (item 6) is the format discriminator; new ciphers land at unused codepoints.

**Byte-pin test coverage:**
- `crates/benten-crypto-suite/tests/tf3a_*.rs` + `crates/benten-crypto-suite/tests/tf4_*.rs` — AEAD round-trip + codepoint dispatch pins.
- `crates/benten-crypto-suite/tests/tf3a_pq_hybrid_wasm32_roundtrip.rs` — same-target PQ-hybrid self-round-trip (recovered-plaintext byte-identity; NOT cross-target wire-byte identity — fresh random ephemeral/nonce per seal), **CI-gated under wasm32-wasip1** by the `crypto-suite-wasm-roundtrip` job in `.github/workflows/wasm-conformance.yml` (F-full R6 R1 finding F-06; the encryption layer is now exercised on the wasm target through wasmtime, not just compile-checked / native-run). **Scope (R10-council F-04):** this gate guards **wasm32-wasip1**, NOT **wasm32-unknown-unknown** — the target the BrowserBackend thin-compute shape (CLAUDE.md baked-in #17) actually ships on. wasm32-wasip1 cleanliness is a strong necessary condition for BrowserBackend but is a distinct target; the wasm32-unknown-unknown crypto round-trip drift-gate is a NAMED CI follow-up (row below).
- `crates/benten-graph/src/aead_wrap.rs` — production wrap path; consumed by every encryption-bearing test.

**M-19 endianness conformance scanner (`benten_crypto_suite::conformance::endianness`).** The flagship M-19 gate
`wire_path_le_survivor_count()` is a REAL source-scanner over `include_str!`-embedded module source (not a
hand-coded `0`): it counts surviving `to_le_bytes` / `from_le_bytes` on any wire/AAD/keying path and MUST report
**0** (live survivor count = **0** at HEAD; F-W0-3 pin). The `WIRE_PATH_SOURCES` site-list it scans embeds **8
crypto-suite source modules** — `aead.rs`, `structural_kdf.rs`, `varsig.rs`, `sizes.rs`, `swap_matrix.rs`,
`envelope.rs`, `vault.rs`, `cipher_suite.rs` (an earlier framing under-counted this set; the array, not the prose
list, is authoritative). This gate covers the crypto-suite's OWN wire surfaces; the **cross-crate** M-19
producers (`benten-graph::aead_wrap`, `benten-platform-foundation::plugin_manifest`) are scanned by their own
crates' tests today. **Intended widened scope:** consolidating the cross-crate producers under one workspace-wide
M-19 survivor scan (so a single gate covers every wire/AAD/keying path in the workspace, not just the
crypto-suite's) is the intended post-v1-beta widening — named in `docs/V1-FROZEN-INTERFACE-DEFERRED.md`.

**FREEZE-WAVE status:** ✅ COVERED — `IROH_BLOCK_SIZE = 16 * 1024` constant pin lives in the aead module's golden-constant tests.

---

## 5. UCAN-Varsig v1 header

**Surface:** UCAN envelope signature header (used by `benten_caps::authorization_grant` + the UCAN proof chain).

**Scope clarification (L11-MIN-1 close at R6-FP-D 2026-05-24):** this row scopes the **header bytes** (Varsig codepoint + signature bytes + multibase prefix). The **UCAN BODY bytes that the header signs over** (the UCAN claim CBOR envelope) are covered structurally by §6 (AuthorizationGrant CBOR) for the AuthorizationGrant-borne UCAN and behaviorally by the chain-validation surface (which exercises the body bytes end-to-end via verify-and-walk pinned tests at `crates/benten-caps/tests/`). An explicit body-bytes byte-pin row is intentionally NOT in this v1-beta inventory — the body's canonical-bytes contract follows DAG-CBOR canonicalization (covered by §1 Phase-1 baseline) + the freeze contract is that the UCAN body shape stays additive (`#[non_exhaustive]` per item 11). A future G-COMP-1 row may add explicit body byte-pins; the current behavioral coverage is per Row D-9 wire-format-deferred posture.

**Wire format:**
- Multiformats Varsig v1 — signature suite codepoint + signature bytes prefixed with the multibase header.
- Sig codepoint table (V1-FROZEN-INTERFACE item 6.2):
  - `HYBRID_ED25519_MLDSA65 = 0x0001` (default; byte-faithful IETF LAMPS composite `id-MLDSA65-Ed25519-SHA512`, OID `1.3.6.1.5.5.7.6.48`, `draft-ietf-lamps-pq-composite-sigs-19` + test-vector commit `f0627ab3`; wire `mldsaSig(3309) ‖ tradSig(64)` = 3373 B ML-DSA-first, raw concat, NO commitment trailer; strip-resistance via shared-`M'`/ctx=Label binding + both-halves-required).
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

## 25. MembershipSet codepoint band + `0x6610` group per-stanza AAD (F-full)

**Surface:** `benten_membership_set::codepoints` (`MEMBERSHIP_SET_ENCRYPTION = 0x6600` / `MEMBERSHIP_SET_GROUP_MULTI_STANZA = 0x6610` / `MEMBERSHIP_SET_RESERVED_0X6620 = 0x6620`) + `benten_membership_set::aad::assemble_group_aad` (the `0x6610` group per-stanza AAD = the BLINDED 11-field set) + `canonical_members_table_bytes` (the NQ-W4 `members_table` snapshot).

**Wire format:**
- The MembershipSet band is `0x6600..=0x66FF` (Inv-18 / NQ-W2 FROZEN-band ownership). Three values assigned at v1-beta.
- `0x6600` set-keying envelope (every `MembershipSetKind` binds here); `0x6610` group multi-stanza per-stanza AAD (the BLINDED 11-field set, big-endian, length-injective per R0.7 §3.10/§4.1); both Sealed-Sender by default.
- **`0x6620` is RESERVED + ENCODE-ONLY at v1-beta** — the `SubsetRef` federation shape is reserved-and-refused (typed-reject) at v1-beta; the value is allocated/encoded in the band but NOT a live decode/dispatch arm until a future additive wave (never a wire break; F-full R6 R1 finding F-21).
- The group AAD 11-field set is OPAQUE bytes across the m-15 GNC-5 crypto-suite seam.
- **B2 sender ORIGIN-AUTH (always-on; `benten_drop::layer_c::group_posture`):** the `0x6610` once-sealed body region is `body_v2 = sig_codepoint(u16 BE) ‖ sender_sig_len(u32 BE) ‖ sender_sig ‖ body`, sealed under the K_Set-derived CEK. `sender_sig` is one per-MESSAGE LAMPS-hybrid `id-MLDSA65-Ed25519-SHA512` (`0x0001`) signature over the domain-separated `M_auth` (`SENDER_AUTH_DOMAIN` ‖ codepoints ‖ sender-DID ‖ body_cid ‖ blinded `audience_set_commitment` ‖ the THREE generation words `[member_key_generation, membership_set_generation, role_assignments_generation]` ‖ stanza_count ‖ body-AAD digest). The per-stanza 11-field AAD is **byte-UNCHANGED** (the sig lives in the body region, not the AAD). Verified post-decrypt against the hybrid key resolved from the recovered sender-DID, recomputing the commitment + generations from the recipient's INDEPENDENTLY-held set-state (F-2/F-3). **Goldens pin wire-SHAPE + a sign→verify round-trip, NOT signature hex** — the ML-DSA half is hedged/randomized (FLAG-6).

**Format version:** `aad_version: u8 = 0x01` prefix on the group AAD; the codepoint band IS the freeze for the set-keying axis.

**Byte-pin test coverage:**
- `crates/benten-crypto-suite/tests/f_cp_codepoint_registry_dispatch.rs` (`MEMBERSHIP_SET_GROUP_MULTI_STANZA == 0x6610` integer pin).
- `crates/benten-membership-set/tests/f_aad_1_members_table_canonical_cbor_length_injective.rs` + `f_aad_2_nine_tuple_injectivity_opaque_boundary.rs` (the 11-field AAD injectivity + canonical-CBOR length-injectivity).
- `crates/benten-membership-set/tests/f_fed_1_2_subset_ref_federation.rs` (`0x6620` reserved-and-refused typed-reject at v1-beta).
- `crates/benten-drop/tests/f_02_group_aad_11field_and_f_01_truncation.rs` (the `F_02_LIVE_SEAL_STANZA0_AAD_HEX` golden). **R9 GAP-1 fixture refresh (NOT a format change):** the golden's 32-byte `audience_set_commitment` component was **regenerated** because the recipient-key representation went placeholder→real — the roster DIDs the commitment hashes over derive from the recipient public-key bytes (`RecipientPublic::to_bytes` = `x25519_pub(32) ‖ mlkem768_ek(1184)`), which changed when the `[u8; 32]` placeholder fingerprint became a real hybrid public key. The AAD **SHAPE / field-set / blinding construction / `aad_version` (`0x01`)** and the cross-engine byte-equality + sign→verify round-trip goldens are **UNCHANGED** — this is a fixture-value refresh, not a wire-format change.

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level; `0x6620` reserved-encode-only.

---

## 26. Layer-C drop band (`0x6500`/`0x6510`/`0x6520`) + B2 sender-origin-auth body region (F-full)

**Surface:** `benten_drop::layer_c` (`LAYER_C_DROP = 0x6500` plaintext-sender / `DROP_TO_RECIPIENT_SEALED_SENDER = 0x6510` Sealed-Sender DEFAULT / `LAYER_C_DROP_MULTI_RECIPIENT = 0x6520` group multi-stanza) + `EncryptedEnvelope` / `BindingContext` / `HpkeRecipientStanza` + the B2 origin-auth `M_auth` binding (`SENDER_AUTH_DOMAIN` + `build_m_auth`). Discharges the R6-round-2 F-06 (the Layer-C drop rows were previously enumerated only via §7's bundle row).

**Wire format:**
- The Layer-C drop band is `0x6500..=0x65FF`. Three values assigned at v1-beta: `0x6500` (plaintext-sender, NON-default), `0x6510` (Sealed-Sender, the v1-beta DEFAULT, BR-1), `0x6520` (`HpkeMultiBase` group multi-stanza).
- **Plaintext AAD field-set (relay-visible) — UNCHANGED by B2.** `0x6500`/`0x6510` single-recipient AAD = `{aad_version(u8), codepoint(u16 BE), audience(u32-BE-lp), body_cid(self-describing CIDv1 36B), recipient_key_generation(u32 BE)}` (`0x6500` additionally appends `lp(sender_did)`, U4). `0x6520` per-stanza AAD = the BLINDED `{aad_version, codepoint, body_cid, recipient_count(u16 BE), audience_set_commitment(32B), stanza_index(u32 BE), stanza_count(u32 BE), recipient_key_generation(u32 BE)}` (+ optional `lp(sender_did)` on the non-default plaintext-sender variant). The KEM is HPKE `mode_base[MLKEM768-X25519]` (`0x647A`); bulk AEAD is ChaCha20-Poly1305.
- **B2 sender ORIGIN-AUTH (always-on; BD-2) — inside the once-sealed body region, NOT on the plaintext wire.** `0x6510` (single): `inner_v2 = lp_u32(sender_did) ‖ sig_codepoint(u16 BE) ‖ sender_sig_len(u32 BE) ‖ sender_sig ‖ body`, sealed under the CEK. `0x6520` (group): `body_v2 = sig_codepoint(u16 BE) ‖ sender_sig_len(u32 BE) ‖ sender_sig ‖ body`, PREPENDED into the once-bulk-sealed body (the per-stanza `sealed_inner = lp_u32(sender_did)` is unchanged). `sender_sig` is one per-MESSAGE LAMPS-hybrid `id-MLDSA65-Ed25519-SHA512` (`0x0001`) signature over `M_auth` (`SENDER_AUTH_DOMAIN` ‖ codepoints ‖ sender-DID ‖ body_cid ‖ audience commitment ‖ generation words ‖ stanza_count ‖ body-AAD digest). The sender-DID + signature are BOTH inside the ciphertext (sender-confidential); the on-wire plaintext AAD field-set is byte-identical to pre-B2.
- **F-11 (R12) — BY-BAND recipient-cardinality width (AS-BUILT freeze note; Ben-CONFIRMED intentional).** The recipient/member-cardinality integer in the group AADs is **BY-BAND asymmetric**: the Layer-C `0x6520` group per-stanza AAD encodes `recipient_count` as **`u16` BE** (this section, line above), while the MembershipSet `0x6610` group per-stanza AAD (§25) encodes `member_count` as **`u32` BE** (the BLINDED 11-field set; verified against `crates/benten-drop/tests/f_02_group_aad_11field_and_f_01_truncation.rs` golden — "4 (member_count)" u32 segment). This is AS-BUILT and both widths are golden-pinned + round-trip-tested; it is **NOT a blocker** (each band's width is internally consistent, and `0x6520`'s `stanza_index`/`stanza_count` are `u32` so the `u16` is only the roster-cardinality field). **RESOLVED (R12):** the `0x6520` u16 vs `0x6610` u32 by-band asymmetry is confirmed **intentional per Ben** — a >65535-recipient single `0x6520` send is out of scope by design (split into multiple sends). The u16 saturation is now enforced with a typed `LayerCError::RecipientCountExceedsBandWidth` at the seal entry (`validate_group_roster_len`, the single choke point) + a named `benten_drop::layer_c::MAX_LAYER_C_GROUP_RECIPIENTS` const (65535) — an over-limit roster typed-rejects, it never panics inside the seal. Unifying the widths to `u32` is a **REJECTED** freeze record (do NOT touch the `0x6610` band). This is a registration + AS-BUILT record, not a change request.
- Verified post-decrypt against the hybrid verifying key resolved from the recovered sender-DID (`benten_id::did::Did::resolve_hybrid`), recomputing the audience commitment + key-generation from the recipient's INDEPENDENTLY-held audience/roster (F-2). The unauthenticated sealed-sender variant is DELETED (an unauthenticated-but-claimed sender = indistinguishable from forgery).

**Format version:** `ENVELOPE_FORMAT_VERSION = 2` (the envelope serialization byte) + `aad_version: u8 = 0x01` (the AAD prefix axis, DISTINCT from the format byte) + `sig_codepoint` (the auth-suite axis, inside the sealed body region).

**Byte-pin test coverage:**
- `crates/benten-drop/tests/f_lc_hpke_encrypt_to_recipient_sealed_sender.rs` (F-LC-1/2/3: single + group round-trip, BLINDED AAD goldens, the substantive B2 `f_lc_3` arms — second-sealer-spoof / second-member-spoof / re-target / stale-generation / strip-PQ-half).
- `crates/benten-drop/tests/f_lc_abuse_control_group_posture_and_inv18.rs` (F-LC-9 group Sealed-Sender posture + Inv-18 AAD field-set golden) + `f_02_group_aad_11field_and_f_01_truncation.rs` (F-02 11-field AAD golden + F-01 truncation) + `f_disc_2_invariant_and_doc_registration_catch_net.rs` (Inv-16/18/20 enforcement).
- **Goldens pin wire-SHAPE + a sign→verify round-trip, NOT a fixed `sender_sig` hex** (the ML-DSA half is hedged/randomized — FLAG-6); the AAD-region goldens are byte-frozen.
- **R9 GAP-1 fixture refresh (NOT a format change).** Where a `0x6520` group AAD golden pins a specific `audience_set_commitment`, that 32-byte component regenerated on the placeholder→real recipient-key transition (the blinded roster derives from `RecipientPublic::to_bytes`, which grew from a `[u8; 32]` fingerprint to `x25519_pub(32) ‖ mlkem768_ek(1184)`). Same as the `0x6610` §25 note: the AAD SHAPE / field-set / blinding / `aad_version` and the round-trip goldens are UNCHANGED — value refresh, not a wire-format change.

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level; B2 origin-auth always-on (pre-`phase-4-meta-core-close` in-place wire change, FLAG-1).

---

## 27. Layer-D bands — DeviceLink (`0x6310..0x631F`) + RemotePermission (`0x6320..0x632F`) + drop timestamp-exclusion (F-full P-III)

**Surface:** `benten_engine::layer_d` — `device_link` (`DEVICE_LINK_BAND_BASE = 0x6310`; `ProvisioningOffer`/`ProvisioningInnerPayload`; `PROVISIONING_WIRE_VERSION = 2`), `remote_permission` (`REMOTE_PERMISSION_BAND_BASE = 0x6320`; `PermissionRequest`/`PermissionGrant`; `REMOTE_PERMISSION_WIRE_VERSION = 2`), and `drop_timestamp` (`LAYER_D_BUCKET_SECS = 3600`; the `DropToRecipient` timestamp-EXCLUSION invariant). Discharges the remainder of the R6-round-2 F-06: §26 added the Layer-C drop rows; this section adds the previously-unenumerated Layer-D bands to the freeze inventory (the Ben P-III freeze sign-off deliverable).

**Wire format:**
- **DeviceLink band `0x6310..=0x631F`** (FREEZE) — multi-device key-wrap `Provisioning*` flow. B's fresh hybrid X25519+ML-KEM-768 recipient keypair receives the HPKE-sealed inner payload (REUSES the Layer-C KEM-DEM, Inv-16). `K_principal` is identity-equivalent → NO forward secrecy (documented by design). Pubkey substitution post-fingerprint (MITM) fails closed (HPKE unwrap on wrong recipient secret); the in-band `provisioning_session_id`-match check rejects substituted payloads and the typed `DeviceLinkError::SessionIdReplayed` arm is present, **but the DURABLE production replay-store (the persisted consumed-session-id set) is NOT wired at v1-beta — DEFERRED to G-COMP-1 per `docs/V1-FROZEN-INTERFACE-DEFERRED.md` Row D-30** (no exploit on a single trusted engine; sessions are short-lived); forged/unsigned offer rejected (`E_DEVICE_ATTESTATION_FORGED`-class).
- **RemotePermission band `0x6320..=0x632F`** (FREEZE) — `PermissionRequest { request_id, requesting_device_did, requesting_device_pubkey, operation: { Decrypt | SignUcanDelegation | RemoteUnlock | ExecuteWorkflow }, reason, timestamp, ephemeral_signing_key, nonce }` (B-signed) → `PermissionGrant { request_id, granted_at, valid_until, operation_result, audit_node_cid }` (A-user-DID-signed). Encoding is **`aad_version: u8 = 0x01` prefix + canonical-TLV length-injective** (NOT DAG-CBOR), every integer big-endian (M-19/M-20). The grant signature binds `audit_node_cid`; an out-of-band codepoint typed-rejects (`UnsupportedOperationCodepoint`).
- **Drop timestamp-EXCLUSION** — `DropToRecipient` / Sealed-Sender carries **NO** `sealed_at` / `valid_until` field (drops are forever-valid per Compromise #62; closes the L6 drop-timestamp HIGH-leak by exclusion). The U28 **1-hour coarse bucket** applies ONLY to DeviceLink + RemotePermission epoch fields, `bucket = (raw_unix_secs / 3600) * 3600` — round-DOWN, NO jitter (NQ-C5 RATIFIED, Ben 2026-06-02; `bucket % 3600 == 0`); bucket ⊥ `valid_until` clock (NQ-T2 RATIFIED).

**Format version:** `PROVISIONING_WIRE_VERSION = 2` / `REMOTE_PERMISSION_WIRE_VERSION = 2` (struct framing axis) + `aad_version: u8 = 0x01` (AAD-prefix axis, DISTINCT from the framing byte — the two orthogonal version axes mirror the MembershipSet `AAD_VERSION = 0x01` pattern).

**Byte-pin test coverage:**
- `crates/benten-engine/tests/f_ld_2_remote_permission_wire_freeze.rs` (F-LD-2: `f_ld_2_operation_wire_encoding_is_big_endian_golden_pin` + `f_ld_2_full_struct_signing_bytes_golden_pin` + signature round-trip + `f_ld_2_out_of_band_codepoint_typed_rejects`).
- `crates/benten-engine/tests/f_ld_4_multi_device_key_wrap_provisioning.rs` (F-LD-4: `f_ld_4_device_link_key_wrap_round_trips_to_device_b` + `f_ld_4_recipient_confidentiality_wrong_secret_rejects_k_principal_exfil` + `f_ld_4_model_only_session_id_replay_pin` (model-only; NOT a production-path pin per Row D-30) + `f_ld_4_forged_offer_signature_rejects` + `f_ld_4_device_link_band_base_pinned` (`0x6310`) + `f_ld_4_k_principal_has_no_forward_secrecy_documented`).
- `crates/benten-engine/tests/f_ld_8_layer_d_timestamp_exclusion.rs` (F-LD-8: `f_ld_8_drop_to_recipient_carries_no_timestamp_field` + the differential `f_ld_8_drop_serialization_has_no_timestamp_bytes` (would-FAIL-on-no-op) + `f_ld_8_device_link_carries_the_one_hour_bucket` (inverse pin) + `f_ld_8_bucket_is_round_down_no_jitter_nq_c5_gated`).
- The Layer-D drops REUSE the Layer-C `0x6610` / `0x6520` BLINDED group-AAD; that AAD shape is golden-pinned by `crates/benten-drop/tests/f_02_group_aad_11field_and_f_01_truncation.rs` (`f_02_live_0x6610_seal_binds_canonical_11_field_aad_golden` + `f_02_local_assembler_matches_canonical_membership_set_byte_for_byte` + `f_02_seal_open_round_trip_under_11_field_aad`; F-01 truncation arms `f_01_0x6610_dropped_stanza_fails_closed` + `f_01_0x6520_dropped_stanza_fails_closed`).

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level; the `0x6310`/`0x6320` bands + the drop timestamp-exclusion invariant are wire-locked (V2 + BE + canonical-TLV from first commit, M-20).

---

## 28. Layer-A vault on-disk AEAD frame (`${BENTEN_DATA_DIR}/vault.cbor`)

<!-- R13 F-17: prose section header renumbered §30 → §28 to restore sequential
     prose-section numbering (the prior §27 prose section folds summary-table
     rows 27/28/29 — the Layer-D bands — into one section, so this is the 28th
     prose section; the summary table below keeps its item-row numbering 1..30,
     which counts wire-surfaces not prose sections — no hex / layout change). -->

**Surface:** `benten_crypto_suite::vault` — `VaultPayload` + `serialize_vault` / `decode_vault` / `open_vault` / `decode_vault_strict`; the DAK derivation `derive_dak` (`benten_crypto_suite::vault`). This is an **at-rest** wire-format-bearing surface (in scope per the redb at-rest precedent — items 1 / 8 / 11 / 12 / 24 all enumerate at-rest formats).

**Wire format (R11 MC-6 frame extension — salt + Argon2id params in-header; pre-freeze):**
- **Outer on-disk frame** is a **hand-rolled magic-prefixed AEAD frame** (NOT a DAG-CBOR-encoded `EncryptedEnvelope`): `magic 0xae | format-version V2 | codepoint(BE u16) | salt(16 B) | m_cost(u32 BE) | t_cost(u32 BE) | p_cost(u32 BE) | nonce_len(u8) | nonce | ct` (`serialize_vault`, `crates/benten-crypto-suite/src/vault.rs`). **R11 MC-6:** the frame now persists the 16-byte Argon2id salt + the `{m_cost, t_cost, p_cost}` params in the header (NON-secret; the standard PBKDF-header shape) so that `vault.cbor` bytes + password ALONE re-derive the DAK and decrypt across a restart (`open_vault` reads salt+params from the frame — no external salt source). Before MC-6 the frame was `magic 0xae | V2 | codepoint(BE) | nonce_len | nonce | ct` and the salt+params lived only in an un-persisted in-RAM struct. Redefining V2 carries no migration burden (F-VA-1: no surviving V1/V2 vault golden vector). The frame is NO LONGER a byte-twin of the `EncryptedEnvelope` symmetric-AEAD wire header (that twin holds only for pre-MC-6 bytes); the vault frame now carries the extra salt+params header that the general envelope does not.
- **AEAD-sealed inner payload** is canonical DAG-CBOR — the vault payload `{ k_principal: [u8;32], user_did_signing_key (HybridSigningKeySerialized, Ed25519⊕ML-DSA-65), user_did_creation_time: u64 }` — canonical field-order is part of the freeze (re-serialize is byte-identical).
- Sealed under XChaCha20-Poly1305 with a **24-byte XNonce** (`VAULT_XNONCE_LEN = 24`, m-4: the vault is reseal-heavy — `K_principal` rotation + multi-device key-wrap re-seals — so a 12-byte ChaCha20 nonce would hit the 2^32 random-nonce birthday bound).
- Wire codepoint = `SymmetricAeadXNonce` `VAULT_SYMMETRIC_AEAD_XNONCE_CODEPOINT = 0x6100` (NOT the 12-byte `SYMMETRIC_AEAD_12B_CODEPOINT = 0x6101` sibling; a 12-byte nonce presented under `0x6100` is typed-rejected at strict decode — codepoint discriminates nonce width).
- DAK derivation: `DAK = HKDF-SHA256( Argon2id(pw, salt; m=19456, t=2, p=1), info = "benten-dak-v1" )` with a 16-byte salt; the frozen `OWASP_DEFAULT` Argon2id params (`m_cost = 19456`, `t_cost = 2`, `p_cost = 1`) + the `"benten-dak-v1"` HKDF info-tag are the freeze (the info-tag is the codepoint slot for a future Argon2id-v2 param set).

**Format version:** the `0x6100` vault codepoint + the `"benten-dak-v1"` HKDF info-tag are the version axes (codepoint-dispatched per CLAUDE.md baked-in #5; a future param set re-versions via a new info-tag / codepoint slot, never an in-place reinterpretation).

**Byte-pin test coverage:**
- `crates/benten-crypto-suite/tests/f_va_1_vault_ondisk_format_freeze.rs` (F-VA-1: `vault_uses_24_byte_xchacha20_nonce` + the `0x6100` codepoint pin + 12-byte-nonce-under-XNonce-codepoint strict-reject + canonical CBOR field-order byte-identity + R11 MC-6 `vault_frame_persists_salt_and_params_in_header` header-offset freeze + `vault_opens_from_bytes_and_password_alone` self-containment).
- `crates/benten-crypto-suite/tests/f_va_2_argon2id_dak_derivation.rs` (F-VA-2: the Argon2id→HKDF DAK derivation determinism + param binding).

**FREEZE-WAVE status:** ✅ COVERED at v1-beta substrate-level; the hand-rolled `magic 0xae | V2 | codepoint | salt(16) | m_cost | t_cost | p_cost | nonce_len | nonce | ct` frame (R11 MC-6 salt+params in-header) + the `0x6100` codepoint + 24-byte XNonce + Argon2id `OWASP_DEFAULT` params + canonical CBOR inner-payload field-order are byte-locked (format-version V2 from first commit, M-20).

---

## Summary

| # | Surface | Format-version discriminator | Byte-pin test | Status |
|---|---|---|---|---|
| 1 | Node/Edge canonical CBOR + sentinel CID | N/A (Phase-1 baseline) | canonical_bytes_fastpath_stable.rs + node_cid.rs (benten-core) | ✅ COVERED |
| 2 | SnapshotBlob v2 | `SNAPSHOT_BLOB_SCHEMA_VERSION = 2` | snapshot_blob_backend.rs + tf11_*.rs | ✅ COVERED |
| 3 | MerkleRangeProof v2 | TBD per Option (b) | — | ⚠️ DEFERRED to G-COMP-1 |
| 4 | Per-chunk AEAD (4-segment AAD per F3 R6 R1 fix-pass) | Cipher codepoint | tf3a_*.rs + tf4_*.rs + canonical_bytes_v1_codepoints_and_aad.rs (aad_per_chunk_canonical_layout_pinned) | ✅ COVERED |
| 5 | UCAN-Varsig v1 header | Sig codepoint | tf3a_ucan_varsig_v1_header_carries_hybrid_signature.rs + tf4_gcore3c_swap_matrix_conformance*.rs | ✅ COVERED |
| 6 | AuthorizationGrant CBOR (audience-pubkey-binding-message at v3 per R6-R2-FP-A; scope-binding at v2 per L3-r1-1 R6 R1) | #[non_exhaustive] + BINDING_SIG_DOMAIN v3 | tf3b_authorization_grant_*.rs + tf3b_scope_substitution_post_sign_rejected.rs + tf3b_audience_substitution_post_sign_rejected.rs | ✅ COVERED |
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
| 18 | PluginManifest (2 shapes: shareable + signing-payload) | CID identity (#18) | crates/benten-platform-foundation/tests/plugin_manifest_full_round_trip.rs + crates/benten-platform-foundation/tests/admin_ui_v0_install_rejects_substituted_bundle_via_peer_did_signature.rs | ✅ COVERED (substrate-level; PQ-hybrid app-layer sig pending per L2-R6-MAJOR-2 fork) |
| 19 | ManifestStore records (PluginManifestRecord) | record schema_version | crates/benten-platform-foundation/tests/ (manifest-store reopen pins) | ✅ COVERED (substrate-level) |
| 20 | HandshakeFrame + HandshakePayload + RevocationEntry | `wire_version: u8` | crates/benten-sync/tests/handshake.rs | ✅ COVERED (substrate-level) |
| 21 | MST proto messages + canonical encoding | message-tagged discriminator | crates/benten-sync/tests/mst_diff.rs + mst_revocation_priority.rs | ✅ COVERED (substrate-level) |
| 22 | Atrium PeerId (== iroh EndpointId byte-identical) | 32-byte width contract | crates/benten-sync/tests/tf3e_zero_conversion_endpoint_id_is_verifying_key.rs + peer_id.rs | ✅ COVERED |
| 23 | LoroDoc canonical export + StampedValue codec | Loro upstream version | crates/benten-sync/tests/loro_lww.rs + loro_rich_type.rs (StampedValue defined in `crates/benten-sync/src/crdt.rs::StampedValue`) | ✅ COVERED (upstream-pinned) |
| 24 | suspension_store on-disk records (crate-private) | per-record discriminator + #[serde(default)] | crates/benten-engine/tests/g12_e_suspension_store_round_trips.rs + redb_suspension_in_process.rs | ✅ COVERED (substrate-level; crate-private — not in public freeze scope) |
| 25 | MembershipSet codepoint band (`0x6600`/`0x6610`/`0x6620`) + `0x6610` group 11-field AAD + B2 body-region sender-sig | `aad_version: u8 = 0x01` + codepoint band + `sig_codepoint` | f_cp_codepoint_registry_dispatch.rs + benten-membership-set/tests/f_aad_1_*.rs + f_aad_2_*.rs + f_fed_1_2_subset_ref_federation.rs + benten-drop/tests/f_lc_hpke_*.rs (B2 0x6610 arms) | ✅ COVERED (substrate-level; `0x6620` reserved-encode-only at v1-beta) |
| 26 | Layer-C drop band (`0x6500`/`0x6510`/`0x6520`) + B2 sender-origin-auth body region | `ENVELOPE_FORMAT_VERSION = 2` + `aad_version: u8 = 0x01` + `sig_codepoint` | benten-drop/tests/f_lc_hpke_encrypt_to_recipient_sealed_sender.rs + f_lc_abuse_control_group_posture_and_inv18.rs + f_02_group_aad_11field_and_f_01_truncation.rs | ✅ COVERED (substrate-level; B2 origin-auth always-on; goldens pin shape+round-trip not sig-hex per FLAG-6) |
| 27 | Layer-D RemotePermission band (`0x6320..0x632F`) — `PermissionRequest`/`PermissionGrant` | `REMOTE_PERMISSION_WIRE_VERSION = 2` + `aad_version: u8 = 0x01` | benten-engine/tests/f_ld_2_remote_permission_wire_freeze.rs (operation BE golden + full-struct signing-bytes golden + out-of-band codepoint typed-reject) | ✅ COVERED (substrate-level; canonical-TLV BE, NOT DAG-CBOR) |
| 28 | Layer-D DeviceLink band (`0x6310..0x631F`) — multi-device key-wrap `Provisioning*` | `PROVISIONING_WIRE_VERSION = 2` + `aad_version: u8 = 0x01` | benten-engine/tests/f_ld_4_multi_device_key_wrap_provisioning.rs (round-trip + pubkey-substitution-reject + session-id-replay MODEL-only pin (Row D-30 deferred) + `0x6310` band pin) + f_02_group_aad_11field_and_f_01_truncation.rs (reused 0x6610 AAD golden) | ✅ COVERED (substrate-level; reuses Layer-C KEM-DEM; K_principal no-FS by design) |
| 29 | Layer-D drop timestamp-EXCLUSION + 1-hr bucket (DeviceLink/RemotePermission only) | structural (NO timestamp field on `DropToRecipient`) + `LAYER_D_BUCKET_SECS = 3600` | benten-engine/tests/f_ld_8_layer_d_timestamp_exclusion.rs (no-timestamp-field + differential no-timestamp-bytes + inverse bucket pin + round-down-no-jitter NQ-C5) | ✅ COVERED (substrate-level; drops forever-valid per #62; NQ-C5/NQ-T2 RATIFIED) |
| 30 | Layer-A vault on-disk AEAD frame (`vault.cbor`) — hand-rolled magic-prefixed frame (R11 MC-6: salt+params in-header; no longer a byte-twin of `EncryptedEnvelope` header), DAG-CBOR inner payload | `0x6100` `SymmetricAeadXNonce` codepoint + 24-byte XNonce + salt(16)+`{m,t,p}`-in-header + Argon2id `OWASP_DEFAULT` (m=19456/t=2/p=1) + `"benten-dak-v1"` HKDF info-tag | benten-crypto-suite/tests/f_va_1_vault_ondisk_format_freeze.rs (24-byte-nonce + `0x6100` codepoint + 12-byte-under-XNonce strict-reject + canonical CBOR field-order + MC-6 salt/params header-offset freeze + bytes+password self-containment) + f_va_2_argon2id_dak_derivation.rs | ✅ COVERED (substrate-level; at-rest format, redb-precedent scope) |

**Outcome (R6 R1 L11 expansion, 2026-05-24; extended at F-full R6 R1 with item 25; B2 sealed-sender origin-auth added item 26; Layer-D bands added items 27-29 at R6-round-2; Layer-A vault at-rest format added item 30 at R6-round-3 per GAP-B):** 29 of 30 surfaces have byte-pin / round-trip coverage at v1-beta substrate-level (items 11-24 added at R6 R1 L11 closure per the lens's phase-wide sweep finding L11-R6-R1-MAJOR-1; item 25 the MembershipSet codepoint band added at F-full R6 R1; item 26 the Layer-C drop band + B2 sender-origin-auth body region added with the sealed-sender-auth mini-ADDL; items 27-29 the Layer-D RemotePermission + DeviceLink + drop-timestamp-exclusion bands added at R6-round-2 — together discharging the R6-round-2 F-06 that the Layer-C drop rows AND the Layer-D bands were under-enumerated; item 30 the Layer-A vault on-disk AEAD frame added at R6-round-3 per GAP-B, closing the at-rest-format gap the redb precedent put in scope). The one DEFERRED public surface (MerkleRangeProof, item 3) is genuinely-not-built (no phantom freeze). The G-COMP-1 wave consumes this expanded inventory for the hex-byte regression-pin sweep per Row D-9 widening. Item 24 is crate-private + retained for completeness; it is NOT in the public freeze contract scope.

¹ **Format note (L11-MIN-2 close at R6-FP-D 2026-05-24):** the glob-form `tf3a_*.rs` / `tf3b_authorization_grant_*.rs` / `tf4_*.rs` cites resolve at wave-time to multiple discrete test files under `crates/benten-crypto-suite/tests/` + `crates/benten-caps/tests/`. The glob-form is intentional for items where the byte-pin coverage spans a test-file family (multiple swap-matrix arms × wire directions); items 1, 2, 7, 8, 9, 10 reference single test files because their byte-pin coverage IS in one file. A future CI inventory-walk lane that resolves these cites should expand the glob via `git ls-files` rather than treating it as a literal path.

---

## P-III Ben decision-point (per V1-FROZEN-INTERFACE.md item 4)

This inventory is the wave-time enumeration; Ben signs the freeze decision separately at the V1-FROZEN-INTERFACE.md item 4 P-III decision-point sweep. The decision-point question Ben answers:

> "Are the 23 covered wire-format surfaces + the deferred MerkleRangeProof surface the COMPLETE v1-beta wire-format inventory (with item 24's suspension_store records noted as crate-private)? Is there any surface NOT listed above whose bytes the v1-beta lock-in needs to bind?"

A "yes, complete" answer locks the inventory; a "no, add X" answer adds the missing surface inline + extends the byte-pin coverage at the same wave.

**R6 R1 expansion provenance (2026-05-24):** items 11-24 were added at R6 R1 phase-close council per L11 lens finding `L11-R6-R1-MAJOR-1` (phase-wide canonical-bytes sweep). The 9-of-10 prior framing was scoped to the G-CORE-9 R4 FREEZE subset; R6 R1 widened to phase-wide which surfaced 14 additional wire-format-bearing surfaces. Per L11 lens recommendation path-(1): expand inventory items 11-24 for the 12 publicly-observable surfaces + retain item 24 (crate-private suspension_store) for completeness.

---

## CI follow-up rows (named-carry)

- **CI-FU-1 (R10-council F-04; reworded F-25 R12) — wasm32-unknown-unknown
  bundle-composition drift-gate.**
  The `crypto-suite-wasm-roundtrip` job (`.github/workflows/wasm-conformance.yml`)
  gates the tf3a PQ-hybrid round-trip on **wasm32-wasip1** (via wasmtime), which
  proves the crypto layer is wasm-CLEAN under wasip1. **F-25 correction:** the
  earlier framing ("add a wasm32-unknown-unknown crypto *round-trip*") is wrong —
  `benten-crypto-suite` is **structurally EXCLUDED from the wasm32-unknown-unknown
  BrowserBackend thin-compute bundle** (CLAUDE.md baked-in #17: the thin-compute
  target ships NO crypto / sync / SANDBOX state; crypto lives on the full-peer
  shape a). So there is nothing to run a *crypto round-trip* against on
  wasm32-unknown-unknown — the crate isn't in that bundle. The correct follow-up
  gate is a **bundle-composition drift-gate**: a wasm32-unknown-unknown build of
  the BrowserBackend thin-compute artifact that ASSERTS `benten-crypto-suite` (and
  the other full-peer-only crates) is NOT linked in — so a future dependency edit
  that accidentally pulls crypto-suite into the browser bundle fires at PR time
  (the structural-exclusion invariant, mirroring the per-crate `wasm32_excluded`
  compile-fence tests). **Destination:** a new `wasm-browser.yml` (or
  `wasm-checks.yml`) job; Phase-4-Meta-Composing browser-runtime CI hardening. Not
  a wire-format change — a CI coverage enhancement. Anchor: R10-council F-04 /
  R12-council F-25.

---

**End of inventory.**
