//! Re-exports of the vetted upstream RustCrypto primitive crates.
//!
//! The integration crate is the ONLY crypto-primitive call site per
//! `crypto-agility-contract:6`. Consumers route through this module:
//!
//! ```ignore
//! // Before (forbidden post-G-CORE-2 in any non-suite crate):
//! use ed25519_dalek::Signature;
//!
//! // After:
//! use benten_crypto_suite::primitives::ed25519_dalek::Signature;
//! ```
//!
//! The re-export *is* the centralization of the direct-dep tree — only
//! THIS crate's `Cargo.toml` direct-deps the primitive crates; the
//! [`crate::boundary::CryptoPrimitiveCallSiteAudit`] scan-step enforces
//! this property by inspecting workspace `Cargo.toml`s.

/// v1-beta classical-signature primitive — Ed25519 via `ed25519-dalek`.
pub use ed25519_dalek;

/// v1-beta PQ-signature primitive — ML-DSA-65 via `ml-dsa`. LIVE
/// end-to-end as of G-CORE-2-FP-1 (2026-05-19) after the iroh 1.0.0-rc.0
/// bump closed the upstream ecosystem fork.
pub use ml_dsa;

// NF-1 PQ⊕PQ end-state signature primitive — SLH-DSA via `slh-dsa` —
// dep reserved for G-CORE-3c (workspace `sha2 0.10` vs slh-dsa's
// required `sha2 0.11` is a deliberate G-CORE-3c bump-and-light wave).
// The codepoint `SigCodepoint::HYBRID_MLDSA65_SLHDSA = 0x0003` is
// reserved + typed-rejects in this wave.

/// v1-default hash primitive — BLAKE3 via `blake3`.
pub use blake3;

/// SHA2 pre-blessed hash fallback (SHA-512/256 = multihash 0x1015).
pub use sha2;

/// SHA3 pre-blessed hash fallback (SHA3-256 = multihash 0x16) +
/// commitment hash.
pub use sha3;

// G-CORE-3 #1301 deps (x25519-dalek / ml-kem / chacha20poly1305 / hkdf)
// land HERE at G-CORE-3 — adding them in G-CORE-2 triggered an unrelated
// `pkcs8 0.11.0-rc.10 → 0.11.0` lockfile re-resolution that broke iroh's
// `ed25519 3.0.0-rc.4` (pinned via iroh-base `=ed25519-dalek 3.0.0-pre.6`).
// Reserved-but-unimplemented codepoint dispatch typed-rejects them in
// this wave; G-CORE-3 will add the deps and route the live impls.
