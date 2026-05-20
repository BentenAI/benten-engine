//! Typed error surface for the integration crate.
//!
//! [`UnsupportedAlgorithm`] is the load-bearing fail-closed variant: every
//! unknown / reserved-unimplemented codepoint surfaces here, **never** a
//! silent fallback to the v1 default. This is what enforces the
//! `RATIFIED-pq-default-reframe-2026-05-19` §4 P2P-interop invariant +
//! `CLAUDE.md` baked-in #5 typed-unsupported clause + the
//! `Veilid`/`MLS`/`NIP-44` deployed-protocol pattern.

use thiserror::Error;

/// The integration crate's typed-reject variant for unknown / reserved /
/// unimplemented codepoints.
///
/// Whenever the codepoint dispatch sees a codepoint value it does not know
/// how to serve, it surfaces one of these variants and returns. There is
/// **never** a silent fallback to the v1 default — that would be a
/// downgrade-attack vector and would silently strand peers that wrote
/// content under a different codepoint.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum UnsupportedAlgorithm {
    /// Unknown / reserved-unimplemented signature codepoint.
    #[error("unsupported signature codepoint 0x{codepoint:04x}")]
    Signature {
        /// The raw codepoint value the dispatch did not know.
        codepoint: u16,
    },
    /// Unknown / reserved-unimplemented hash codepoint.
    #[error("unsupported hash codepoint 0x{codepoint:04x}")]
    Hash {
        /// The raw multihash codepoint value the dispatch did not know.
        codepoint: u64,
    },
    /// Unknown / reserved-unimplemented cipher-suite codepoint. (G-CORE-3
    /// `#1301` lights the live impls; this wave reserves the typed-reject
    /// surface ahead.)
    #[error("unsupported cipher-suite codepoint 0x{codepoint:04x}")]
    CipherSuite {
        /// The raw cipher-suite codepoint value the dispatch did not know.
        codepoint: u16,
    },
}

/// Verification-time errors for [`crate::sig::SignatureSuite::verify`].
///
/// These are surfaced for non-cryptographic-success outcomes; a successful
/// verify returns `Ok(())`. The hybrid construction's load-bearing safety
/// property — strip-resistance — surfaces here as
/// [`VerifyError::HybridHalfMissing`] /
/// [`VerifyError::StripResistanceViolated`]: there is no `Ok(())` arm that
/// silently accepts a stripped / single-half / cross-message-spliced sig.
#[derive(Debug, Error)]
pub enum VerifyError {
    /// The classical Ed25519 half failed to verify.
    #[error("Ed25519 (classical) half verify failed")]
    ClassicalVerifyFailed,
    /// The PQ (ML-DSA-65) half failed to verify.
    #[error("ML-DSA-65 (PQ) half verify failed")]
    PqVerifyFailed,
    /// The hybrid signature was missing one of its two halves (stripped or
    /// truncated). Fail-closed strip-resistance pin.
    #[error("hybrid signature missing a required half: {0}")]
    HybridHalfMissing(&'static str),
    /// The committing construction detected a substitution: each half was
    /// individually valid but they did not jointly bind the same message
    /// (cross-message half splice).
    #[error("hybrid strip-resistance violated: {0}")]
    StripResistanceViolated(&'static str),
    /// A classical-only suite was handed a hybrid-codepoint signature; it
    /// refuses to silently accept by ignoring the PQ half (that would be a
    /// silent downgrade).
    #[error("classical-only suite refuses hybrid-codepoint signature (silent-downgrade defense)")]
    CodepointMismatch,
    /// The underlying primitive returned a malformed-key error.
    #[error("malformed key material: {0}")]
    MalformedKey(&'static str),
    /// The underlying primitive returned a malformed-signature error.
    #[error("malformed signature material: {0}")]
    MalformedSignature(&'static str),
    /// Codepoint-dispatch fallthrough.
    #[error(transparent)]
    Unsupported(#[from] UnsupportedAlgorithm),
}

impl PartialEq for VerifyError {
    fn eq(&self, other: &Self) -> bool {
        // Lightweight, variant-discriminant equality (sufficient for tests
        // that `matches!` the typed arm; serde-stable Eq across primitive
        // crate boundaries is intentionally NOT promised).
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

/// Crate-level error envelope.
#[derive(Debug, Error)]
pub enum CryptoError {
    /// Typed-unsupported codepoint dispatch failure.
    #[error(transparent)]
    Unsupported(#[from] UnsupportedAlgorithm),
    /// Signature verification failure.
    #[error(transparent)]
    Verify(#[from] VerifyError),
}
