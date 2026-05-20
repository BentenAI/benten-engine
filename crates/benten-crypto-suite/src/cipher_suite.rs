//! Cipher-suite codepoint dispatch surface (G-CORE-3 #1301 hook).
//!
//! **THIS WAVE (G-CORE-2) reserves the codepoint dispatch + typed-unsupported
//! arm only.** The live X25519⊕ML-KEM-768 hybrid KEM at codepoint `0x647a`
//! (vendored ~30-LOC X-Wing-style combiner over `ml-kem` + `x25519-dalek` +
//! `sha3`) + the ChaCha20-Poly1305 bulk layer + HKDF derivation are
//! G-CORE-3's deliverable; they call the typed API surfaced here.
//!
//! Per `crypto-agility-contract:6`: the integration crate is the ONLY
//! crypto-primitive call site. The KEM/AEAD/KDF deps (`x25519-dalek`,
//! `ml-kem`, `chacha20poly1305`, `hkdf`) are declared in this crate's
//! Cargo.toml so when G-CORE-3 wires the live impls there is no new
//! workspace direct-dep on a primitive.
//!
//! ## Re-exports for G-CORE-3
//!
//! The component primitive crates will be re-exported via
//! `crate::primitives::*` (e.g. `x25519_dalek`, `ml_kem`,
//! `chacha20poly1305`, `hkdf`) when G-CORE-3 adds them as direct deps
//! of THIS crate alongside the live cipher-suite impl.

pub use crate::codepoint::CipherSuiteCodepoint;
use crate::error::UnsupportedAlgorithm;

/// G-CORE-3-hook cipher-suite dispatcher. At G-CORE-2 every arm typed-
/// rejects — G-CORE-3 flips `0x647a` to live and lights the swap matrix.
pub struct CipherSuite {
    codepoint: CipherSuiteCodepoint,
}

impl CipherSuite {
    /// The v1-beta DEFAULT codepoint per RATIFIED-pq-default-reframe §1:
    /// X25519⊕ML-KEM-768 hybrid KEM at `0x647a` + ChaCha20-Poly1305 bulk.
    /// **G-CORE-2 does NOT build the live impl — the dispatch typed-rejects
    /// at this wave; G-CORE-3 #1301 lights it.**
    #[must_use]
    pub const fn v1_default_codepoint() -> CipherSuiteCodepoint {
        CipherSuiteCodepoint::HYBRID_X25519_MLKEM768
    }

    /// Resolve a cipher-suite codepoint. **All arms typed-reject in this
    /// wave** (G-CORE-3 turns them live).
    pub fn resolve(codepoint: CipherSuiteCodepoint) -> Result<Self, UnsupportedAlgorithm> {
        codepoint.resolve()?;
        // Unreachable at this wave — all codepoints typed-reject above.
        Ok(Self { codepoint })
    }

    /// The dispatched codepoint.
    #[must_use]
    pub const fn codepoint(&self) -> CipherSuiteCodepoint {
        self.codepoint
    }
}
