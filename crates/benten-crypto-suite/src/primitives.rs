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

// NF-1 PQ⊕PQ end-state signature primitive — SLH-DSA via `slh-dsa`.
// SHIPPED at G-CORE-3c: the `slh-dsa 0.2.0-rc.5` direct dep is live in
// `Cargo.toml` and the codepoint `SigCodepoint::HYBRID_MLDSA65_SLHDSA =
// 0x0003` is a built sign+verify arm (`swap_matrix::SwapMatrix`'s
// pure-PQ-sole-trust-path). It is NOT re-exported here (used internally
// in `swap_matrix.rs` only); the arm is STRUCTURALLY NON-DEFAULT until
// the audit-landed flag flips (C11b safety invariant; NF-2 / C-GM-AUDIT).

/// v1-default hash primitive — BLAKE3 via `blake3`.
pub use blake3;

/// SHA2 pre-blessed hash fallback (SHA-512/256 = multihash 0x1015).
pub use sha2;

/// SHA3 pre-blessed hash fallback (SHA3-256 = multihash 0x16) +
/// commitment hash.
pub use sha3;

// G-CORE-3 #1301 encryption-side deps (x25519-dalek / libcrux-ml-kem /
// chacha20poly1305 / hkdf) are SHIPPED: all are live direct deps in
// `Cargo.toml` and routed to production impls — x25519-dalek +
// libcrux-ml-kem in `mlkem.rs` + `cipher_suite.rs` (X-Wing-hybrid wrap),
// chacha20poly1305 in `aead.rs` (bulk AEAD), hkdf in `structural_kdf.rs`
// + `vault.rs` (HKDF-SHA256 derivation). They are used internally and NOT
// re-exported here (external consumers route through the typed wrap/seal/
// derive APIs, not the raw primitive crates).
