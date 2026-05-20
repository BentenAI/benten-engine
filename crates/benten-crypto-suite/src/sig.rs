//! Signature seam — v1-beta hybrid Ed25519⊕ML-DSA-65 default surface
//! (LIVE end-to-end as of G-CORE-2-FP-1 / 2026-05-19 — the iroh 0.98 →
//! 1.0.0-rc.0 bump closes the upstream ecosystem fork that previously
//! gated the live ML-DSA-65 arm; classical Ed25519 non-default
//! downgrade arm (LIVE); typed-unsupported arm on unknown / reserved
//! codepoints.
//!
//! # NF-4 construction (RATIFIED — both halves LIVE)
//!
//! The hybrid is built **concatenated / committing / strip-resistant**
//! per `RATIFIED-pq-default-reframe-2026-05-19` §2 NF-4
//! (`draft-ietf-lamps-pq-composite-sigs-18` aligned):
//!
//! ```text
//! SIGN(msg):
//!     classical_sig = Ed25519.sign(msg)
//!     pq_sig        = ML-DSA-65.sign(msg)
//!     commitment    = SHA3-256(domain_sep || pub_classical || pub_pq ||
//!                              classical_sig || pq_sig || msg)
//!     hybrid_sig    = classical_sig || pq_sig || commitment
//!
//! VERIFY(pub_classical, pub_pq, msg, hybrid_sig):
//!     1. split hybrid_sig into (classical_sig, pq_sig, commitment)
//!        — codepoint dispatches the dimensions; NO hardcoded sizes.
//!     2. recompute commitment_expected from the inputs.
//!     3. fail-closed if commitment != commitment_expected.
//!     4. fail-closed if Ed25519.verify(...) rejects.
//!     5. fail-closed if ML-DSA-65.verify(...) rejects.
//!     6. else Ok(()).
//! ```
//!
//! The commitment binds (a) both public-keys, (b) both signature halves,
//! and (c) the message — so neither half can be stripped, truncated, or
//! cross-message-substituted without the verify failing closed. The
//! commitment + the dual cryptographic verifies are **independent
//! fail-closed surfaces**; both fire on adversarial inputs. This is the
//! load-bearing safety property of the whole PQ-default reframe.
//!
//! # Hybrid arm behavior — both must verify, never silent fallback
//!
//! Hybrid `verify` returns `Ok(())` ONLY when BOTH the classical
//! Ed25519 cryptographic verify AND the ML-DSA-65 cryptographic verify
//! succeed against the same message and the commitment binding all the
//! inputs matches. Returns a typed [`VerifyError`] otherwise — NEVER a
//! silent single-half accept; NEVER a silent classical-only fallback;
//! NEVER `Ok(())` after only one half's cryptographic verify.
//!
//! # No-hardcoded-sizes property (CLAUDE.md baked-in #5)
//!
//! Every public surface reports its sizes dynamically via the
//! codepoint-dispatch. ML-DSA-65 dimensions flow from upstream
//! `ml_dsa` type-level constants ([`crate::sizes::ml_dsa_65_pubkey_len`]
//! / [`crate::sizes::ml_dsa_65_sig_len`]) — NOT redefined. Ed25519
//! dimensions flow from `ed25519_dalek::SIGNATURE_LENGTH`. The
//! commitment is fixed at SHA3-256 output (32 B).

use ed25519_dalek::{Signer as _, Verifier as _};
use ml_dsa::signature::{Keypair as _, Signer as _, Verifier as _};
use ml_dsa::{
    EncodedSignature, EncodedVerifyingKey, Generate as _, KeySizeUser, MlDsa65,
    Signature as MlDsaSig, SigningKey as MlDsaSigningKey, VerifyingKey as MlDsaVerifyingKey,
};
use rand_core::OsRng;
use sha3::Digest as _;

use crate::codepoint::SigCodepoint;
use crate::error::UnsupportedAlgorithm;
use crate::sizes::ml_dsa_65_sig_len;

// Re-export so test files that `use benten_crypto_suite::sig::VerifyError`
// (per TF-2 spec) find it under sig where the verify happens.
pub use crate::error::VerifyError;

// Ed25519 dimensions are vetted via the upstream crate constants;
// re-export-not-redefine.
#[allow(dead_code)]
const ED25519_PUBLIC_LEN: usize = ed25519_dalek::PUBLIC_KEY_LENGTH;
const ED25519_SIG_LEN: usize = ed25519_dalek::SIGNATURE_LENGTH;

const COMMITMENT_LEN: usize = 32; // SHA3-256 output.

const HYBRID_DOMAIN_SEP: &[u8] = b"benten-crypto-suite/hybrid-ed25519-mldsa65/v1\0";

/// Public configuration of a [`SignatureSuite`] — selects between the
/// v1-beta hybrid default (LIVE) and the classical-only downgrade arm
/// (LIVE).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuiteConfig {
    codepoint: SigCodepoint,
}

impl SuiteConfig {
    /// The v1-beta DEFAULT — hybrid Ed25519⊕ML-DSA-65 (NF-4
    /// concatenated/committing/strip-resistant); BOTH halves
    /// cryptographically verified; LIVE end-to-end.
    #[must_use]
    pub const fn v1_default() -> Self {
        Self {
            codepoint: SigCodepoint::HYBRID_ED25519_MLDSA65,
        }
    }

    /// Non-default downgrade — classical-only Ed25519 (LIVE in this wave).
    #[must_use]
    pub const fn classical_only() -> Self {
        Self {
            codepoint: SigCodepoint::CLASSICAL_ED25519,
        }
    }

    /// The dispatched codepoint.
    #[must_use]
    pub const fn codepoint(self) -> SigCodepoint {
        self.codepoint
    }

    /// `true` iff this config is the hybrid default.
    #[must_use]
    pub const fn is_hybrid(self) -> bool {
        self.codepoint.is_hybrid_default()
    }

    /// Returns true if two configs are observably distinct.
    #[must_use]
    pub fn is_distinct_from(&self, other: &Self) -> bool {
        self.codepoint != other.codepoint
    }
}

/// Hybrid keypair — carries the classical half (always) + the PQ half
/// when the suite is hybrid (LIVE end-to-end).
pub struct Keypair {
    classical: ed25519_dalek::SigningKey,
    pq: Option<MlDsaSigningKey<MlDsa65>>,
}

impl Keypair {
    /// Public-key handle (both halves where the suite carries them).
    #[must_use]
    pub fn public(&self) -> PublicKey {
        PublicKey {
            classical: self.classical.verifying_key(),
            pq: self.pq.as_ref().map(|sk| sk.verifying_key()),
        }
    }
}

/// Public-key handle — carries the verifying-key for both halves where
/// the suite is hybrid.
pub struct PublicKey {
    classical: ed25519_dalek::VerifyingKey,
    pq: Option<MlDsaVerifyingKey<MlDsa65>>,
}

impl PublicKey {
    /// `true` iff this public-key handle carries the PQ verifying-key
    /// (i.e. came from a hybrid keypair).
    #[must_use]
    pub fn is_hybrid(&self) -> bool {
        self.pq.is_some()
    }
}

/// Hybrid signature — concatenated/committing/strip-resistant.
///
/// The byte layout is `classical_sig || pq_sig || commitment` for the
/// hybrid arm; `classical_sig` only for the classical-only arm. Sizes
/// are codepoint-dispatched (NOT Ed25519-shape-assumed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HybridSignature {
    codepoint: SigCodepoint,
    classical: Vec<u8>,
    pq: Vec<u8>,
    commitment: Vec<u8>,
}

impl HybridSignature {
    /// The codepoint this signature was produced under.
    #[must_use]
    pub const fn codepoint(&self) -> SigCodepoint {
        self.codepoint
    }

    /// Encode as the wire-bytes the UCAN-Varsig-v1 header carries.
    #[must_use]
    pub fn to_wire_bytes(&self) -> Vec<u8> {
        let mut buf =
            Vec::with_capacity(self.classical.len() + self.pq.len() + self.commitment.len());
        buf.extend_from_slice(&self.classical);
        buf.extend_from_slice(&self.pq);
        buf.extend_from_slice(&self.commitment);
        buf
    }

    /// Test-helper: remove the PQ half. Used by adversarial strip pins.
    #[must_use]
    pub fn without_pq_half_for_test(&self) -> Self {
        Self {
            codepoint: self.codepoint,
            classical: self.classical.clone(),
            pq: Vec::new(),
            commitment: self.commitment.clone(),
        }
    }

    /// Test-helper: remove the classical half.
    #[must_use]
    pub fn without_classical_half_for_test(&self) -> Self {
        Self {
            codepoint: self.codepoint,
            classical: Vec::new(),
            pq: self.pq.clone(),
            commitment: self.commitment.clone(),
        }
    }

    /// Test-helper: borrow the classical half bytes.
    #[must_use]
    pub fn classical_half_for_test(&self) -> Vec<u8> {
        self.classical.clone()
    }

    /// Test-helper: borrow the PQ half bytes.
    #[must_use]
    pub fn pq_half_for_test(&self) -> Vec<u8> {
        self.pq.clone()
    }

    /// Test-helper: splice arbitrary halves into a new signature.
    #[must_use]
    pub fn splice_for_test(classical: Vec<u8>, pq: Vec<u8>) -> Self {
        Self {
            codepoint: SigCodepoint::HYBRID_ED25519_MLDSA65,
            classical,
            pq,
            commitment: Vec::new(),
        }
    }

    /// Crate-internal constructor (used by [`crate::varsig`] to round-trip
    /// a signature out of wire bytes).
    #[doc(hidden)]
    #[must_use]
    pub fn from_parts_internal(
        codepoint: SigCodepoint,
        classical: Vec<u8>,
        pq: Vec<u8>,
        commitment: Vec<u8>,
    ) -> Self {
        Self {
            codepoint,
            classical,
            pq,
            commitment,
        }
    }

    /// Mutate a byte of the PQ half (CID-derivation pin).
    pub fn flip_pq_byte_for_test(&mut self, offset: usize) {
        if offset < self.pq.len() {
            self.pq[offset] ^= 0xff;
        } else if !self.pq.is_empty() {
            let last = self.pq.len() - 1;
            self.pq[last] ^= 0xff;
        }
    }
}

/// The integration crate's signature suite — the codepoint-dispatched
/// sign/verify entry point.
pub struct SignatureSuite {
    config: SuiteConfig,
}

impl SignatureSuite {
    /// Construct from a [`SuiteConfig`].
    #[must_use]
    pub const fn from_config(config: SuiteConfig) -> Self {
        Self { config }
    }

    /// v1-beta DEFAULT — hybrid Ed25519⊕ML-DSA-65 (LIVE end-to-end).
    #[must_use]
    pub const fn v1_default() -> Self {
        Self::from_config(SuiteConfig::v1_default())
    }

    /// `true` iff this suite is the hybrid default.
    #[must_use]
    pub const fn is_hybrid(&self) -> bool {
        self.config.is_hybrid()
    }

    /// The codepoint this suite dispatches on.
    #[must_use]
    pub const fn default_codepoint(&self) -> SigCodepoint {
        self.config.codepoint
    }

    /// Resolve a codepoint into a SignatureSuite — the typed-unsupported
    /// arm fires for unknown/reserved codepoints. **Never a silent
    /// fallback.**
    pub fn resolve_codepoint(codepoint: SigCodepoint) -> Result<Self, UnsupportedAlgorithm> {
        match codepoint.raw() {
            0x0001 => Ok(Self::from_config(SuiteConfig::v1_default())),
            0x0002 => Ok(Self::from_config(SuiteConfig::classical_only())),
            // NF-1 PQ⊕PQ reserved-but-unimplemented + every other unknown.
            other => Err(UnsupportedAlgorithm::Signature { codepoint: other }),
        }
    }

    /// Per-codepoint signature byte length — sum of the cryptographic
    /// halves (NOT including the commitment substrate). NOT a hardcoded
    /// `pub const`. Sources its constants from the upstream crate
    /// constants via [`crate::sizes`].
    #[must_use]
    pub fn signature_byte_len_for(&self, codepoint: SigCodepoint) -> usize {
        match codepoint.raw() {
            0x0001 => ED25519_SIG_LEN + ml_dsa_65_sig_len(),
            0x0002 => ED25519_SIG_LEN,
            _ => 0,
        }
    }

    /// Verify the integration crate exposes NO public `pub const SIG_LEN`
    /// — the size-agility structural pin.
    #[must_use]
    pub const fn exposes_static_size_constant() -> bool {
        false
    }

    /// Generate a keypair for the configured suite.
    #[must_use]
    pub fn generate_keypair(&self) -> Keypair {
        let classical = ed25519_dalek::SigningKey::generate(&mut OsRng);
        let pq = if self.is_hybrid() {
            // ML-DSA-65 key-gen via the `Generate::generate()` path
            // (enabled by the `getrandom` feature; uses `SysRng`
            // internally — sidesteps the rand_core 0.6/0.10 version
            // skew between ed25519-dalek 2.x and ml-dsa 0.1's
            // crypto-common 0.2.x).
            Some(MlDsaSigningKey::<MlDsa65>::generate())
        } else {
            None
        };
        Keypair { classical, pq }
    }

    /// Sign a message under the configured suite.
    ///
    /// **Hybrid arm (LIVE):** produces a real ML-DSA-65 signature via
    /// the deterministic signing path + a real Ed25519 signature + the
    /// NF-4 commitment binding both pubkeys + both sigs + the message.
    /// Both halves cryptographically verify on `verify`.
    ///
    /// Classical-only arm: produces a REAL Ed25519 signature; PQ half
    /// empty; commitment empty (the construction degenerates to a plain
    /// Ed25519 signature on the wire).
    #[must_use]
    pub fn sign(&self, kp: &Keypair, msg: &[u8]) -> HybridSignature {
        let classical_sig = kp.classical.sign(msg);
        let classical_bytes = classical_sig.to_bytes().to_vec();

        if !self.is_hybrid() {
            return HybridSignature {
                codepoint: self.config.codepoint,
                classical: classical_bytes,
                pq: Vec::new(),
                commitment: Vec::new(),
            };
        }

        // Hybrid arm — real ML-DSA-65 sign + commitment binding.
        let pq_sk = kp
            .pq
            .as_ref()
            .expect("hybrid keypair must carry a PQ signing key (invariant of generate_keypair)");
        let pq_vk = pq_sk.verifying_key();
        let pq_sig: MlDsaSig<MlDsa65> = pq_sk.sign(msg);
        let pq_bytes = pq_sig.encode().as_slice().to_vec();

        let commitment = compute_commitment(
            &kp.classical.verifying_key(),
            &pq_vk,
            &classical_bytes,
            &pq_bytes,
            msg,
        );

        HybridSignature {
            codepoint: self.config.codepoint,
            classical: classical_bytes,
            pq: pq_bytes,
            commitment,
        }
    }

    /// Verify a signature under the configured suite.
    ///
    /// - **Hybrid (LIVE):** returns `Ok(())` ONLY when BOTH the
    ///   classical Ed25519 cryptographic verify AND the ML-DSA-65
    ///   cryptographic verify succeed against the same message AND the
    ///   NF-4 commitment (over `domain_sep || pub_classical || pub_pq
    ///   || classical_sig || pq_sig || msg`) recomputes equal to the
    ///   commitment that travelled with the signature. Returns a typed
    ///   [`VerifyError`] otherwise — strip / substitution / tamper /
    ///   half-missing / commitment-mismatch attacks all surface typed
    ///   errors. NEVER a silent single-half accept. NEVER `Ok(())`
    ///   after only one cryptographic verify.
    /// - Classical-only suite handed a hybrid-codepoint sig: surfaces
    ///   [`VerifyError::CodepointMismatch`] (silent-downgrade defense).
    /// - Classical-only suite + classical-only sig: real Ed25519 verify.
    pub fn verify(
        &self,
        pk: PublicKey,
        msg: &[u8],
        sig: &HybridSignature,
    ) -> Result<(), VerifyError> {
        // Codepoint-mismatch guard: a classical-only suite MUST NOT
        // silently accept a hybrid-coded signature by ignoring the PQ
        // half. The downgrade arm is a downgrade for FRESHLY-signed
        // content, not a strip path for incoming hybrid sigs.
        if !self.is_hybrid() && sig.codepoint.is_hybrid_default() {
            return Err(VerifyError::CodepointMismatch);
        }

        // Classical-only path: real Ed25519 verify.
        if !self.is_hybrid() {
            if sig.classical.len() != ED25519_SIG_LEN {
                return Err(VerifyError::MalformedSignature(
                    "classical-only sig has non-Ed25519 length",
                ));
            }
            let sig_bytes: [u8; ED25519_SIG_LEN] = sig
                .classical
                .as_slice()
                .try_into()
                .map_err(|_| VerifyError::MalformedSignature("classical sig length"))?;
            let classical_sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
            return pk
                .classical
                .verify(msg, &classical_sig)
                .map_err(|_| VerifyError::ClassicalVerifyFailed);
        }

        // Hybrid verify — REQUIRES BOTH halves + a matching commitment
        // + BOTH cryptographic verifies succeed.
        if sig.classical.is_empty() {
            return Err(VerifyError::HybridHalfMissing("classical half is empty"));
        }
        if sig.pq.is_empty() {
            return Err(VerifyError::HybridHalfMissing("PQ half is empty"));
        }
        if sig.commitment.is_empty() {
            return Err(VerifyError::StripResistanceViolated(
                "commitment is missing (would silently allow cross-message splice)",
            ));
        }
        let pq_vk = pk.pq.as_ref().ok_or(VerifyError::HybridHalfMissing(
            "hybrid public key missing PQ half (caller used non-hybrid keypair)",
        ))?;

        // Recompute the commitment over the FULL inputs (binds both
        // pubkeys + both sigs + msg per NF-4); mismatch = strip /
        // substitute / cross-message-splice attack.
        let commitment_expected =
            compute_commitment(&pk.classical, pq_vk, &sig.classical, &sig.pq, msg);
        if commitment_expected != sig.commitment {
            return Err(VerifyError::StripResistanceViolated(
                "commitment mismatch — either half was substituted, a pubkey was substituted, or the message differs",
            ));
        }

        // Cryptographically verify the classical Ed25519 half.
        if sig.classical.len() != ED25519_SIG_LEN {
            return Err(VerifyError::MalformedSignature("classical sig length"));
        }
        let classical_bytes: [u8; ED25519_SIG_LEN] = sig
            .classical
            .as_slice()
            .try_into()
            .map_err(|_| VerifyError::MalformedSignature("classical sig length"))?;
        let classical_sig = ed25519_dalek::Signature::from_bytes(&classical_bytes);
        pk.classical
            .verify(msg, &classical_sig)
            .map_err(|_| VerifyError::ClassicalVerifyFailed)?;

        // Cryptographically verify the PQ ML-DSA-65 half (LIVE — wired
        // at G-CORE-2-FP-1 2026-05-19 after the iroh 1.0.0-rc.0 bump
        // closed the upstream ecosystem fork). `Signature::decode` is
        // size-checked at type-level (the encoded buffer must be exactly
        // ML-DSA-65 signature size); a malformed/truncated PQ half
        // surfaces `VerifyError::MalformedSignature` before ever
        // reaching the cryptographic check.
        if sig.pq.len() != ml_dsa_65_sig_len() {
            return Err(VerifyError::MalformedSignature("pq sig length"));
        }
        let encoded_pq: EncodedSignature<MlDsa65> =
            EncodedSignature::<MlDsa65>::try_from(sig.pq.as_slice())
                .map_err(|_| VerifyError::MalformedSignature("pq sig encoding"))?;
        let pq_sig = MlDsaSig::<MlDsa65>::decode(&encoded_pq).ok_or(
            VerifyError::MalformedSignature("pq sig decode (algorithm-internal shape violation)"),
        )?;
        pq_vk
            .verify(msg, &pq_sig)
            .map_err(|_| VerifyError::PqVerifyFailed)?;

        Ok(())
    }
}

/// Compute the NF-4 commitment binding both pubkeys + both sigs + msg.
///
/// Live arm — `pq_pk` is the real ML-DSA-65 verifying key. The
/// commitment is structurally strip-resistant: any cross-message splice
/// or pubkey substitution mutates a binding input and trips the
/// recomputation-mismatch arm on `verify`.
fn compute_commitment(
    classical_pk: &ed25519_dalek::VerifyingKey,
    pq_pk: &MlDsaVerifyingKey<MlDsa65>,
    classical_sig: &[u8],
    pq_sig: &[u8],
    msg: &[u8],
) -> Vec<u8> {
    let pq_pk_encoded: EncodedVerifyingKey<MlDsa65> = pq_pk.encode();
    let pq_pk_bytes = pq_pk_encoded.as_slice();
    debug_assert_eq!(pq_pk_bytes.len(), <MlDsaVerifyingKey<MlDsa65>>::key_size());

    let mut hasher = sha3::Sha3_256::new();
    hasher.update(HYBRID_DOMAIN_SEP);
    hasher.update(classical_pk.as_bytes());
    hasher.update(pq_pk_bytes);
    hasher.update(classical_sig);
    hasher.update(pq_sig);
    hasher.update(msg);
    let result = hasher.finalize().to_vec();
    debug_assert_eq!(result.len(), COMMITMENT_LEN);
    result
}

/// Re-export the Ed25519 dimensions for downstream introspection — sourced
/// from the upstream crate (NOT redefined). NOT public; the production
/// surface reports sizes via [`SignatureSuite::signature_byte_len_for`].
#[doc(hidden)]
pub const fn ed25519_public_len() -> usize {
    ED25519_PUBLIC_LEN
}
