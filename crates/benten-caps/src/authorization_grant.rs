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
    /// G-CORE-3e: UCAN `exp` field (seconds-since-epoch). `u64::MAX`
    /// means "no expiry" (the wave-3b synthetic fixtures use this as
    /// the default; the wave-3e replay-attack pins set this to a past
    /// value to trigger the typed `UcanExpired` arm at handler time).
    /// The expiry check fires at the wave-3e ALPN handler boundary —
    /// `verify_binding` does NOT check this field (binding-sig
    /// validation is orthogonal to wall-clock validity, per the
    /// "binding is the foundation" §R3 contract).
    #[serde(default = "default_exp_secs")]
    pub exp_secs: u64,
    /// G-CORE-3e: UCAN `nbf` field (seconds-since-epoch) — not-before
    /// time. `0` means "no nbf restriction" (the wave-3b synthetic
    /// fixtures' default; the wave-3e nbf-in-future pin sets this to
    /// a future value to trigger the typed `UcanNotYetValid` arm).
    #[serde(default)]
    pub nbf_secs: u64,
    /// G-CORE-3e: a sentinel marking the grant as referencing an
    /// unresolvable peer-DID — couples §4.25 sync-hydrate denial +
    /// the §5 "unresolvable peer-DID at recheck" adversarial pattern.
    /// `false` is the production default; the wave-3e
    /// `tf3e_unresolvable_peer_did_yields_typed_unresolved_deny` pin
    /// sets this to `true` to trigger the typed `UnresolvedDeny` arm
    /// at handler time.
    ///
    /// This field **IS** covered by `binding_sig` verification (it's
    /// part of the [`UcanEnvelope`] serde shape, so `binding_message`
    /// CBOR-encodes it via serde-derive auto-inclusion; there is no
    /// `#[serde(skip)]` attribute — `#[serde(default)]` only controls
    /// deserialize-default behavior, not serialize-omission). However,
    /// the handler ARM 1 unresolved-peer short-circuit fires **BEFORE**
    /// ARM 5 binding-sig verification, so a grant with this flag set
    /// never reaches the binding-sig check — the runtime semantic is
    /// "unresolved-peer-deny short-circuits ALL other validation
    /// including binding-sig". Production grants with this flag set
    /// would still verify their binding-sig under the ARM-1-bypassed
    /// code path.
    ///
    /// The production wire-up at G-CORE-3e replaces this in-grant
    /// sentinel with a real peer-resolution call against the
    /// RotationLog. NEVER promote this to a production grant fixture —
    /// it is the sentinel pattern, NOT a real grant arm.
    #[serde(default)]
    pub unresolved_peer: bool,
}

/// Serde default for [`UcanEnvelope::exp_secs`] — `u64::MAX` (no
/// expiry). Distinct from `nbf_secs` whose default of `0` means "no
/// not-before restriction" (and `0` would mean "expires at epoch" if
/// used as the `exp` default, the silent fail-open shape).
#[allow(clippy::unnecessary_wraps)]
const fn default_exp_secs() -> u64 {
    u64::MAX
}

impl UcanEnvelope {
    /// Construct a synthetic envelope for the named audience.
    #[must_use]
    pub fn synthetic_for_test(audience: Cid) -> Self {
        Self {
            audience,
            discriminator: 0,
            exp_secs: u64::MAX,
            nbf_secs: 0,
            unresolved_peer: false,
        }
    }

    /// Construct a synthetic envelope distinct from `synthetic_for_test`
    /// — used by adversarial pins.
    #[must_use]
    pub fn synthetic_for_test_distinct(audience: Cid, discriminator: u32) -> Self {
        Self {
            audience,
            discriminator,
            exp_secs: u64::MAX,
            nbf_secs: 0,
            unresolved_peer: false,
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
    /// G-CORE-3e: the audience's Ed25519 verifying-key bytes — used
    /// by the wave-3e ALPN handler to check that the connection's
    /// verified `EndpointId` (which IS the requester's pubkey per Spike
    /// A2's zero-conversion plumbing) matches the UCAN audience. A
    /// mismatch yields typed `UcanAudienceMismatch` at the handler
    /// boundary. Optional for back-compat with wave-3b's
    /// `issue_envelopes_for_test` helper which uses a Cid-form
    /// audience and doesn't carry the raw pubkey bytes.
    #[serde(default, with = "serde_bytes_opt")]
    pub audience_pubkey: Option<Vec<u8>>,
    /// G-CORE-3e: the structured `RestrictedSpec` scope this grant
    /// authorises. The wave-3e per-request handler resolves the
    /// requested `ciphertext_hash` against this spec's `roots`
    /// allowlist (per the `with_hashes` constructor pattern); requests
    /// for hashes NOT in the allowlist yield typed
    /// `UcanBlobsRequestNotInScope`. Optional for back-compat with
    /// wave-3b's `issue_envelopes_for_test` helper which doesn't
    /// carry a scope (binding-sig validation is the wave-3b concern,
    /// scope-check is the wave-3e concern).
    #[serde(default)]
    pub scope: Option<crate::restricted_spec::RestrictedSpec>,
}

/// Helper module for `#[serde(with = "serde_bytes_opt")]` — folds
/// `Option<Vec<u8>>` through `serde_bytes` so DAG-CBOR encodes the
/// inner Vec as a byte-string when present and as `null` when absent.
mod serde_bytes_opt {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub(super) fn serialize<S>(value: &Option<Vec<u8>>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // serde's `#[serde(with = ...)]` codegen requires the signature
        // `fn serialize(value: &Option<Vec<u8>>, ser: S)` — accepting
        // `Option<&Vec<u8>>` here would break the macro contract. The
        // ref-option clippy lint flags the idiomatic stdlib-style
        // signature but doesn't apply to serde's required shape.
        #![allow(clippy::ref_option)]
        match value {
            None => ser.serialize_none(),
            Some(bytes) => serde_bytes::Bytes::new(bytes).serialize(ser),
        }
    }

    pub(super) fn deserialize<'de, D>(de: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Accept either a byte-string (Some) or null (None). DAG-CBOR
        // decodes `null` as `Option::None` natively when the field
        // type is Option<_>, so an explicit Option wrapper around the
        // ByteBuf form is the right shape.
        let opt: Option<serde_bytes::ByteBuf> = Option::deserialize(de)?;
        Ok(opt.map(serde_bytes::ByteBuf::into_vec))
    }
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

    /// Test-only constructor (wave-3b envelope-shaped) — generates a
    /// fresh Ed25519 signing keypair, signs the canonical `(ucan,
    /// key_material, audience)` message, and returns a verifiable
    /// grant. Mirrors the `synthetic_for_test` pattern (Cid +
    /// production-type companions per the established workspace
    /// convention).
    ///
    /// Wave-3b consumers (binding-sig tamper-detection pins) call this
    /// directly. Wave-3e consumers use the production-shaped
    /// [`Self::issue_for_test`] (keypair + audience-pubkey + scope +
    /// expiry) which threads the wave-3e scope + audience-pubkey
    /// fields automatically.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationGrantError::Serialization`] if CBOR
    /// encoding of either half fails (synthetic test fixtures should
    /// not, but the path is fallible by construction).
    pub fn issue_envelopes_for_test(
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
            audience_pubkey: None,
            scope: None,
        })
    }

    /// G-CORE-3e test constructor (wave-3e production-shaped) —
    /// builds a verifiable grant from the issuer's full
    /// `benten_id::keypair::Keypair`, the audience's `PublicKey`, a
    /// `RestrictedSpec` scope, and an absolute `exp_secs` expiry.
    /// Signs the binding-message with the issuer's actual signing
    /// key (per the production wire-up at G-CORE-3e where the issuer
    /// has a real Atrium identity).
    ///
    /// Couples the wave-3b binding-sig pipeline (`verify_binding`)
    /// with the wave-3e handler-layer scope + expiry + audience-pubkey
    /// fields the ALPN handler dispatches on.
    ///
    /// `nbf_secs` defaults to `0` (no not-before restriction).
    #[must_use]
    pub fn issue_for_test(
        issuer_kp: &benten_id::keypair::Keypair,
        audience_pubkey: &benten_id::keypair::PublicKey,
        scope: crate::restricted_spec::RestrictedSpec,
        exp_secs: u64,
    ) -> Self {
        Self::issue_with_nbf_for_test(issuer_kp, audience_pubkey, scope, 0, exp_secs)
    }

    /// G-CORE-3e test constructor — variant of [`Self::issue_for_test`]
    /// that also sets the UCAN `nbf` field. Used by
    /// `tf3e_ucan_nbf_in_future_typed_not_yet_valid` to exercise the
    /// not-before defense-in-depth check.
    #[must_use]
    pub fn issue_with_nbf_for_test(
        issuer_kp: &benten_id::keypair::Keypair,
        audience_pubkey: &benten_id::keypair::PublicKey,
        scope: crate::restricted_spec::RestrictedSpec,
        nbf_secs: u64,
        exp_secs: u64,
    ) -> Self {
        // Derive audience CID from the audience's pubkey bytes —
        // content-addressing under the `From<[u8; 32]> for Cid` arm
        // (BLAKE3-form construction; pubkey bytes are the 32-byte
        // digest). The wave-3e production wire-up will route through
        // the proper `did:key` CID derivation when grants flow from
        // the real DID-registration surface.
        let audience_bytes = audience_pubkey.to_bytes();
        let audience_cid: Cid = audience_bytes.into();

        let ucan = UcanEnvelope {
            audience: audience_cid,
            discriminator: 0,
            exp_secs,
            nbf_secs,
            unresolved_peer: false,
        };
        let key_material = KeyMaterial::synthetic_for_test();

        let signing_key = SigningKey::from_bytes(&issuer_kp.secret_bytes_unprotected());
        let verifying_key = signing_key.verifying_key();
        let msg = Self::binding_message(&ucan, &key_material, &audience_cid)
            .expect("synthetic UcanEnvelope + KeyMaterial CBOR-encode infallibly in test fixtures");
        let sig: Signature = signing_key.sign(&msg);

        Self {
            ucan,
            key_material,
            binding_sig: sig.to_bytes().to_vec(),
            audience_binding: audience_cid,
            issuer_verifying_key: verifying_key.to_bytes().to_vec(),
            audience_pubkey: Some(audience_bytes.to_vec()),
            scope: Some(scope),
        }
    }

    /// G-CORE-3e test constructor — variant of [`Self::issue_for_test`]
    /// that marks the grant's UCAN as referencing an unresolvable
    /// peer-DID (sentinel pattern). Used by
    /// `tf3e_unresolvable_peer_did_yields_typed_unresolved_deny` to
    /// exercise the wave-3e `UnresolvedDeny` arm at handler time.
    /// NEVER use this in production fixtures — the sentinel exists
    /// to assert the handler NEVER admits an unresolvable peer.
    #[must_use]
    pub fn issue_with_unresolved_peer_for_test(
        issuer_kp: &benten_id::keypair::Keypair,
        audience_pubkey: &benten_id::keypair::PublicKey,
        scope: crate::restricted_spec::RestrictedSpec,
        exp_secs: u64,
    ) -> Self {
        let mut grant = Self::issue_for_test(issuer_kp, audience_pubkey, scope, exp_secs);
        grant.ucan.unresolved_peer = true;
        // The unresolved_peer flag IS part of `binding_message` (it
        // serializes through serde-derive auto-inclusion on the
        // UcanEnvelope CBOR shape — see the field doc-comment for the
        // serde-derived inclusion analysis). Mutating it post-sign
        // therefore DOES invalidate `binding_sig`. We don't re-sign
        // here because the handler ARM 1 unresolved-peer short-circuit
        // fires BEFORE ARM 5 binding-sig verification — the runtime
        // semantic is `unresolved-peer-deny short-circuits ALL other
        // validation including binding-sig`. The fixture is therefore
        // brittle to a future ARM-reorder; see the
        // `g-core-3e-mr7-test-fixture-re-sign-discipline` carry in
        // `docs/future/phase-4-backlog.md` for the proper re-sign
        // hardening + the discriminator-asserting safety net the
        // tf3e corpus already carries.
        grant
    }

    /// G-CORE-3e test constructor — produces a deliberately
    /// MALFORMED grant whose `binding_sig` is structurally invalid
    /// (zero-filled). Used by
    /// `tf3e_per_request_ucan_validation_rejects_malformed_grant` to
    /// exercise the handler's rejection arm BEFORE any bytes flow.
    /// The handler MUST reject this grant via `BindingSigInvalid` (or
    /// `GrantValidation`) WITHOUT dispatching to iroh-blobs.
    #[must_use]
    pub fn malformed_for_test(audience_kp: &benten_id::keypair::Keypair) -> Self {
        let audience_pubkey = audience_kp.public_key();
        let audience_bytes = audience_pubkey.to_bytes();
        let audience_cid: Cid = audience_bytes.into();

        // Build the surface but with a structurally-invalid
        // binding_sig (all zeros). The verify_binding path WILL
        // reject — that is the wave-3e contract this fixture
        // exercises.
        Self {
            ucan: UcanEnvelope::synthetic_for_test(audience_cid),
            key_material: KeyMaterial::synthetic_for_test(),
            binding_sig: vec![0u8; 64],
            audience_binding: audience_cid,
            // Random verifying-key bytes — the verify call will
            // either fail to parse OR fail the cryptographic check.
            // We use a freshly-generated key to avoid any ambiguity.
            issuer_verifying_key: SigningKey::generate(&mut OsRng)
                .verifying_key()
                .to_bytes()
                .to_vec(),
            audience_pubkey: Some(audience_bytes.to_vec()),
            scope: Some(crate::restricted_spec::RestrictedSpec::new()),
        }
    }

    /// G-CORE-3e test helper — returns a stable CID identifying this
    /// grant (BLAKE3 over the canonical-bytes encoding). Used by the
    /// wave-3e revocation pin
    /// `tf3e_revoked_grant_yields_typed_revoked` to key the
    /// handler-side revocation store. Production wire-up at G-CORE-3e
    /// derives the grant CID at issue time through the established
    /// `benten-id` content-addressing seam.
    #[doc(hidden)]
    #[must_use]
    pub fn grant_cid_for_test(&self) -> Cid {
        let bytes = self
            .to_canonical_bytes()
            .expect("synthetic test grant CBOR-encodes infallibly");
        Cid::from_blake3_digest(*blake3::hash(&bytes).as_bytes())
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
            audience_pubkey: self.audience_pubkey.clone(),
            scope: self.scope.clone(),
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
            audience_pubkey: self.audience_pubkey.clone(),
            scope: self.scope.clone(),
        }
    }
}
