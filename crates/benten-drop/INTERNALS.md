# benten-drop — Internals

A plain-English, code-grounded tour of the `benten-drop` crate — the **14th workspace crate** added in Phase-4-Meta-Core (G-CORE-3f at PR #1340 minting the Drop bundle format; G-CORE-3f follow-ups + the F-full Layer-C encrypt-to-recipient / Sealed-Sender build landing through G-CORE-9 FREEZE). Read-only audit. Audience: a developer landing in this crate fresh and trying to find the load-bearing seams. Last refreshed: 2026-07-02 against the `phase-4-meta-core/r9-base` freeze base `091fb12f` (R9 GAP-1 real recipient keying + F-04 Layer-C-section + file-inventory refresh).

---

## 1. What this crate does

`benten-drop` carries **two** Sharing & Confidentiality (S&C) surfaces:

1. The **self-contained offline-bundle format** — a `DropBundle` is a single **CBOR-on-disk artifact** that packages everything a recipient needs to consume a shared subgraph *without a live publisher online* (the offline complement to the live G-CORE-3e ALPN serve path). §1–§2 below cover this half.
2. The **online Layer-C encrypt-to-recipient / Sealed-Sender** surface (`layer_c.rs`, the LARGEST file in the crate) — the codepoint-dispatched HPKE `mode_base` encrypt-to-recipient + Sealed-Sender drops (`0x6510` / `0x6520` / the `0x6610` MembershipSet group via `group_posture`). §3-LAYER-C covers this half.

The rest of §1's narrative describes the offline `DropBundle` half.

A `DropBundle` carries four things:

1. **What can be shared** — an authorized snapshot of a `RestrictedScope`-shaped SubgraphSpec (defines the bounded subgraph the bundle covers).
2. **The payload** — per-Node AEAD ciphertexts for the subgraph's content (`Vec<EncryptedNode>` produced by `benten-graph::aead_wrap`).
3. **The authority** — ONE signed `AuthorizationGrant` `{ucan, key_material, binding_sig}` that authorizes the recipient to derive keys + decrypt content (per `RATIFIED-sharing-and-confidentiality-2026-05-21.md` §R3).
4. **Envelope integrity** — an envelope-level Ed25519 signature by the issuer over the bundle header, sealing the outer layer (defense-in-depth per Spike G).

The bundle is consumed **offline** (filesystem-only, no live publisher) — this is the **Mode 2** sendme→Drop deployment shape per the 3-mode taxonomy in `.addl/phase-4-meta/00-implementation-plan.md` §3 G-CORE-3 def input-constraints refinement #6.

### Three modes (only modes 1 + 2 ship at v1-beta)

- **Mode 1 — OnlinePull** — recipient pulls from a live G-CORE-3e ALPN serve endpoint. The bundle wire-shape mode discriminator `DropContentMode::OnlinePull` names this for protocol negotiation.
- **Mode 2 — OfflineDrop** — recipient consumes the CBOR-on-disk bundle without a live publisher. This is the substantive shape this crate ships.
- **Mode 3 — InlineTiny** (≤16 KiB bundle inlined into a share URL) — **DEFERRED post-v1-beta**. Attempting to construct or parse a Mode-3 bundle yields typed `DropBundleError::UnsupportedDropMode`; the `DropContentMode` enum is `#[non_exhaustive]` so the future Mode-3 variant lands additively.

### Defense-in-depth (per Spike G)

Two cryptographic layers protect bundle integrity:

- **Envelope-sig (outer)** — Ed25519 signature by the issuer over the bundle header (`version`, `spec_cid`, `audience`, `mode`, `auth_grant`, `content_root_hash`, `key_material_hash`). Verifies the bundle header was not tampered post-issue. ErrorCode `E_DROP_BUNDLE_ENVELOPE_SIG_INVALID` fires on verification failure.
- **Per-Node AEAD authentication tags (inner)** — each `EncryptedNode` carries its own AAD-bound AEAD tag from `benten-crypto-suite::aead::wrap`. A tamper in any `content[i]` ciphertext fails at AEAD authentication during decrypt — *even if the envelope-sig still verifies*, because the envelope-sig binds the content **root hash** (not every byte). The defense-in-depth overhead is **<12%** vs envelope-only per Spike G measurement, pinned by `tf3f_envelope_plus_per_node_sig_defense_in_depth.rs`.

### Revocation reach (R6 reality, per RATIFIED-S&C §R6)

Drop bundles are **forever-valid once distributed**: revocation of the embedded UCAN cuts FUTURE serves on the online (G-CORE-3e) ALPN path, but already-derived keys remain decryptable (cryptographic limit — the key material is already in the recipient's hands). The mitigation is **tight `nbf`/`exp` + periodic key rotation**. This is an OPEN ARCHITECTURAL TRADE-OFF per Compromise #62 (SECURITY-POSTURE.md; revocation-reach was re-pointed from the in-tree #31 occupant per BR-2 — #31 is now the LAMPS Composite ML-DSA EUF-CMA-only compromise). Cross-referenced by `tf3f_revocation_reach_forever_valid_documented` pins.

---

## 2. Dependency chain

**Workspace deps (in):** `benten-core` (Cid, canonical-bytes serialization), `benten-caps` (`AuthorizationGrant`, `RestrictedScope` SubgraphSpec shape, UCAN body), `benten-crypto-suite` (Ed25519 envelope-sig + per-Node AEAD seam; the only crypto-primitive call site per CLAUDE.md baked-in #5), `benten-graph` (`aead_wrap::EncryptedNode` payload type — `EncryptedContent` is a type-alias), `benten-errors` (`E_DROP_BUNDLE_*` ErrorCode catalog).

**External deps:** `serde` + `serde_ipld_dagcbor` (CBOR-on-disk canonical bytes), `ed25519-dalek` is reached *only via `benten-crypto-suite`*, `thiserror`.

**Consumers (out):** at v1-beta, primarily the integration test suite + future Phase-4-Meta-Composing admin UI surface for Drop bundle authoring + admin import. Not consumed by the engine evaluator hot path (Drop bundles are filesystem artifacts; the engine consumes their *unwrapped* form once decrypted).

**Crates explicitly NOT reached:** `benten-engine`, `benten-eval`, `benten-platform-foundation`, `benten-renderer-tauri` — Drop bundles are a filesystem artifact format, not an engine-runtime concern.

**Features:** none (single-target).

---

## 3. Files inventory in `src/` (~3.8k LOC across 4 files)

**`layer_c.rs` is the largest file in the crate** — the crate is NOT offline-bundle-only. The offline `DropBundle` format (`lib.rs` / `bundle.rs` / `envelope_sig.rs`) is one half; the online **Layer-C encrypt-to-recipient / Sealed-Sender** surface (`layer_c.rs`, §3-LAYER-C below) is the other, larger half.

- **`lib.rs`** (94 LOC) — crate-level doc (the 3-mode taxonomy + defense-in-depth + revocation reach narrative). Pub re-exports: `DROP_BUNDLE_MAX_SIZE_BYTES`, `DropBundle`, `DropBundleError`, `DropBundleVersion`, `DropContentMode`, `EncryptedContent` + the `layer_c` module (`RecipientPublic` / `RecipientSecret` re-exports, `EncryptedEnvelope`, seal/open surface). `#![forbid(unsafe_code)]` + `#![deny(rust_2018_idioms)]`.

- **`bundle.rs`** (877 LOC) — the substantive offline-bundle surface. Owns:
  - **`DropBundle`** — the top-level CBOR-on-disk envelope. Fields: `version: DropBundleVersion`, `spec: RestrictedScopeSpec`, `audience: Did`, `mode: DropContentMode`, `auth_grant: AuthorizationGrant`, `content: Vec<EncryptedContent>`, `envelope_sig: EnvelopeSignature`. CBOR-serialized via `serde_ipld_dagcbor`.
  - **`DropBundleVersion`** — version discriminator (`V1` + `Synthetic` test-only arm). `#[non_exhaustive]`. Unknown reader-side versions surface `E_DROP_BUNDLE_VERSION_UNSUPPORTED`.
  - **`DropContentMode`** — `OnlinePull` (mode 1) or `OfflineDrop` (mode 2). `#[non_exhaustive]`. **No `InlineTiny` arm** at v1-beta (mode 3 deferred). Synthetic Mode-3 construction at runtime fires `E_DROP_BUNDLE_MODE3_INLINE_REJECTED`.
  - **`EncryptedContent`** — type-alias of `benten_graph::aead_wrap::EncryptedNode` (the per-Recipe content cell carried in the bundle).
  - **`DropBundleError`** — typed errors: `EnvelopeSignatureInvalid`, `VersionUnsupported`, `UnsupportedDropMode`, `BundleTooLarge`, `CborDecode`, `AeadAuthenticationFailed`, `AuthorizationGrantBindingInvalid`. Flows through the workspace `benten-errors::ErrorCode` catalog.
  - **`DROP_BUNDLE_MAX_SIZE_BYTES = 4096`** — 4 KiB upper bound on 5-Recipe bundles (Spike G measurement ~2688 bytes).
  - **Construction + parse** — `DropBundle::seal(issuer_keypair, spec, audience, mode, auth_grant, content) -> Result<Self, DropBundleError>` + `DropBundle::open(bytes, recipient_keymaterial) -> Result<UnwrappedBundle, DropBundleError>` (high-level surface; internal helpers handle the envelope-sig verify + per-Node AEAD unwrap loop).

- **`envelope_sig.rs`** (166 LOC) — the envelope-level Ed25519 signature helpers. `EnvelopeSignature` wire-bytes type + sign / verify functions over the bundle header transcript. Routes through `benten-crypto-suite::sig::SignatureSuite` rather than calling `ed25519-dalek` directly (per the only-call-site rule). The signed transcript is the canonical-byte serialization of `(version, spec_cid, audience, mode, auth_grant, content_root_hash, key_material_hash)` — note `content_root_hash` is hashed-over-payload (not each ciphertext byte), so per-Node AEAD-tag verification is the inner defense layer.

- **`layer_c.rs`** (2670 LOC, the LARGEST file in the crate) — the **online Layer-C encrypt-to-recipient / Sealed-Sender** surface (G-CORE-3f / F-full Layer-C). See §3-LAYER-C below.

### 3-LAYER-C. The Layer-C encrypt-to-recipient surface (`layer_c.rs`)

`layer_c.rs` is the codepoint-dispatched encrypt-to-recipient surface — a body is bulk-sealed under a fresh content-encryption-key (CEK), the CEK is HPKE-key-wrapped to the recipient via the unified X25519⊕ML-KEM-768 X-Wing KEM at codepoint `0x647a`, and a codepoint-discriminated plaintext AAD binds the recipient-targeting metadata. Per CLAUDE.md baked-in #5 all crypto routes through `benten_crypto_suite` — this module is concat / framing glue only. **The recipient key types are REAL (R9 GAP-1):** `layer_c` re-exports `RecipientPublic` / `RecipientSecret` from `benten_crypto_suite::cipher_suite`; seal takes `&RecipientPublic`, open takes `&RecipientSecret` (genuine OS-RNG entropy, unrecoverable from the public key; the deleted placeholder derived `sk = pk + 0x80`).

- **`0x6510` Sealed-Sender single-recipient DEFAULT (BR-1)** — `seal_sealed_sender` / `open_single`. The sender-DID lives INSIDE the ciphertext (recovered post-decrypt); the on-wire AAD binds ONLY `{aad_version, codepoint, audience, body_cid, recipient_key_generation}` — never the sender-DID. The paired non-default plaintext-sender path is `seal_plaintext_sender` (`0x6500`, binds the sender-DID into the plaintext AAD, U4).
- **`0x6520` group multi-stanza** — `seal_group_multi` / `seal_group_multi_plaintext_sender` / `open_group_stanza`. One `HpkeRecipientStanza` per recipient. The DEFAULT group send HONORS Sealed-Sender: each stanza's plaintext AAD is BLINDED — it carries the `audience_set_commitment` over the *sorted* recipient roster (never the raw roster; closes the #61 social-graph leak), `stanza_index`, `stanza_count` (truncation defense), and `recipient_key_generation`.
- **B2 sender ORIGIN-AUTHENTICATION (always-on, BD-2)** — every Sealed-Sender send carries, inside the once-sealed body region, one per-MESSAGE LAMPS-hybrid `id-MLDSA65-Ed25519-SHA512` (`0x0001`) signature over the domain-separated binding `M_auth` (`build_m_auth` / `SENDER_AUTH_DOMAIN`). The recipient resolves the recovered sender-DID to its hybrid verifying key and verifies BOTH halves post-decrypt, fail-closed (`SenderOriginAuthFailed`). SOUNDNESS-CRITICAL (F-2): the recipient re-derives the audience commitment + key-epoch generations from the set-state it INDEPENDENTLY HOLDS, never the wire value — so a re-target flips the commitment and a stale-generation replay flips a generation word.
- **`EncryptedEnvelope` (Inv-16)** — the codepoint-dispatched envelope (`HpkeBase` single / `HpkeMultiBase` group); `BindingContext` (the AAD source); `HpkeRecipientStanza`; `LayerCError` (`#[non_exhaustive]`).
- **Submodules:** `group_posture` (the `0x6610` MembershipSet group seal/open — `seal_membership_set_group` / `open_membership_set_group` + `GroupSealParams` / `GroupVerifyContext` / `GroupSealedEnvelope` / `GroupError`), `abuse_control`, `sealed_aad`, and the `domain_registry_mirror` byte-equality test.

**Endianness:** every wire integer is big-endian (M-19); the AAD leads with the dedicated `AAD_VERSION` (`0x01`) byte, DISTINCT from the envelope `ENVELOPE_FORMAT_VERSION` (`0x02`).

---

## 4. Test pins of note (in `tests/`)

Discoverable via `git ls-files crates/benten-drop/tests/tf3f_*.rs`. The G-CORE-3f wave shipped pins covering:

- **`tf3f_envelope_plus_per_node_sig_defense_in_depth.rs`** — the per-Node-AEAD-plus-envelope-sig defense-in-depth family (the G-CORE-3f wave consolidated the separate tamper pins here): `tf3f_per_node_ciphertext_tamper_detected_envelope_sig_still_valid` (per-Node AEAD authentication tag rejects payload-tampering even with a still-valid envelope-sig), `tf3f_tampered_envelope_sig_fails_before_per_node_decrypt` (envelope-sig forgery detection), and `tf3f_per_node_attestation_size_overhead_under_12_percent` (the <12% defense-in-depth overhead validated at Spike G).
- **`tf3f_no_mode3_inline_tiny_arm.rs`** — Mode-3 inline-tiny rejected at construct + parse time (defer-to-post-v1 contract): `tf3f_drop_content_mode_no_inline_tiny_arm` + `tf3f_inline_tiny_synthetic_rejected_via_unsupported_version_or_mode_typed`.
- **`tf3f_drop_bundle_offline_consume.rs`** — offline-consume round-trip + the size-cap measurement (`tf3f_dropbundle_5_recipe_bundle_within_size_envelope`, against `DROP_BUNDLE_MAX_SIZE_BYTES`) + unknown-`DropBundleVersion` rejection on the read side (`tf3f_dropbundle_future_version_yields_typed_unsupported_drop_version`).
- **`tf3f_revocation_reach_forever_valid_documented.rs`** — Spike G + RATIFIED-S&C §R6 reality documented + asserted at the codebase boundary (the *behavioral* assertion is that the bundle remains decryptable after upstream UCAN revocation, with the security narrative pointing the reader to the documented compromise + tight-`nbf`/`exp` mitigation).

---

## 5. Phase-4-Meta-Core wave provenance

- **G-CORE-3f (PR #1340)** — crate minted; Drop bundle format substantive landing; 3 new ErrorCodes (`DropBundleEnvelopeSigInvalid`, `DropBundleVersionUnsupported`, `DropBundleMode3InlineRejected`); CATALOG 168→171 across the G-CORE-3 family mints.
- **G-CORE-9 R1/R2/R3/R4/R4b FREEZE wave** — V1-FROZEN-INTERFACE item 15(j) coverage; the Drop bundle wire-bytes + the `DropBundle` / `DropBundleVersion` / `DropContentMode` pub surface enters the cargo-public-api required-failing baseline at `docs/public-api/benten-drop.txt`.

---

## 6. Load-bearing seams (where future-Phase-4-Meta-Composing work plugs in)

- **`DropContentMode` Mode-3 InlineTiny** — additive arm landing per CLAUDE.md additive-codepoint discipline. The enum is `#[non_exhaustive]` so adding the variant is non-breaking. Mode-3 will need a corresponding bundle-size renegotiation (smaller cap; encoded for inlining into share URLs).
- **`DropBundleVersion`** — additive version mint when wire format evolves. `#[non_exhaustive]` enum; old reader rejects unknown new version with `E_DROP_BUNDLE_VERSION_UNSUPPORTED` (forever-supported old-version property per V1-FROZEN-INTERFACE item 14 invariant).
- **Multi-recipient bundle** — current shape carries a single `audience: Did`. Multi-recipient extension (per RATIFIED-S&C future work) is an additive field landing on the next `DropBundleVersion::V2`.
- **Codepoint disambiguation (F-full R6 R1 finding F-21).** A Layer-C drop is **NOT** a MembershipSet. The Layer-C drop multi-recipient codepoint is `0x6520` (`LAYER_C_DROP_MULTI_RECIPIENT`, the R0.7-blinded Layer-C group multi-stanza band); the MembershipSet keying band is the separate `0x6600..=0x66FF` (`0x6600` set-keying / `0x6610` group AAD / `0x6620` reserved). The MembershipSet `0x6620` value is **reserved + ENCODE-ONLY at v1-beta** (the `SubsetRef` federation shape is reserved-and-refused / typed-reject; never a live decode arm until a future additive wave). The two bands are separately-frozen, codepoint-discriminated byte-strings — see `docs/CRYPTO-CODEPOINTS.md §4.0` + `docs/V1-WIRE-FORMAT-INVENTORY.md` item 25.
- **iroh-blobs swap-in** — the offline-Drop path complements the online G-CORE-3e ALPN path; future work to bridge the two (e.g., a Drop bundle that hyperlinks back to a still-live source for fresh content) is a Phase-4-Meta-Composing window deliverable.

---

## 7. v1-FROZEN-INTERFACE.md coupling

This crate's public surface IS frozen as part of **V1-FROZEN-INTERFACE.md item 15** (Sharing & Confidentiality public surface). Specifically:

- **Item 15(a) SubgraphSpec primitive** — `DropBundle::spec` carries a `RestrictedScopeSpec` shaped per item 15(a)'s 4-thing thin core.
- **Item 15(d) AuthorizationGrant envelope** — `DropBundle::auth_grant` carries the ONE signed `AuthorizationGrant {ucan, key_material, binding_sig}` per item 15(d).
- **Item 15(i) Revocation reach** — the forever-valid-once-distributed property is documented at Compromise #62 (SECURITY-POSTURE.md; re-pointed from the in-tree #31 occupant per BR-2) per item 15(i); the open architectural trade-off is MITIGATED (not closed) by tight `nbf`/`exp` + key rotation.

The cargo-public-api baseline at `docs/public-api/benten-drop.txt` is the byte-stable v1-beta surface; drift fails CI per the G-CORE-9 R1 Fork 3 FREEZE-FLIP.
