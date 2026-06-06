# benten-drop — Internals

A plain-English, code-grounded tour of the `benten-drop` crate — the **14th workspace crate** added in Phase-4-Meta-Core (G-CORE-3f at PR #1340 minting the Drop bundle format; G-CORE-3f follow-ups landing through G-CORE-9 FREEZE). Read-only audit. Audience: a developer landing in this crate fresh and trying to find the load-bearing seams. Last refreshed: 2026-05-24 against main HEAD `a0b75637` (post `phase-4-meta-core/r4b-r1-fix-pass` base `4bbc4cac`).

---

## 1. What this crate does

`benten-drop` is the **self-contained offline-bundle format** for Benten's Sharing & Confidentiality (S&C) stack. A `DropBundle` is a single **CBOR-on-disk artifact** that packages everything a recipient needs to consume a shared subgraph *without a live publisher online* — the offline complement to the live G-CORE-3e ALPN serve path.

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

Drop bundles are **forever-valid once distributed**: revocation of the embedded UCAN cuts FUTURE serves on the online (G-CORE-3e) ALPN path, but already-derived keys remain decryptable (cryptographic limit — the key material is already in the recipient's hands). The mitigation is **tight `nbf`/`exp` + periodic key rotation**. This is an OPEN ARCHITECTURAL TRADE-OFF per Compromise #31 (SECURITY-POSTURE.md). Cross-referenced by `tf3f_revocation_reach_forever_valid_documented` pins.

---

## 2. Dependency chain

**Workspace deps (in):** `benten-core` (Cid, canonical-bytes serialization), `benten-caps` (`AuthorizationGrant`, `RestrictedScope` SubgraphSpec shape, UCAN body), `benten-crypto-suite` (Ed25519 envelope-sig + per-Node AEAD seam; the only crypto-primitive call site per CLAUDE.md baked-in #5), `benten-graph` (`aead_wrap::EncryptedNode` payload type — `EncryptedContent` is a type-alias), `benten-errors` (`E_DROP_BUNDLE_*` ErrorCode catalog).

**External deps:** `serde` + `serde_ipld_dagcbor` (CBOR-on-disk canonical bytes), `ed25519-dalek` is reached *only via `benten-crypto-suite`*, `thiserror`.

**Consumers (out):** at v1-beta, primarily the integration test suite + future Phase-4-Meta-Composing admin UI surface for Drop bundle authoring + admin import. Not consumed by the engine evaluator hot path (Drop bundles are filesystem artifacts; the engine consumes their *unwrapped* form once decrypted).

**Crates explicitly NOT reached:** `benten-engine`, `benten-eval`, `benten-platform-foundation`, `benten-renderer-tauri` — Drop bundles are a filesystem artifact format, not an engine-runtime concern.

**Features:** none (single-target).

---

## 3. Files inventory in `src/` (~1.0k LOC across 3 files)

- **`lib.rs`** (93 LOC) — crate-level doc (the 3-mode taxonomy + defense-in-depth + revocation reach narrative). Pub re-exports: `DROP_BUNDLE_MAX_SIZE_BYTES`, `DropBundle`, `DropBundleError`, `DropBundleVersion`, `DropContentMode`, `EncryptedContent`. `#![forbid(unsafe_code)]` + `#![deny(rust_2018_idioms)]`.

- **`bundle.rs`** (793 LOC) — the substantive surface. Owns:
  - **`DropBundle`** — the top-level CBOR-on-disk envelope. Fields: `version: DropBundleVersion`, `spec: RestrictedScopeSpec`, `audience: Did`, `mode: DropContentMode`, `auth_grant: AuthorizationGrant`, `content: Vec<EncryptedContent>`, `envelope_sig: EnvelopeSignature`. CBOR-serialized via `serde_ipld_dagcbor`.
  - **`DropBundleVersion`** — version discriminator (`V1` + `Synthetic` test-only arm). `#[non_exhaustive]`. Unknown reader-side versions surface `E_DROP_BUNDLE_VERSION_UNSUPPORTED`.
  - **`DropContentMode`** — `OnlinePull` (mode 1) or `OfflineDrop` (mode 2). `#[non_exhaustive]`. **No `InlineTiny` arm** at v1-beta (mode 3 deferred). Synthetic Mode-3 construction at runtime fires `E_DROP_BUNDLE_MODE3_INLINE_REJECTED`.
  - **`EncryptedContent`** — type-alias of `benten_graph::aead_wrap::EncryptedNode` (the per-Recipe content cell carried in the bundle).
  - **`DropBundleError`** — typed errors: `EnvelopeSignatureInvalid`, `VersionUnsupported`, `UnsupportedDropMode`, `BundleTooLarge`, `CborDecode`, `AeadAuthenticationFailed`, `AuthorizationGrantBindingInvalid`. Flows through the workspace `benten-errors::ErrorCode` catalog.
  - **`DROP_BUNDLE_MAX_SIZE_BYTES = 4096`** — 4 KiB upper bound on 5-Recipe bundles (Spike G measurement ~2688 bytes).
  - **Construction + parse** — `DropBundle::seal(issuer_keypair, spec, audience, mode, auth_grant, content) -> Result<Self, DropBundleError>` + `DropBundle::open(bytes, recipient_keymaterial) -> Result<UnwrappedBundle, DropBundleError>` (high-level surface; internal helpers handle the envelope-sig verify + per-Node AEAD unwrap loop).

- **`envelope_sig.rs`** (148 LOC) — the envelope-level Ed25519 signature helpers. `EnvelopeSignature` wire-bytes type + sign / verify functions over the bundle header transcript. Routes through `benten-crypto-suite::sig::SignatureSuite` rather than calling `ed25519-dalek` directly (per the only-call-site rule). The signed transcript is the canonical-byte serialization of `(version, spec_cid, audience, mode, auth_grant, content_root_hash, key_material_hash)` — note `content_root_hash` is hashed-over-payload (not each ciphertext byte), so per-Node AEAD-tag verification is the inner defense layer.

---

## 4. Test pins of note (in `tests/`)

Discoverable via `git ls-files crates/benten-drop/tests/tf3f_*.rs`. The G-CORE-3f wave shipped pins covering:

- **`tf3f_envelope_plus_per_node_sig_defense_in_depth.rs`** — PIN 3 measures the <12% defense-in-depth overhead validated at Spike G.
- **`tf3f_envelope_sig_tamper_rejected.rs`** — envelope-sig forgery detection.
- **`tf3f_per_node_aead_tamper_rejected.rs`** — per-Node AEAD authentication tag rejects payload-tampering even with valid envelope-sig.
- **`tf3f_mode3_inline_rejected.rs`** — Mode-3 inline-tiny rejected at construct + parse time (defer-to-post-v1 contract).
- **`tf3f_version_unsupported_*.rs`** — unknown `DropBundleVersion` rejection on the read side.
- **`tf3f_revocation_reach_forever_valid_documented.rs`** — Spike G + RATIFIED-S&C §R6 reality documented + asserted at the codebase boundary (the *behavioral* assertion is that the bundle remains decryptable after upstream UCAN revocation, with the security narrative pointing the reader to the documented compromise + tight-`nbf`/`exp` mitigation).
- **`tf3f_*_bundle_under_4k_bytes.rs`** — size-cap measurement against `DROP_BUNDLE_MAX_SIZE_BYTES`.

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
- **Item 15(i) Revocation reach** — the forever-valid-once-distributed property is documented at Compromise #31 (SECURITY-POSTURE.md) per item 15(i); the open architectural trade-off is MITIGATED (not closed) by tight `nbf`/`exp` + key rotation.

The cargo-public-api baseline at `docs/public-api/benten-drop.txt` is the byte-stable v1-beta surface; drift fails CI per the G-CORE-9 R1 Fork 3 FREEZE-FLIP.
