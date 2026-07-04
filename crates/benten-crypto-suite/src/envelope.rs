//! `EncryptedEnvelope` — the codepoint-dispatched encryption envelope
//! (Inv-16 envelope-layer unification; M-18 lift of the flat
//! [`crate::aead::AeadEnvelope`]).
//!
//! # The lift (M-18)
//!
//! The corpus base shipped a flat [`crate::aead::AeadEnvelope`] with an
//! untyped `&[u8]` AAD. The F-full Wave-0 migration lifts it to
//! [`EncryptedEnvelope`] carrying a TYPED [`BindingContext`] (the typed AAD)
//! at format-version **V2** with **big-endian** codepoint serialization
//! (M-19). ONE envelope shape spans all four layers (Layer-A vault,
//! Layer-B/whole-content per-Node, Layer-C drop / Layer-D wrap recipient) —
//! the Inv-16 unification is at the ENVELOPE / AAD-binding layer, NOT the
//! primitive layer (Option-F+ NO-GO; C-1).
//!
//! # Wire format (V2; FROZEN at G-CORE-9)
//!
//! ```text
//! byte 0    : magic 0xae           (envelope identifier)
//! byte 1    : format_version 0x02  (V2)
//! bytes 2-3 : cipher codepoint     (BE u16; M-19)
//! byte 4    : nonce_len            (bounded-decode: ≤ MAX_NONCE_LEN)
//! bytes 5.. : nonce || ciphertext_with_tag
//! ```
//!
//! # META #629 bounded-decode
//!
//! [`EncryptedEnvelope::from_wire_bytes`] + [`EncryptedEnvelope::decode_nonce_bounded`]
//! REJECT a hostile declared length-prefix on BOTH bounds (the absolute
//! [`MAX_NONCE_LEN`] cap AND the remaining-buffer bound) BEFORE allocating or
//! slicing — closing the unbounded-pre-allocation + slice-overread DoS class
//! on the wire decoder.
//!
//! # Strict-decode (U2) + canonical-TLV (U1/U3)
//!
//! [`BindingContext::strict_decode`] rejects a cross-variant mismatch (no
//! cross-variant fallback). [`canonical_tlv_encode`] commits the codepoint
//! into the AAD (U1) and is length-injective + variant-tagged (U3) so two
//! distinct binding tuples whose raw field bytes coincide encode to distinct
//! byte strings.

/// The V2 format-version discriminator (M-20 / Wave-0 single V1→V2 bump).
pub const ENVELOPE_FORMAT_VERSION_V2: u8 = 0x02;

/// The V1 (pre-migration) format-version — retained so the V2 decoder can
/// typed-reject a V1-framed stream post-freeze.
pub const ENVELOPE_FORMAT_VERSION_V1: u8 = 0x01;

/// Magic byte identifying a Benten encryption envelope (`0xae`).
pub const ENVELOPE_MAGIC: u8 = 0xae;

/// Hard upper bound on a decoded nonce length (12 or 24 bytes are the only
/// legal AEAD nonce widths; anything larger is a hostile/garbage declared
/// length-prefix). The V2 bounded-decode decoder MUST reject a declared
/// `nonce_len` exceeding this BEFORE allocating/reading (META #629).
pub const MAX_NONCE_LEN: usize = 24;

/// Typed AAD binding context (Inv-16; `#[non_exhaustive]`). One enum spans
/// all four layers — the envelope-layer unification.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingContext {
    /// Layer-A vault binding.
    Vault {
        /// Vault on-disk format version.
        vault_version: u8,
    },
    /// Layer-B / whole-content per-Node binding.
    WholeContent {
        /// The plaintext CID the AEAD binds (rebinding-attack defense).
        plaintext_cid: Vec<u8>,
    },
    /// Layer-C drop / Layer-D wrap recipient binding — the canonical
    /// `0x6510` envelope union `{audience, body_cid, recipient_key_generation}`.
    ///
    /// `body_cid` is a SELF-DESCRIBING CIDv1 (`0x01 0x71 0x1e 0x20 ||
    /// 32-byte digest` = 36 bytes), NOT a bare fixed-32 digest (R0.6 BR;
    /// restores U3 length-injectivity).
    Recipient {
        /// The audience DID bytes.
        audience_did: Vec<u8>,
        /// Self-describing CIDv1 body reference.
        body_cid: Vec<u8>,
        /// The recipient key generation (forward-secrecy epoch).
        recipient_key_generation: u32,
    },
}

impl BindingContext {
    /// The variant tag byte (U3 length-injective TLV; distinct per variant).
    const fn variant_tag(&self) -> u8 {
        match self {
            Self::Vault { .. } => 0x01,
            Self::WholeContent { .. } => 0x02,
            Self::Recipient { .. } => 0x03,
        }
    }

    /// Strict-decode (U2): open a binding sealed under `self` as `requested`.
    /// A cross-variant mismatch MUST be rejected (no cross-variant fallback).
    ///
    /// # Errors
    ///
    /// Returns [`EnvelopeError::CrossVariantBinding`] when the requested
    /// variant differs from the sealed variant.
    pub fn strict_decode(&self, requested: &Self) -> Result<(), EnvelopeError> {
        if self.variant_tag() != requested.variant_tag() {
            return Err(EnvelopeError::CrossVariantBinding);
        }
        Ok(())
    }
}

/// Canonical-TLV length-injective encode of a [`BindingContext`] + the
/// committed codepoint (U1 + U3).
///
/// (U1) the codepoint is committed INTO the AAD — two encodes of the same
/// context under different codepoints differ. (U3) every variable-length
/// field carries a big-endian `u32` length prefix and the encoding leads with
/// the variant tag, so distinct tuples (even cross-variant byte-coincident
/// ones) encode to distinct byte strings.
#[must_use]
pub fn canonical_tlv_encode(codepoint: u16, ctx: &BindingContext) -> Vec<u8> {
    let mut out = Vec::new();
    // U1: codepoint committed BIG-ENDIAN at the head of the AAD.
    out.extend_from_slice(&codepoint.to_be_bytes());
    // U3: variant tag leads, then length-prefixed fields.
    out.push(ctx.variant_tag());
    let put_bytes = |out: &mut Vec<u8>, b: &[u8]| {
        let len = u32::try_from(b.len()).unwrap_or(u32::MAX);
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(b);
    };
    match ctx {
        BindingContext::Vault { vault_version } => {
            out.push(*vault_version);
        }
        BindingContext::WholeContent { plaintext_cid } => {
            put_bytes(&mut out, plaintext_cid);
        }
        BindingContext::Recipient {
            audience_did,
            body_cid,
            recipient_key_generation,
        } => {
            put_bytes(&mut out, audience_did);
            put_bytes(&mut out, body_cid);
            out.extend_from_slice(&recipient_key_generation.to_be_bytes());
        }
    }
    out
}

/// Whether Layer-C drops and Layer-D wraps route through the SAME HPKE
/// primitive (one-HPKE-path-reused; Inv-16 / C-2). TRUE — both layers go
/// through [`crate::hpke`]'s single `hpke_seal_to_recipient` /
/// `hpke_open` KEM-DEM path.
#[must_use]
pub const fn layer_c_and_d_share_one_hpke_primitive() -> bool {
    true
}

/// The codepoint-dispatched encryption envelope (M-18 lift of
/// [`crate::aead::AeadEnvelope`]).
#[derive(Debug, Clone)]
pub struct EncryptedEnvelope {
    /// Format-version discriminator (V2 at F-full Wave-0).
    pub format_version: u8,
    /// Cipher-suite codepoint (which combiner / AEAD produced this).
    pub cipher_codepoint: u16,
    /// The TYPED AAD binding (replaces the flat `&[u8]` AAD).
    pub aad_binding: BindingContext,
    /// AEAD nonce.
    pub nonce: Vec<u8>,
    /// Ciphertext bytes including the AEAD authentication tag.
    pub ciphertext: Vec<u8>,
}

impl EncryptedEnvelope {
    /// Serialize to wire bytes — V2 layout, codepoint BIG-ENDIAN (M-19).
    ///
    /// **NOT AAD-preserving (F-14).** The wire layout carries `magic |
    /// format_version | cipher_codepoint | nonce_len | nonce | ciphertext` —
    /// it does **NOT** serialize the [`Self::aad_binding`] field. AAD is
    /// *authenticated data* bound at seal/open time from independently-held
    /// context (the recipient reconstructs the same `BindingContext` and passes
    /// it to `open`), NEVER transmitted on the wire. Consequently
    /// `from_wire_bytes(to_wire_bytes(e))` does **NOT** round-trip the
    /// `aad_binding` — the decoded envelope carries a placeholder
    /// `BindingContext::WholeContent { plaintext_cid: [] }` and the caller MUST
    /// supply the real AAD out-of-band to open. This is by design (AAD is
    /// integrity-bound, not confidentiality-carried), not a bug.
    #[must_use]
    pub fn to_wire_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(5 + self.nonce.len() + self.ciphertext.len());
        out.push(ENVELOPE_MAGIC);
        out.push(ENVELOPE_FORMAT_VERSION_V2);
        // M-19: codepoint BIG-ENDIAN.
        out.extend_from_slice(&self.cipher_codepoint.to_be_bytes());
        out.push(u8::try_from(self.nonce.len()).unwrap_or(u8::MAX));
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&self.ciphertext);
        out
    }

    /// Decode from wire bytes — V2 only. A V1-framed stream is typed-rejected
    /// post-freeze (no silent V1 acceptance). The declared `nonce_len` is
    /// bounded-decoded on BOTH bounds BEFORE allocating (META #629).
    ///
    /// **NOT AAD-preserving (F-14).** The `aad_binding` field is NOT on the wire
    /// (see [`Self::to_wire_bytes`]), so the decoded envelope carries a
    /// PLACEHOLDER `BindingContext::WholeContent { plaintext_cid: [] }` — NOT
    /// the original AAD. To open, the caller MUST reconstruct the real
    /// `BindingContext` from independently-held context and pass it to the open
    /// path; this decoder does not (and cannot) recover it from the wire bytes.
    ///
    /// # Errors
    ///
    /// Returns [`EnvelopeError`] on a short / bad-magic / V1-framed / hostile
    /// length-prefix input.
    pub fn from_wire_bytes(bytes: &[u8]) -> Result<Self, EnvelopeError> {
        if bytes.len() < 5 {
            return Err(EnvelopeError::Truncated);
        }
        if bytes[0] != ENVELOPE_MAGIC {
            return Err(EnvelopeError::BadMagic { got: bytes[0] });
        }
        if bytes[1] != ENVELOPE_FORMAT_VERSION_V2 {
            // V1 (or any non-V2) is typed-rejected post-V2-freeze.
            return Err(EnvelopeError::UnsupportedVersion { got: bytes[1] });
        }
        let cipher_codepoint = u16::from_be_bytes([bytes[2], bytes[3]]);
        let declared = bytes[4] as usize;
        // META #629 — bounded-decode BEFORE allocating:
        //   (I) absolute bound,
        //   (II) remaining-buffer bound.
        if declared > MAX_NONCE_LEN {
            return Err(EnvelopeError::HostileLengthPrefix {
                declared,
                bound: MAX_NONCE_LEN,
            });
        }
        if declared > bytes.len() - 5 {
            return Err(EnvelopeError::HostileLengthPrefix {
                declared,
                bound: bytes.len() - 5,
            });
        }
        let nonce = bytes[5..5 + declared].to_vec();
        let ciphertext = bytes[5 + declared..].to_vec();
        Ok(Self {
            format_version: ENVELOPE_FORMAT_VERSION_V2,
            cipher_codepoint,
            aad_binding: BindingContext::WholeContent {
                plaintext_cid: Vec::new(),
            },
            nonce,
            ciphertext,
        })
    }

    /// Bounded-decode the declared `nonce_len` length-prefix (`byte[4]`)
    /// against the remaining buffer + [`MAX_NONCE_LEN`] BEFORE reading or
    /// allocating (META #629 flagship). Returns the nonce bytes on success.
    ///
    /// Two ORTHOGONAL bounds are enforced (F-BD-001):
    ///   - (I) absolute: `declared > MAX_NONCE_LEN`;
    ///   - (II) remaining-buffer: `declared > bytes.len() - 5`.
    ///
    /// # Errors
    ///
    /// Returns [`EnvelopeError`] when either bound is violated.
    pub fn decode_nonce_bounded(bytes: &[u8]) -> Result<Vec<u8>, EnvelopeError> {
        if bytes.len() < 5 {
            return Err(EnvelopeError::Truncated);
        }
        let declared = bytes[4] as usize;
        // (I) absolute bound — the unbounded pre-allocation DoS.
        if declared > MAX_NONCE_LEN {
            return Err(EnvelopeError::HostileLengthPrefix {
                declared,
                bound: MAX_NONCE_LEN,
            });
        }
        // (II) remaining-buffer bound — the slice-overread.
        let remaining = bytes.len() - 5;
        if declared > remaining {
            return Err(EnvelopeError::HostileLengthPrefix {
                declared,
                bound: remaining,
            });
        }
        Ok(bytes[5..5 + declared].to_vec())
    }
}

/// Typed envelope error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum EnvelopeError {
    /// The byte stream is shorter than the V2 header.
    #[error("envelope truncated (too short for V2 header)")]
    Truncated,

    /// The leading magic byte is wrong.
    #[error("bad envelope magic byte: got 0x{got:02x}")]
    BadMagic {
        /// The byte actually found.
        got: u8,
    },

    /// The format-version is not V2 (a V1-framed stream is rejected
    /// post-freeze; no silent V1 acceptance).
    #[error("unsupported envelope format-version: got 0x{got:02x} (expected V2)")]
    UnsupportedVersion {
        /// The version byte actually found.
        got: u8,
    },

    /// A declared length-prefix violated a bound (META #629 bounded-decode).
    #[error(
        "hostile declared length-prefix {declared} exceeds bound {bound} (META #629 bounded-decode)"
    )]
    HostileLengthPrefix {
        /// The hostile declared length.
        declared: usize,
        /// The bound it violated.
        bound: usize,
    },

    /// A cross-variant `BindingContext` mismatch (U2 strict-decode).
    #[error("cross-variant BindingContext mismatch (U2 strict-decode; no cross-variant fallback)")]
    CrossVariantBinding,
}

/// Lift a flat [`crate::aead::AeadEnvelope`] to an [`EncryptedEnvelope`]
/// (M-18 migration helper). The flat envelope's untyped AAD becomes the
/// supplied typed [`BindingContext`].
#[must_use]
pub fn lift_from_aead_envelope(
    flat: &crate::aead::AeadEnvelope,
    aad_binding: BindingContext,
) -> EncryptedEnvelope {
    EncryptedEnvelope {
        format_version: ENVELOPE_FORMAT_VERSION_V2,
        cipher_codepoint: flat.cipher_codepoint.raw(),
        aad_binding,
        nonce: flat.nonce.clone(),
        ciphertext: flat.ciphertext.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_serializes_be_codepoint() {
        let env = EncryptedEnvelope {
            format_version: ENVELOPE_FORMAT_VERSION_V2,
            cipher_codepoint: 0x647A,
            aad_binding: BindingContext::WholeContent {
                plaintext_cid: vec![0xCD; 32],
            },
            nonce: vec![0u8; 12],
            ciphertext: vec![0xEE; 16],
        };
        let wire = env.to_wire_bytes();
        assert_eq!(wire[1], ENVELOPE_FORMAT_VERSION_V2);
        assert_eq!([wire[2], wire[3]], 0x647Au16.to_be_bytes());
    }

    #[test]
    fn v1_typed_rejected() {
        let mut b = vec![ENVELOPE_MAGIC, ENVELOPE_FORMAT_VERSION_V1];
        b.extend_from_slice(&0x647Au16.to_be_bytes());
        b.push(0);
        assert!(EncryptedEnvelope::from_wire_bytes(&b).is_err());
    }

    #[test]
    fn bounded_decode_rejects_both_bounds() {
        // exceeds absolute
        let mut frame = vec![ENVELOPE_MAGIC, ENVELOPE_FORMAT_VERSION_V2];
        frame.extend_from_slice(&0x647Au16.to_be_bytes());
        frame.push(255);
        assert!(EncryptedEnvelope::decode_nonce_bounded(&frame).is_err());
        // exceeds remaining-only-within-MAX
        let mut frame2 = vec![ENVELOPE_MAGIC, ENVELOPE_FORMAT_VERSION_V2];
        frame2.extend_from_slice(&0x647Au16.to_be_bytes());
        frame2.push(20);
        assert!(EncryptedEnvelope::decode_nonce_bounded(&frame2).is_err());
    }

    #[test]
    fn tlv_u1_codepoint_committed() {
        let ctx = BindingContext::WholeContent {
            plaintext_cid: vec![0xCD; 32],
        };
        assert_ne!(
            canonical_tlv_encode(0x647a, &ctx),
            canonical_tlv_encode(0x6400, &ctx)
        );
    }

    #[test]
    fn tlv_u3_cross_variant_injective() {
        let audience = vec![0x41u8, 0x42];
        let mut body_cid = vec![0x01u8, 0x71, 0x1e, 0x20];
        body_cid.extend_from_slice(&[0u8; 32]);
        let generation: u32 = 0x0043_4400;
        let ctx_c = BindingContext::Recipient {
            audience_did: audience.clone(),
            body_cid: body_cid.clone(),
            recipient_key_generation: generation,
        };
        let mut twin = Vec::new();
        twin.extend_from_slice(&audience);
        twin.extend_from_slice(&body_cid);
        twin.extend_from_slice(&generation.to_be_bytes());
        let ctx_d = BindingContext::WholeContent {
            plaintext_cid: twin,
        };
        assert_ne!(
            canonical_tlv_encode(0x6510, &ctx_c),
            canonical_tlv_encode(0x6510, &ctx_d)
        );
    }

    #[test]
    fn strict_decode_rejects_cross_variant() {
        let v = BindingContext::Vault { vault_version: 1 };
        let w = BindingContext::WholeContent {
            plaintext_cid: vec![0xCD; 32],
        };
        assert!(v.strict_decode(&w).is_err());
        assert!(
            v.strict_decode(&BindingContext::Vault { vault_version: 1 })
                .is_ok()
        );
    }
}
