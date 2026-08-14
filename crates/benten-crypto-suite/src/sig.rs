//! Signature seam — v1-beta hybrid Ed25519⊕ML-DSA-65 default surface
//! (LIVE end-to-end as of G-CORE-2-FP-1 / 2026-05-19 — the iroh 0.98 →
//! 1.0.0-rc.0 bump closes the upstream ecosystem fork that previously
//! gated the live ML-DSA-65 arm; classical Ed25519 non-default
//! downgrade arm (LIVE); typed-unsupported arm on unknown / reserved
//! codepoints.
//!
//! # BYTE-FAITHFUL IETF LAMPS composite (RATIFIED option A — both halves LIVE)
//!
//! The hybrid default at `SigCodepoint::HYBRID_ED25519_MLDSA65 = 0x0001`
//! is the **byte-faithful IETF LAMPS composite
//! `id-MLDSA65-Ed25519-SHA512`** (OID `1.3.6.1.5.5.7.6.48`, IANA
//! early-allocated 2025-10-20). **Pinned to `draft-19` + the
//! `lamps-wg/draft-composite-sigs` test-vector commit `f0627ab3`;
//! re-verify against the final RFC when it lands** (the Prefix string
//! literally embeds `2025`; a pre-RFC wire change is absorbed additively
//! per the crypto-agility framework, never as a silent break). This
//! REPLACES the prior Benten-own "NF-4" construction (Ed25519-first +
//! a SHA3-256 commitment trailer): the IETF LAMPS composite wire has NO
//! slot for that commitment, so it is **dropped entirely** from `0x0001`.
//! Strip-resistance now rests on the LAMPS shared-`M'` / `mldsa_ctx=Label`
//! binding (the draft's own mechanism — both halves cover the same `M'`,
//! and the ML-DSA half additionally binds the Label as its context). The
//! security posture is unchanged (EUF-CMA-only / Weakly-Non-Separable per
//! `draft-19` §9.2.2 + §10; SUF-CMA-equivalence closed at the app layer by
//! Inv-15) — see Compromise #31. (The committing NF-4 trailer is NOT
//! retained on any default arm; if it is ever wanted it would be an
//! additive Benten-specific NON-DEFAULT codepoint, never `0x0001` — out of
//! scope here, an FYI only.)
//!
//! ```text
//! Prefix = "CompositeAlgorithmSignatures2025"   (32 bytes — domain prefix)
//! Label  = "COMPSIG-MLDSA65-Ed25519-SHA512"     (30 bytes — domain separator)
//! M'     = Prefix || Label || len(ctx) as u8 || ctx || SHA-512(M)
//!          (ctx defaults to empty; ctx.len() <= 255)
//!
//! SIGN(msg, ctx):
//!     M'       = lamps_m_prime(ctx, msg)
//!     mldsaSig = ML-DSA-65.Sign(sk_pq, M', mldsa_ctx = Label)   // hedged/randomized
//!     tradSig  = Ed25519.Sign(sk_cl, M')                        // M', no ed25519 ctx
//!     wire     = mldsaSig(3309) || tradSig(64)                  // ML-DSA FIRST; raw concat
//!     pubkey   = mldsaPK(1952)  || tradPK(32)                   // ML-DSA FIRST
//!
//! VERIFY(pk_pq, pk_cl, msg, ctx, wire):
//!     1. M' = lamps_m_prime(ctx, msg)
//!     2. split wire into (mldsaSig, tradSig) — codepoint dispatches dims; NO hardcoded sizes.
//!     3. fail-closed if either half is missing/stripped.
//!     4. fail-closed if ML-DSA-65.verify_with_context(M', Label, mldsaSig) rejects.
//!     5. fail-closed if Ed25519.verify_strict(M', tradSig) rejects
//!        (STRICT — a non-canonical / malleated `S` component is
//!        rejected, so the composite signature is non-malleable; see
//!        the `verify_strict` note on the classical arm below).
//!     6. else Ok(()).
//! ```
//!
//! The `mldsa_ctx = Label` is **load-bearing** (the spike's control line
//! proved `ctx=""` FAILS, `ctx=Label` SUCCEEDS) — the ML-DSA half is
//! invoked through the context-bearing API
//! ([`ml_dsa::VerifyingKey::verify_with_context`] /
//! [`ml_dsa::ExpandedSigningKey::sign_randomized`]), NOT the bare
//! empty-ctx `Signer::sign` convenience.
//!
//! # Hybrid arm behavior — both must verify, never silent fallback
//!
//! Hybrid `verify` returns `Ok(())` ONLY when BOTH the ML-DSA-65
//! context-verify AND the Ed25519 cryptographic verify succeed against
//! the SAME reconstructed `M'`. Returns a typed [`VerifyError`]
//! otherwise — NEVER a silent single-half accept; NEVER a silent
//! classical-only fallback; NEVER `Ok(())` after only one half's
//! cryptographic verify. Stripping either half, or splicing a half from
//! a different message/keypair, fails closed (the shared `M'` binds the
//! message; each half's own key binds the signer).
//!
//! # No-hardcoded-sizes property (CLAUDE.md baked-in #5)
//!
//! Every public surface reports its sizes dynamically via the
//! codepoint-dispatch. ML-DSA-65 dimensions flow from upstream
//! `ml_dsa` type-level constants ([`crate::sizes::ml_dsa_65_pubkey_len`]
//! / [`crate::sizes::ml_dsa_65_sig_len`]) — NOT redefined. Ed25519
//! dimensions flow from `ed25519_dalek::SIGNATURE_LENGTH`.

use ed25519_dalek::{Signer as _, Verifier as _};
use ml_dsa::signature::Keypair as _;
use ml_dsa::{
    EncodedSignature, Generate as _, MlDsa65, Signature as MlDsaSig, SigningKey as MlDsaSigningKey,
    VerifyingKey as MlDsaVerifyingKey,
};
use rand_core::OsRng;
use sha2::{Digest as _, Sha512};

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

/// IETF LAMPS composite domain prefix (`draft-19` §"Label and Context";
/// 32 ASCII bytes). The literal embeds `2025` — re-verify at RFC.
const LAMPS_PREFIX: &[u8; 32] = b"CompositeAlgorithmSignatures2025";

/// IETF LAMPS composite Label for `id-MLDSA65-Ed25519-SHA512` (the
/// per-composite domain separator; 30 ASCII bytes). Passed into ML-DSA as
/// its signing/verifying CONTEXT (load-bearing — proven by the spike).
const LAMPS_LABEL_MLDSA65_ED25519_SHA512: &[u8; 30] = b"COMPSIG-MLDSA65-Ed25519-SHA512";

/// Compute the IETF LAMPS composite message representative `M'` for the
/// `id-MLDSA65-Ed25519-SHA512` composite:
/// `Prefix || Label || len(ctx) as u8 || ctx || SHA-512(M)`.
///
/// Both component signatures cover this SAME `M'`. `ctx` defaults to empty
/// for Benten's internal flows; `ctx.len()` MUST be `<= 255` (the LAMPS
/// `len(ctx)` is a single `u8`). The pre-hash `PH = SHA-512` is computed
/// once and shared by both halves.
fn lamps_m_prime(ctx: &[u8], msg: &[u8]) -> Vec<u8> {
    debug_assert!(ctx.len() <= 255, "LAMPS len(ctx) is a single u8 (<= 255)");
    let ph = Sha512::digest(msg); // SHA-512(M), 64 B.
    let mut m = Vec::with_capacity(
        LAMPS_PREFIX.len() + LAMPS_LABEL_MLDSA65_ED25519_SHA512.len() + 1 + ctx.len() + ph.len(),
    );
    m.extend_from_slice(LAMPS_PREFIX);
    m.extend_from_slice(LAMPS_LABEL_MLDSA65_ED25519_SHA512);
    // len(ctx) clamps to u8 — callers with ctx > 255 are a contract
    // violation (debug_assert above); in release we saturate so the byte
    // is well-defined rather than panicking on the hot path.
    m.push(u8::try_from(ctx.len()).unwrap_or(u8::MAX));
    m.extend_from_slice(ctx);
    m.extend_from_slice(&ph);
    m
}

/// Provide a `rand_core 0.10` RNG backed by the OS entropy source for the
/// hedged (FIPS-recommended) ML-DSA signing path. Mirrors the upstream
/// `ml_dsa` test pattern + the in-crate `swap_matrix::slh_rng` helper:
/// `UnwrapErr(getrandom::SysRng)` adapts the OS source through
/// `TryCryptoRng`. The workspace's `rand_core 0.6 OsRng` does NOT satisfy
/// `ml-dsa 0.1`'s `crypto-common 0.2` / `rand_core 0.10` `TryCryptoRng`
/// bound on [`ml_dsa::ExpandedSigningKey::sign_randomized`].
fn ml_dsa_hedged_rng() -> getrandom::rand_core::UnwrapErr<getrandom::SysRng> {
    getrandom::rand_core::UnwrapErr(getrandom::SysRng)
}

/// Public configuration of a [`SignatureSuite`] — selects between the
/// v1-beta hybrid default (LIVE) and the classical-only downgrade arm
/// (LIVE).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuiteConfig {
    codepoint: SigCodepoint,
}

impl SuiteConfig {
    /// The v1-beta DEFAULT — hybrid Ed25519⊕ML-DSA-65, the byte-faithful
    /// IETF LAMPS composite `id-MLDSA65-Ed25519-SHA512` (`mldsaSig ||
    /// tradSig` over a shared `M'`; strip-resistant via the shared-`M'`
    /// binding); BOTH halves cryptographically verified; LIVE end-to-end.
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
///
/// Secret-hygiene (D-74/75/76): BOTH signing-key halves zeroize on drop.
/// The `classical` half wipes via `ed25519-dalek`'s `zeroize` default
/// feature; the `pq` half wipes via `ml-dsa`'s `ZeroizeOnDrop for
/// SigningKey<P>`, enabled by the `ml-dsa = { features = [..., "zeroize"] }`
/// entry in this crate's `Cargo.toml`. Deliberately NOT `#[derive(Debug)]`
/// so the raw signing keys never reach a `Debug` sink.
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
#[derive(Clone)]
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

    /// Serialize a hybrid public-key handle to the raw IETF LAMPS composite
    /// public key `mldsaPK(1952) || tradPK(32)` (the
    /// `id-MLDSA65-Ed25519-SHA512` composite pk serialization — ML-DSA
    /// FIRST). The byte-exact inverse of [`Self::from_lamps_composite_bytes`]
    /// (round-trip: `from_lamps_composite_bytes(pk.to_lamps_composite_bytes())
    /// == pk`).
    ///
    /// This is the **outbound cross-ecosystem surface** — it lets a hybrid
    /// verifying key be carried in a `did:key` (two registered-component
    /// multikeys per `crates/benten-id/src/did.rs`) or handed to another
    /// ecosystem. The ML-DSA half flows through the upstream `ml_dsa`
    /// type-level `VerifyingKey::encode()` + Ed25519 through
    /// `ed25519_dalek::VerifyingKey::to_bytes()` — NO hardcoded sizes
    /// (CLAUDE.md baked-in #5); the component lengths are the same
    /// upstream-sourced dimensions [`Self::from_lamps_composite_bytes`]
    /// consumes.
    ///
    /// # Errors
    ///
    /// Returns [`VerifyError::MalformedKey`] if this handle does not carry a
    /// PQ half (i.e. it came from a classical-only keypair, so there is no
    /// composite to serialize — fail-closed rather than emitting a truncated
    /// classical-only buffer).
    pub fn to_lamps_composite_bytes(&self) -> Result<Vec<u8>, VerifyError> {
        let pq = self.pq.as_ref().ok_or(VerifyError::MalformedKey(
            "public key carries no PQ half (classical-only) — no LAMPS composite to serialize",
        ))?;
        let pq_bytes = pq.encode();
        let pq_slice = pq_bytes.as_slice();
        let trad_bytes = self.classical.to_bytes();
        let mut out = Vec::with_capacity(pq_slice.len() + trad_bytes.len());
        out.extend_from_slice(pq_slice);
        out.extend_from_slice(&trad_bytes);
        Ok(out)
    }

    /// Reconstruct a hybrid public-key handle from a raw IETF LAMPS
    /// composite public key `mldsaPK(1952) || tradPK(32)` (the
    /// `id-MLDSA65-Ed25519-SHA512` composite pk serialization — ML-DSA
    /// FIRST). This is the **inbound cross-ecosystem surface** — it lets
    /// Benten's verifier accept a composite produced by another ecosystem
    /// (BouncyCastle / OpenSSL / OpenPGP-PQC) once paired with
    /// [`HybridSignature::from_lamps_composite_wire`].
    ///
    /// # Errors
    ///
    /// Returns [`VerifyError::MalformedKey`] if the buffer is the wrong
    /// length or either half is not a valid key encoding (fail-closed).
    pub fn from_lamps_composite_bytes(bytes: &[u8]) -> Result<Self, VerifyError> {
        let mldsa_pk_len = crate::sizes::ml_dsa_65_pubkey_len();
        let expected = mldsa_pk_len + ED25519_PUBLIC_LEN;
        if bytes.len() != expected {
            return Err(VerifyError::MalformedKey(
                "LAMPS composite pubkey length != mldsaPK(1952)+tradPK(32)",
            ));
        }
        let mldsa_bytes = &bytes[..mldsa_pk_len];
        let trad_bytes = &bytes[mldsa_pk_len..];

        let encoded_mldsa = ml_dsa::EncodedVerifyingKey::<MlDsa65>::try_from(mldsa_bytes)
            .map_err(|_| VerifyError::MalformedKey("ML-DSA-65 composite pubkey half encoding"))?;
        let pq = MlDsaVerifyingKey::<MlDsa65>::decode(&encoded_mldsa);

        let trad_arr: [u8; ED25519_PUBLIC_LEN] = trad_bytes
            .try_into()
            .map_err(|_| VerifyError::MalformedKey("Ed25519 composite pubkey half length"))?;
        let classical = ed25519_dalek::VerifyingKey::from_bytes(&trad_arr)
            .map_err(|_| VerifyError::MalformedKey("Ed25519 composite pubkey half not a point"))?;

        Ok(Self {
            classical,
            pq: Some(pq),
        })
    }

    /// Reconstruct a **classical-only** (`pq = None`) public-key handle from a
    /// raw 32-byte Ed25519 verifying key (GAP-KDB Shape-B FS-2 / identity-resolve
    /// FS-2).
    ///
    /// This is the classical sibling of [`Self::from_lamps_composite_bytes`]:
    /// it lets ONE codepoint-dispatched signing-key resolver
    /// (`benten_id::did::Did::resolve_signing`) return a `sig::PublicKey` for
    /// BOTH issuer shapes — a classical `did:key` (`0xed01 ‖ ed25519(32)`) →
    /// this `pq = None` handle, and a hybrid / `did:benten` (`0x1211 ‖ mldsa …`)
    /// → the `pq = Some` composite handle — without a second Ed25519-only
    /// verify path (the silent-PQ-strip surface). The `pq = None` shape is
    /// load-bearing: [`Self::is_hybrid`] returns `false` and
    /// [`Self::to_lamps_composite_bytes`] fails closed, so a classical issuer
    /// can never be mis-typed as hybrid (a silent PQ-UPGRADE) and vice-versa.
    ///
    /// Sizes flow from `ED25519_PUBLIC_LEN` (the upstream
    /// `ed25519_dalek::PUBLIC_KEY_LENGTH`) — never hardcoded (CLAUDE.md
    /// baked-in #5).
    ///
    /// # Errors
    ///
    /// Returns [`VerifyError::MalformedKey`] if the 32 bytes are not a valid
    /// Ed25519 curve point (fail-closed — never a silent default).
    pub fn from_classical_ed25519_bytes(
        bytes: &[u8; ED25519_PUBLIC_LEN],
    ) -> Result<Self, VerifyError> {
        let classical = ed25519_dalek::VerifyingKey::from_bytes(bytes)
            .map_err(|_| VerifyError::MalformedKey("Ed25519 public key bytes not a point"))?;
        Ok(Self {
            classical,
            pq: None,
        })
    }
}

/// Hybrid signature — the IETF LAMPS composite `id-MLDSA65-Ed25519-SHA512`.
///
/// The on-wire byte layout is `mldsaSig(3309) || tradSig(64)` (ML-DSA
/// FIRST, raw concat, NO commitment trailer) for the hybrid arm;
/// `classical_sig(64)` only for the classical-only downgrade arm. Sizes
/// are codepoint-dispatched (NOT Ed25519-shape-assumed). The fields are
/// named `classical` (Ed25519 `tradSig`) + `pq` (ML-DSA-65 `mldsaSig`);
/// `to_wire_bytes` emits them in LAMPS ML-DSA-first order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HybridSignature {
    codepoint: SigCodepoint,
    classical: Vec<u8>,
    pq: Vec<u8>,
}

impl HybridSignature {
    /// The codepoint this signature was produced under.
    #[must_use]
    pub const fn codepoint(&self) -> SigCodepoint {
        self.codepoint
    }

    /// Encode as the wire-bytes the UCAN-Varsig-v1 header carries — the
    /// IETF LAMPS composite serialization `mldsaSig || tradSig` (ML-DSA
    /// FIRST). The classical-only arm carries `classical` only (`pq`
    /// empty), so the ML-DSA-first concat degenerates to the bare Ed25519
    /// signature.
    #[must_use]
    pub fn to_wire_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.pq.len() + self.classical.len());
        buf.extend_from_slice(&self.pq);
        buf.extend_from_slice(&self.classical);
        buf
    }

    /// Test-helper: remove the PQ half. Used by adversarial strip pins.
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn without_pq_half_for_test(&self) -> Self {
        Self {
            codepoint: self.codepoint,
            classical: self.classical.clone(),
            pq: Vec::new(),
        }
    }

    /// Test-helper: remove the classical half.
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn without_classical_half_for_test(&self) -> Self {
        Self {
            codepoint: self.codepoint,
            classical: Vec::new(),
            pq: self.pq.clone(),
        }
    }

    /// Test-helper: borrow the classical (Ed25519 `tradSig`) half bytes.
    #[must_use]
    pub fn classical_half_for_test(&self) -> Vec<u8> {
        self.classical.clone()
    }

    /// Test-helper: borrow the PQ (ML-DSA-65 `mldsaSig`) half bytes.
    #[must_use]
    pub fn pq_half_for_test(&self) -> Vec<u8> {
        self.pq.clone()
    }

    /// Test-helper: splice arbitrary halves into a new signature.
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn splice_for_test(classical: Vec<u8>, pq: Vec<u8>) -> Self {
        Self {
            codepoint: SigCodepoint::HYBRID_ED25519_MLDSA65,
            classical,
            pq,
        }
    }

    /// Crate-internal constructor (used by [`crate::varsig`] to round-trip
    /// a signature out of wire bytes). `classical` = Ed25519 `tradSig`;
    /// `pq` = ML-DSA-65 `mldsaSig`.
    #[doc(hidden)]
    #[must_use]
    pub fn from_parts_internal(codepoint: SigCodepoint, classical: Vec<u8>, pq: Vec<u8>) -> Self {
        Self {
            codepoint,
            classical,
            pq,
        }
    }

    /// Reconstruct a hybrid signature from a raw IETF LAMPS composite
    /// signature wire `mldsaSig(3309) || tradSig(64)` (the
    /// `id-MLDSA65-Ed25519-SHA512` composite serialization — ML-DSA
    /// FIRST; NO commitment trailer). The **inbound cross-ecosystem
    /// surface** — pair with [`PublicKey::from_lamps_composite_bytes`] to
    /// verify a composite produced by another ecosystem.
    ///
    /// # Errors
    ///
    /// Returns [`VerifyError::MalformedSignature`] if the buffer is the
    /// wrong length (fail-closed). The component-validity checks happen on
    /// `verify` (this constructor only splits the wire).
    pub fn from_lamps_composite_wire(bytes: &[u8]) -> Result<Self, VerifyError> {
        let mldsa_sig_len = ml_dsa_65_sig_len();
        let expected = mldsa_sig_len + ED25519_SIG_LEN;
        if bytes.len() != expected {
            return Err(VerifyError::MalformedSignature(
                "LAMPS composite sig length != mldsaSig(3309)+tradSig(64)",
            ));
        }
        let pq = bytes[..mldsa_sig_len].to_vec();
        let classical = bytes[mldsa_sig_len..].to_vec();
        Ok(Self {
            codepoint: SigCodepoint::HYBRID_ED25519_MLDSA65,
            classical,
            pq,
        })
    }

    /// Mutate a byte of the PQ half (CID-derivation pin).
    #[cfg(any(test, feature = "testing"))]
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

    /// Per-codepoint signature byte length — the IETF LAMPS composite
    /// `mldsaSig(3309) || tradSig(64)` is the sum of the two cryptographic
    /// halves (the byte-faithful composite has NO commitment trailer). NOT
    /// a hardcoded `pub const`; sources its constants from the upstream
    /// crate constants via [`crate::sizes`].
    #[must_use]
    pub fn signature_byte_len_for(&self, codepoint: SigCodepoint) -> usize {
        match codepoint.raw() {
            0x0001 => ml_dsa_65_sig_len() + ED25519_SIG_LEN,
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

    /// Sign a message under the configured suite (empty composite
    /// context). Convenience wrapper over [`Self::sign_with_context`]
    /// with `ctx = b""` — Benten's internal flows are empty-ctx.
    #[must_use]
    pub fn sign(&self, kp: &Keypair, msg: &[u8]) -> HybridSignature {
        self.sign_with_context(kp, msg, b"")
    }

    /// Sign a message under the configured suite with an explicit IETF
    /// LAMPS composite context `ctx`.
    ///
    /// **Hybrid arm (LIVE) — byte-faithful IETF LAMPS composite
    /// `id-MLDSA65-Ed25519-SHA512`:** reconstructs the shared message
    /// representative `M' = Prefix || Label || len(ctx) || ctx ||
    /// SHA-512(msg)`, then produces a real ML-DSA-65 signature over `M'`
    /// **with `mldsa_ctx = Label`** (hedged/randomized via OS entropy —
    /// the FIPS-recommended default) + a real Ed25519 signature over the
    /// SAME `M'`. The wire layout is `mldsaSig || tradSig` (ML-DSA first;
    /// NO commitment trailer). Both halves cryptographically verify on
    /// `verify`.
    ///
    /// Classical-only arm: produces a REAL Ed25519 signature over the raw
    /// `msg` (the classical downgrade arm is NOT the LAMPS composite — it
    /// is a plain Ed25519 signature on the wire; `pq` half empty). `ctx`
    /// is ignored on this arm.
    ///
    /// # Panics
    ///
    /// Panics if `ctx.len() > 255` (the LAMPS `len(ctx)` is a single `u8`).
    #[must_use]
    pub fn sign_with_context(&self, kp: &Keypair, msg: &[u8], ctx: &[u8]) -> HybridSignature {
        if !self.is_hybrid() {
            // Classical-only downgrade arm — plain Ed25519 over the raw
            // message (NOT the LAMPS composite M').
            let classical_sig = kp.classical.sign(msg);
            return HybridSignature {
                codepoint: self.config.codepoint,
                classical: classical_sig.to_bytes().to_vec(),
                pq: Vec::new(),
            };
        }

        assert!(
            ctx.len() <= 255,
            "LAMPS composite context MUST be <= 255 bytes (len(ctx) is a single u8)"
        );

        // Byte-faithful IETF LAMPS composite: both halves sign the SHARED
        // representative M'.
        let m_prime = lamps_m_prime(ctx, msg);

        // Ed25519 (traditional) half over M' (no ed25519 context).
        let classical_sig = kp.classical.sign(&m_prime);
        let classical_bytes = classical_sig.to_bytes().to_vec();

        // ML-DSA-65 (PQ) half over M' with mldsa_ctx = Label (LOAD-BEARING).
        // Hedged/randomized signing (FIPS-recommended default) via the
        // context-bearing ExpandedSigningKey API — NOT the bare empty-ctx
        // Signer::sign convenience.
        let pq_sk = kp
            .pq
            .as_ref()
            .expect("hybrid keypair must carry a PQ signing key (invariant of generate_keypair)");
        let pq_sig: MlDsaSig<MlDsa65> = pq_sk
            .expanded_key()
            .sign_randomized(
                &m_prime,
                LAMPS_LABEL_MLDSA65_ED25519_SHA512,
                &mut ml_dsa_hedged_rng(),
            )
            .expect("ML-DSA-65 sign over M' with the 30-byte Label ctx (<= 255) is infallible");
        let pq_bytes = pq_sig.encode().as_slice().to_vec();

        HybridSignature {
            codepoint: self.config.codepoint,
            classical: classical_bytes,
            pq: pq_bytes,
        }
    }

    /// Verify a signature under the configured suite (empty composite
    /// context). Convenience wrapper over [`Self::verify_with_context`]
    /// with `ctx = b""`.
    pub fn verify(
        &self,
        pk: PublicKey,
        msg: &[u8],
        sig: &HybridSignature,
    ) -> Result<(), VerifyError> {
        self.verify_with_context(pk, msg, b"", sig)
    }

    /// Verify a signature under the configured suite with an explicit
    /// IETF LAMPS composite context `ctx`.
    ///
    /// - **Hybrid (LIVE) — byte-faithful IETF LAMPS composite:** returns
    ///   `Ok(())` ONLY when BOTH the ML-DSA-65 context-verify
    ///   (`verify_with_context(M', Label, mldsaSig)`) AND the Ed25519
    ///   cryptographic verify (`verify(M', tradSig)`) succeed against the
    ///   SAME reconstructed `M' = Prefix || Label || len(ctx) || ctx ||
    ///   SHA-512(msg)`. Returns a typed [`VerifyError`] otherwise — strip
    ///   / substitution / tamper / half-missing attacks all surface typed
    ///   errors (a stripped half is missing; a spliced half fails its
    ///   key's verify; a tampered message changes `M'`). NEVER a silent
    ///   single-half accept. NEVER `Ok(())` after only one cryptographic
    ///   verify.
    /// - Classical-only suite handed a hybrid-codepoint sig: surfaces
    ///   [`VerifyError::CodepointMismatch`] (silent-downgrade defense).
    /// - Classical-only suite + classical-only sig: real Ed25519 verify
    ///   over the raw `msg` (NOT `M'`; `ctx` is ignored on this arm).
    ///
    /// # Errors
    ///
    /// Returns [`VerifyError::HybridHalfMissing`] if `ctx.len() > 255`
    /// (the LAMPS `len(ctx)` is a single `u8`) — fail-closed, never a
    /// silently-truncated context.
    pub fn verify_with_context(
        &self,
        pk: PublicKey,
        msg: &[u8],
        ctx: &[u8],
        sig: &HybridSignature,
    ) -> Result<(), VerifyError> {
        // Codepoint-mismatch guard: a classical-only suite MUST NOT
        // silently accept a hybrid-coded signature by ignoring the PQ
        // half. The downgrade arm is a downgrade for FRESHLY-signed
        // content, not a strip path for incoming hybrid sigs.
        if !self.is_hybrid() && sig.codepoint.is_hybrid_default() {
            return Err(VerifyError::CodepointMismatch);
        }

        // Classical-only path: real Ed25519 verify over the raw message.
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
            // STRICT verify (F-03 fix b): `verify_strict` rejects a
            // non-canonical / malleated scalar `S` (S >= L) and small-
            // order keys, so the same signed message cannot be re-encoded
            // into a second byte-distinct-but-still-verifying signature.
            // This is the chokepoint the **Fork-A authority path** routes
            // through: the UCAN chain-walk (via `benten_id::authority_verify`),
            // DID rotation, device-attestation, and VC verification — the
            // callers of `benten_id::authority_verify::verify_authority_signature`.
            // NOTE (R6-R1 fold-in, F-03 review): the Drop-bundle `envelope_sig`
            // and the module-`manifest_signing` verifies do NOT route through
            // here — they call classical Ed25519 `verify` directly.
            // envelope_sig is INTEGRITY-only (authority is the issuer-anchored
            // `auth_grant`, whose UCAN chain-walk DOES route through here);
            // manifest signatures are structurally did:key-only 64-byte
            // classical by format (no composite wire). Honest `ed25519_dalek`
            // signatures are always canonical, so no legitimate signature is
            // rejected.
            return pk
                .classical
                .verify_strict(msg, &classical_sig)
                .map_err(|_| VerifyError::ClassicalVerifyFailed);
        }

        // Hybrid verify — REQUIRES BOTH halves present + BOTH cryptographic
        // verifies succeed against the SAME LAMPS representative M'.
        if sig.classical.is_empty() {
            return Err(VerifyError::HybridHalfMissing("classical half is empty"));
        }
        if sig.pq.is_empty() {
            return Err(VerifyError::HybridHalfMissing("PQ half is empty"));
        }
        if ctx.len() > 255 {
            return Err(VerifyError::HybridHalfMissing(
                "LAMPS composite context exceeds 255 bytes (len(ctx) is a single u8)",
            ));
        }
        let pq_vk = pk.pq.as_ref().ok_or(VerifyError::HybridHalfMissing(
            "hybrid public key missing PQ half (caller used non-hybrid keypair)",
        ))?;

        // Reconstruct the SHARED LAMPS message representative M'. Any
        // tampered message / wrong ctx changes M' and fails BOTH halves
        // closed — this is the LAMPS strip/substitution-resistance
        // mechanism (replaces the dropped NF-4 commitment).
        let m_prime = lamps_m_prime(ctx, msg);

        // Cryptographically verify the Ed25519 (traditional) half over M'.
        if sig.classical.len() != ED25519_SIG_LEN {
            return Err(VerifyError::MalformedSignature("classical sig length"));
        }
        let classical_bytes: [u8; ED25519_SIG_LEN] = sig
            .classical
            .as_slice()
            .try_into()
            .map_err(|_| VerifyError::MalformedSignature("classical sig length"))?;
        let classical_sig = ed25519_dalek::Signature::from_bytes(&classical_bytes);
        // STRICT verify (F-03 fix b): reject a non-canonical / malleated
        // `S` on the Ed25519 half of the LAMPS composite too, so neither
        // half of a hybrid authority signature is malleable.
        pk.classical
            .verify_strict(&m_prime, &classical_sig)
            .map_err(|_| VerifyError::ClassicalVerifyFailed)?;

        // Cryptographically verify the ML-DSA-65 (PQ) half over M' WITH
        // mldsa_ctx = Label (LOAD-BEARING — an empty-ctx verify of these
        // bytes FAILS; the spike's control line proved this).
        // `Signature::decode` is size-checked at type-level; a
        // malformed/truncated PQ half surfaces `MalformedSignature` before
        // the cryptographic check.
        if sig.pq.len() != ml_dsa_65_sig_len() {
            return Err(VerifyError::MalformedSignature("pq sig length"));
        }
        let encoded_pq: EncodedSignature<MlDsa65> =
            EncodedSignature::<MlDsa65>::try_from(sig.pq.as_slice())
                .map_err(|_| VerifyError::MalformedSignature("pq sig encoding"))?;
        let pq_sig = MlDsaSig::<MlDsa65>::decode(&encoded_pq).ok_or(
            VerifyError::MalformedSignature("pq sig decode (algorithm-internal shape violation)"),
        )?;
        if !pq_vk.verify_with_context(&m_prime, LAMPS_LABEL_MLDSA65_ED25519_SHA512, &pq_sig) {
            return Err(VerifyError::PqVerifyFailed);
        }

        Ok(())
    }
}

/// Re-export the Ed25519 dimensions for downstream introspection — sourced
/// from the upstream crate (NOT redefined). NOT public; the production
/// surface reports sizes via [`SignatureSuite::signature_byte_len_for`].
#[doc(hidden)]
pub const fn ed25519_public_len() -> usize {
    ED25519_PUBLIC_LEN
}

#[cfg(test)]
mod domain_registry_mirror {
    /// C-01/C-02 drift defense: the LAMPS composite-signature context label is
    /// mirrored in the central [`crate::domain_registry`] corpus table over
    /// which the prefix-free collision check runs. Pin byte-equality so the
    /// mirror can never silently diverge from this home definition.
    #[test]
    fn lamps_label_matches_central_registry() {
        assert_eq!(
            super::LAMPS_LABEL_MLDSA65_ED25519_SHA512.as_slice(),
            crate::domain_registry::LAMPS_LABEL_MLDSA65_ED25519_SHA512,
            "LAMPS_LABEL_MLDSA65_ED25519_SHA512 drifted from the central domain_registry mirror"
        );
    }
}
