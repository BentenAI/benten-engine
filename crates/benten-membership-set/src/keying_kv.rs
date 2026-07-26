//! The Inv-19 type-restricted `K(V)` derivation API.
//!
//! `K(V)` (the membership version-node key, F4-014) may ONLY be derived for an
//! **immutable** content-addressed target. Inv-19 forbids binding key material
//! to a MUTABLE [`benten_core::version::Anchor`] CID: the Anchor's CURRENT
//! pointer moves across an `append_version`, so a key bound to an Anchor would
//! silently re-target as the chain advances. The two permitted targets are an
//! immutable Version-Node-CID and a MembershipSet identity (Inv-20 clause-f —
//! set-identity is Inv-21-stable).
//!
//! This module is the **type-restriction front-door**: the
//! [`CidTarget`]-typed [`derive_kv`] is the sole SUPPORTED `K(V)` derivation
//! API.
//!
//! **Honest scope of the Inv-19 guarantee (R6-tail F-06).** The rejection is a
//! RUNTIME typed-reject (`Err(`[`KvError::TargetNotImmutable`]`)`), NOT a
//! compile-time / type-level impossibility: [`CidTarget::MutableAnchor`] is a
//! publicly constructible variant, and the BLAKE3-KDF context label
//! `crate::keying::KV_DERIVE_CONTEXT` is public (and deliberately mirrored as
//! `benten_crypto_suite::domain_registry::KV_DERIVE_CONTEXT` for the
//! prefix-free domain-tag corpus), so a caller that bypasses this front-door
//! can reproduce `K(V)` bytes for a forbidden target. The invariant is held by
//! the discipline of going through [`derive_kv`] plus its runtime check — do
//! NOT read this module as a structural impossibility. (Not a live bypass at
//! HEAD: `derive_kv` has zero production callers; the production keying-path
//! wiring is Row D-64 in `docs/V1-FROZEN-INTERFACE-DEFERRED.md`.)
//!
//! It lives in a NEW submodule (re-exported from [`crate::keying`]) so the
//! `K(V)` type-restriction concern stays disjoint from the gossip keyed-MAC
//! routing that also lives in `keying`. The actual KDF primitive is the BLAKE3
//! KDF (`blake3::derive_key`) — a vetted upstream, NOT a forked construction
//! (#5: never fork crypto primitives).

/// The kind of CID a `K(V)` derivation is being asked to bind to (Inv-19).
///
/// ONLY [`CidTarget::ImmutableVersionNode`] and [`CidTarget::MembershipSet`]
/// are permitted targets; a [`CidTarget::MutableAnchor`] MUST be rejected with
/// [`KvError::TargetNotImmutable`].
///
/// # The KDF binds the FULL self-describing CIDv1 (Ben-ratified 2026-06-05)
///
/// Each arm carries the **full 36-byte self-describing CIDv1**
/// (`0x01 0x71 0x1e 0x20 || the 32-byte BLAKE3 digest`), NOT a bare fixed-32-byte
/// digest. A bare 32-byte digest would bake a hash-width assumption into the
/// keying input, contradicting the crypto-agility framing (#5): the
/// self-describing multiformats prefix is the permanent commitment, and the key
/// must be derived over those self-describing bytes so a future hash-width swap
/// re-domain-separates the key automatically (R0.7 §body_cid BR, lines 603-607;
/// applies to BOTH `0x6510`/`0x6610`). [`derive_kv`] feeds these 36 bytes into
/// the BLAKE3 KDF; this restores the R4-frozen `K(V)` golden.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CidTarget {
    /// An immutable Version-Node CID — the canonical content-addressed target
    /// whose bytes never change (PERMITTED; the Inv-19-safe binding). Carries
    /// the FULL 36-byte self-describing CIDv1 (the KDF input).
    ImmutableVersionNode([u8; 36]),
    /// A MembershipSet identity target (PERMITTED — set-identity is
    /// Inv-21-stable per Inv-20 clause-f). Carries the FULL 36-byte
    /// self-describing CIDv1 (the KDF input).
    MembershipSet([u8; 36]),
    /// A mutable [`benten_core::version::Anchor`] CID — its CURRENT pointer
    /// moves, so a key bound here would silently re-target (FORBIDDEN by
    /// Inv-19). Carries the FULL 36-byte self-describing CIDv1 for shape
    /// parity, though it is rejected before any derivation runs.
    MutableAnchor([u8; 36]),
}

/// The typed error surface for the Inv-19 `K(V)` type-restriction.
///
/// `TargetNotImmutable` crosses the public surface (the Inv-19 rejection is an
/// observable typed failure), so it carries a first-class
/// `benten_errors::ErrorCode` via [`KvError::error_code`] (§3.5g mirror).
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum KvError {
    /// The derivation target is not an immutable Version-Node-CID (or a
    /// MembershipSet identity) — an Inv-19 violation. Binding `K(V)` to a
    /// mutable Anchor (whose CURRENT pointer moves) would silently re-target
    /// the key; the API fails closed instead.
    #[error("K(V) derivation target is not an immutable Version-Node-CID (Inv-19)")]
    TargetNotImmutable,
}

impl KvError {
    /// The stable `benten_errors::ErrorCode` for this error (§3.5g catalog
    /// mirror). The Inv-19 type-restriction rejection maps to the dedicated
    /// `E_KV_TARGET_NOT_IMMUTABLE` catalog code.
    #[must_use]
    pub fn error_code(&self) -> benten_errors::ErrorCode {
        match self {
            KvError::TargetNotImmutable => benten_errors::ErrorCode::KvTargetNotImmutable,
        }
    }
}

/// Derive the membership version-node key `K(V)` for a content-addressed
/// target, ENFORCING the Inv-19 type-restriction.
///
/// ACCEPTS [`CidTarget::ImmutableVersionNode`] / [`CidTarget::MembershipSet`];
/// REJECTS [`CidTarget::MutableAnchor`] with [`KvError::TargetNotImmutable`].
/// The KDF primitive is the BLAKE3 KDF (`blake3::derive_key`, a vetted
/// upstream — NOT a forked construction; #5 ONLY-call-site discipline).
///
/// The KDF INPUT is the **full 36-byte self-describing CIDv1** carried by the
/// permitted arm (`0x01 0x71 0x1e 0x20 || the 32-byte BLAKE3 digest`), NOT a
/// bare 32-byte digest — hashing the self-describing bytes avoids baking a
/// hash-width assumption into the keying input (R0.7 §body_cid BR;
/// crypto-agility #5). The context label is unchanged.
///
/// # Errors
///
/// Returns [`KvError::TargetNotImmutable`] if the target is a mutable Anchor
/// CID — binding key material to an Anchor (whose CURRENT pointer moves) would
/// silently re-target the key as the chain advances (the bug Inv-19 forbids).
pub fn derive_kv(target: CidTarget) -> Result<[u8; 32], KvError> {
    // The FULL self-describing CIDv1 (36 bytes) is the KDF input (Ben-ratified
    // 2026-06-05; R0.7 §body_cid BR / crypto-agility #5) — NOT the bare 32-byte
    // digest, which would bake a hash-width assumption into the keying input.
    let cid_bytes = match target {
        CidTarget::ImmutableVersionNode(cid) | CidTarget::MembershipSet(cid) => cid,
        CidTarget::MutableAnchor(_) => return Err(KvError::TargetNotImmutable),
    };
    // Reuse the canonical K(V) BLAKE3-KDF context label that lives alongside
    // the keying formula in `keying` (single source of truth — the formula and
    // its type-restriction agree on the same domain-separation label).
    Ok(blake3::derive_key(
        crate::keying::KV_DERIVE_CONTEXT,
        &cid_bytes,
    ))
}
