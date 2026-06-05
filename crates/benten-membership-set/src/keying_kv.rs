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
//! [`CidTarget`]-typed [`derive_kv`] is the only public way to derive `K(V)`.
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CidTarget {
    /// An immutable Version-Node CID — the canonical content-addressed target
    /// whose bytes never change (PERMITTED; the Inv-19-safe binding).
    ImmutableVersionNode([u8; 32]),
    /// A MembershipSet identity target (PERMITTED — set-identity is
    /// Inv-21-stable per Inv-20 clause-f).
    MembershipSet([u8; 32]),
    /// A mutable [`benten_core::version::Anchor`] CID — its CURRENT pointer
    /// moves, so a key bound here would silently re-target (FORBIDDEN by
    /// Inv-19).
    MutableAnchor([u8; 32]),
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
/// # Errors
///
/// Returns [`KvError::TargetNotImmutable`] if the target is a mutable Anchor
/// CID — binding key material to an Anchor (whose CURRENT pointer moves) would
/// silently re-target the key as the chain advances (the bug Inv-19 forbids).
pub fn derive_kv(target: CidTarget) -> Result<[u8; 32], KvError> {
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
