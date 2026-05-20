//! `benten-crypto-suite` — the ONE thin Benten-owned signature / hash /
//! cipher-suite agility integration crate.
//!
//! # The crypto-agility contract this crate implements (G-CORE-2 / #1300)
//!
//! Per `CLAUDE.md` baked-in #5 +
//! `RATIFIED-crypto-agility-2026-05-18.md` +
//! `RATIFIED-pq-default-reframe-2026-05-19.md`:
//!
//! - The **permanent** commitment is the **multiformats framing**
//!   (CIDv1 / multihash / multicodec / `did:key` / UCAN-Varsig).
//!   Algorithms are v1 *default implementations selected within* that
//!   framing.
//! - **This crate is the ONLY crypto-primitive call site.** #1301
//!   and the hash seam call its *typed* API; they never instantiate
//!   a primitive themselves (`crypto-agility-contract:6`).
//! - **Never fork / never reimplement primitives** — this crate is
//!   concat / hash / codepoint / envelope GLUE only over vetted
//!   upstream RustCrypto crates (`ed25519-dalek`, `ml-dsa`,
//!   `slh-dsa`, `x25519-dalek`, `chacha20poly1305`, `hkdf`, `sha2`,
//!   `sha3`, `blake3`, `ml-kem`).
//! - **Never hardcode key/sig/ciphertext sizes** anywhere. Sizes are
//!   reported dynamically by codepoint (the load-bearing v1-PRODUCTION
//!   correctness property — ML-DSA-65 dims ~1952 B key / ~3309 B sig
//!   now flow on the *default* path under the PQ-default reframe).
//! - Codepoint dispatch has a **typed `UnsupportedAlgorithm` arm —
//!   NEVER a silent fallback** (P2P-mainstream choice per Veilid,
//!   MLS-RFC9420, Nostr NIP-44; age's silent-ignore is the deliberately
//!   rejected outlier).
//!
//! # v1-beta defaults (NF-4 ratified)
//!
//! - **Signature default = hybrid Ed25519 ⊕ ML-DSA-65**
//!   (`SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001`):
//!   *concatenated, committing, strip-resistant*. The message is
//!   independently signed with both keys; both signatures travel
//!   together with a commitment that binds them to the message; **both
//!   MUST verify** or the verify fails closed. Aligned with the IETF
//!   `draft-ietf-lamps-pq-composite-sigs` line. The classical half
//!   (Ed25519) is the audited security floor; the construction means
//!   unaudited PQC is never the SOLE trust path.
//!
//! - **Hash default = BLAKE3-256** (multihash `0x1e`) with pre-blessed
//!   agile fallbacks SHA-512/256 (multihash `0x1015`) + SHA3-256
//!   (multihash `0x16`). Hash is PQ-UNAFFECTED — Grover quadratic-only
//!   (256-bit safe indefinitely).
//!
//! # Non-default downgrade arms (built + conformance-testable)
//!
//! - **Classical-only Ed25519** (`SigCodepoint::CLASSICAL_ED25519 = 0x0002`)
//!   — a real, built downgrade config (NOT paper); the suite returned by
//!   `SignatureSuite::from_config(SuiteConfig::classical_only())`
//!   round-trips sign/verify on the production path and refuses to
//!   silently accept a hybrid-codepoint signature.
//!
//! # Reserved-but-unimplemented codepoints (NF-1 PQ⊕PQ end-state)
//!
//! - **Signature PQ⊕PQ = ML-DSA-65 ⊕ SLH-DSA**
//!   (`SigCodepoint::HYBRID_MLDSA65_SLHDSA = 0x0003`) — reserved-now,
//!   conformance-built at G-CORE-3c (the full swap matrix). This wave
//!   typed-rejects it with [`UnsupportedAlgorithm::Signature`].
//! - **KEM PQ⊕PQ = ML-KEM-768 ⊕ HQC** — reserved codepoint;
//!   build-trigger = FIPS 207 (HQC-KEM) published as a *final* standard
//!   (NIST-projected 2027); FIPS-207 *draft* (~early-2026) is the
//!   early-warning. Until then HQC is reserved-unimplemented.
//!
//! # Cipher-suite (G-CORE-3 / #1301) deferred surface
//!
//! The KEM/AEAD primitive deps (`x25519-dalek`, `ml-kem`,
//! `chacha20poly1305`, `hkdf`) are declared HERE so G-CORE-3 plugs into
//! a real typed surface ([`cipher_suite::CipherSuiteCodepoint`]) without
//! adding a new workspace dep. G-CORE-3 lights up the
//! `0x647a` X25519⊕ML-KEM-768 hybrid KEM (the vendored ~30-LOC X-Wing-style
//! combiner over `ml-kem` + `x25519-dalek` + `sha3` — stable-but-non-WG
//! IETF Independent Submission draft, Benten-owned). In this wave the
//! cipher-suite codepoints typed-reject via the same
//! [`UnsupportedAlgorithm`] arm.
//!
//! # Module map
//!
//! - [`codepoint`] — typed codepoint enums + dispatch table.
//! - [`sig`] — signature seam (hybrid default; classical downgrade;
//!   typed-unsupported arm; the [`sig::HybridSignature`] +
//!   [`sig::SignatureSuite`] surface).
//! - [`hash`] — hash seam (BLAKE3 default + SHA-512/256 + SHA3-256
//!   agile fallbacks; typed-unsupported arm).
//! - [`varsig`] — UCAN-Varsig-v1 header round-trip for the hybrid
//!   signature.
//! - [`sizes`] — the size-touching surface aggregator (struct /
//!   DAG-CBOR / redb / CID / napi / fixtures) — the canonical
//!   no-hardcoded-sizes substrate the cross-surface ML-DSA-65 vector
//!   round-trips through.
//! - [`cipher_suite`] — typed surface for G-CORE-3 #1301
//!   (reserved-but-unimplemented at this wave; live impls land in
//!   G-CORE-3).
//! - [`error`] — typed errors including [`error::UnsupportedAlgorithm`].
//! - [`boundary`] — the call-site-audit surface (TF-2 grep-pin
//!   substrate; asserts this crate is the only one that direct-deps the
//!   primitive crates).
//! - [`discharge`] — the #835 `from_string_unchecked` discharge marker
//!   (verify-and-execute, not prose).
//! - [`primitives`] — re-exports of the primitive crates so in-flight
//!   consumers route their imports through this crate (`use
//!   benten_crypto_suite::primitives::ed25519_dalek` instead of `use
//!   ed25519_dalek`). The re-export *is* the centralization of the
//!   direct-dep tree — the only crate that direct-deps the primitive
//!   crates is THIS one.

#![doc(html_root_url = "https://docs.rs/benten-crypto-suite/0.0.0/")]

pub mod boundary;
pub mod cipher_suite;
pub mod codepoint;
pub mod discharge;
pub mod error;
pub mod hash;
pub mod primitives;
pub mod sig;
pub mod sizes;
pub mod varsig;

// Convenience re-exports of the most-used typed surface.
pub use crate::codepoint::{HashCodepoint, SigCodepoint};
pub use crate::error::{CryptoError, UnsupportedAlgorithm, VerifyError};
pub use crate::hash::HashSeam;
pub use crate::sig::{HybridSignature, SignatureSuite, SuiteConfig};
pub use crate::varsig::{UcanVarsigV1Header, VarsigError};
