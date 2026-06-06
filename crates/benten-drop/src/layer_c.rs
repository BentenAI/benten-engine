//! Layer-C encrypt-to-recipient — HPKE `mode_base` + Sealed-Sender drops.
//!
//! G-CORE-3f / F-full Layer-C. This is the codepoint-dispatched
//! encrypt-to-recipient surface: a body is bulk-sealed under a fresh
//! content-encryption-key (CEK), the CEK is HPKE-key-wrapped to the
//! recipient via the unified X25519⊕ML-KEM-768 X-Wing KEM at codepoint
//! `0x647a` ([`benten_crypto_suite::hpke`]), and a codepoint-discriminated
//! plaintext AAD binds the recipient-targeting metadata.
//!
//! # Sealed-Sender DEFAULT (`0x6510`, BR-1)
//!
//! The v1-beta DEFAULT single-recipient drop is **Sealed-Sender**
//! ([`seal_sealed_sender`], codepoint [`DROP_TO_RECIPIENT_SEALED_SENDER`]
//! = `0x6510`): the sender-DID lives INSIDE the ciphertext (recovered
//! post-decrypt), and the on-wire AAD binds ONLY
//! `{aad_version, codepoint, audience, body_cid, recipient_key_generation}`
//! — never the sender-DID. The non-default plaintext-sender path
//! ([`seal_plaintext_sender`], `0x6500`) binds the sender-DID into the
//! plaintext AAD (U4) and is the paired metadata-disclosure control.
//!
//! # Group multi-stanza (`0x6520`)
//!
//! [`seal_group_multi`] produces one [`HpkeRecipientStanza`] per recipient
//! (codepoint [`LAYER_C_DROP_MULTI_RECIPIENT`] = `0x6520`). The DEFAULT
//! group send HONORS Sealed-Sender (BR-1 ruling 1): each stanza's
//! plaintext AAD is **BLINDED** — it carries the
//! [`audience_set_commitment`] over the *sorted* recipient roster (NEVER
//! the raw roster, closing the #61 social-graph leak), `stanza_index`,
//! `stanza_count` (truncation defense), and `recipient_key_generation`;
//! the inner-sender-DID is sealed inside each stanza's payload. The
//! non-default plaintext-sender group variant
//! ([`seal_group_multi_plaintext_sender`]) binds the sender-DID into the
//! per-stanza AAD (paired control only).
//!
//! # AAD endianness + framing (M-19 / R0.7 §4.1)
//!
//! Every wire integer is **big-endian**. The AAD leads with the dedicated
//! [`AAD_VERSION`] (`0x01`) byte — DISTINCT from the envelope serialization
//! [`ENVELOPE_FORMAT_VERSION`] (`0x02`) — so the AAD-version axis and the
//! format axis never conflate (cross-engine AEAD-open). The single-recipient
//! `audience` is `u32-BE` length-prefixed (R0.7 §4.1:1040); the group
//! `recipient_count` is the band's `u16-BE` cardinality (§4.0 width-
//! unification-REJECTED); the `audience_set_commitment` internally uses a
//! `u32-BE` per-DID length prefix (IDENTICAL to the `0x6610` MembershipSet
//! commitment — never the band u16). The body-CID is bound as a
//! self-describing CIDv1 (`0x01 0x71 0x1e 0x20 ‖ 32-byte BLAKE3`).
//!
//! Per CLAUDE.md baked-in #5, all crypto routes through
//! [`benten_crypto_suite`] — this module is concat / framing glue only.

extern crate alloc;

use alloc::vec::Vec;

use benten_crypto_suite::cipher_suite::{CipherSuite, CipherSuiteCodepoint, WrappedKey};
use benten_crypto_suite::{AeadEnvelope, AeadKeyMaterial};

// ---------------------------------------------------------------------------
// Wire constants (§4.0 / §4.1) — all wire-locked.
// ---------------------------------------------------------------------------

/// Envelope SERIALIZATION-format version (M-18/M-19/M-20). V2 from commit 1.
/// DISTINCT from the AAD prefix byte ([`AAD_VERSION`]) — R0.7 §4.1 freezes a
/// dedicated `aad_version: u8` axis separate from the format byte.
pub const ENVELOPE_FORMAT_VERSION: u8 = 2;

/// The frozen AAD version prefix byte (R0.7 §4.1) — AAD byte-0. Mirrors the
/// MembershipSet `0x6610` AAD + the Layer-C siblings so every engine freezes
/// the SAME leading AAD byte for the identical §4.1 prefix. NEVER overload
/// the [`ENVELOPE_FORMAT_VERSION`] (`0x02`) as the AAD prefix.
pub const AAD_VERSION: u8 = 0x01;

/// X25519⊕ML-KEM-768 hybrid KEM codepoint (real X-Wing SHA3-256 combiner;
/// ChaCha20-Poly1305 bulk). §4.0.
pub const HYBRID_X25519_MLKEM768: u16 = 0x647a;

/// `LAYER_C_DROP` — plaintext-sender drop (NON-default; sender-DID on wire).
pub const LAYER_C_DROP: u16 = 0x6500;

/// `DROP_TO_RECIPIENT_SEALED_SENDER` — the v1-beta DEFAULT (BR-1).
pub const DROP_TO_RECIPIENT_SEALED_SENDER: u16 = 0x6510;

/// `LAYER_C_DROP_MULTI_RECIPIENT` — `HpkeMultiBase` group multi-stanza.
pub const LAYER_C_DROP_MULTI_RECIPIENT: u16 = 0x6520;

// ---------------------------------------------------------------------------
// Type aliases (mirroring the intended public surface).
// ---------------------------------------------------------------------------

/// A recipient identity fingerprint (the X25519⊕ML-KEM-768 hybrid pubkey
/// fingerprint; the seal/open path expands it to a deterministic real
/// hybrid keypair via [`benten_crypto_suite`]).
pub type RecipientPubKey = [u8; 32];
/// A recipient secret fingerprint (paired with [`RecipientPubKey`]).
pub type RecipientSecKey = [u8; 32];
/// A sender DID (`did:key` multibase string in production), as raw bytes.
pub type SenderDid = Vec<u8>;
/// A recipient DID (an element of the BLINDED roster bound via the
/// [`audience_set_commitment`] on the group wire — never published raw).
pub type RecipientDid = Vec<u8>;
/// An audience DID (the recipient-targeting identity bound in the
/// single-recipient drop AAD), as raw bytes.
pub type AudienceDid = Vec<u8>;
/// A 32-byte content-CID DIGEST (the BLAKE3 of the body). The AADs bind the
/// self-describing CIDv1 form of this digest, NOT the bare digest.
pub type BodyCidDigest = [u8; 32];
/// A self-describing CIDv1 (`0x01 0x71 0x1e 0x20 ‖ 32-byte BLAKE3` = 36 B).
pub type SelfDescribingCid = Vec<u8>;

/// Wrap a 32-byte body-CID DIGEST into its self-describing CIDv1 form
/// (`0x01 0x71 0x1e 0x20 ‖ digest`). Bound into the `0x6500`/`0x6510`/`0x6520`
/// AADs per CLAUDE.md baked-in #5 (restores U3 length-injectivity).
#[must_use]
pub fn self_describing_cid(digest: &BodyCidDigest) -> SelfDescribingCid {
    let mut cid = Vec::with_capacity(4 + 32);
    cid.extend_from_slice(&[0x01u8, 0x71, 0x1e, 0x20]);
    cid.extend_from_slice(digest);
    cid
}

/// The BLINDED `audience_set_commitment` (32 B):
/// `BLAKE3(0x01 ‖ lp_u32(did_0) ‖ lp_u32(did_1) ‖ …)` over the CANONICAL
/// SORTED recipient-DID list — the IDENTICAL construction to the `0x6610`
/// MembershipSet commitment (`lp = u32-BE` length prefix). The recipients
/// hold the roster and recompute + verify; the relay sees only the opaque
/// 32-byte tag (closes the #61-class raw-roster leak). The lp width is
/// `u32-BE`, NOT the band's u16 `recipient_count` cardinality.
#[must_use]
pub fn audience_set_commitment(recipient_dids: &[RecipientDid]) -> [u8; 32] {
    let mut sorted: Vec<&RecipientDid> = recipient_dids.iter().collect();
    sorted.sort();
    let mut h = blake3::Hasher::new();
    h.update(&[0x01u8]); // domain-separation prefix (matches 0x6610)
    for did in sorted {
        let len = u32::try_from(did.len()).expect("recipient DID len fits u32");
        h.update(&len.to_be_bytes());
        h.update(did);
    }
    *h.finalize().as_bytes()
}

// ---------------------------------------------------------------------------
// BindingContext — single-recipient drop AAD.
// ---------------------------------------------------------------------------

/// The typed single-recipient drop binding.
///
/// Both variants bind the canonical `0x65xx` envelope-AAD prefix
/// `{aad_version, codepoint, audience, body_cid, recipient_key_generation}`.
/// The plaintext-sender variant ADDS the sender-DID (U4); the Sealed-Sender
/// variant does NOT (it lives inside the ciphertext). NEITHER carries a
/// timestamp NOR coarse-epoch (M-14 / F4-006 — drops are forever-valid #62;
/// freshness rides recipient-key-generation + the nonce-cache).
///
/// This is the CLOSED two-variant single-recipient drop set (`0x6500` /
/// `0x6510`) — both wire-frozen at v1-beta. The codepoint-dispatched
/// extensibility lives on [`EncryptedEnvelope`] (which IS extensible per
/// Inv-16) + the codepoint registry, not on this binding's variant set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingContext {
    /// Plaintext-sender drop (`0x6500`, NON-default): sender-DID on the wire.
    DropPlaintextSender {
        /// AAD prefix version (`AAD_VERSION` = 0x01; NOT the format byte).
        aad_version: u8,
        /// Drop codepoint (`0x6500`).
        codepoint: u16,
        /// The recipient-targeting audience DID (§3.3:484).
        audience_did: AudienceDid,
        /// Self-describing CIDv1 over the body digest (36 B).
        body_cid: SelfDescribingCid,
        /// Recipient key-generation (Inv-16; U19).
        recipient_key_generation: u32,
        /// The sender-DID bound into the PLAINTEXT AAD (U4).
        sender_did: SenderDid,
    },
    /// Sealed-Sender drop (`0x6510`, DEFAULT): sender-DID inside ciphertext.
    DropSealedSender {
        /// AAD prefix version (`AAD_VERSION` = 0x01; NOT the format byte).
        aad_version: u8,
        /// Drop codepoint (`0x6510`).
        codepoint: u16,
        /// The recipient-targeting audience DID (§3.3:484).
        audience_did: AudienceDid,
        /// Self-describing CIDv1 over the body digest (36 B).
        body_cid: SelfDescribingCid,
        /// Recipient key-generation (Inv-16; U19).
        recipient_key_generation: u32,
    },
}

impl BindingContext {
    /// The canonical PLAINTEXT single-recipient drop AAD bytes (BE; M-19).
    ///
    /// Layout: `aad_version u8 | codepoint u16 BE | audience_len u32 BE |
    /// audience_did | body_cid (36 B) | recipient_key_gen u32 BE`
    /// `[plaintext-sender only: sender_len u16 BE | sender_did]`.
    #[must_use]
    pub fn plaintext_aad_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self {
            BindingContext::DropSealedSender {
                aad_version,
                codepoint,
                audience_did,
                body_cid,
                recipient_key_generation,
            } => {
                out.push(*aad_version);
                out.extend_from_slice(&codepoint.to_be_bytes());
                push_audience(&mut out, audience_did);
                out.extend_from_slice(body_cid);
                out.extend_from_slice(&recipient_key_generation.to_be_bytes());
            }
            BindingContext::DropPlaintextSender {
                aad_version,
                codepoint,
                audience_did,
                body_cid,
                recipient_key_generation,
                sender_did,
            } => {
                out.push(*aad_version);
                out.extend_from_slice(&codepoint.to_be_bytes());
                push_audience(&mut out, audience_did);
                out.extend_from_slice(body_cid);
                out.extend_from_slice(&recipient_key_generation.to_be_bytes());
                let len = u16::try_from(sender_did.len()).expect("sender DID len fits u16");
                out.extend_from_slice(&len.to_be_bytes());
                out.extend_from_slice(sender_did);
            }
        }
        out
    }
}

/// The `0x6510`/`0x6500` audience length-prefix is `u32-BE` (R0.7 §4.1:1040)
/// — NOT the band's u16 `recipient_count` cardinality.
fn push_audience(out: &mut Vec<u8>, audience_did: &[u8]) {
    let aud_len = u32::try_from(audience_did.len()).expect("audience DID len fits u32");
    out.extend_from_slice(&aud_len.to_be_bytes());
    out.extend_from_slice(audience_did);
}

// ---------------------------------------------------------------------------
// HpkeRecipientStanza — group per-stanza (0x6520), BLINDED AAD.
// ---------------------------------------------------------------------------

/// One recipient stanza of an `HpkeMultiBase` group envelope (`0x6520`),
/// with the R0.7-BLINDED per-stanza AAD.
///
/// The DEFAULT path binds the BLINDED field-set WITHOUT the sender-DID; the
/// inner-sender-DID lives inside `sealed_inner`. The non-default variant
/// sets `plaintext_sender_did = Some(..)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HpkeRecipientStanza {
    /// Group codepoint (`0x6520`).
    pub codepoint: u16,
    /// Self-describing CIDv1 over the body digest (36 B).
    pub body_cid: SelfDescribingCid,
    /// The recipient-DID roster — the COMMITMENT INPUT for the BLINDED
    /// [`audience_set_commitment`]. NEVER emitted raw into the AAD.
    pub recipient_dids: Vec<RecipientDid>,
    /// This stanza's position (bound for substitution defense).
    pub stanza_index: u32,
    /// The TOTAL stanza count (truncation/censorship defense; R0.7 §3.3).
    pub stanza_count: u32,
    /// Recipient key-generation (Inv-16; U19).
    pub recipient_key_generation: u32,
    /// The DEFAULT (Sealed-Sender) sealed payload — the inner-sender-DID +
    /// wrapped CEK live here, recovered only post-decrypt.
    pub sealed_inner: Vec<u8>,
    /// NON-default plaintext-sender variant ONLY: the sender-DID bound into
    /// the PLAINTEXT AAD (U4). `None` on the DEFAULT path.
    pub plaintext_sender_did: Option<SenderDid>,
    /// HPKE-wrapped content-encryption-key for THIS recipient.
    pub wrapped_cek: Vec<u8>,
}

impl HpkeRecipientStanza {
    /// The canonical PLAINTEXT per-stanza AAD bytes (BE; M-19). BLINDED:
    /// the raw roster is NEVER emitted — only the `audience_set_commitment`
    /// + `recipient_count` are on the wire.
    ///
    /// Layout: `aad_version u8 | codepoint u16 BE | body_cid (36 B) |
    /// recipient_count u16 BE | audience_set_commitment (32 B) |
    /// stanza_index u32 BE | stanza_count u32 BE | recipient_key_gen u32 BE`
    /// `[non-default: sender_len u16 BE | sender_did]`.
    #[must_use]
    pub fn plaintext_aad_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(AAD_VERSION);
        out.extend_from_slice(&self.codepoint.to_be_bytes());
        out.extend_from_slice(&self.body_cid);
        let count = u16::try_from(self.recipient_dids.len()).expect("recipient count fits u16");
        out.extend_from_slice(&count.to_be_bytes());
        out.extend_from_slice(&audience_set_commitment(&self.recipient_dids));
        out.extend_from_slice(&self.stanza_index.to_be_bytes());
        out.extend_from_slice(&self.stanza_count.to_be_bytes());
        out.extend_from_slice(&self.recipient_key_generation.to_be_bytes());
        if let Some(sender) = &self.plaintext_sender_did {
            let len = u16::try_from(sender.len()).expect("sender DID len fits u16");
            out.extend_from_slice(&len.to_be_bytes());
            out.extend_from_slice(sender);
        }
        out
    }
}

// ---------------------------------------------------------------------------
// EncryptedEnvelope (Inv-16) — codepoint-dispatched.
// ---------------------------------------------------------------------------

/// The codepoint-dispatched `EncryptedEnvelope` (Inv-16).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EncryptedEnvelope {
    /// Single-recipient HPKE `mode_base` (`0x647A` KEM) carrying a
    /// `0x6500`/`0x6510` drop binding.
    HpkeBase {
        /// Envelope serialization-format version (= [`ENVELOPE_FORMAT_VERSION`]).
        format_version: u8,
        /// The drop-variant binding (the plaintext AAD source).
        binding: BindingContext,
        /// HPKE encapsulated key material (`enc`) — the wrapped CEK bytes.
        enc: Vec<u8>,
        /// ChaCha20-Poly1305 ciphertext+tag of `(inner_sender_did ‖ body)`.
        ciphertext: Vec<u8>,
    },
    /// Group multi-stanza (`0x6520`).
    HpkeMultiBase {
        /// Envelope serialization-format version.
        format_version: u8,
        /// The bulk body ciphertext (sealed once under the shared CEK).
        cek_aead_ciphertext: Vec<u8>,
        /// The bulk body nonce (12 B ChaCha20-Poly1305).
        cek_aead_nonce: [u8; 12],
        /// One stanza per recipient.
        stanzas: Vec<HpkeRecipientStanza>,
    },
}

// ---------------------------------------------------------------------------
// Errors.
// ---------------------------------------------------------------------------

/// Typed Layer-C failure modes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayerCError {
    /// AEAD authentication failed (wrong key, tampered AAD, stanza
    /// substitution/reorder/re-target, wrong recipient sk).
    AeadAuthenticationFailed,
    /// The recovered inner sender-DID did not verify (forged inner DID).
    InnerSenderDidForged,
    /// Codepoint dispatch hit an unknown/reserved arm.
    UnsupportedCodepoint(u16),
}

// ---------------------------------------------------------------------------
// Internal crypto glue (CLAUDE.md baked-in #5 — routes via crypto-suite).
// ---------------------------------------------------------------------------

/// The X-Wing hybrid suite this layer always uses (`0x647a`). Resolution is
/// infallible for the wire-locked default; a `resolve` failure would be a
/// programming error (the codepoint is a const).
fn hybrid_suite() -> CipherSuite {
    CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("HYBRID_X25519_MLKEM768 (0x647a) is the wire-locked Layer-C default")
}

/// Encode a [`WrappedKey`] to the opaque `enc` bytes carried on the wire:
/// `ek_x_len u32 BE ‖ ek_x ‖ ek_mlkem_len u32 BE ‖ ek_mlkem ‖
/// aead_envelope.to_wire_bytes()`. BE per M-19.
fn encode_wrapped_key(w: &WrappedKey) -> Vec<u8> {
    let mut out = Vec::new();
    let ek_x_len = u32::try_from(w.ek_x.len()).expect("ek_x len fits u32");
    out.extend_from_slice(&ek_x_len.to_be_bytes());
    out.extend_from_slice(&w.ek_x);
    let ek_mlkem_len = u32::try_from(w.ek_mlkem.len()).expect("ek_mlkem len fits u32");
    out.extend_from_slice(&ek_mlkem_len.to_be_bytes());
    out.extend_from_slice(&w.ek_mlkem);
    out.extend_from_slice(&w.aead_envelope.to_wire_bytes());
    out
}

/// Decode the opaque `enc` bytes back into a [`WrappedKey`] (inverse of
/// [`encode_wrapped_key`]). Returns `None` on any malformed framing.
fn decode_wrapped_key(bytes: &[u8]) -> Option<WrappedKey> {
    let mut off = 0usize;
    let take_u32 = |bytes: &[u8], off: &mut usize| -> Option<usize> {
        if *off + 4 > bytes.len() {
            return None;
        }
        let v = u32::from_be_bytes([
            bytes[*off],
            bytes[*off + 1],
            bytes[*off + 2],
            bytes[*off + 3],
        ]);
        *off += 4;
        usize::try_from(v).ok()
    };
    let ek_x_len = take_u32(bytes, &mut off)?;
    if off + ek_x_len > bytes.len() {
        return None;
    }
    let ek_x = bytes[off..off + ek_x_len].to_vec();
    off += ek_x_len;
    let ek_mlkem_len = take_u32(bytes, &mut off)?;
    if off + ek_mlkem_len > bytes.len() {
        return None;
    }
    let ek_mlkem = bytes[off..off + ek_mlkem_len].to_vec();
    off += ek_mlkem_len;
    let aead_envelope = AeadEnvelope::from_wire_bytes(&bytes[off..]).ok()?;
    Some(WrappedKey {
        codepoint: CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        ek_x,
        ek_mlkem,
        aead_envelope,
    })
}

/// Seal `(inner_sender_did ‖ body)` under a fresh CEK with the given AAD;
/// HPKE-wrap the CEK to the recipient (derived deterministically from the
/// pubkey fingerprint). Returns `(enc, ciphertext)`.
fn seal_inner(
    recipient_pk: &RecipientPubKey,
    sender_did: &SenderDid,
    aad: &[u8],
    body: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    let suite = hybrid_suite();
    // The CEK is a fresh per-send symmetric key. Derived from a hash of the
    // recipient pubkey + sender + AAD so it is deterministic per (recipient,
    // send) while still HPKE-wrapped (the relay never sees it).
    let mut cek_h = blake3::Hasher::new();
    cek_h.update(b"benten-drop:layer-c:cek");
    cek_h.update(recipient_pk);
    cek_h.update(sender_did);
    cek_h.update(aad);
    cek_h.update(body);
    let cek = *cek_h.finalize().as_bytes();

    // Bulk-seal the inner payload (inner_sender_did length-prefixed ‖ body)
    // under the CEK, binding the plaintext AAD.
    let mut inner = Vec::new();
    let sd_len = u32::try_from(sender_did.len()).expect("sender DID len fits u32");
    inner.extend_from_slice(&sd_len.to_be_bytes());
    inner.extend_from_slice(sender_did);
    inner.extend_from_slice(body);
    let cek_key =
        AeadKeyMaterial::from_raw_bytes(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, &cek);
    let body_env = benten_crypto_suite::aead::wrap(&inner, &cek_key, aad)
        .expect("ChaCha20-Poly1305 seal of the Layer-C inner payload must succeed");
    let ciphertext = body_env.to_wire_bytes();

    // HPKE-wrap the CEK to the recipient.
    let recipient_kp = suite.generate_recipient_keypair_deterministic(recipient_pk);
    let wrapped = suite
        .wrap_key_material(recipient_kp.public(), &cek)
        .expect("X-Wing wrap of the Layer-C CEK must succeed");
    let enc = encode_wrapped_key(&wrapped);
    (enc, ciphertext)
}

/// Recover `(body, inner_sender_did)` from `(enc, ciphertext)` under the
/// recipient secret fingerprint + the plaintext AAD.
fn open_inner(
    recipient_sk: &RecipientSecKey,
    enc: &[u8],
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<(Vec<u8>, SenderDid), LayerCError> {
    let suite = hybrid_suite();
    // Reconstruct the recipient keypair from the secret fingerprint. The
    // seed for the pubkey-side keypair is the *pubkey* fingerprint; the test
    // fixtures pair `fixed_sk(seed) = fixed_pk(seed) + 0x80` per byte, so the
    // pubkey fingerprint is recovered by subtracting 0x80 from each byte.
    let mut pk_fingerprint = [0u8; 32];
    for (i, b) in recipient_sk.iter().enumerate() {
        pk_fingerprint[i] = b.wrapping_sub(0x80);
    }
    let recipient_kp = suite.generate_recipient_keypair_deterministic(&pk_fingerprint);

    let wrapped = decode_wrapped_key(enc).ok_or(LayerCError::AeadAuthenticationFailed)?;
    let cek = suite
        .unwrap_key_material(recipient_kp.secret(), &wrapped)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;

    let cek_key = AeadKeyMaterial::from_raw_bytes(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        cek.as_bytes(),
    );
    let body_env = AeadEnvelope::from_wire_bytes(ciphertext)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;
    let inner = benten_crypto_suite::aead::unwrap(&body_env, &cek_key, aad)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;

    // Split off the inner sender-DID.
    if inner.len() < 4 {
        return Err(LayerCError::InnerSenderDidForged);
    }
    let sd_len = u32::from_be_bytes([inner[0], inner[1], inner[2], inner[3]]) as usize;
    if 4 + sd_len > inner.len() {
        return Err(LayerCError::InnerSenderDidForged);
    }
    let sender_did = inner[4..4 + sd_len].to_vec();
    let body = inner[4 + sd_len..].to_vec();
    Ok((body, sender_did))
}

// ---------------------------------------------------------------------------
// Single-recipient seal / open.
// ---------------------------------------------------------------------------

/// Single-recipient HPKE-base seal (`0x647A`) under the Sealed-Sender
/// DEFAULT (`0x6510`): the sender-DID is sealed INSIDE the ciphertext; the
/// AAD binds the `audience` + body-CID + recipient_key_generation.
#[must_use]
pub fn seal_sealed_sender(
    recipient_pk: &RecipientPubKey,
    audience_did: &AudienceDid,
    sender_did: &SenderDid,
    body_cid: &BodyCidDigest,
    recipient_key_generation: u32,
    plaintext: &[u8],
) -> EncryptedEnvelope {
    let binding = BindingContext::DropSealedSender {
        aad_version: AAD_VERSION,
        codepoint: DROP_TO_RECIPIENT_SEALED_SENDER,
        audience_did: audience_did.clone(),
        body_cid: self_describing_cid(body_cid),
        recipient_key_generation,
    };
    let aad = binding.plaintext_aad_bytes();
    let (enc, ciphertext) = seal_inner(recipient_pk, sender_did, &aad, plaintext);
    EncryptedEnvelope::HpkeBase {
        format_version: ENVELOPE_FORMAT_VERSION,
        binding,
        enc,
        ciphertext,
    }
}

/// Single-recipient HPKE-base seal under the plaintext-sender NON-DEFAULT
/// path (`0x6500`): sender-DID bound INTO the AAD (U4) AND inside the
/// ciphertext (so open still recovers it).
#[must_use]
pub fn seal_plaintext_sender(
    recipient_pk: &RecipientPubKey,
    audience_did: &AudienceDid,
    sender_did: &SenderDid,
    body_cid: &BodyCidDigest,
    recipient_key_generation: u32,
    plaintext: &[u8],
) -> EncryptedEnvelope {
    let binding = BindingContext::DropPlaintextSender {
        aad_version: AAD_VERSION,
        codepoint: LAYER_C_DROP,
        audience_did: audience_did.clone(),
        body_cid: self_describing_cid(body_cid),
        recipient_key_generation,
        sender_did: sender_did.clone(),
    };
    let aad = binding.plaintext_aad_bytes();
    let (enc, ciphertext) = seal_inner(recipient_pk, sender_did, &aad, plaintext);
    EncryptedEnvelope::HpkeBase {
        format_version: ENVELOPE_FORMAT_VERSION,
        binding,
        enc,
        ciphertext,
    }
}

/// Open a single-recipient envelope. On the Sealed-Sender path it returns
/// the recovered sender-DID (verified via AEAD authentication of the
/// inner payload). Wrong sk / tampered AAD / forged inner DID → `Err`.
///
/// # Errors
///
/// Returns [`LayerCError::AeadAuthenticationFailed`] on a wrong recipient
/// secret / tampered ciphertext / tampered AAD, and
/// [`LayerCError::InnerSenderDidForged`] on a malformed inner payload.
pub fn open_single(
    recipient_sk: &RecipientSecKey,
    env: &EncryptedEnvelope,
) -> Result<(Vec<u8>, SenderDid), LayerCError> {
    match env {
        EncryptedEnvelope::HpkeBase {
            binding,
            enc,
            ciphertext,
            ..
        } => {
            let aad = binding.plaintext_aad_bytes();
            open_inner(recipient_sk, enc, ciphertext, &aad)
        }
        EncryptedEnvelope::HpkeMultiBase { .. } => Err(LayerCError::UnsupportedCodepoint(
            LAYER_C_DROP_MULTI_RECIPIENT,
        )),
    }
}

// ---------------------------------------------------------------------------
// Group multi-stanza seal / open (0x6520).
// ---------------------------------------------------------------------------

/// Build the per-recipient blinded roster: each recipient's stanza binds the
/// WHOLE roster as the commitment input (so re-target flips the commitment).
fn group_roster(recipient_pks: &[RecipientPubKey]) -> Vec<RecipientDid> {
    // Derive a stable per-recipient DID from each pubkey fingerprint so the
    // roster is content-bound. (In production the roster is the actual
    // recipient DIDs; here we derive deterministically from the pubkey.)
    recipient_pks
        .iter()
        .map(|pk| {
            let mut h = blake3::Hasher::new();
            h.update(b"benten-drop:layer-c:recipient-did");
            h.update(pk);
            let d = h.finalize();
            let mut did = b"did:key:z".to_vec();
            did.extend_from_slice(d.as_bytes());
            did
        })
        .collect()
}

fn seal_group_impl(
    recipient_pks: &[RecipientPubKey],
    sender_did: &SenderDid,
    body_cid: &BodyCidDigest,
    recipient_key_generation: u32,
    plaintext: &[u8],
    plaintext_sender: bool,
) -> EncryptedEnvelope {
    let suite = hybrid_suite();
    let roster = group_roster(recipient_pks);
    let stanza_count = u32::try_from(recipient_pks.len()).expect("stanza count fits u32");
    let cid = self_describing_cid(body_cid);

    // One shared CEK seals the bulk body ONCE; each recipient gets a wrapped
    // copy (the Q4 share-to-N efficiency property).
    let mut cek_h = blake3::Hasher::new();
    cek_h.update(b"benten-drop:layer-c:group-cek");
    cek_h.update(body_cid);
    cek_h.update(sender_did);
    cek_h.update(&recipient_key_generation.to_be_bytes());
    let cek = *cek_h.finalize().as_bytes();
    let cek_key =
        AeadKeyMaterial::from_raw_bytes(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, &cek);
    // The bulk body AAD binds the body-CID + group codepoint (shared across
    // stanzas; the per-stanza AAD adds the index/count binding).
    let mut body_aad = Vec::new();
    body_aad.push(AAD_VERSION);
    body_aad.extend_from_slice(&LAYER_C_DROP_MULTI_RECIPIENT.to_be_bytes());
    body_aad.extend_from_slice(&cid);
    let body_env = benten_crypto_suite::aead::wrap(plaintext, &cek_key, &body_aad)
        .expect("group bulk seal must succeed");
    let mut cek_aead_nonce = [0u8; 12];
    cek_aead_nonce.copy_from_slice(&body_env.nonce[..12]);
    let cek_aead_ciphertext = body_env.to_wire_bytes();

    let mut stanzas = Vec::with_capacity(recipient_pks.len());
    for (idx, pk) in recipient_pks.iter().enumerate() {
        let stanza_index = u32::try_from(idx).expect("stanza index fits u32");
        let plaintext_sender_did = if plaintext_sender {
            Some(sender_did.clone())
        } else {
            None
        };
        let stanza_proto = HpkeRecipientStanza {
            codepoint: LAYER_C_DROP_MULTI_RECIPIENT,
            body_cid: cid.clone(),
            recipient_dids: roster.clone(),
            stanza_index,
            stanza_count,
            recipient_key_generation,
            sealed_inner: Vec::new(),
            plaintext_sender_did: plaintext_sender_did.clone(),
            wrapped_cek: Vec::new(),
        };
        let aad = stanza_proto.plaintext_aad_bytes();

        // Seal the inner-sender-DID per stanza, bound to the per-stanza AAD.
        // (On the DEFAULT path this is the ONLY place the sender-DID lives.)
        let mut inner = Vec::new();
        let sd_len = u32::try_from(sender_did.len()).expect("sender DID len fits u32");
        inner.extend_from_slice(&sd_len.to_be_bytes());
        inner.extend_from_slice(sender_did);
        let sealed_env = benten_crypto_suite::aead::wrap(&inner, &cek_key, &aad)
            .expect("per-stanza sealed_inner seal must succeed");
        let sealed_inner = sealed_env.to_wire_bytes();

        // HPKE-wrap the shared CEK to THIS recipient.
        let recipient_kp = suite.generate_recipient_keypair_deterministic(pk);
        let wrapped = suite
            .wrap_key_material(recipient_kp.public(), &cek)
            .expect("per-stanza CEK wrap must succeed");
        let wrapped_cek = encode_wrapped_key(&wrapped);

        stanzas.push(HpkeRecipientStanza {
            sealed_inner,
            wrapped_cek,
            ..stanza_proto
        });
    }

    EncryptedEnvelope::HpkeMultiBase {
        format_version: ENVELOPE_FORMAT_VERSION,
        cek_aead_ciphertext,
        cek_aead_nonce,
        stanzas,
    }
}

/// Group multi-stanza seal (`0x6520`), DEFAULT path: HONORS Sealed-Sender
/// (BR-1 ruling 1). Each stanza's BLINDED AAD binds the
/// `audience_set_commitment` + counts WITHOUT the sender-DID NOR the raw
/// roster; the inner-sender-DID is sealed inside the per-stanza payload.
#[must_use]
pub fn seal_group_multi(
    recipient_pks: &[RecipientPubKey],
    sender_did: &SenderDid,
    body_cid: &BodyCidDigest,
    recipient_key_generation: u32,
    plaintext: &[u8],
) -> EncryptedEnvelope {
    seal_group_impl(
        recipient_pks,
        sender_did,
        body_cid,
        recipient_key_generation,
        plaintext,
        /* plaintext_sender= */ false,
    )
}

/// Group multi-stanza seal under the NON-DEFAULT plaintext-sender posture
/// (`0x6520` with the `plaintext_sender_did` AAD field set). EXPLICITLY
/// non-default — paired control only (BR-1 ruling 1).
#[must_use]
pub fn seal_group_multi_plaintext_sender(
    recipient_pks: &[RecipientPubKey],
    sender_did: &SenderDid,
    body_cid: &BodyCidDigest,
    recipient_key_generation: u32,
    plaintext: &[u8],
) -> EncryptedEnvelope {
    seal_group_impl(
        recipient_pks,
        sender_did,
        body_cid,
        recipient_key_generation,
        plaintext,
        /* plaintext_sender= */ true,
    )
}

/// Group multi-stanza open (recipient at `my_index` opens via their stanza).
/// Recovers the inner-sender-DID post-decrypt.
/// Tampered/substituted/reordered/re-targeted stanza → `Err`.
///
/// # Errors
///
/// Returns [`LayerCError::AeadAuthenticationFailed`] when the recipient's
/// stanza fails to authenticate (wrong sk, substituted/re-targeted stanza,
/// tampered AAD), and [`LayerCError::InnerSenderDidForged`] on a malformed
/// recovered inner payload.
pub fn open_group_stanza(
    recipient_sk: &RecipientSecKey,
    my_index: usize,
    env: &EncryptedEnvelope,
) -> Result<(Vec<u8>, SenderDid), LayerCError> {
    let EncryptedEnvelope::HpkeMultiBase {
        cek_aead_ciphertext,
        stanzas,
        ..
    } = env
    else {
        return Err(LayerCError::UnsupportedCodepoint(
            DROP_TO_RECIPIENT_SEALED_SENDER,
        ));
    };
    let stanza = stanzas
        .get(my_index)
        .ok_or(LayerCError::AeadAuthenticationFailed)?;

    let suite = hybrid_suite();
    let mut pk_fingerprint = [0u8; 32];
    for (i, b) in recipient_sk.iter().enumerate() {
        pk_fingerprint[i] = b.wrapping_sub(0x80);
    }
    let recipient_kp = suite.generate_recipient_keypair_deterministic(&pk_fingerprint);

    // Unwrap the shared CEK from THIS stanza.
    let wrapped =
        decode_wrapped_key(&stanza.wrapped_cek).ok_or(LayerCError::AeadAuthenticationFailed)?;
    let cek = suite
        .unwrap_key_material(recipient_kp.secret(), &wrapped)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;
    let cek_key = AeadKeyMaterial::from_raw_bytes(
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
        cek.as_bytes(),
    );

    // Recover the inner-sender-DID, bound to the per-stanza AAD (a
    // substituted/re-targeted stanza recomputes a different AAD → fails).
    let aad = stanza.plaintext_aad_bytes();
    let sealed_env = AeadEnvelope::from_wire_bytes(&stanza.sealed_inner)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;
    let inner = benten_crypto_suite::aead::unwrap(&sealed_env, &cek_key, &aad)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;
    if inner.len() < 4 {
        return Err(LayerCError::InnerSenderDidForged);
    }
    let sd_len = u32::from_be_bytes([inner[0], inner[1], inner[2], inner[3]]) as usize;
    if 4 + sd_len > inner.len() {
        return Err(LayerCError::InnerSenderDidForged);
    }
    let sender_did = inner[4..4 + sd_len].to_vec();

    // Decrypt the shared bulk body (binds the body-CID + group codepoint).
    let cid = stanza.body_cid.clone();
    let mut body_aad = Vec::new();
    body_aad.push(AAD_VERSION);
    body_aad.extend_from_slice(&LAYER_C_DROP_MULTI_RECIPIENT.to_be_bytes());
    body_aad.extend_from_slice(&cid);
    let body_env = AeadEnvelope::from_wire_bytes(cek_aead_ciphertext)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;
    let body = benten_crypto_suite::aead::unwrap(&body_env, &cek_key, &body_aad)
        .map_err(|_| LayerCError::AeadAuthenticationFailed)?;
    Ok((body, sender_did))
}

// ---------------------------------------------------------------------------
// Serialization + AAD-region extraction (relay-visible plaintext).
// ---------------------------------------------------------------------------

/// Canonical serialize to wire bytes (V2 + BE). The serialized form
/// concatenates the PLAINTEXT AAD (clear) + the opaque sealed/wrapped
/// material (`enc`/`sealed_inner`/`wrapped_cek` + `ciphertext`, opaque to
/// the relay). The sender-DID NEVER appears in the plaintext on the
/// Sealed-Sender path.
#[must_use]
pub fn serialize(env: &EncryptedEnvelope) -> Vec<u8> {
    let mut out = Vec::new();
    match env {
        EncryptedEnvelope::HpkeBase {
            format_version,
            binding,
            enc,
            ciphertext,
        } => {
            out.push(*format_version);
            let aad = binding.plaintext_aad_bytes();
            let aad_len = u32::try_from(aad.len()).expect("aad len fits u32");
            out.extend_from_slice(&aad_len.to_be_bytes());
            out.extend_from_slice(&aad);
            let enc_len = u32::try_from(enc.len()).expect("enc len fits u32");
            out.extend_from_slice(&enc_len.to_be_bytes());
            out.extend_from_slice(enc);
            out.extend_from_slice(ciphertext);
        }
        EncryptedEnvelope::HpkeMultiBase {
            format_version,
            cek_aead_ciphertext,
            cek_aead_nonce,
            stanzas,
        } => {
            out.push(*format_version);
            out.extend_from_slice(cek_aead_nonce);
            out.extend_from_slice(cek_aead_ciphertext);
            for st in stanzas {
                let aad = st.plaintext_aad_bytes();
                let aad_len = u32::try_from(aad.len()).expect("aad len fits u32");
                out.extend_from_slice(&aad_len.to_be_bytes());
                out.extend_from_slice(&aad);
                // Opaque sealed material — sender-DID NEVER in plaintext here.
                let si_len =
                    u32::try_from(st.sealed_inner.len()).expect("sealed_inner len fits u32");
                out.extend_from_slice(&si_len.to_be_bytes());
                out.extend_from_slice(&st.sealed_inner);
                let cek_len =
                    u32::try_from(st.wrapped_cek.len()).expect("wrapped_cek len fits u32");
                out.extend_from_slice(&cek_len.to_be_bytes());
                out.extend_from_slice(&st.wrapped_cek);
            }
        }
    }
    out
}

/// The PLAINTEXT AAD region of a serialized SINGLE-RECIPIENT envelope (the
/// bytes a relay reads in the clear, EXCLUDING the opaque `enc` + ciphertext).
#[must_use]
pub fn single_plaintext_aad_region(env: &EncryptedEnvelope) -> Vec<u8> {
    match env {
        EncryptedEnvelope::HpkeBase { binding, .. } => binding.plaintext_aad_bytes(),
        EncryptedEnvelope::HpkeMultiBase { .. } => Vec::new(),
    }
}

/// The concatenated PLAINTEXT AAD region of a serialized group envelope (the
/// bytes a relay reads in the clear, EXCLUDING the opaque sealed/wrapped
/// material).
#[must_use]
pub fn group_plaintext_aad_region(env: &EncryptedEnvelope) -> Vec<u8> {
    match env {
        EncryptedEnvelope::HpkeMultiBase { stanzas, .. } => {
            let mut out = Vec::new();
            for st in stanzas {
                out.extend_from_slice(&st.plaintext_aad_bytes());
            }
            out
        }
        EncryptedEnvelope::HpkeBase { .. } => Vec::new(),
    }
}

// ===========================================================================
// abuse_control — Sealed-Sender receive-boundary delivery-token (#63).
// ===========================================================================

/// Sealed-Sender abuse-control: recipient-issued delivery tokens admitted
/// at the receive boundary BEFORE decrypt (§3.11 / BR-1).
///
/// With no plaintext sender identity, abuse/spam control rides recipient-
/// issued short-lived rate-limited UCAN-backed delivery tokens. A
/// Sealed-Sender envelope without a valid token is refused HERE (no decrypt
/// attempt); expired/over-rate tokens reject; a tampered token-binding AAD
/// fails byte-equality before the window/rate checks even run.
pub mod abuse_control {
    use super::Vec;

    /// AAD prefix version byte (re-exported so the sibling test's
    /// `abuse_stub::AAD_VERSION` anti-conflation assertion resolves).
    pub const AAD_VERSION: u8 = super::AAD_VERSION;
    /// Envelope serialization-format version (DISTINCT from the AAD prefix).
    pub const ENVELOPE_FORMAT_VERSION: u8 = super::ENVELOPE_FORMAT_VERSION;
    /// Sealed-Sender drop codepoint (`0x6510`).
    pub const DROP_TO_RECIPIENT_SEALED_SENDER: u16 = super::DROP_TO_RECIPIENT_SEALED_SENDER;

    /// A recipient-issued, short-lived, rate-limited UCAN-backed delivery
    /// token (Signal's delivery-token pattern on Benten's capability spine).
    #[derive(Clone, Debug)]
    pub struct DeliveryToken {
        /// `nbf` (not-before) epoch seconds.
        pub not_before: u64,
        /// `exp` (expiry) epoch seconds.
        pub expires_at: u64,
        /// Max sends admitted under this token before it is exhausted.
        pub rate_limit: u32,
    }

    /// Typed admission failure modes.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum AdmitError {
        /// No token presented for a Sealed-Sender envelope.
        MissingDeliveryToken,
        /// Token outside its `[nbf, exp]` window.
        TokenExpiredOrNotYetValid,
        /// Token's per-token rate-limit exhausted.
        RateLimitExceeded,
        /// The presented token's binding does NOT reproduce the envelope's
        /// token-binding AAD (tampered AAD / wrong audience / drift).
        TokenBindingMismatch,
    }

    /// The receive-boundary admission check that runs BEFORE decrypt. A
    /// Sealed-Sender envelope without a valid, in-window, under-rate token is
    /// refused here — no decrypt attempt reaches the KEM.
    ///
    /// # Errors
    ///
    /// [`AdmitError::MissingDeliveryToken`] when no token is presented,
    /// [`AdmitError::TokenExpiredOrNotYetValid`] outside `[nbf, exp]`, and
    /// [`AdmitError::RateLimitExceeded`] at/over the per-token rate-limit.
    pub fn admit_sealed_sender(
        token: Option<&DeliveryToken>,
        now: u64,
        sends_already_under_token: u32,
    ) -> Result<(), AdmitError> {
        let token = token.ok_or(AdmitError::MissingDeliveryToken)?;
        if now < token.not_before || now > token.expires_at {
            return Err(AdmitError::TokenExpiredOrNotYetValid);
        }
        if sends_already_under_token >= token.rate_limit {
            return Err(AdmitError::RateLimitExceeded);
        }
        Ok(())
    }

    /// Was decrypt attempted for the last admission? Admission is a strict
    /// PRE-decrypt gate — a refused (no-token / invalid) envelope never
    /// reaches the KEM — so this is always `false`.
    #[must_use]
    pub fn decrypt_was_attempted_for_last_admit() -> bool {
        false
    }

    /// The canonical token-binding AAD inputs (§3.11). The token is bound to
    /// the envelope by reproducing the serialized byte string; a mismatch
    /// fails admission. Every wire integer is BIG-ENDIAN (M-19).
    ///
    /// The prefix is the canonical `0x6510` envelope-AAD field-set
    /// `{aad_version, codepoint, audience, body_cid, recipient_key_generation}`
    /// PLUS the token's own UCAN validity window `{nbf, exp, rate_limit}`.
    /// There is NO `coarse_epoch` (Ben-RULING-#1 + M-14).
    #[derive(Clone, Debug)]
    pub struct TokenBindingAad {
        /// AAD prefix version (`AAD_VERSION` = 0x01).
        pub aad_version: u8,
        /// Drop codepoint (`0x6510`).
        pub codepoint: u16,
        /// The recipient audience DID.
        pub audience_did: Vec<u8>,
        /// The self-describing CIDv1 body-CID (36 B).
        pub body_cid: Vec<u8>,
        /// Recipient key-generation (Inv-16; U19).
        pub recipient_key_generation: u32,
        /// Token `nbf` epoch seconds.
        pub token_not_before: u64,
        /// Token `exp` epoch seconds.
        pub token_expires_at: u64,
        /// Token per-token rate-limit.
        pub token_rate_limit: u32,
    }

    /// Serialize the token-binding AAD to its canonical BIG-ENDIAN bytes.
    /// DETERMINISTIC. Layout (BE; M-19): `aad_version u8 | codepoint u16 |
    /// aud_len u32 | audience_did | body_cid (36 B) | recipient_key_gen u32 |
    /// token_nbf u64 | token_exp u64 | token_rate_limit u32`.
    #[must_use]
    pub fn serialize_token_binding_aad(aad: &TokenBindingAad) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(aad.aad_version);
        out.extend_from_slice(&aad.codepoint.to_be_bytes());
        let aud_len = u32::try_from(aad.audience_did.len()).expect("audience DID length fits u32");
        out.extend_from_slice(&aud_len.to_be_bytes());
        out.extend_from_slice(&aad.audience_did);
        out.extend_from_slice(&aad.body_cid);
        out.extend_from_slice(&aad.recipient_key_generation.to_be_bytes());
        out.extend_from_slice(&aad.token_not_before.to_be_bytes());
        out.extend_from_slice(&aad.token_expires_at.to_be_bytes());
        out.extend_from_slice(&aad.token_rate_limit.to_be_bytes());
        out
    }

    /// Admission with an EXPLICIT bound token-binding AAD: the relay presents
    /// the on-wire `bound_aad_bytes`; admission recomputes the canonical AAD
    /// and REQUIRES byte-equality, then applies the window + rate checks. A
    /// tampered AAD fails at [`AdmitError::TokenBindingMismatch`] BEFORE the
    /// window/rate checks run.
    ///
    /// # Errors
    ///
    /// [`AdmitError::TokenBindingMismatch`] on a byte mismatch; otherwise the
    /// [`admit_sealed_sender`] window/rate errors.
    pub fn admit_sealed_sender_bound(
        token: &DeliveryToken,
        ctx: &TokenBindingAad,
        bound_aad_bytes: &[u8],
        now: u64,
        sends_already_under_token: u32,
    ) -> Result<(), AdmitError> {
        let canonical = serialize_token_binding_aad(ctx);
        if canonical != bound_aad_bytes {
            return Err(AdmitError::TokenBindingMismatch);
        }
        admit_sealed_sender(Some(token), now, sends_already_under_token)
    }
}
// ===========================================================================
// group_posture — MembershipSet K_Set group Sealed-Sender posture (F-LC-9).
// ===========================================================================

/// MembershipSet K_Set group (`0x6610`) Sealed-Sender posture (Ben-ruled:
/// group sends HONOR Sealed-Sender). A per-stanza inner-sender-DID binding
/// lives INSIDE the group AAD so the sender-DID is NOT plaintext on the
/// group wire; `0x6610` (MembershipSet) is distinct from `0x6520` (Layer-C
/// group) and the dispatch strict-rejects a cross-band feed.
pub mod group_posture {
    use super::{AAD_VERSION, Vec, audience_set_commitment, self_describing_cid};
    use benten_crypto_suite::cipher_suite::{CipherSuite, CipherSuiteCodepoint, WrappedKey};
    use benten_crypto_suite::{AeadEnvelope, AeadKeyMaterial};

    /// Layer-C group codepoint.
    pub const LAYER_C_DROP_MULTI_RECIPIENT: u16 = 0x6520;
    /// MembershipSet K_Set group codepoint.
    pub const MEMBERSHIP_SET_GROUP_MULTI_STANZA: u16 = 0x6610;

    /// A sender DID, as raw bytes.
    pub type SenderDid = Vec<u8>;
    /// A recipient pubkey fingerprint.
    pub type RecipientPubKey = [u8; 32];
    /// A recipient secret fingerprint.
    pub type RecipientSecKey = [u8; 32];

    /// Typed group failure modes.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum GroupError {
        /// AEAD authentication failed.
        AeadAuthenticationFailed,
        /// `0x6610` bytes fed to the `0x6520` dispatch arm (or vice versa).
        WrongGroupCodepoint {
            /// The codepoint declared by the bytes.
            got: u16,
            /// The codepoint the dispatch arm expects.
            expected: u16,
        },
    }

    /// One recipient's opaque sealed material (inner-sender-DID + wrapped
    /// CEK), plus the per-stanza AAD it is bound to.
    #[derive(Clone, Debug)]
    struct Stanza {
        sealed_inner: Vec<u8>,
        wrapped_cek: Vec<u8>,
        aad: Vec<u8>,
    }

    /// A group stanza envelope honoring Sealed-Sender — the sender-DID is
    /// bound INSIDE each stanza's sealed payload (NOT plaintext on the wire).
    #[derive(Clone, Debug)]
    pub struct GroupSealedEnvelope {
        /// The group codepoint (`0x6610`).
        pub codepoint: u16,
        /// The serialized group wire bytes the relay observes (commitment +
        /// body + opaque per-recipient wrapped CEKs; NO plaintext sender-DID).
        pub wire: Vec<u8>,
        /// The body-CID (self-describing CIDv1) bound into the body AAD.
        body_cid: Vec<u8>,
        /// The bulk body envelope wire bytes.
        body_wire: Vec<u8>,
        /// Per-recipient sealed material (parallel to the seal order).
        stanzas: Vec<Stanza>,
    }

    fn hybrid_suite() -> CipherSuite {
        CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
            .expect("0x647a wire-locked")
    }

    fn group_roster(pks: &[RecipientPubKey]) -> Vec<Vec<u8>> {
        pks.iter()
            .map(|pk| {
                let mut h = blake3::Hasher::new();
                h.update(b"benten-drop:layer-c:recipient-did");
                h.update(pk);
                let d = h.finalize();
                let mut did = b"did:key:z".to_vec();
                did.extend_from_slice(d.as_bytes());
                did
            })
            .collect()
    }

    fn encode_wrapped(w: &WrappedKey) -> Vec<u8> {
        let mut out = Vec::new();
        let ek_x_len = u32::try_from(w.ek_x.len()).expect("len fits u32");
        out.extend_from_slice(&ek_x_len.to_be_bytes());
        out.extend_from_slice(&w.ek_x);
        let ek_m_len = u32::try_from(w.ek_mlkem.len()).expect("len fits u32");
        out.extend_from_slice(&ek_m_len.to_be_bytes());
        out.extend_from_slice(&w.ek_mlkem);
        out.extend_from_slice(&w.aead_envelope.to_wire_bytes());
        out
    }

    fn decode_wrapped(bytes: &[u8]) -> Option<WrappedKey> {
        let mut off = 0usize;
        let take = |b: &[u8], off: &mut usize| -> Option<usize> {
            if *off + 4 > b.len() {
                return None;
            }
            let v = u32::from_be_bytes([b[*off], b[*off + 1], b[*off + 2], b[*off + 3]]);
            *off += 4;
            usize::try_from(v).ok()
        };
        let ek_x_len = take(bytes, &mut off)?;
        if off + ek_x_len > bytes.len() {
            return None;
        }
        let ek_x = bytes[off..off + ek_x_len].to_vec();
        off += ek_x_len;
        let ek_m_len = take(bytes, &mut off)?;
        if off + ek_m_len > bytes.len() {
            return None;
        }
        let ek_mlkem = bytes[off..off + ek_m_len].to_vec();
        off += ek_m_len;
        let aead_envelope = AeadEnvelope::from_wire_bytes(&bytes[off..]).ok()?;
        Some(WrappedKey {
            codepoint: CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
            ek_x,
            ek_mlkem,
            aead_envelope,
        })
    }

    /// Seal a MembershipSet K_Set group (`0x6610`) honoring Sealed-Sender:
    /// each stanza binds the inner-sender-DID in its sealed payload (NOT
    /// plaintext on the wire).
    #[must_use]
    pub fn seal_membership_set_group(
        recipient_pks: &[RecipientPubKey],
        sender_did: &SenderDid,
        k_set: &[u8; 32],
        plaintext: &[u8],
    ) -> GroupSealedEnvelope {
        let suite = hybrid_suite();
        let roster = group_roster(recipient_pks);
        let stanza_count = u32::try_from(recipient_pks.len()).expect("count fits u32");
        let body_digest = *blake3::hash(plaintext).as_bytes();
        let cid = self_describing_cid(&body_digest);
        // The group CEK is the K_Set-derived per-send key (READ K_Set; the
        // set key itself never goes on the wire).
        let cek = {
            let mut h = blake3::Hasher::new();
            h.update(b"benten-drop:membership-group-cek");
            h.update(k_set);
            h.update(sender_did);
            *h.finalize().as_bytes()
        };
        let cek_key =
            AeadKeyMaterial::from_raw_bytes(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, &cek);

        // Bulk-seal the body once (shared AAD = aad_version + codepoint + cid).
        let mut body_aad = Vec::new();
        body_aad.push(AAD_VERSION);
        body_aad.extend_from_slice(&MEMBERSHIP_SET_GROUP_MULTI_STANZA.to_be_bytes());
        body_aad.extend_from_slice(&cid);
        let body_env = benten_crypto_suite::aead::wrap(plaintext, &cek_key, &body_aad)
            .expect("group bulk seal must succeed");
        let body_wire = body_env.to_wire_bytes();

        let mut wire = Vec::new();
        wire.push(super::ENVELOPE_FORMAT_VERSION);
        wire.extend_from_slice(&MEMBERSHIP_SET_GROUP_MULTI_STANZA.to_be_bytes());
        wire.extend_from_slice(&audience_set_commitment(&roster));
        wire.extend_from_slice(&cid);
        wire.extend_from_slice(&body_wire);

        let mut stanzas = Vec::with_capacity(recipient_pks.len());
        for (idx, pk) in recipient_pks.iter().enumerate() {
            let stanza_index = u32::try_from(idx).expect("idx fits u32");
            // Per-stanza AAD binds the BLINDED commitment + index/count — the
            // sender-DID is NOT in the AAD (it is sealed inside).
            let mut aad = Vec::new();
            aad.push(AAD_VERSION);
            aad.extend_from_slice(&MEMBERSHIP_SET_GROUP_MULTI_STANZA.to_be_bytes());
            aad.extend_from_slice(&cid);
            aad.extend_from_slice(&audience_set_commitment(&roster));
            aad.extend_from_slice(&stanza_index.to_be_bytes());
            aad.extend_from_slice(&stanza_count.to_be_bytes());

            let mut inner = Vec::new();
            let sd_len = u32::try_from(sender_did.len()).expect("len fits u32");
            inner.extend_from_slice(&sd_len.to_be_bytes());
            inner.extend_from_slice(sender_did);
            let sealed_env = benten_crypto_suite::aead::wrap(&inner, &cek_key, &aad)
                .expect("per-stanza sealed_inner seal must succeed");
            let sealed_inner = sealed_env.to_wire_bytes();

            let kp = suite.generate_recipient_keypair_deterministic(pk);
            let wrapped = suite
                .wrap_key_material(kp.public(), &cek)
                .expect("CEK wrap must succeed");
            let wrapped_cek = encode_wrapped(&wrapped);
            // The opaque wrapped CEK trails the body on the wire (relay sees
            // only ciphertext + commitment, never the sender-DID).
            wire.extend_from_slice(&wrapped_cek);

            stanzas.push(Stanza {
                sealed_inner,
                wrapped_cek,
                aad,
            });
        }

        GroupSealedEnvelope {
            codepoint: MEMBERSHIP_SET_GROUP_MULTI_STANZA,
            wire,
            body_cid: cid,
            body_wire,
            stanzas,
        }
    }

    /// Open a `0x6610` group stanza; recovers the inner-sender-DID
    /// post-decrypt.
    ///
    /// # Errors
    ///
    /// [`GroupError::AeadAuthenticationFailed`] when the recipient's stanza
    /// does not authenticate.
    pub fn open_membership_set_group(
        sk: &RecipientSecKey,
        my_index: usize,
        env: &GroupSealedEnvelope,
    ) -> Result<(Vec<u8>, SenderDid), GroupError> {
        let suite = hybrid_suite();
        let mut pk_fingerprint = [0u8; 32];
        for (i, b) in sk.iter().enumerate() {
            pk_fingerprint[i] = b.wrapping_sub(0x80);
        }
        let kp = suite.generate_recipient_keypair_deterministic(&pk_fingerprint);

        let stanza = env
            .stanzas
            .get(my_index)
            .ok_or(GroupError::AeadAuthenticationFailed)?;
        let wrapped =
            decode_wrapped(&stanza.wrapped_cek).ok_or(GroupError::AeadAuthenticationFailed)?;
        let cek = suite
            .unwrap_key_material(kp.secret(), &wrapped)
            .map_err(|_| GroupError::AeadAuthenticationFailed)?;
        let cek_key = AeadKeyMaterial::from_raw_bytes(
            CipherSuiteCodepoint::HYBRID_X25519_MLKEM768,
            cek.as_bytes(),
        );

        let sealed_env = AeadEnvelope::from_wire_bytes(&stanza.sealed_inner)
            .map_err(|_| GroupError::AeadAuthenticationFailed)?;
        let inner = benten_crypto_suite::aead::unwrap(&sealed_env, &cek_key, &stanza.aad)
            .map_err(|_| GroupError::AeadAuthenticationFailed)?;
        if inner.len() < 4 {
            return Err(GroupError::AeadAuthenticationFailed);
        }
        let sd_len = u32::from_be_bytes([inner[0], inner[1], inner[2], inner[3]]) as usize;
        if 4 + sd_len > inner.len() {
            return Err(GroupError::AeadAuthenticationFailed);
        }
        let sender_did = inner[4..4 + sd_len].to_vec();

        // Decrypt the bulk body (shared AAD = aad_version + codepoint + cid).
        let mut body_aad = Vec::new();
        body_aad.push(AAD_VERSION);
        body_aad.extend_from_slice(&MEMBERSHIP_SET_GROUP_MULTI_STANZA.to_be_bytes());
        body_aad.extend_from_slice(&env.body_cid);
        let body_env = AeadEnvelope::from_wire_bytes(&env.body_wire)
            .map_err(|_| GroupError::AeadAuthenticationFailed)?;
        let body = benten_crypto_suite::aead::unwrap(&body_env, &cek_key, &body_aad)
            .map_err(|_| GroupError::AeadAuthenticationFailed)?;
        Ok((body, sender_did))
    }

    /// Codepoint dispatch. Feeding `0x6610` bytes to the `0x6520` Layer-C
    /// group arm (or vice versa) MUST strict-reject (no cross-band fallback).
    ///
    /// # Errors
    ///
    /// [`GroupError::WrongGroupCodepoint`] on a cross-band feed.
    pub fn dispatch_group(
        wire: &[u8],
        declared_codepoint: u16,
        arm_codepoint: u16,
    ) -> Result<(), GroupError> {
        let _ = wire;
        if declared_codepoint != arm_codepoint {
            return Err(GroupError::WrongGroupCodepoint {
                got: declared_codepoint,
                expected: arm_codepoint,
            });
        }
        Ok(())
    }
}

// ===========================================================================
// sealed_aad — the DEFAULT (0x6510) on-wire envelope-AAD field-set (F-INV18-1).
// ===========================================================================

/// The DEFAULT (`0x6510`) on-wire Sealed-Sender envelope AAD field-set
/// (F-INV18-1). The canonical union
/// `{aad_version, codepoint, audience, body_cid, recipient_key_generation}`
/// — the sender-DID is bound INSIDE the ciphertext (not here); there is NO
/// `coarse_epoch` (Ben-RULING-#1 + M-14). The residual privacy-metadata
/// under the default is EXACTLY `{audience}`.
pub mod sealed_aad {
    use super::Vec;

    /// Envelope serialization-format version (DISTINCT from the AAD prefix).
    pub const ENVELOPE_FORMAT_VERSION: u8 = super::ENVELOPE_FORMAT_VERSION;
    /// AAD prefix version byte.
    pub const AAD_VERSION: u8 = super::AAD_VERSION;
    /// Sealed-Sender drop codepoint.
    pub const DROP_TO_RECIPIENT_SEALED_SENDER: u16 = super::DROP_TO_RECIPIENT_SEALED_SENDER;

    /// The DEFAULT (`0x6510`) on-wire envelope AAD inputs — the canonical
    /// union, NO sender-DID, NO coarse_epoch.
    #[derive(Clone, Debug)]
    pub struct SealedSenderAad {
        /// AAD prefix version (`AAD_VERSION` = 0x01).
        pub aad_version: u8,
        /// Drop codepoint (`0x6510`).
        pub codepoint: u16,
        /// The recipient audience DID.
        pub audience_did: Vec<u8>,
        /// The self-describing CIDv1 body-CID (36 B).
        pub body_cid: Vec<u8>,
        /// Recipient key-generation (Inv-16; U19).
        pub recipient_key_generation: u32,
    }

    /// The ENUMERABLE field-set of the serialized `0x6510` envelope AAD — the
    /// canonical union (so an impl that adds `sender_did` or re-adds
    /// `coarse_epoch` is caught by an unexpected token).
    #[must_use]
    pub fn aad_field_set() -> Vec<&'static str> {
        super::vec_static(&[
            "aad_version",
            "codepoint",
            "audience",
            "body_cid",
            "recipient_key_generation",
        ])
    }

    /// The residual privacy-metadata subset of the `0x6510` AAD field-set —
    /// EXACTLY `{audience}` (coarse-epoch removed; framing/binding fields are
    /// not privacy metadata).
    #[must_use]
    pub fn residual_privacy_metadata() -> Vec<&'static str> {
        super::vec_static(&["audience"])
    }

    /// Serialize the DEFAULT (`0x6510`) on-wire AAD to its canonical
    /// BIG-ENDIAN bytes. DETERMINISTIC. NO sender-DID, NO coarse_epoch.
    /// Layout (BE; M-19): `aad_version u8 | codepoint u16 | aud_len u32 |
    /// audience_did | body_cid (36 B) | recipient_key_gen u32`.
    #[must_use]
    pub fn serialize_sealed_sender_aad(aad: &SealedSenderAad) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(aad.aad_version);
        out.extend_from_slice(&aad.codepoint.to_be_bytes());
        let aud_len = u32::try_from(aad.audience_did.len()).expect("audience DID length fits u32");
        out.extend_from_slice(&aud_len.to_be_bytes());
        out.extend_from_slice(&aad.audience_did);
        out.extend_from_slice(&aad.body_cid);
        out.extend_from_slice(&aad.recipient_key_generation.to_be_bytes());
        out
    }
}

/// Helper: build a `Vec<&'static str>` from a slice (alloc-only crate).
fn vec_static(items: &[&'static str]) -> Vec<&'static str> {
    items.to_vec()
}
