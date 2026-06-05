//! Keying glue — the thin keying-mechanism that delegates ALL crypto primitives
//! to `benten-crypto-suite` (the #5 ONLY-call-site discipline).
//!
//! # `K(N)` content keys (RBAC-gated)
//!
//! Content-bearing roles derive per-Node keys; **Invitee (RoleId=0) derives
//! ZERO content** (M-11 hard floor — no `K(N)`, pre-acceptance handshake only).
//! The membership crate NEVER constructs an HKDF / AEAD / KEM primitive
//! directly: the production derivation routes through
//! `benten_crypto_suite::structural_kdf` (`derive_root` / `derive_step`).
//!
//! # `K(V)` version-node keying (F4-014)
//!
//! Membership version-node keys derive via
//! `blake3::derive_key("benten-membership-set:K(V):v1", cid)` — the BLAKE3 KDF
//! context-string derivation (a vetted-upstream primitive; no fork). The engine
//! owns the `K(V)` type-restriction (the engine `f_inv19_1` K(V) arm); this
//! crate owns the *keying* (the context-string + the derive). The two agree on
//! the context-string label so cross-wave derivation is byte-identical.

extern crate alloc;
use alloc::vec::Vec;

use benten_crypto_suite::structural_kdf::{StructuralKdfKey, derive_step};

use crate::kind::MEMBERSHIP_SET_ENCRYPTION;
use crate::role::RoleId;

/// The `K(V)` membership-version-node KDF context-string (F4-014). The engine's
/// `K(V)` type-restriction agrees on this label so the derivation is
/// cross-wave byte-identical.
pub const KV_DERIVE_CONTEXT: &str = "benten-membership-set:K(V):v1";

/// The membership-set canonical content-derivation edge-label (the per-Node
/// `K(N)` step is labelled `"member-content"` along the keying walk).
pub const MEMBER_CONTENT_EDGE_LABEL: &[u8] = b"member-content";

/// Derive the per-Node content key `K(N)` for a member at `role`, given the
/// predecessor key and the Node CID along a canonical-path walk edge.
///
/// **Invitee derives `None`** (the M-11 zero-content floor). Every
/// content-bearing role (`Viewer`/`Member`/`Moderator`/`Admin`) derives a key
/// by delegating to `benten_crypto_suite::structural_kdf::derive_step` (the
/// path-tagged HKDF-SHA256 step; #5 ONLY-call-site — this crate never
/// reimplements HKDF). Returns the raw 32-byte material for the AEAD-key feed.
#[must_use]
pub fn derive_member_step_key(
    role: RoleId,
    predecessor: &StructuralKdfKey,
    edge_label: &[u8],
    node_cid: &[u8],
) -> Option<[u8; 32]> {
    if !role.derives_content() {
        // Invitee = pre-acceptance handshake only: NO content key (M-11).
        return None;
    }
    let k = derive_step(predecessor, edge_label, node_cid);
    Some(k.as_bytes())
}

/// The per-Node content-key derivation context-string (the BLAKE3 KDF
/// domain-separation label for the node-CID-only `K(N)` derivation path).
pub const MEMBER_CONTENT_DERIVE_CONTEXT: &str = "benten-membership-set:K(N):v1";

/// Derive a member's per-Node content key `K(N)` from a Node CID, RBAC-gated by
/// `role`. **Invitee derives `None`** (the M-11 zero-content floor); every
/// content-bearing role derives a deterministic 32-byte key via
/// `blake3::derive_key` (a vetted-upstream KDF — #5 ONLY-call-site; the
/// membership crate never reimplements a KDF).
///
/// This is the node-CID-only convenience derivation; the full path-tagged
/// keying walk (with a real predecessor key) uses [`derive_member_step_key`].
#[must_use]
pub fn derive_member_content_key(role: RoleId, node_cid: &[u8]) -> Option<Vec<u8>> {
    if !role.derives_content() {
        // Invitee = pre-acceptance handshake only: NO content key (M-11).
        return None;
    }
    Some(blake3::derive_key(MEMBER_CONTENT_DERIVE_CONTEXT, node_cid).to_vec())
}

/// Whether a role is granted a read capability (Invitee gets none). The
/// membership-side mirror of [`RoleId::grants_read_cap`].
#[must_use]
pub const fn grants_read_cap(role: RoleId) -> bool {
    role.grants_read_cap()
}

/// Whether a role derives any content (any `K(N)`). The membership-side mirror
/// of [`RoleId::derives_content`] used by the keying gate.
#[must_use]
pub const fn role_derives_content(role: RoleId) -> bool {
    role.derives_content()
}

/// Derive the membership version-node key `K(V)` via
/// `blake3::derive_key(KV_DERIVE_CONTEXT, cid)` (F4-014). The context-string is
/// the cross-wave agreement point with the engine's `K(V)` type-restriction.
#[must_use]
pub fn derive_version_node_key(cid: &[u8]) -> [u8; 32] {
    blake3::derive_key(KV_DERIVE_CONTEXT, cid)
}

/// The set-keying codepoint every Kind binds to (`0x6600`) — re-exported here
/// so keying callers don't reach into [`crate::kind`] for the constant.
pub const SET_KEYING_CODEPOINT: u16 = MEMBERSHIP_SET_ENCRYPTION;

/// Convenience: feed a content key into the AEAD-key newtype path (the keying
/// glue's only crypto-primitive touchpoint is the typed crypto-suite API). The
/// raw 32-byte material is what the crypto-suite AEAD wrap consumes.
#[must_use]
pub fn content_key_bytes(k: &StructuralKdfKey) -> Vec<u8> {
    k.as_bytes().to_vec()
}
