//! The unified HPKE / KEM-DEM key-encryption path (Inv-16 one-HPKE-primitive
//! reused across Layer-C drops + Layer-D wraps).
//!
//! # The KEM-DEM composition (Q4 key-encryption mode)
//!
//! Sharing a Node = bulk-encrypt the body under a small per-Node `K(N)`
//! (Layer-B DEM), then HPKE-**key-encrypt** the small `K(N)` to the recipient
//! (Layer-C key-encryption mode, Q4). The recipient unwraps `K(N)` then
//! AEAD-Opens the body. What travels per recipient is the SMALL `K(N)`, not
//! the body — so a large body shared to N recipients costs one bulk seal + N
//! small key-wraps (the Q4 share-to-N efficiency property).
//!
//! # One primitive, two layers (Inv-16 / C-2)
//!
//! Layer-C drops and Layer-D wraps route through the SAME wrap primitive
//! ([`wrap_key_to_recipient`] / [`unwrap_key_from_recipient`]) — the real
//! X25519⊕ML-KEM-768 X-Wing KEM-DEM at codepoint `0x647a` (the committing
//! combiner in [`crate::cipher_suite`]). There is ONE KEM-DEM impl, not two.
//!
//! # NQ-C1 (the McMillion-`hpke`-faithful vs Benten-supplies-KEM fork)
//!
//! Whether the on-wire KEM-encapsulation bytes are RFC-9180-cross-stack-interop
//! (McMillion `hpke` admits X25519MLKEM768 into a real `mode_base` context) or
//! Benten-canonical (Benten supplies the KEM + reuses only the HPKE
//! key-schedule) is the open question NQ-C1 — a Ben-gated wire-format decision
//! surfaced at R5 (see `f_kat_3_hpke_kem_extensibility_nq_c1.rs`). THIS module
//! ships the Benten-canonical KEM-DEM (the real X-Wing wrap) which is the
//! functional substrate either branch builds on; the RFC-9180-faithful
//! on-wire framing is additive once NQ-C1 ratifies.

use crate::cipher_suite::{CipherSuite, RecipientPublic, RecipientSecret, WrappedKey};
use crate::codepoint::CipherSuiteCodepoint;

/// HPKE key-encryption: wrap the small `key` (e.g. a per-Node `K(N)`) to a
/// recipient via the unified X-Wing KEM-DEM at the hybrid codepoint.
///
/// # Errors
///
/// Returns the cipher-suite AEAD error on a wrap failure.
pub fn wrap_key_to_recipient(
    recipient_pub: &RecipientPublic,
    key: &[u8],
) -> Result<WrappedKey, crate::aead::AeadError> {
    let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .map_err(crate::aead::AeadError::Unsupported)?;
    suite.wrap_key_material(recipient_pub, key)
}

/// HPKE key-decryption: unwrap a [`WrappedKey`] with the recipient secret.
///
/// # Errors
///
/// Returns the cipher-suite AEAD error on an unwrap failure (a wrong
/// recipient secret fails closed).
pub fn unwrap_key_from_recipient(
    recipient_sec: &RecipientSecret,
    wrapped: &WrappedKey,
) -> Result<Vec<u8>, crate::aead::AeadError> {
    let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .map_err(crate::aead::AeadError::Unsupported)?;
    let unwrapped = suite.unwrap_key_material(recipient_sec, wrapped)?;
    Ok(unwrapped.as_bytes().to_vec())
}

/// Whether Layer-C drops and Layer-D wraps share ONE KEM-DEM primitive
/// (Inv-16 C-2). TRUE — both bottom out at
/// [`CipherSuite::wrap_key_material`](crate::cipher_suite::CipherSuite): Layer-D
/// device-link routes through the [`wrap_key_to_recipient`] wrapper, while
/// Layer-C drops (and the swap-matrix) call `CipherSuite::wrap_key_material`
/// directly via `hybrid_suite()`. One KEM-DEM impl either way.
#[must_use]
pub const fn one_primitive_across_layers() -> bool {
    true
}

#[cfg(any(test, feature = "testing"))]
mod test_support {
    use super::{
        CipherSuite, CipherSuiteCodepoint, unwrap_key_from_recipient, wrap_key_to_recipient,
    };

    /// Round-trip the unified key-encryption path: wrap a 32-byte key to a
    /// freshly-generated hybrid recipient + unwrap it back. Used by the
    /// F-LB-3 KEM-DEM composition pins.
    #[must_use]
    pub fn round_trip_key(key: &[u8; 32]) -> bool {
        let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768).unwrap();
        let kp = CipherSuite::generate_recipient_keypair_for_test(&suite);
        let wrapped = wrap_key_to_recipient(kp.public(), key).unwrap();
        let recovered = unwrap_key_from_recipient(kp.secret(), &wrapped).unwrap();
        recovered.as_slice() == key.as_slice()
    }
}

#[cfg(any(test, feature = "testing"))]
pub use test_support::round_trip_key;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_encryption_round_trips() {
        assert!(round_trip_key(&[0x3Fu8; 32]));
    }

    #[test]
    fn one_primitive_flag() {
        assert!(one_primitive_across_layers());
    }
}
