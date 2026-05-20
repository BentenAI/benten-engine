//! UCAN-Varsig-v1 header round-trip for the hybrid signature.
//!
//! The Varsig v1 header carries the signature codepoint + the
//! concatenated hybrid signature payload. Encoding round-trips bytes
//! exactly; decoding fails closed on an unknown codepoint (typed
//! [`VarsigError::UnsupportedCodepoint`], NOT a silent classical
//! fallback).
//!
//! # Wire format (G-CORE-2 internal; G-CORE-9 freezes for v1)
//!
//! ```text
//! [magic (1 B = 0xb5) | version (1 B = 0x01) | codepoint (2 B LE) | payload (variable)]
//! ```
//!
//! Sizes are codepoint-dispatched (no hardcoded Ed25519 assumption).

use thiserror::Error;

use crate::codepoint::SigCodepoint;
use crate::sig::HybridSignature;

const VARSIG_MAGIC: u8 = 0xb5;
const VARSIG_V1: u8 = 0x01;

// Codepoint-dispatched dimensions sourced from the upstream crates' constants
// (NOT a Benten redefinition).
const ED25519_SIG_LEN: usize = ed25519_dalek::SIGNATURE_LENGTH;
// ML-DSA-65 FIPS 204 Category-3 fixed signature size.
const ML_DSA_65_SIG_LEN: usize = 3309;
// SHA3-256 commitment fixed output.
const COMMITMENT_LEN: usize = 32;

/// UCAN-Varsig-v1 header carrying a hybrid signature on the wire.
pub struct UcanVarsigV1Header {
    bytes: Vec<u8>,
}

impl UcanVarsigV1Header {
    /// Encode a hybrid signature into a Varsig v1 header.
    #[must_use]
    pub fn encode_hybrid(sig: &HybridSignature) -> Self {
        let payload = sig.to_wire_bytes();
        let cp = sig.codepoint().raw();
        let mut bytes = Vec::with_capacity(4 + payload.len());
        bytes.push(VARSIG_MAGIC);
        bytes.push(VARSIG_V1);
        bytes.extend_from_slice(&cp.to_le_bytes());
        bytes.extend_from_slice(&payload);
        Self { bytes }
    }

    /// Borrow the encoded bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// `true` iff this header advertises the hybrid-default codepoint
    /// (sanity-check that the encoder did not silently downgrade).
    #[must_use]
    pub fn advertises_hybrid(&self) -> bool {
        if self.bytes.len() < 4 {
            return false;
        }
        let cp = u16::from_le_bytes([self.bytes[2], self.bytes[3]]);
        cp == SigCodepoint::HYBRID_ED25519_MLDSA65.raw()
    }

    /// Decode the header — fail-closed on unknown codepoint (NEVER a
    /// silent classical fallback).
    pub fn decode(bytes: &[u8]) -> Result<DecodedVarsig, VarsigError> {
        if bytes.len() < 4 {
            return Err(VarsigError::Truncated);
        }
        if bytes[0] != VARSIG_MAGIC {
            return Err(VarsigError::BadMagic { got: bytes[0] });
        }
        if bytes[1] != VARSIG_V1 {
            return Err(VarsigError::UnsupportedVersion { got: bytes[1] });
        }
        let codepoint_raw = u16::from_le_bytes([bytes[2], bytes[3]]);
        let codepoint = SigCodepoint::from_raw(codepoint_raw);
        // Typed-unsupported dispatch — fail-closed on unknown codepoints.
        codepoint
            .resolve()
            .map_err(|e| VarsigError::UnsupportedCodepoint {
                codepoint: codepoint_raw,
                reason: format!("{e}"),
            })?;

        let payload = &bytes[4..];
        let sig = decode_payload(codepoint, payload)?;
        Ok(DecodedVarsig {
            codepoint,
            signature: sig,
        })
    }

    /// Test-helper: synthesize a header carrying a raw (possibly unknown)
    /// codepoint — used by adversarial pins.
    #[must_use]
    pub fn with_raw_codepoint_for_test(codepoint: u16) -> Self {
        let mut bytes = Vec::with_capacity(8);
        bytes.push(VARSIG_MAGIC);
        bytes.push(VARSIG_V1);
        bytes.extend_from_slice(&codepoint.to_le_bytes());
        // Add a few zero-bytes of payload — the decode will fail at the
        // codepoint resolve step BEFORE consuming payload.
        bytes.extend_from_slice(&[0u8; 4]);
        Self { bytes }
    }
}

fn decode_payload(codepoint: SigCodepoint, payload: &[u8]) -> Result<HybridSignature, VarsigError> {
    match codepoint.raw() {
        0x0001 => {
            let expected = ED25519_SIG_LEN + ML_DSA_65_SIG_LEN + COMMITMENT_LEN;
            if payload.len() != expected {
                return Err(VarsigError::Truncated);
            }
            let classical = payload[..ED25519_SIG_LEN].to_vec();
            let pq = payload[ED25519_SIG_LEN..ED25519_SIG_LEN + ML_DSA_65_SIG_LEN].to_vec();
            let commitment = payload[ED25519_SIG_LEN + ML_DSA_65_SIG_LEN..].to_vec();
            Ok(HybridSignature::from_parts_internal(
                codepoint, classical, pq, commitment,
            ))
        }
        0x0002 => {
            if payload.len() != ED25519_SIG_LEN {
                return Err(VarsigError::Truncated);
            }
            Ok(HybridSignature::from_parts_internal(
                codepoint,
                payload.to_vec(),
                Vec::new(),
                Vec::new(),
            ))
        }
        // Should be unreachable per `resolve()` gate at the boundary.
        other => Err(VarsigError::UnsupportedCodepoint {
            codepoint: other,
            reason: "unknown codepoint reached payload decode".into(),
        }),
    }
}

/// A successfully-decoded Varsig v1 header.
#[derive(Debug)]
pub struct DecodedVarsig {
    codepoint: SigCodepoint,
    signature: HybridSignature,
}

impl DecodedVarsig {
    /// The decoded signature.
    #[must_use]
    pub fn signature(&self) -> &HybridSignature {
        &self.signature
    }

    /// The decoded codepoint.
    #[must_use]
    pub const fn codepoint(&self) -> SigCodepoint {
        self.codepoint
    }
}

/// Varsig decode errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum VarsigError {
    /// Header truncated below minimum length.
    #[error("varsig header truncated")]
    Truncated,
    /// Bad magic byte (not `0xb5`).
    #[error("varsig bad magic byte: 0x{got:02x}")]
    BadMagic { got: u8 },
    /// Unsupported Varsig version (G-CORE-2 ships v1 only).
    #[error("varsig unsupported version: {got}")]
    UnsupportedVersion { got: u8 },
    /// Unknown / reserved codepoint — fail-closed.
    #[error("varsig unsupported codepoint 0x{codepoint:04x}: {reason}")]
    UnsupportedCodepoint { codepoint: u16, reason: String },
}
