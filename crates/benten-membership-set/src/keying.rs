//! The multi-stanza-HPKE keying glue (mechanism-half).
//!
//! This module is the membership crate's keying minimum. It derives the
//! membership-scoped keys and hands OPAQUE bytes to `benten-crypto-suite` for
//! every actual crypto primitive (#5 ONLY-call-site). It NEVER constructs a
//! low-level crypto primitive directly (no SHA-3 / ChaCha20 / ML-KEM / ML-DSA /
//! X25519 primitive imports) — the F-CRATE-2 grep-fence enforces this; the only
//! primitive used here is the BLAKE3 KDF (a vetted upstream, the project hash).
//!
//! Two membership-scoped derivations:
//!
//! - **`K(V)` (membership version-node key, F4-014)** — derived via
//!   `blake3::derive_key("benten-membership-set:K(V):v1", cid)`, the BLAKE3
//!   KDF (a vetted upstream primitive, not a forked construction). The engine
//!   owns the `K(V)` type-restriction (the `f_inv19_1` cross-wave pin); this
//!   crate owns the keying formula.
//! - **`K(N)` (per-Node content key)** — the membership-set content key the
//!   role-keyed [`crate::role::derive_member_content_key`] gate hands out for
//!   content-bearing roles. Derived from the membership content-key domain so
//!   it is distinct from `K(V)`.

/// The BLAKE3-KDF context label for the membership version-node key `K(V)`
/// (F4-014). `K(V) = blake3::derive_key("benten-membership-set:K(V):v1", cid)`.
pub const KV_DERIVE_CONTEXT: &str = "benten-membership-set:K(V):v1";

/// The BLAKE3-KDF context label for the per-Node content key `K(N)`.
pub const KN_DERIVE_CONTEXT: &str = "benten-membership-set:K(N):v1";

// The membership version-node key `K(V)` derivation (F4-014) lives in the
// [`crate::keying_kv`] submodule as the Inv-19 TYPE-RESTRICTED front-door
// (`derive_kv(CidTarget) -> Result<[u8; 32], KvError>`): it ACCEPTS an
// immutable Version-Node-CID / MembershipSet identity and REJECTS a mutable
// Anchor CID (the bug Inv-19 forbids — an Anchor's CURRENT pointer moves, so a
// key bound there would silently re-target). The `K(V)` type-restriction is a
// distinct concern from the gossip keyed-MAC routing that also lives in this
// module, so it is sliced into `keying_kv` (the w-gov-audit / w-ms-sync
// file-disjointness slice). The typed surface is re-exported here so callers
// reach it as `benten_membership_set::keying::{derive_kv, CidTarget, KvError}`.
pub use crate::keying_kv::{CidTarget, KvError, derive_kv};

/// Derive a per-Node content key `K(N)` for a Node CID. Uses the BLAKE3 KDF
/// (distinct context label from `K(V)` so the two derivations never collide).
#[must_use]
pub fn derive_member_key(node_cid: &[u8]) -> Vec<u8> {
    blake3::derive_key(KN_DERIVE_CONTEXT, node_cid).to_vec()
}
