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
//! # v1-beta defaults (byte-faithful IETF LAMPS composite)
//!
//! - **Signature default = hybrid Ed25519 ⊕ ML-DSA-65**
//!   (`SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001`): the
//!   **byte-faithful IETF LAMPS composite `id-MLDSA65-Ed25519-SHA512`**
//!   (OID `1.3.6.1.5.5.7.6.48`; pinned to `draft-19` + test-vector commit
//!   `f0627ab3`, re-verify at RFC). Both halves independently sign the
//!   SHARED message representative `M' = Prefix || Label || len(ctx) ||
//!   ctx || SHA-512(M)` (the ML-DSA half with `mldsa_ctx = Label`); the
//!   wire is `mldsaSig || tradSig` (ML-DSA FIRST; NO commitment trailer);
//!   **both MUST verify** or the verify fails closed. Strip-resistance
//!   rests on the shared-`M'` / `mldsa_ctx=Label` binding (the LAMPS
//!   mechanism). The classical half (Ed25519) is the audited security
//!   floor; the construction means unaudited PQC is never the SOLE trust
//!   path. (This REPLACES the prior Benten-own "NF-4" Ed25519-first +
//!   SHA3-256-commitment construction; security posture unchanged —
//!   EUF-CMA-only / WNS per Compromise #31, SUF-equivalence closed by
//!   Inv-15.)
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
//!   (`SigCodepoint::HYBRID_MLDSA65_SLHDSA = 0x0003`) — reserved
//!   swap-matrix arm — typed-rejected by default at `SigCodepoint::resolve`
//!   + `SignatureSuite::resolve_codepoint` + `varsig.rs::decode_payload`
//!   per the C11b safety gate. **G-CORE-3c TERMINAL shipped the full
//!   swap matrix retaining 0x0003 as dispatcher-typed-rejected**; reachable
//!   only via the audit-gated `SwapMatrix::try_pure_pq_sole_trust_path`
//!   constructor which returns `AuditNotLandedPurePqRejected` at v1-beta
//!   (mirrors 0x647c framing).
//! - **KEM PQ⊕PQ = ML-KEM-768 ⊕ HQC** — reserved codepoint;
//!   build-trigger = FIPS 207 (HQC-KEM) published as a *final* standard
//!   (NIST-projected 2027); FIPS-207 *draft* (~early-2026) is the
//!   early-warning. Until then HQC is reserved-unimplemented.
//!
//! # Cipher-suite (G-CORE-3 / #1301) — LIVE at v1-beta
//!
//! The KEM/AEAD primitive deps (`x25519-dalek`, `ml-kem`,
//! `chacha20poly1305`, `hkdf`) are declared HERE so G-CORE-3 plugs into
//! a real typed surface ([`cipher_suite::CipherSuiteCodepoint`]) without
//! adding a new workspace dep. **G-CORE-3a CANARY shipped `0x647a`
//! X25519⊕ML-KEM-768 hybrid KEM LIVE** (the vendored ~30-LOC X-Wing-style
//! combiner over `libcrux-ml-kem` (via `crate::mlkem`; RustCrypto `ml-kem` is
//! the dev-only KAT witness) + `x25519-dalek` + `sha3` — stable-but-non-WG
//! IETF Independent Submission draft, Benten-owned) + **`0x6400`
//! classical-only X25519 downgrade arm LIVE.** **G-CORE-3c TERMINAL shipped
//! the full swap matrix** retaining `0x647c` (pure-PQ ML-KEM-768-only swap-matrix
//! arm) + `0x0003` (NF-1 sig end-state) as dispatcher-typed-rejected per the
//! C11b safety gate. The remaining reserved cipher-suite codepoints
//! (`0x647b` NF-1 ML-KEM-768⊕HQC end-state + `0x0000` no-encryption) stay
//! reserved-typed-reject via [`UnsupportedAlgorithm`] at the cipher-suite
//! dispatcher level; `0x647c` is reachable ONLY via the named
//! [`swap_matrix::SwapMatrix::try_pure_pq_sole_trust_path`] constructor which
//! gates on `AUDIT_LANDED_PURE_PQ_FLAG`.
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
pub mod conformance;
pub mod discharge;
pub mod domain_registry;
pub mod envelope;
pub mod error;
pub mod hash;
pub mod hpke;
pub(crate) mod mlkem;
pub mod primitives;
pub mod registry;
pub mod sig;
pub mod sizes;
pub mod structural_kdf;
pub mod swap_matrix;
pub mod varsig;
pub mod vault;

// Convenience re-exports of the most-used typed surface.
pub use crate::aead::{AeadEnvelope, AeadError, AeadKeyMaterial};
pub use crate::cipher_suite::{
    X_WING_LABEL, X25519_PUBLIC_LEN, X25519_SECRET_LEN, classical_combine, combine_x_wing,
    x_wing_combiner_preimage,
};
pub use crate::codepoint::{CipherSuiteCodepoint, CodepointLifecycle, HashCodepoint, SigCodepoint};
pub use crate::envelope::{
    BindingContext, ENVELOPE_FORMAT_VERSION_V1, ENVELOPE_FORMAT_VERSION_V2, ENVELOPE_MAGIC,
    EncryptedEnvelope, EnvelopeError, MAX_NONCE_LEN, canonical_tlv_encode,
    layer_c_and_d_share_one_hpke_primitive,
};
pub use crate::error::{CryptoError, UnsupportedAlgorithm, VerifyError};
pub use crate::hash::HashSeam;
pub use crate::sig::{HybridSignature, SignatureSuite, SuiteConfig};
pub use crate::structural_kdf::{StructuralKdfKey, derive_root, derive_step};
pub use crate::swap_matrix::{
    AUDIT_LANDED_PURE_PQ_FLAG, PureKemEnc, PureKemKeypair, PurePqNf1SignatureArm, PureSigPubkey,
    PureSigVec, SwapDecrypted, SwapEnvelope, SwapKeypair, SwapMatrix, SwapMatrixError,
    SwapPublicKey, SwapRecipientKeypair, SwapRecipientPublic, SwapRecipientSecret,
    audit_landed_pure_pq_flag,
};
// R18 C4 + D-74/75/76: the KAT-fixture structs + the `PureKemDec` recovered-
// shared-secret handle are TEST-ONLY conformance surfaces — gated off the
// frozen default-feature public-api surface (their loaders /
// `ml_kem_768_decapsulate_for_test` are already
// `#[cfg(any(test, feature = "testing"))]`).
#[cfg(any(test, feature = "testing"))]
pub use crate::swap_matrix::{KemKatVector, PureKemDec, SignatureKatVector};
pub use crate::varsig::{UcanVarsigV1Header, VarsigError};
pub use crate::vault::{
    Argon2idParams, DAK_HKDF_INFO_TAG, OWASP_DEFAULT, UnlockedKeyMaterial, VaultEngine, VaultError,
    VaultPayload, derive_dak,
};
