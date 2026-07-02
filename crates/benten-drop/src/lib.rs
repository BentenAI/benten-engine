//! Drop bundle format — G-CORE-3f Phase-4-Meta-Core.
//!
//! # What this crate is
//!
//! `benten-drop` is the **self-contained offline-bundle format** for
//! Benten's Sharing & Confidentiality stack. A `DropBundle` is a single
//! CBOR-on-disk artifact that:
//!
//! 1. Carries an authorized snapshot of a `RestrictedScope`-shaped
//!    SubgraphSpec (the **what-can-be-shared** half).
//! 2. Carries the per-Node AEAD ciphertexts for the subgraph's content
//!    (the **payload** half — `Vec<EncryptedNode>` produced by
//!    `benten-graph::aead_wrap`).
//! 3. Carries the ONE signed `AuthorizationGrant`
//!    `{ucan, key_material, binding_sig}` that authorizes the recipient
//!    to derive keys + decrypt content (the **authority** half — per
//!    `RATIFIED-sharing-and-confidentiality-2026-05-21.md` §R3).
//! 4. Is sealed with an **envelope-level Ed25519 signature** over the
//!    bundle header (defense-in-depth outer layer per Spike G).
//!
//! The bundle is consumed **offline** (filesystem-only, no live
//! publisher) — this is the **Mode 2** sendme→Drop deployment shape
//! per the 3-mode taxonomy in
//! `.addl/phase-4-meta/00-implementation-plan.md` §3 G-CORE-3 def
//! input-constraints refinement #6 (L341). **Mode 3** (inline-tiny:
//! bundle ≤16KiB inlined into share URL) is **deferred to post-v1**;
//! attempting to construct or parse a Mode-3 bundle yields typed
//! [`DropBundleError::UnsupportedDropMode`].
//!
//! # Defense-in-depth (per Spike G)
//!
//! Two cryptographic layers protect bundle integrity:
//!
//! - **Envelope-sig (outer)**: an Ed25519 signature by the issuer over
//!   the bundle header (`version`, `spec_cid`, `audience`, `mode`,
//!   `auth_grant`, `content_root_hash`, `key_material_hash`). Verifies
//!   the bundle header was not tampered post-issue.
//! - **Per-Node AEAD authentication tags (inner)**: each
//!   `EncryptedNode` carries its own AAD-bound AEAD tag
//!   (`benten-crypto-suite::aead::wrap`). A tamper in any `content[i]`
//!   ciphertext fails at AEAD authentication during decrypt — even if
//!   the envelope-sig still verifies (because the envelope-sig binds
//!   the content **root hash**, not every byte).
//!
//! Spike G measured the per-Node-sig defense-in-depth overhead at
//! **<12%** vs envelope-only — proved out by
//! `tf3f_envelope_plus_per_node_sig_defense_in_depth.rs` PIN 3.
//!
//! # Revocation reach (R6 reality)
//!
//! Per `RATIFIED-sharing-and-confidentiality-2026-05-21.md` §R6,
//! Drop bundles are **forever-valid once distributed**: revocation of
//! the embedded UCAN cuts FUTURE serves on the online (G-CORE-3e)
//! ALPN path, but already-derived keys remain decryptable
//! (cryptographic limit — the key material is already in the
//! recipient's hands). The mitigation is **tight `nbf`/`exp` +
//! periodic key rotation**. Documented at
//! `docs/SECURITY-POSTURE.md` § "Revocation reach"; cross-referenced
//! by `tf3f_revocation_reach_forever_valid_documented` pins.
//!
//! # Surface
//!
//! - [`DropBundle`] — the top-level CBOR-on-disk envelope.
//! - [`DropBundleVersion`] — version discriminator (known +
//!   `Synthetic` test-only arm).
//! - [`DropContentMode`] — `OnlinePull` (mode 1) or `OfflineDrop`
//!   (mode 2). **No `InlineTiny` arm** (mode 3 deferred to post-v1).
//! - [`EncryptedContent`] — the per-Recipe content cell carried in
//!   the bundle (alias of `benten_graph::aead_wrap::EncryptedNode`).
//! - [`DropBundleError`] — typed errors for the bundle surface.
//! - [`DROP_BUNDLE_MAX_SIZE_BYTES`] — 4 KiB upper bound on
//!   5-Recipe bundles (Spike G measurement ~2688 bytes).
//!
//! # ErrorCode mints
//!
//! - `E_DROP_BUNDLE_ENVELOPE_SIG_INVALID` — envelope-level Ed25519
//!   signature does not verify against the bundle header.
//! - `E_DROP_BUNDLE_VERSION_UNSUPPORTED` — reader sees an unknown
//!   version discriminator.
//! - `E_DROP_BUNDLE_MODE3_INLINE_REJECTED` — synthetic Mode-3
//!   inline-tiny bundle was rejected at parse time (defer-to-post-v1
//!   contract).

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod bundle;
pub mod envelope_sig;
pub mod layer_c;

pub use bundle::{
    DROP_BUNDLE_MAX_SIZE_BYTES, DropBundle, DropBundleError, DropBundleVersion, DropContentMode,
    EncryptedContent,
};
