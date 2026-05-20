//! Hash seam — BLAKE3 default + SHA-512/256 + SHA3-256 agile fallbacks.
//!
//! Hash is **PQ-UNAFFECTED** per `CLAUDE.md` baked-in #5 — Grover's
//! attack is quadratic-only; 256-bit hashes retain ~128-bit pre-image
//! resistance indefinitely under known quantum constraints. The seam
//! exists for *algorithm-agility* (multihash `0x1e` BLAKE3 is still
//! draft → permanent promotion is tracked) not for PQ — the pre-blessed
//! agile fallbacks SHA-512/256 (`0x1015`, FIPS, permanent) and SHA3-256
//! (`0x16`, permanent) are the deliberate hedge.

use sha2::Digest as _;
use sha3::Digest as _;

use crate::codepoint::HashCodepoint;
use crate::error::UnsupportedAlgorithm;

/// Hash dispatcher — typed surface over BLAKE3 / SHA-512/256 / SHA3-256.
///
/// The struct wraps the resolved [`HashCodepoint`] + the corresponding
/// hash impl behind a single [`HashSeam::digest`] entry point. Sizes are
/// **NOT hardcoded** anywhere on the public surface — output length is
/// reported per-codepoint via [`HashSeam::output_byte_len`].
#[derive(Debug)]
pub struct HashSeam {
    codepoint: HashCodepoint,
}

impl HashSeam {
    /// Resolve a codepoint into a [`HashSeam`] handle. Unknown / reserved
    /// codepoints surface [`UnsupportedAlgorithm::Hash`] — **never a
    /// silent BLAKE3 fallback**.
    pub fn for_codepoint(codepoint: HashCodepoint) -> Result<Self, UnsupportedAlgorithm> {
        codepoint.resolve()?;
        Ok(Self { codepoint })
    }

    /// The v1 default hash codepoint = BLAKE3 (multihash `0x1e`).
    #[must_use]
    pub const fn default_codepoint() -> HashCodepoint {
        HashCodepoint::BLAKE3
    }

    /// Hash the input under this seam's codepoint, returning the
    /// codepoint-derived digest bytes.
    #[must_use]
    pub fn digest(&self, input: &[u8]) -> Vec<u8> {
        match self.codepoint.raw() {
            0x1e => blake3::hash(input).as_bytes().to_vec(),
            0x1015 => {
                // SHA-512/256: SHA-512 truncated to 256 bits per FIPS.
                let mut hasher = sha2::Sha512_256::new();
                hasher.update(input);
                hasher.finalize().to_vec()
            }
            0x16 => {
                let mut hasher = sha3::Sha3_256::new();
                hasher.update(input);
                hasher.finalize().to_vec()
            }
            // Unreachable per `for_codepoint`'s pre-resolve gate.
            other => unreachable!("unsupported hash codepoint 0x{other:x} bypassed resolve()"),
        }
    }

    /// Per-codepoint output byte length. NOT hardcoded — derived from
    /// the codepoint's resolved impl.
    #[must_use]
    pub fn output_byte_len(&self) -> usize {
        match self.codepoint.raw() {
            // BLAKE3 default-output and both SHA-256 variants are 32 B.
            0x1e | 0x1015 | 0x16 => 32,
            other => unreachable!("unsupported hash codepoint 0x{other:x} bypassed resolve()"),
        }
    }

    /// Borrow the codepoint this seam is dispatched on.
    #[must_use]
    pub const fn codepoint(&self) -> HashCodepoint {
        self.codepoint
    }
}
