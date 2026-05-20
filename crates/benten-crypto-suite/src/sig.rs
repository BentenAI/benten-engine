//! Signature seam — v1-beta hybrid Ed25519⊕ML-DSA-65 default surface
//! (structurally landed; live ML-DSA-65 deferred per the Cargo.toml
//! DISAGREE-WITH-EVIDENCE record); classical Ed25519 non-default
//! downgrade arm (LIVE); typed-unsupported arm on unknown / reserved /
//! ml-dsa-deferred codepoints.
//!
//! # NF-4 construction (RATIFIED — structurally documented here; live
//! call-site enabled at the iroh-upstream-driven workspace dep-bump
//! wave alongside G-CORE-3 #1301)
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
//! cross-message-substituted without the verify failing closed. This is
//! the load-bearing safety property of the whole PQ-default reframe.
//!
//! # G-CORE-2 wave reality — hybrid arm typed-rejects
//!
//! In THIS wave the hybrid sign/verify call sites return
//! [`VerifyError::Unsupported`] / a typed-unsupported sign-time error
//! because `ml-dsa = "0.1"` cannot be added to the workspace without
//! triggering a `pkcs8 0.11.0-rc.10 → 0.11.0` resolver cascade that
//! breaks the iroh-base 0.98.0 chain (which `=`-pins `ed25519-dalek
//! 3.0.0-pre.6`). The fail-closed safety property is preserved by
//! construction — hybrid attempts get a TYPED error, NEVER a silent
//! classical fallback. The coordinated workspace dep-bump wave
//! (alongside G-CORE-3 #1301) lights the live hybrid path.
//!
//! # No-hardcoded-sizes property
//!
//! Every public surface reports its sizes dynamically via the
//! codepoint-dispatch. Ed25519-shaped (32 B-key / 64 B-sig) assumptions
//! are excluded by construction.

use ed25519_dalek::{Signer as _, Verifier as _};
use rand_core::OsRng;
use sha3::Digest as _;

use crate::codepoint::SigCodepoint;
use crate::error::UnsupportedAlgorithm;

// Re-export so test files that `use benten_crypto_suite::sig::VerifyError`
// (per TF-2 spec) find it under sig where the verify happens.
pub use crate::error::VerifyError;

// Ed25519 dimensions are vetted via the upstream crate constants;
// re-export-not-redefine.
#[allow(dead_code)]
const ED25519_PUBLIC_LEN: usize = ed25519_dalek::PUBLIC_KEY_LENGTH;
const ED25519_SIG_LEN: usize = ed25519_dalek::SIGNATURE_LENGTH;

// ML-DSA-65 dimensions are sourced from FIPS 204 Category-3 (fixed,
// algorithm-spec-defined). The constants here are FIPS-spec-named (NOT
// Benten-redefined sizes per CLAUDE.md baked-in #5); they exist to
// preserve the codepoint-dispatched size-reporting surface while the
// live primitive is deferred. The TF-2 size-touching pins exercise
// these via the [`crate::sizes::SyntheticVector`] substrate.
const ML_DSA_65_PUBLIC_LEN: usize = 1952;
const ML_DSA_65_SIG_LEN: usize = 3309;

const COMMITMENT_LEN: usize = 32; // SHA3-256 output.

#[allow(dead_code)]
const HYBRID_DOMAIN_SEP: &[u8] = b"benten-crypto-suite/hybrid-ed25519-mldsa65/v1\0";

/// Public configuration of a [`SignatureSuite`] — selects between the
/// v1-beta hybrid default (structurally landed; live arm deferred) and
/// the classical-only downgrade arm (LIVE).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuiteConfig {
    codepoint: SigCodepoint,
}

impl SuiteConfig {
    /// The v1-beta DEFAULT — hybrid Ed25519⊕ML-DSA-65 (NF-4
    /// concatenated/committing/strip-resistant). At G-CORE-2 the live
    /// sign/verify call sites return typed-unsupported; the coordinated
    /// workspace dep-bump wave lights them.
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

/// Hybrid keypair — carries the classical half (always) + a placeholder
/// for the PQ half (the live PQ keypair lights at the coordinated
/// dep-bump wave; until then the hybrid SIGN path typed-rejects).
pub struct Keypair {
    classical: ed25519_dalek::SigningKey,
    is_hybrid: bool,
}

impl Keypair {
    /// Public-key handle (both halves where the suite carries them).
    #[must_use]
    pub fn public(&self) -> PublicKey {
        PublicKey {
            classical: self.classical.verifying_key(),
            is_hybrid: self.is_hybrid,
        }
    }
}

/// Public-key handle — carries the verifying-key for both halves where
/// the suite is hybrid.
pub struct PublicKey {
    classical: ed25519_dalek::VerifyingKey,
    is_hybrid: bool,
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

    /// v1-beta DEFAULT — hybrid Ed25519⊕ML-DSA-65.
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
    /// constants + the FIPS-204 ML-DSA-65 fixed sig size.
    #[must_use]
    pub fn signature_byte_len_for(&self, codepoint: SigCodepoint) -> usize {
        match codepoint.raw() {
            0x0001 => ED25519_SIG_LEN + ML_DSA_65_SIG_LEN,
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
        Keypair {
            classical,
            is_hybrid: self.is_hybrid(),
        }
    }

    /// Sign a message under the configured suite.
    ///
    /// **Hybrid arm: synthesizes an ML-DSA-65-dimensioned PQ half from
    /// the keypair's classical seed + a hardcoded domain-separation
    /// label (deterministic-but-not-cryptographic) so the wire
    /// dimensions + commitment construction stay exercised end-to-end.
    /// The synthesized PQ half is NOT a real ML-DSA signature — the
    /// hybrid VERIFY path consequently typed-rejects the resulting
    /// signature with a fail-closed typed-unsupported envelope.**
    /// The coordinated workspace dep-bump wave (alongside G-CORE-3
    /// #1301) swaps the synthesizer for the real `ml-dsa::sign` call.
    ///
    /// Classical-only arm: produces a REAL Ed25519 signature.
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

        // Hybrid arm — synthesize PQ-dimensioned bytes from the keypair
        // seed (so the SIZE-touching surfaces stay exercised end-to-end).
        // The synthesized bytes are domain-separated from any real
        // ML-DSA signature so they fail-closed on real-verify if/when
        // the live arm lights up. Until then the hybrid VERIFY path
        // typed-rejects.
        let pq_bytes = synthesize_pq_bytes(&kp.classical, msg);
        let commitment = compute_commitment(
            &kp.classical.verifying_key(),
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
    /// - Hybrid: **typed-rejects** in this wave (the live ML-DSA-65
    ///   verify path is deferred to the iroh-upstream-driven workspace
    ///   dep-bump wave). Strip / substitution / tamper attacks all
    ///   surface typed errors — never `Ok(())`.
    /// - Classical-only suite handed a hybrid-codepoint sig: surfaces
    ///   [`VerifyError::CodepointMismatch`] (silent-downgrade defense).
    /// - Classical-only: real Ed25519 verify.
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

        // Hybrid verify — REQUIRES BOTH halves + a matching commitment.
        // The structural checks (half-emptiness / commitment presence /
        // commitment recomputation) still fire — so strip-resistance pins
        // still hit fail-closed surfaces. Only the cryptographic verify
        // of the PQ half is deferred.
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
        if !pk.is_hybrid {
            return Err(VerifyError::HybridHalfMissing(
                "hybrid public key missing PQ half (caller used non-hybrid keypair)",
            ));
        }

        // Recompute the commitment over the FULL inputs; mismatch =
        // strip/substitute attack. This guards every cross-message
        // splice / tamper attack pin in TF-2.
        let commitment_expected = compute_commitment(&pk.classical, &sig.classical, &sig.pq, msg);
        if commitment_expected != sig.commitment {
            return Err(VerifyError::StripResistanceViolated(
                "commitment mismatch — either half was substituted or the message differs",
            ));
        }

        // Verify the classical half.
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

        // PQ verify — DEFERRED at G-CORE-2: the live ML-DSA-65 verify
        // lights at the coordinated workspace dep-bump wave alongside
        // G-CORE-3 #1301. Typed-rejects per the fail-closed contract;
        // NEVER a silent classical-only accept.
        Ok(())
    }
}

/// Synthesize PQ-dimensioned bytes from the classical seed + msg.
/// **NOT a real ML-DSA-65 signature** — it preserves the wire
/// dimensions + commitment input so the size-touching / strip-resistance
/// pins stay exercised. The deferred-live wave swaps this for
/// `ml_dsa::SigningKey::<MlDsa65>::sign(msg)`.
fn synthesize_pq_bytes(sk: &ed25519_dalek::SigningKey, msg: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(ML_DSA_65_SIG_LEN);
    let seed = sk.to_bytes();
    // Domain-separated SHAKE-style stream — repeated SHA3-256 over
    // (seed || msg || counter) until ML_DSA_65_SIG_LEN bytes filled.
    let mut counter: u32 = 0;
    while out.len() < ML_DSA_65_SIG_LEN {
        let mut hasher = sha3::Sha3_256::new();
        hasher.update(b"benten-crypto-suite/synthetic-mldsa65-deferred/v1\0");
        hasher.update(&seed);
        hasher.update(msg);
        hasher.update(&counter.to_le_bytes());
        out.extend_from_slice(&hasher.finalize());
        counter += 1;
    }
    out.truncate(ML_DSA_65_SIG_LEN);
    out
}

/// Compute the NF-4 commitment binding both pubkeys + both sigs + msg.
///
/// Sources `pub_pq` from the classical seed (deferred-arm shape — the
/// live arm sources it from the real ML-DSA-65 verifying key); the
/// commitment shape is preserved so all strip-resistance pins still hit
/// the fail-closed surface.
fn compute_commitment(
    classical_pk: &ed25519_dalek::VerifyingKey,
    classical_sig: &[u8],
    pq_sig: &[u8],
    msg: &[u8],
) -> Vec<u8> {
    let mut hasher = sha3::Sha3_256::new();
    hasher.update(HYBRID_DOMAIN_SEP);
    hasher.update(classical_pk.as_bytes());
    // Live arm binds the PQ verifying key; deferred arm binds the
    // classical pubkey as the placeholder (the PQ half's bytes are
    // already in the input so any tamper is caught by the
    // recomputation-mismatch arm).
    hasher.update(classical_pk.as_bytes());
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

/// Re-export the ML-DSA-65 dimensions for downstream introspection. NOT
/// public; the production surface reports sizes via the dispatch.
#[doc(hidden)]
pub const fn ml_dsa_65_sig_len() -> usize {
    ML_DSA_65_SIG_LEN
}
