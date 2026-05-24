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
//! adding a new workspace dep. **G-CORE-3a (CANARY) flips `0x647a`
//! X25519⊕ML-KEM-768 hybrid KEM (the vendored ~30-LOC X-Wing-style
//! combiner over `ml-kem` + `x25519-dalek` + `sha3` — stable-but-non-WG
//! IETF Independent Submission draft, Benten-owned) + `0x6400`
//! classical-only X25519 downgrade arm to LIVE.** The remaining
//! cipher-suite codepoints (`0x647b` NF-1 ML-KEM-768⊕HQC end-state +
//! `0x647c` pure-PQ ML-KEM-768-only swap-matrix arm +
//! `0x0000` no-encryption) stay reserved-typed-reject via
//! [`UnsupportedAlgorithm`] at the cipher-suite dispatcher level until
//! G-CORE-3c's full swap-matrix wave (`0x647c` is reachable ONLY via
//! the named [`swap_matrix::SwapMatrix::try_pure_pq_sole_trust_path`]
//! constructor which gates on `AUDIT_LANDED_PURE_PQ_FLAG`).
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
//!   (G-CORE-3a CANARY: `0x647a` hybrid + `0x6400` classical-X25519
//!   downgrade LIVE; `0x647b` NF-1 ML-KEM⊕HQC + `0x647c` pure-PQ
//!   ML-KEM-only + `0x0000` no-encryption remain reserved-typed-reject
//!   at the cipher-suite dispatcher; the pure-PQ arm is reachable only
//!   via the named `SwapMatrix::try_pure_pq_sole_trust_path` constructor).
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

pub mod aead;
pub mod boundary;
pub mod cipher_suite;
pub mod codepoint;
pub mod discharge;
pub mod error;
pub mod hash;
pub mod primitives;
pub mod sig;
pub mod sizes;
pub mod structural_kdf;
pub mod swap_matrix;
pub mod varsig;

// Convenience re-exports of the most-used typed surface.
pub use crate::aead::{AeadEnvelope, AeadError, AeadKeyMaterial};
pub use crate::codepoint::{CipherSuiteCodepoint, HashCodepoint, SigCodepoint};
pub use crate::error::{CryptoError, UnsupportedAlgorithm, VerifyError};
pub use crate::hash::HashSeam;
pub use crate::sig::{HybridSignature, SignatureSuite, SuiteConfig};
pub use crate::structural_kdf::{StructuralKdfKey, derive_root, derive_step};
pub use crate::swap_matrix::{
    AUDIT_LANDED_PURE_PQ_FLAG, KemKatVector, PureKemDec, PureKemEnc, PureKemKeypair,
    PurePqNf1SignatureArm, PureSigPubkey, PureSigVec, SignatureKatVector, SwapDecrypted,
    SwapEnvelope, SwapKeypair, SwapMatrix, SwapMatrixError, SwapPublicKey, SwapRecipientKeypair,
    SwapRecipientPublic, SwapRecipientSecret, audit_landed_pure_pq_flag,
};
pub use crate::varsig::{UcanVarsigV1Header, VarsigError};
