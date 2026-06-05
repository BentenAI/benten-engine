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

/// Compute the BLINDED iroh-gossip rendezvous topic (F-GOSSIP-2 / §3.9 /
/// Compromise #61). The topic is a keyed MAC over `(K_Set, set_id, generation)`
/// with **NO time input**, so every current-generation member derives the SAME
/// topic from any wall-clock:
///
/// ```text
/// topic = truncate_32(blake3::keyed_hash(
///     K_Set,
///     membership_set_id || BE(generation)
/// ))
/// ```
///
/// This routes the keyed MAC through the SAME §3.9 primitive
/// ([`crate::aad::membership_set_id_commitment`] family): R0.7 §4.1: `HMAC` =
/// `blake3::keyed_hash` — the native BLAKE3 keyed MAC; the crate carries **no**
/// `hmac`/`sha2` dependency (#5 ONLY-call-site, NEVER a forked construction).
/// BLAKE3's output is 32 bytes wide, so the `truncate_32` is the identity.
///
/// ## Preimage = `membership_set_id || BE(generation)` (NO domain label)
///
/// The R0.7 §3.9 (authoritative Transport section, plan lines 865-866) freezes
/// the gossip-topic primitive as
/// `blake3::keyed_hash(K_Set, membership_set_id || generation_summary)` — i.e.
/// **WITHOUT** the `"benten:setid:v1"` domain-separation label that the §3.10
/// `membership_set_id_commitment` AAD field carries. The two are therefore
/// distinct keyed-MAC preimages over the SAME primitive (the topic appends
/// `BE(generation)`; the AAD commitment prepends the label) — they never
/// collide. The R4-frozen golden vector (`f_gossip_2_topic_absolute_golden_vector_be`)
/// is computed against THIS no-label §3.9 construction. (The dispatch brief's
/// M-20 line and §3.10's "the SAME construction §3.9 uses" prose both echoed the
/// *labelled* commitment formula; that is an imprecise cross-reference — §3.9
/// owns the gossip topic and its frozen primitive is the no-label form, which
/// the existing golden confirms byte-for-byte. FLAGGED-FOR-BEN.)
///
/// Byte-order is freeze-gating: the generation counter is encoded
/// **big-endian** (F4-024 M-20). An observer without `K_Set` cannot link or
/// recover `membership_set_id` from the topic (it is a keyed hash). A fork that
/// rotates the generation rotates the topic (#61 fingerprint defense).
///
/// `gossip = liveness-only`: this topic is the rendezvous label for the
/// notification channel. Convergence is backed by the MST anti-entropy backstop
/// (`benten_sync::mst`), NEVER gossip — dropping all gossip still converges.
#[must_use]
pub fn gossip_topic(k_set: &[u8; 32], membership_set_id: &[u8], generation: u32) -> [u8; 32] {
    let mut msg = Vec::with_capacity(membership_set_id.len() + 4);
    msg.extend_from_slice(membership_set_id);
    // generation_summary = the set-generation counter (BE u32), NOT a per-member
    // vector (O-5 / m-11). BIG-ENDIAN is the freeze-gating byte order (F4-024).
    msg.extend_from_slice(&generation.to_be_bytes());
    blake3::keyed_hash(k_set, &msg).into()
}
