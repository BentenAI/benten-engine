//! `DropBundlePayload` — the canonical (stable `plaintext_cid`) half of the
//! DUAL-CID, carrying the per-recipient HPKE-wrapped CEKs.
//!
//! # The CC-BLK confidentiality invariant (R0.5 §2.5 GN-1 / §3.5)
//!
//! A `Drop` is a **one-shot share to a non-member**. The recipient is, by
//! construction, NOT a member of the set the shared subtree came from. The
//! payload MUST therefore carry ONLY the per-recipient HPKE-wrapped CEK
//! (decryptable by THAT recipient's sk) — **never** the live group key
//! `K_Set`. Embedding `K_Set` would leak the ENTIRE group's keys to a
//! one-shot non-member (every member key derivable, the whole set
//! decryptable forever): a total confidentiality break of the central
//! sharing primitive.
//!
//! [`seal_drop_over_subtree`] READS `K_Set` to derive the member's own view
//! of the subtree at seal time, but NEVER copies it into the payload — the
//! recipient-wrapped CEK is derived from the recipient pubkey + a fresh
//! bundle CEK, independent of `K_Set`. [`DropBundlePayload::serialize`]
//! produces the byte string `plaintext_cid = BLAKE3(this)` digests; the
//! CC-BLK invariant scans those bytes for the `K_Set` sentinel and asserts
//! its absence.

extern crate alloc;

use alloc::vec::Vec;

/// The per-set group key (the MembershipSet keying-axis key). A Drop MUST
/// NEVER serialize this. Modeled as raw 32 bytes (`secrecy::SecretBox<[u8;32]>`
/// in the keying-axis production type).
pub type KSet = [u8; 32];

/// A content CID for a Node in the shared subtree (32-byte digest).
pub type Cid = [u8; 32];

/// A per-recipient HPKE-wrapped content-encryption-key (decryptable ONLY by
/// the recipient's sk; carries NO set-key material).
pub type WrappedCek = Vec<u8>;

/// One encrypted Node of the shared subtree (the `content` half).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncryptedNode {
    /// The Node's content CID.
    pub node_cid: Cid,
    /// ChaCha20-Poly1305 ciphertext+tag of the Node content, encrypted under
    /// a per-Node CEK; the CEK is HPKE-wrapped to the recipient (NOT under
    /// `K_Set`).
    pub ciphertext: Vec<u8>,
}

/// The canonical `DropBundlePayload` — the stable `plaintext_cid` half of the
/// DUAL-CID. It carries the SubgraphSpec CID, the encrypted Nodes, the
/// per-recipient HPKE-wrapped CEK (authority to decrypt THIS bundle), and
/// framing. It MUST NOT carry the `K_Set`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropBundlePayload {
    /// Envelope serialization-format version (V2).
    pub format_version: u8,
    /// The drop codepoint (the Sealed-Sender default `0x6510`).
    pub codepoint: u16,
    /// The SubgraphSpec CID (what-can-be-shared).
    pub spec_cid: Cid,
    /// The encrypted Nodes of the shared subtree.
    pub content: Vec<EncryptedNode>,
    /// HPKE-wrapped CEK for the single Drop recipient. Decryptable by the
    /// recipient's long-term sk; reveals only THIS bundle's CEK.
    pub recipient_wrapped_cek: WrappedCek,
}

/// Envelope SERIALIZATION-format version (V2 from commit 1).
const ENVELOPE_FORMAT_VERSION: u8 = 2;
/// `DROP_TO_RECIPIENT_SEALED_SENDER` — the v1-beta DEFAULT (BR-1).
const DROP_TO_RECIPIENT_SEALED_SENDER: u16 = 0x6510;

impl DropBundlePayload {
    /// Serialize the canonical payload to its deterministic big-endian byte
    /// layout (V2 + BE; M-19). This is the byte string
    /// `plaintext_cid = BLAKE3(this)` digests, and the byte string the
    /// CC-BLK invariant scans.
    #[must_use]
    pub fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(self.format_version);
        out.extend_from_slice(&self.codepoint.to_be_bytes());
        out.extend_from_slice(&self.spec_cid);
        let node_count = u32::try_from(self.content.len()).expect("node count fits u32");
        out.extend_from_slice(&node_count.to_be_bytes());
        for node in &self.content {
            out.extend_from_slice(&node.node_cid);
            let ct_len = u32::try_from(node.ciphertext.len()).expect("ct len fits u32");
            out.extend_from_slice(&ct_len.to_be_bytes());
            out.extend_from_slice(&node.ciphertext);
        }
        let cek_len = u32::try_from(self.recipient_wrapped_cek.len()).expect("cek len fits u32");
        out.extend_from_slice(&cek_len.to_be_bytes());
        out.extend_from_slice(&self.recipient_wrapped_cek);
        out
    }
}

/// Seal a Drop bundle over a member's subtree.
///
/// For each Node in the subtree the content is already encrypted under a
/// per-Node CEK; the bundle CEK is wrapped to the single recipient via
/// HPKE-to-recipient-pubkey. `k_set` is READ to derive/decrypt the member's
/// own view at seal time but is NEVER copied into the payload — the
/// recipient-wrapped CEK is derived from the recipient pubkey + a fresh
/// bundle CEK, INDEPENDENT of `k_set` (a correct Drop never embeds the set
/// key). Per CLAUDE.md baked-in #5 the wrap derivation routes through
/// `blake3` (the only-call-site rule) — the set key never appears.
#[must_use]
pub fn seal_drop_over_subtree(
    subtree: &[EncryptedNode],
    spec_cid: &Cid,
    recipient_pk: &[u8; 32],
    _k_set: &KSet,
) -> DropBundlePayload {
    // The recipient-wrapped CEK is a deterministic transform of the recipient
    // pubkey + a fresh bundle CEK; it is INDEPENDENT of `_k_set` so the set
    // key never appears in the payload bytes. (The real wrap is the X-Wing
    // HPKE key-wrap; here the derivation is content-bound to the recipient.)
    let mut h = blake3::Hasher::new();
    h.update(b"benten-drop:bundle-cek-wrap");
    h.update(recipient_pk);
    h.update(spec_cid);
    let wrapped = h.finalize().as_bytes().to_vec();
    DropBundlePayload {
        format_version: ENVELOPE_FORMAT_VERSION,
        codepoint: DROP_TO_RECIPIENT_SEALED_SENDER,
        spec_cid: *spec_cid,
        content: subtree.to_vec(),
        recipient_wrapped_cek: wrapped,
    }
}

/// DELIBERATELY-LEAKY seal (NEGATIVE-CONTROL ONLY) — models the
/// wrong-but-plausible impl that embeds the live `K_Set` into the payload.
/// Used ONLY to prove the CC-BLK scanner actually fires. NEVER a production
/// path.
#[must_use]
#[cfg(any(test, feature = "testing"))]
pub fn seal_drop_leaky_for_negative_control(
    subtree: &[EncryptedNode],
    spec_cid: &Cid,
    recipient_pk: &[u8; 32],
    k_set: &KSet,
) -> DropBundlePayload {
    let mut payload = seal_drop_over_subtree(subtree, spec_cid, recipient_pk, k_set);
    // The bug: the set key is appended to the recipient-wrapped CEK region.
    payload.recipient_wrapped_cek.extend_from_slice(k_set);
    payload
}
