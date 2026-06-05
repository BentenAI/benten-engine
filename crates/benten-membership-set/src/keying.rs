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

/// Derive the membership version-node key `K(V)` for a version-node CID
/// (F4-014). Uses the BLAKE3 KDF (`blake3::derive_key`), a vetted upstream
/// primitive — NOT a forked construction.
///
/// The engine owns the `K(V)` *type-restriction* (the `f_inv19_1` cross-wave
/// pin restricts which Node kinds may key off `K(V)`); this crate owns the
/// *keying formula* the engine restriction agrees with.
#[must_use]
pub fn derive_kv(version_node_cid: &[u8]) -> [u8; 32] {
    blake3::derive_key(KV_DERIVE_CONTEXT, version_node_cid)
}

/// Derive a per-Node content key `K(N)` for a Node CID. Uses the BLAKE3 KDF
/// (distinct context label from `K(V)` so the two derivations never collide).
#[must_use]
pub fn derive_member_key(node_cid: &[u8]) -> Vec<u8> {
    blake3::derive_key(KN_DERIVE_CONTEXT, node_cid).to_vec()
}
