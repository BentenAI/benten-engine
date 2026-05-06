//! [`MultiSigSurface`] trait + [`Ed25519SingleKey`] default impl
//! (G14-A2 wave-4a').
//!
//! ## Crypto-minor-2 contract
//!
//! Trait surface (load-bearing across G14-A2 → post-Phase-3
//! v1-assessment-window per D-PHASE-3-24):
//!
//! ```ignore
//! pub trait MultiSigSurface {
//!     type Signature;
//!     type Error;
//!     fn sign(&self, msg: &[u8]) -> Result<Self::Signature, Self::Error>;
//!     fn verify(&self, msg: &[u8], sig: &Self::Signature) -> Result<(), Self::Error>;
//!     fn threshold(&self) -> u32;
//!     fn participants(&self) -> u32;
//! }
//! ```
//!
//! [`Ed25519SingleKey`] is the load-bearing default impl that carries
//! Phase-3 identity work. [`ThresholdMultiSig`] is a compile-only
//! placeholder demonstrating the extension point lands; the body
//! returns [`crate::errors::MultiSigError::PostPhase3`] per
//! D-PHASE-3-24 deferral.
//!
//! ## Cag-5 + D-PHASE-3-24 contract
//!
//! NO recovery-protocol-specific behavior in Phase 3. The source-grep
//! audit at
//! `crates/benten-id/tests/multi_sig.rs::multi_sig_surface_no_recovery_protocol_specific_behavior_in_phase_3`
//! pins this — the non-comment surface area of THIS file MUST NOT
//! name `Shamir`, `MLS`, `social_recovery`, `TPM`, or
//! `hardware_escrow` (the comment lines mentioning them as deferred
//! protocols are intentionally above the source-grep filter, which
//! strips comment-only lines per `crypto-r4-r1-minor-2` hardening).

use ed25519_dalek::{Signature, Signer, Verifier};

use crate::errors::MultiSigError;
use crate::keypair::{Keypair, PublicKey};

/// The load-bearing identity-recovery + multi-signature trait surface.
///
/// Phase 3 ships [`Ed25519SingleKey`] as the only fully-implemented
/// impl. Future post-Phase-3 work (after the v1-milestone-gate
/// assessment per CLAUDE.md baked-in #15) will land additional impls
/// (k-of-n threshold, social-recovery, etc.) without breaking this
/// trait signature.
pub trait MultiSigSurface {
    /// Signature type produced by [`MultiSigSurface::sign`].
    type Signature;
    /// Error type produced by either [`MultiSigSurface::sign`] or
    /// [`MultiSigSurface::verify`].
    type Error;

    /// Produce a signature over `msg`.
    fn sign(&self, msg: &[u8]) -> Result<Self::Signature, Self::Error>;

    /// Verify `sig` over `msg`.
    fn verify(&self, msg: &[u8], sig: &Self::Signature) -> Result<(), Self::Error>;

    /// Threshold count — minimum participants required to produce a
    /// valid signature. `1` for [`Ed25519SingleKey`].
    fn threshold(&self) -> u32;

    /// Total participant count. `1` for [`Ed25519SingleKey`].
    fn participants(&self) -> u32;
}

/// Default impl wrapping a single [`Keypair`].
///
/// `threshold == 1`, `participants == 1`. Sign/verify route to the
/// underlying `ed25519-dalek` primitives; failures surface as
/// [`MultiSigError::BadSignature`].
pub struct Ed25519SingleKey {
    keypair: Keypair,
}

impl Ed25519SingleKey {
    /// Construct from an owned [`Keypair`].
    pub fn new(keypair: Keypair) -> Self {
        Self { keypair }
    }

    /// Borrow the underlying public key.
    pub fn public_key(&self) -> &PublicKey {
        self.keypair.public_key()
    }
}

impl MultiSigSurface for Ed25519SingleKey {
    type Signature = Signature;
    type Error = MultiSigError;

    fn sign(&self, msg: &[u8]) -> Result<Self::Signature, Self::Error> {
        Ok(self.keypair.sign(msg))
    }

    fn verify(&self, msg: &[u8], sig: &Self::Signature) -> Result<(), Self::Error> {
        self.keypair
            .public_key()
            .as_verifying_key()
            .verify(msg, sig)
            .map_err(|_| MultiSigError::BadSignature)
    }

    fn threshold(&self) -> u32 {
        1
    }

    fn participants(&self) -> u32 {
        1
    }
}

/// Compile-only placeholder demonstrating the trait extension point.
///
/// Per D-PHASE-3-24, the concrete recovery protocol (k-of-n
/// threshold, etc.) is deferred to post-Phase-3 v1-assessment-window.
/// This struct exists so `crates/benten-id/tests/multi_sig.rs::
/// multi_sig_surface_threshold_extension_point_present` can pin
/// that downstream crates can implement [`MultiSigSurface`] for new
/// types (the trait is not sealed). Calling `sign` / `verify`
/// returns [`MultiSigError::PostPhase3`].
pub struct ThresholdMultiSig {
    /// Configured threshold; ignored at G14-A2 (the surface is
    /// shape-only).
    pub threshold: u32,
    /// Configured participants; ignored at G14-A2.
    pub participants: u32,
}

/// Type alias for [`ThresholdMultiSig`]'s signature shape. Flat
/// `Vec<u8>` keeps the canonical-bytes hand-off shape simple at the
/// extension-point demo level; concrete impl-time signature shape
/// will live with the chosen recovery protocol.
pub type ThresholdSignature = Vec<u8>;

impl MultiSigSurface for ThresholdMultiSig {
    type Signature = ThresholdSignature;
    type Error = MultiSigError;

    fn sign(&self, _msg: &[u8]) -> Result<Self::Signature, Self::Error> {
        Err(MultiSigError::PostPhase3)
    }

    fn verify(&self, _msg: &[u8], _sig: &Self::Signature) -> Result<(), Self::Error> {
        Err(MultiSigError::PostPhase3)
    }

    fn threshold(&self) -> u32 {
        self.threshold
    }

    fn participants(&self) -> u32 {
        self.participants
    }
}
