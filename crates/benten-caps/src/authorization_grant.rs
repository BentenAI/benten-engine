//! `AuthorizationGrant` — ONE signed artifact `{ucan, key_material,
//! binding_sig}` per RATIFIED-S&C 2026-05-21 §R3.
//!
//! ## The R3 contract
//!
//! Per the post-spike ratification:
//!
//! > "ONE signed artifact `{ucan, key_material, binding_sig}` where
//! > `binding_sig` is the issuer's signature over the CBOR-encoded
//! > `(ucan, key_material)` tuple bound to the same audience. Validators
//! > check `binding_sig` BEFORE consulting the UCAN scope or the key
//! > material — the binding is the foundation."
//!
//! The binding's "load-bearing third leg" is the **audience**:
//! `binding_sig` covers `(canonical_bytes(ucan) ||
//! canonical_bytes(key_material) || canonical_bytes(audience))` so a
//! grant cannot be repointed at a different audience without
//! re-signing (the wrong-audience-swap attack defense, P-3 / A-3).
//!
//! ## Adversary outcomes covered
//!
//! - **A-1 stolen-UCAN-without-keys.** Attacker swaps `key_material`;
//!   `binding_sig` no longer verifies →
//!   [`AuthorizationGrantError::BindingMismatch`].
//! - **A-2 stolen-keys-without-UCAN.** Attacker swaps `ucan`;
//!   `binding_sig` no longer verifies →
//!   [`AuthorizationGrantError::BindingMismatch`].
//! - **A-3 wrong-audience-swap.** Grant bound to audience X presented
//!   for audience Y; `binding_sig` covers Y's audience binding too →
//!   [`AuthorizationGrantError::AudienceMismatch`].
//!
//! ## Signing path
//!
//! Routed through `benten_crypto_suite::primitives::ed25519_dalek` per
//! `crypto-agility-contract:6` (the ONE crypto-primitive call site in
//! the workspace lives in `benten-crypto-suite`; consumers route
//! through its re-exports — see `benten-id` for the established
//! pattern). The signing keypair is carried inside the grant for the
//! `*_for_test` constructors used by the wave-3b RED-PHASE pins; the
//! production issuer-keypair routing lands at G-CORE-3e (sync) +
//! G-CORE-3f (Drop bundle) when the grant flows over real wire
//! protocols.
//!
//! ## Cross-surface canonical-bytes parity
//!
//! `to_canonical_bytes` / `from_canonical_bytes` produce the SAME bytes
//! whether the grant is destined for an online ALPN handler
//! (G-CORE-3e) or an offline Drop bundle (G-CORE-3f). One envelope
//! shape; both surfaces (P-3.2).

#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_crypto_suite::primitives::ed25519_dalek::{
    Signature, Signer, SigningKey, Verifier, VerifyingKey,
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

/// Synthetic UCAN envelope for the wave-3b RED-PHASE pins.
///
/// The production UCAN type lives in `benten-id::ucan::Ucan`; the
/// `AuthorizationGrant` surface accepts a *handle* shape so the
/// binding-sig contract can be exercised without coupling to the full
/// UCAN parse/validate pipeline at the cap-policy layer. The
/// production wire-up at G-CORE-3e will introduce a `From<Ucan>`
/// conversion + threaded canonical-bytes parity test.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UcanEnvelope {
    /// The audience DID-CID this UCAN is bound to.
    pub audience: Cid,
    /// Synthetic discriminator — distinguishes test fixtures so the
    /// `with_swapped_ucan_for_test` helper can construct a verifiably
    /// distinct envelope.
    pub discriminator: u32,
}

impl UcanEnvelope {
    /// Construct a synthetic envelope for the named audience.
    #[must_use]
    pub fn synthetic_for_test(audience: Cid) -> Self {
        Self {
            audience,
            discriminator: 0,
        }
    }

    /// Construct a synthetic envelope distinct from `synthetic_for_test`
    /// — used by adversarial pins.
    #[must_use]
    pub fn synthetic_for_test_distinct(audience: Cid, discriminator: u32) -> Self {
        Self {
            audience,
            discriminator,
        }
    }
}

/// Synthetic key-material handle for the wave-3b RED-PHASE pins.
///
/// The production key-material type is `benten_crypto_suite::aead::
/// KeyMaterial` (G-CORE-3a CANARY). The cap-layer surface mints its
/// own envelope shape so the binding-sig contract can be exercised
/// without coupling the wave-3b pin closure to the cipher-suite
/// internals. The production wire-up at G-CORE-3e/3f introduces the
/// real-type bridge.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyMaterial {
    /// Opaque material bytes. Synthetic test fixtures carry
    /// deterministic content; real key material is a wrapped AEAD key
    /// at the production wire-up.
    pub bytes: Vec<u8>,
}

impl KeyMaterial {
    /// Construct synthetic key material for tests.
    #[must_use]
    pub fn synthetic_for_test() -> Self {
        Self {
            bytes: vec![0xAA; 32],
        }
    }

    /// Construct synthetic key material distinct from `synthetic_for_test`
    /// — used by adversarial pins to swap halves.
    #[must_use]
    pub fn synthetic_for_test_distinct(discriminator: u8) -> Self {
        let mut bytes = vec![0u8; 32];
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = discriminator.wrapping_add(i as u8);
        }
        Self { bytes }
    }
}

/// ONE signed artifact: `{ucan, key_material, binding_sig}` plus the
/// load-bearing audience binding + the issuer's verifying-key bytes
/// the validator uses to verify `binding_sig`.
///
/// Per RATIFIED-S&C 2026-05-21 §R3:
///
/// > "Validators check `binding_sig` BEFORE consulting the UCAN scope
/// > or the key material — the binding is the foundation."
///
/// [`Self::verify_binding`] enforces that ordering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationGrant {
    /// The UCAN half (audience + delegation chain seam).
    pub ucan: UcanEnvelope,
    /// The key-material half (per-DID wrapped key or grant-derived
    /// material at the production wire-up).
    pub key_material: KeyMaterial,
    /// The issuer's binding signature over `(ucan, key_material,
    /// audience)` canonical bytes. Verified before either the UCAN or
    /// the key material is consulted.
    pub binding_sig: Vec<u8>,
    /// The audience the issuer bound the grant to. The validator's
    /// supplied audience must match this exactly.
    pub audience_binding: Cid,
    /// The issuer's verifying-key bytes (Ed25519). Threaded through
    /// the grant so the binding-sig can be verified without a separate
    /// keypair-resolution surface at the cap-policy layer. Production
    /// wire-up at G-CORE-3e introduces a key-resolution-via-DID-rotation-log
    /// path.
    #[serde(with = "serde_bytes")]
    pub issuer_verifying_key: Vec<u8>,
}

/// Typed `AuthorizationGrant` failure.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AuthorizationGrantError {
    /// The binding signature does not verify against the carried
    /// `(ucan, key_material, audience)` tuple — covers stolen-UCAN-
    /// without-keys (A-1) + stolen-keys-without-UCAN (A-2).
    #[error(
        "binding signature mismatch (E_AUTHORIZATION_GRANT_BINDING_SIG_INVALID): ucan or key_material does not match the issuer's signed binding"
    )]
    BindingMismatch {
        /// Human-readable detail of the mismatch (which half differs +
        /// the verify-failure shape). Test fixtures populate with a
        /// short narrative; production wire-up emits structured fields.
        detail: String,
    },
    /// The supplied audience does not match the audience the issuer
    /// bound the grant to — covers wrong-audience-swap (A-3).
    #[error(
        "audience mismatch (E_AUTHORIZATION_GRANT_BINDING_SIG_INVALID): grant was bound to {bound_audience:?} but verify was called with {presented_audience:?}"
    )]
    AudienceMismatch {
        /// The audience the issuer bound the grant to at issue time.
        bound_audience: Cid,
        /// The audience the validator supplied at verify time.
        presented_audience: Cid,
    },
    /// Serialization-layer failure — malformed inputs at
    /// encode/decode time.
    #[error("authorization grant serialization failure: {0}")]
    Serialization(String),
    /// Cryptographic verifying-key parsing failure.
    #[error("authorization grant verifying-key malformed: {0}")]
    VerifyingKeyMalformed(&'static str),
}

/// Domain-separation tag for the binding-sig message construction.
/// Distinct per surface so a signature on the grant body cannot be
/// reinterpreted as a signature on some other workspace surface.
const BINDING_SIG_DOMAIN: &[u8] = b"benten/g-core-3b/authorization-grant/v1";

impl AuthorizationGrant {
    /// Compute the canonical message the issuer signs / the validator
    /// re-checks. Concatenates the domain-separation tag, the CBOR
    /// canonical bytes of the UCAN, the CBOR canonical bytes of the
    /// key material, and the audience CID bytes — in that fixed
    /// order. Both sides MUST agree byte-for-byte for the binding-sig
    /// to verify.
    fn binding_message(
        ucan: &UcanEnvelope,
        key_material: &KeyMaterial,
        audience: &Cid,
    ) -> Result<Vec<u8>, AuthorizationGrantError> {
        let ucan_bytes = serde_ipld_dagcbor::to_vec(ucan)
            .map_err(|e| AuthorizationGrantError::Serialization(e.to_string()))?;
        let km_bytes = serde_ipld_dagcbor::to_vec(key_material)
            .map_err(|e| AuthorizationGrantError::Serialization(e.to_string()))?;
        let mut msg =
            Vec::with_capacity(BINDING_SIG_DOMAIN.len() + ucan_bytes.len() + km_bytes.len() + 36);
        msg.extend_from_slice(BINDING_SIG_DOMAIN);
        msg.extend_from_slice(&ucan_bytes);
        msg.extend_from_slice(&km_bytes);
        msg.extend_from_slice(audience.as_bytes());
        Ok(msg)
    }

    /// Test-only constructor — generates a fresh Ed25519 signing
    /// keypair, signs the canonical `(ucan, key_material, audience)`
    /// message, and returns a verifiable grant. Mirrors the
    /// `synthetic_for_test` pattern (Cid + production-type companions
    /// per the established workspace convention).
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationGrantError::Serialization`] if CBOR
    /// encoding of either half fails (synthetic test fixtures should
    /// not, but the path is fallible by construction).
    pub fn issue_for_test(
        ucan: UcanEnvelope,
        key_material: KeyMaterial,
        audience: Cid,
    ) -> Result<Self, AuthorizationGrantError> {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        let msg = Self::binding_message(&ucan, &key_material, &audience)?;
        let sig: Signature = signing_key.sign(&msg);

        Ok(Self {
            ucan,
            key_material,
            binding_sig: sig.to_bytes().to_vec(),
            audience_binding: audience,
            issuer_verifying_key: verifying_key.to_bytes().to_vec(),
        })
    }

    /// Verify the binding signature + audience binding.
    ///
    /// Order of checks (per RATIFIED §R3 "binding is the foundation"):
    /// 1. Audience binding — supplied audience must match
    ///    `audience_binding`. Returns
    ///    [`AuthorizationGrantError::AudienceMismatch`] otherwise.
    /// 2. Verifying-key parse — the issuer's Ed25519 key must parse.
    /// 3. Signature parse + cryptographic verify against the
    ///    re-constructed canonical binding message. Any failure
    ///    returns [`AuthorizationGrantError::BindingMismatch`] —
    ///    covers A-1 (stolen UCAN), A-2 (stolen keys), and any other
    ///    tampering of either half.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationGrantError::AudienceMismatch`],
    /// [`AuthorizationGrantError::VerifyingKeyMalformed`],
    /// [`AuthorizationGrantError::BindingMismatch`], or
    /// [`AuthorizationGrantError::Serialization`] per the per-arm
    /// semantics above.
    pub fn verify_binding(&self, audience: Cid) -> Result<(), AuthorizationGrantError> {
        // 1. Audience binding check.
        if self.audience_binding != audience {
            return Err(AuthorizationGrantError::AudienceMismatch {
                bound_audience: self.audience_binding,
                presented_audience: audience,
            });
        }

        // 2. Verifying-key parse.
        let vk_bytes: [u8; 32] = self
            .issuer_verifying_key
            .as_slice()
            .try_into()
            .map_err(|_| {
                AuthorizationGrantError::VerifyingKeyMalformed("expected 32-byte Ed25519 key")
            })?;
        let vk = VerifyingKey::from_bytes(&vk_bytes).map_err(|_| {
            AuthorizationGrantError::VerifyingKeyMalformed("invalid Ed25519 verifying key bytes")
        })?;

        // 3. Signature parse + cryptographic verify.
        let sig_bytes: [u8; 64] = self.binding_sig.as_slice().try_into().map_err(|_| {
            AuthorizationGrantError::BindingMismatch {
                detail: "binding_sig has non-Ed25519 length".to_string(),
            }
        })?;
        let sig = Signature::from_bytes(&sig_bytes);

        let msg = Self::binding_message(&self.ucan, &self.key_material, &audience)?;
        vk.verify(&msg, &sig)
            .map_err(|_| AuthorizationGrantError::BindingMismatch {
                detail: "Ed25519 verify failed against re-constructed (ucan, key_material, audience) message".to_string(),
            })
    }

    /// Serialize to canonical DAG-CBOR bytes for transmission /
    /// storage. The bytes are identical across online (ALPN) and
    /// offline (Drop bundle) surfaces per P-3.2.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationGrantError::Serialization`] if CBOR
    /// encoding fails.
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, AuthorizationGrantError> {
        serde_ipld_dagcbor::to_vec(self)
            .map_err(|e| AuthorizationGrantError::Serialization(e.to_string()))
    }

    /// Parse from canonical DAG-CBOR bytes.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationGrantError::Serialization`] on malformed
    /// input.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, AuthorizationGrantError> {
        serde_ipld_dagcbor::from_slice(bytes)
            .map_err(|e| AuthorizationGrantError::Serialization(e.to_string()))
    }

    /// Test-helper — swap the `key_material` half AFTER issue. Used by
    /// the A-1 stolen-UCAN-without-keys adversarial pin to
    /// demonstrate the binding-sig detects the tamper.
    #[must_use]
    pub fn with_swapped_key_material_for_test(&self, km: KeyMaterial) -> Self {
        Self {
            ucan: self.ucan.clone(),
            key_material: km,
            binding_sig: self.binding_sig.clone(),
            audience_binding: self.audience_binding,
            issuer_verifying_key: self.issuer_verifying_key.clone(),
        }
    }

    /// Test-helper — swap the `ucan` half AFTER issue. Used by the
    /// A-2 stolen-keys-without-UCAN adversarial pin to demonstrate
    /// the binding-sig detects the tamper.
    #[must_use]
    pub fn with_swapped_ucan_for_test(&self, ucan: UcanEnvelope) -> Self {
        Self {
            ucan,
            key_material: self.key_material.clone(),
            binding_sig: self.binding_sig.clone(),
            audience_binding: self.audience_binding,
            issuer_verifying_key: self.issuer_verifying_key.clone(),
        }
    }
}
