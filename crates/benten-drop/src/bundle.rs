//! `DropBundle` — the CBOR-on-disk envelope for offline-Drop sharing.
//!
//! See crate-level docs for the architectural framing. This module
//! holds the on-the-wire shape + the encode / decode + the offline-
//! consume pipeline.

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use benten_caps::authorization_grant::{AuthorizationGrant, UcanEnvelope};
use benten_caps::restricted_spec::RestrictedScope;
use benten_core::Cid;
use benten_graph::aead_wrap::{EncryptedNode, decode_encrypted_node, encode_encrypted_node};
use benten_id::keypair::Keypair;
use serde::{Deserialize, Serialize};

use crate::envelope_sig::{
    EnvelopeSigError, build_envelope_message, sign_envelope, verify_envelope,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Upper bound on a 5-Recipe DropBundle (Spike G measurement was
/// ~2688 bytes). The 4 KiB ceiling gives roughly 50% slack over the
/// measured value. Exceeding this is a possible envelope bloat /
/// non-canonical CBOR signal — surfaced by
/// `tf3f_dropbundle_5_recipe_bundle_within_size_envelope`.
pub const DROP_BUNDLE_MAX_SIZE_BYTES: usize = 4 * 1024;

/// The current Drop bundle on-the-wire version. Future on-the-wire
/// changes bump this value AND require a reader update; readers
/// encountering an unknown version yield typed
/// [`DropBundleError::UnsupportedDropVersion`] — never silent skip.
pub const DROP_BUNDLE_VERSION_V1: u16 = 1;

// ---------------------------------------------------------------------------
// Version + mode
// ---------------------------------------------------------------------------

/// The Drop bundle on-the-wire version discriminator. The `V1` arm is
/// the only one that round-trips through the production path; the
/// `Synthetic` arm is test-only (RED-phase pins synthesize "future"
/// version values to assert the typed-reject contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "tag", content = "value")]
#[non_exhaustive]
pub enum DropBundleVersion {
    /// v1 — the only production-known version at G-CORE-3f.
    V1,
    /// Test-only: synthesize an arbitrary u16 version discriminator
    /// so the future-version-rejection pin can assert the typed
    /// `UnsupportedDropVersion` path.
    Synthetic(u16),
    /// Catch-all for a genuine FUTURE on-the-wire version tag
    /// (e.g. `{"tag":"V2"}`) that this reader does not know.
    /// `#[serde(other)]` maps every unrecognized tag here, so the
    /// version-check (`is_v1`) routes it to the
    /// advertised typed [`DropBundleError::UnsupportedDropVersion`]
    /// (F-DROP-VER-FWD / Row D-83) rather than surfacing a GENERIC serde
    /// codec error. This is a DESERIALIZE-only catch-all — `#[serde(other)]`
    /// variants are never serialized, so adding it leaves the `V1` +
    /// `Synthetic` wire bytes byte-identical (verified: the freeze/golden
    /// round-trips are unchanged).
    #[serde(other)]
    UnknownVersion,
}

impl DropBundleVersion {
    /// Sentinel reported by [`DropBundleVersion::as_u16`] for the
    /// [`DropBundleVersion::UnknownVersion`] catch-all. The real future
    /// tag string is discarded by `#[serde(other)]`, so the numeric
    /// `seen` field carries a documented "unreadable future version"
    /// sentinel rather than a fabricated value.
    const UNKNOWN_VERSION_SENTINEL: u16 = u16::MAX;

    fn as_u16(self) -> u16 {
        match self {
            Self::V1 => DROP_BUNDLE_VERSION_V1,
            Self::Synthetic(n) => n,
            Self::UnknownVersion => Self::UNKNOWN_VERSION_SENTINEL,
        }
    }

    fn is_v1(self) -> bool {
        matches!(self, Self::V1)
            || matches!(self, Self::Synthetic(n) if n == DROP_BUNDLE_VERSION_V1)
    }
}

/// The two allowed Drop content-delivery modes per
/// `00-implementation-plan.md` §3 G-CORE-3 def input-constraints
/// refinement #6 (L341).
///
/// **No `InlineTiny` arm.** Mode 3 (bundle ≤16KiB inlined into share
/// URL) is **deferred to post-v1**; the absence is structural — a
/// future commit adding an `InlineTiny` variant breaks the exhaustive
/// match in `tf3f_drop_content_mode_no_inline_tiny_arm` (which
/// intentionally has no `_` wildcard).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropContentMode {
    /// Mode 1 — recipient pulls live from the publisher via the
    /// G-CORE-3e ALPN custom-handler path. Online; respects UCAN
    /// revocation.
    OnlinePull,
    /// Mode 2 — bundle is sealed offline; consumer reads from
    /// filesystem (USB stick / shared drive / etc.). No network; no
    /// live publisher; bundle is forever-valid once distributed per
    /// the §R6 revocation-reach reality.
    OfflineDrop,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Typed `DropBundle` failure modes.
///
/// All variants map to stable `ErrorCode` discriminants in
/// `benten-errors` so cross-language renderers can render the bundle
/// rejections without losing the wire-side stable name.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DropBundleError {
    /// The envelope-level Ed25519 signature does not verify against
    /// the (re-built) bundle header. Maps to
    /// `E_DROP_BUNDLE_ENVELOPE_SIG_INVALID`. Failing here means
    /// envelope tamper — the outer layer of defense-in-depth.
    #[error("envelope signature mismatch (E_DROP_BUNDLE_ENVELOPE_SIG_INVALID): {detail}")]
    EnvelopeSignatureInvalid {
        /// Human-readable failure detail (which surface of the header
        /// the verify-failure points at).
        detail: String,
    },

    /// Reader sees a `DropBundleVersion` discriminator it does not
    /// know. Maps to `E_DROP_BUNDLE_VERSION_UNSUPPORTED`. Per
    /// `00-implementation-plan.md` §3 G-CORE-3 def input-constraints
    /// (F-3: typed reject, never silent skip).
    #[error(
        "unsupported Drop bundle version (E_DROP_BUNDLE_VERSION_UNSUPPORTED): seen={seen}, known=[1]"
    )]
    UnsupportedDropVersion {
        /// The unknown version discriminator read off the wire.
        seen: u16,
    },

    /// Parser saw a Mode-3 (InlineTiny) bundle discriminator that is
    /// deferred-to-post-v1. Maps to
    /// `E_DROP_BUNDLE_MODE3_INLINE_REJECTED`. The bundle is
    /// rejected at parse time per the defer-to-post-v1 contract.
    #[error(
        "Mode-3 (InlineTiny) Drop bundle rejected (E_DROP_BUNDLE_MODE3_INLINE_REJECTED): {detail}"
    )]
    UnsupportedDropMode {
        /// Human-readable detail (which arm tripped + the deferral
        /// citation).
        detail: String,
    },

    /// One of the per-Node AEAD authentication tags failed to
    /// verify — the inner layer of defense-in-depth caught a
    /// content-only tamper. Maps via the existing
    /// `benten-graph::aead_wrap::AeadError` chain.
    #[error("per-Node AEAD authentication failed at content[{index}]: {detail}")]
    PerNodeAeadAuthenticationFailed {
        /// Index in the `content` array where the failure surfaced.
        index: usize,
        /// Underlying AEAD error detail.
        detail: String,
    },

    /// A per-Node signature (when present) did not verify against
    /// the carried verifying key. This is the secondary integrity
    /// layer — distinct from the AEAD-tag layer above.
    ///
    /// Reserved-but-unconstructed at v1-beta (register-then-enforce, Row
    /// D-66): the `per_node_attestation` field is a frozen sized placeholder,
    /// so nothing emits this variant yet. It lands when the deferred typed
    /// per-Node-signature upgrade lands (Phase-4-Meta-Composing).
    #[error("per-Node signature invalid at content[{index}]: {detail}")]
    PerNodeSignatureInvalid {
        /// Index in the `content` array where the failure surfaced.
        index: usize,
        /// Human-readable verify-failure detail.
        detail: String,
    },

    /// The bundle's `issuer_verifying_key` (the key the envelope-sig
    /// verifies under — an attacker-controllable header field anchored
    /// to nothing on its own) does NOT match the authoritative issuer of
    /// the `AuthorizationGrant` the recipient trusts
    /// (`auth_grant.issuer_verifying_key`, cryptographically self-bound
    /// via the grant's 7-segment binding-message). Maps to
    /// `E_DROP_BUNDLE_ENVELOPE_ISSUER_MISMATCH`. Fires on the recipient
    /// trust path ([`DropBundle::consume_offline`]) AFTER the grant
    /// binding verifies — closing the strip attack where an attacker
    /// re-authors the header, mints a fresh keypair, re-signs the
    /// envelope message, and overwrites `issuer_verifying_key` (a
    /// signature-by-nobody). Verify-time only; no wire byte changes.
    ///
    /// The first-class `ErrorCode` mirror is
    /// [`DropBundleError::code`] → `ErrorCode::DropBundleEnvelopeIssuerMismatch`
    /// (`E_DROP_BUNDLE_ENVELOPE_ISSUER_MISMATCH`). The catalog surface itself is
    /// reserved (boundary-lift at the G-CORE-9 v1-interface freeze, same as the
    /// sibling drop codes); the live production typed arm is THIS variant.
    #[error("envelope issuer mismatch (E_DROP_BUNDLE_ENVELOPE_ISSUER_MISMATCH): {detail}")]
    EnvelopeIssuerMismatch {
        /// Human-readable detail of the mismatch (the bundle
        /// envelope-sig key is not the trusted grant's issuer).
        detail: String,
    },

    /// The carried `AuthorizationGrant` did not verify against the
    /// supplied audience. Composes with `AuthorizationGrantError`.
    #[error("authorization grant verification failed: {0}")]
    AuthorizationGrantFailed(#[from] benten_caps::authorization_grant::AuthorizationGrantError),

    /// CBOR encode / decode failure.
    #[error("CBOR codec failure: {0}")]
    CodecError(String),
}

impl From<EnvelopeSigError> for DropBundleError {
    fn from(e: EnvelopeSigError) -> Self {
        Self::EnvelopeSignatureInvalid {
            detail: format!("{e}"),
        }
    }
}

impl DropBundleError {
    /// First-class stable [`benten_errors::ErrorCode`] mirror for this
    /// typed failure (§3.5g item 6). Cross-language renderers surface the
    /// wire-side stable code without losing the typed arm.
    ///
    /// The catalog surfaces for these codes are RESERVED at v1-beta (the
    /// boundary-lift into the engine-wide outbound-Drop API lands at the
    /// G-CORE-9 v1-interface freeze), so the codes carry a
    /// `reachability: ignore` catalog annotation — they are referenced only
    /// here, in this mapper arm, not constructed. The live production typed
    /// arms are the `DropBundleError` variants themselves.
    #[must_use]
    pub fn code(&self) -> benten_errors::ErrorCode {
        use benten_errors::ErrorCode;
        match self {
            Self::EnvelopeSignatureInvalid { .. } => ErrorCode::DropBundleEnvelopeSigInvalid,
            Self::UnsupportedDropVersion { .. } => ErrorCode::DropBundleVersionUnsupported,
            Self::UnsupportedDropMode { .. } => ErrorCode::DropBundleMode3InlineRejected,
            Self::EnvelopeIssuerMismatch { .. } => ErrorCode::DropBundleEnvelopeIssuerMismatch,
            // The AEAD-tag / per-Node-sig / grant-composition / codec arms map
            // to their existing first-class codes.
            Self::PerNodeAeadAuthenticationFailed { .. } => ErrorCode::AeadRebindingAttackDetected,
            Self::PerNodeSignatureInvalid { .. } => ErrorCode::DropBundleEnvelopeSigInvalid,
            Self::AuthorizationGrantFailed(_) => ErrorCode::AuthorizationGrantBindingSigInvalid,
            Self::CodecError(_) => ErrorCode::Serialize,
        }
    }
}

// ---------------------------------------------------------------------------
// EncryptedContent
// ---------------------------------------------------------------------------

/// The per-Recipe content cell carried in a `DropBundle`.
///
/// Internally a CBOR-on-disk cell holds the storage-layer encoded
/// bytes of a `benten_graph::aead_wrap::EncryptedNode` (via
/// `encode_encrypted_node` / `decode_encrypted_node`). The wrapper
/// shape lets the Drop layer be serde-derived even though
/// `EncryptedNode` itself is not (storage-layer envelope shape lives
/// behind the `to_wire_bytes` / `from_wire_bytes` private CBOR
/// representation per G-CORE-3d).
///
/// **⚠️ Wire-format coupling — frozen atomically at G-CORE-9.** The
/// Drop bundle's on-disk layout is content-addressed via the inner
/// `encode_encrypted_node` CBOR format, which the storage-layer
/// (`benten_graph::aead_wrap`) explicitly marks NOT-frozen pre-v1.
/// Both layers are atomically frozen at the G-CORE-9 v1-interface
/// freeze wave; before that, the Drop wire-format is provisional. The
/// `DropVersion` discriminator + the typed `UnsupportedDropVersion`
/// reject path ensure forward-incompat blobs surface a typed error
/// (no silent-mis-decode) until the freeze locks the layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedContent {
    /// Storage-encoded `EncryptedNode` bytes (via
    /// `benten_graph::aead_wrap::encode_encrypted_node`).
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
}

impl EncryptedContent {
    /// Wrap an `EncryptedNode` for transport in a `DropBundle`.
    ///
    /// # Errors
    ///
    /// Returns [`DropBundleError::CodecError`] if the storage-layer
    /// encode fails (only possible on chunk-count overflow for
    /// implausibly-large Nodes — won't fire for Drop bundle fixtures).
    pub fn from_encrypted_node(node: &EncryptedNode) -> Result<Self, DropBundleError> {
        let bytes = encode_encrypted_node(node)
            .map_err(|e| DropBundleError::CodecError(format!("encode_encrypted_node: {e}")))?;
        Ok(Self { bytes })
    }

    /// Decode this cell into an `EncryptedNode`.
    ///
    /// # Errors
    ///
    /// Returns [`DropBundleError::CodecError`] on malformed storage-
    /// encoded bytes.
    pub fn to_encrypted_node(&self) -> Result<EncryptedNode, DropBundleError> {
        decode_encrypted_node(&self.bytes)
            .map_err(|e| DropBundleError::CodecError(format!("decode_encrypted_node: {e}")))
    }
}

// ---------------------------------------------------------------------------
// DropBundle
// ---------------------------------------------------------------------------

/// The Drop bundle CBOR-on-disk envelope.
///
/// # On-disk layout (CBOR fields, in declaration order)
///
/// - `version`: [`DropBundleVersion`] — fail-typed on unknown.
/// - `mode`: [`DropContentMode`] — no `InlineTiny` arm.
/// - `spec_cid`: [`Cid`] — the canonical CID of the
///   [`RestrictedScope`] this bundle authorizes a snapshot of.
/// - `audience`: [`Cid`] — the recipient DID this bundle is bound to
///   (matches the `auth_grant.audience_binding`).
/// - `auth_grant`: [`AuthorizationGrant`] — the ONE signed artifact
///   `{ucan, key_material, binding_sig}` per RATIFIED §R3.
/// - `content`: `Vec<EncryptedContent>` — the per-Recipe AEAD
///   ciphertexts (each is a serialized `EncryptedNode` via
///   `encode_encrypted_node`).
/// - `restricted_spec`: [`RestrictedScope`] — the selector shape
///   (chain-validator anchor; the spec_cid above identifies which
///   spec this bundle is for; the body is carried for offline
///   consume so the recipient does not need a network lookup).
/// - `per_node_attestation`: optional opaque bytes blob carried
///   as a SIZE-RESERVED placeholder (130-byte sized marker). At the
///   G-CORE-9 v1-beta freeze this field is FROZEN as an inert,
///   reserved-size opaque blob — the real per-Node-signature
///   construction (a typed `Vec<Signature>` parallel to `content`)
///   was DEFERRED past the freeze (Row D-66); the freeze did NOT
///   upgrade the shape. Defense-in-depth on the shipped v1-beta path
///   is genuinely 2 layers (envelope-sig + per-Node-AEAD-tag); the
///   third layer (per-Node-sig validation) is the deferred addition,
///   and the `DropBundleError::PerNodeSignatureInvalid` typed-reject
///   defined here is reserved-but-unconstructed (register-then-enforce)
///   at v1-beta. The size-reservation pin keeps the ~12% Spike G
///   ceiling visible so the deferred per-Node-signature upgrade lands
///   ADDITIVELY (no wire-format surprise) when it arrives.
/// - `envelope_sig`: `Vec<u8>` — Ed25519 signature over the bundle
///   header (everything else above). Verified BEFORE any per-Node
///   decrypt is attempted.
/// - `issuer_verifying_key`: `Vec<u8>` — 32-byte Ed25519 verifying
///   key bytes for `envelope_sig`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropBundle {
    /// On-the-wire version discriminator.
    pub version: DropBundleVersion,
    /// Delivery mode — `OnlinePull` or `OfflineDrop` (never
    /// `InlineTiny` — Mode 3 deferred to post-v1).
    pub mode: DropContentMode,
    /// CID of the `RestrictedScope` this bundle delivers.
    pub spec_cid: Cid,
    /// Audience this bundle is bound to (matches grant audience).
    pub audience: Cid,
    /// The signed `{ucan, key_material, binding_sig}` artifact.
    pub auth_grant: AuthorizationGrant,
    /// Per-Recipe AEAD ciphertexts (serialized as opaque
    /// storage-encoded byte blobs).
    pub content: Vec<EncryptedContent>,
    /// The selector shape (chain-validator anchor + offline
    /// material).
    pub restricted_spec: RestrictedScope,
    /// Per-Node attestation blob — non-empty when the bundle is
    /// built with per-Node-sig defense-in-depth ON; empty (or
    /// shortened) for the envelope-only-for-test variant. FROZEN at the
    /// G-CORE-9 v1-beta freeze as an inert, reserved-size opaque blob;
    /// the typed `Vec<Signature>` (parallel to `content`) upgrade was
    /// DEFERRED past the freeze (Row D-66), so this stays a `Vec<u8>`
    /// size-reservation at v1-beta. The deferred upgrade lands additively.
    #[serde(with = "serde_bytes")]
    pub per_node_attestation: Vec<u8>,
    /// Ed25519 envelope signature bytes.
    #[serde(with = "serde_bytes")]
    pub envelope_sig: Vec<u8>,
    /// 32-byte Ed25519 verifying-key bytes for `envelope_sig`.
    #[serde(with = "serde_bytes")]
    pub issuer_verifying_key: Vec<u8>,
}

impl DropBundle {
    /// Number of `EncryptedContent` entries carried.
    #[must_use]
    pub fn content_count(&self) -> usize {
        self.content.len()
    }

    /// Encode the bundle to canonical DAG-CBOR bytes for on-disk
    /// distribution.
    ///
    /// # Errors
    ///
    /// Returns [`DropBundleError::CodecError`] if CBOR encoding fails.
    pub fn to_cbor_bytes(&self) -> Result<Vec<u8>, DropBundleError> {
        serde_ipld_dagcbor::to_vec(self).map_err(|e| DropBundleError::CodecError(format!("{e}")))
    }

    /// Decode a bundle from canonical DAG-CBOR bytes. Validates the
    /// version + mode discriminators (rejecting unknown versions +
    /// Mode-3 inline-tiny). DOES NOT verify the envelope-sig — call
    /// [`Self::verify_envelope_signature`] for that.
    ///
    /// # Errors
    ///
    /// - [`DropBundleError::CodecError`] for malformed CBOR.
    /// - [`DropBundleError::UnsupportedDropVersion`] for unknown
    ///   version discriminators.
    /// - [`DropBundleError::UnsupportedDropMode`] for Mode-3
    ///   inline-tiny bundles (deferred to post-v1).
    pub fn parse_cbor_bytes(bytes: &[u8]) -> Result<Self, DropBundleError> {
        // R19 (Row D-80 / D-67): fail-closed bounded-decode guard. Reject
        // over-cap input BEFORE the CBOR deserialize so an oversized/hostile
        // blob is a typed reject, not an OOM. `DROP_BUNDLE_MAX_SIZE_BYTES`
        // (4 KiB) is ~50% over the measured 5-Recipe bundle ceiling, so no
        // legitimate bundle is rejected here.
        if bytes.len() > DROP_BUNDLE_MAX_SIZE_BYTES {
            return Err(DropBundleError::CodecError(format!(
                "Drop bundle exceeds DROP_BUNDLE_MAX_SIZE_BYTES: got {} bytes, max {}",
                bytes.len(),
                DROP_BUNDLE_MAX_SIZE_BYTES
            )));
        }

        let bundle: Self = serde_ipld_dagcbor::from_slice(bytes)
            .map_err(|e| DropBundleError::CodecError(format!("{e}")))?;

        // Version check first — an unknown future version is the
        // typed-reject path per `00-implementation-plan.md` §3
        // G-CORE-3 def input-constraints (F-3).
        if !bundle.version.is_v1() {
            return Err(DropBundleError::UnsupportedDropVersion {
                seen: bundle.version.as_u16(),
            });
        }

        Ok(bundle)
    }

    // -----------------------------------------------------------------
    // Envelope-sig surface
    // -----------------------------------------------------------------

    /// Verify the envelope-level Ed25519 signature against the
    /// bundle header. DOES NOT touch the per-Node content (the inner
    /// layer of defense-in-depth — exercised by
    /// [`Self::consume_offline`]).
    ///
    /// # Errors
    ///
    /// Returns [`DropBundleError::EnvelopeSignatureInvalid`] on any
    /// of: malformed verifying-key bytes, malformed signature bytes,
    /// Ed25519 cryptographic verification failure.
    pub fn verify_envelope_signature(&self) -> Result<(), DropBundleError> {
        let msg = build_envelope_message(self);
        verify_envelope(&self.issuer_verifying_key, &self.envelope_sig, &msg)
            .map_err(DropBundleError::from)
    }

    // -----------------------------------------------------------------
    // Offline-consume pipeline
    // -----------------------------------------------------------------

    /// Consume the bundle offline. Sequential layers:
    ///
    /// 1. **Envelope-sig verify** — the outer defense-in-depth layer.
    ///    Fails BEFORE any decrypt attempt (no wasted work; no
    ///    information leak about content validity).
    /// 2. **Grant binding verify** — the
    ///    [`AuthorizationGrant::verify_binding`] check (A-1/A-2/A-3
    ///    tamper detection per RATIFIED §R3).
    /// 2b. **Envelope-issuer anchor** (F-INJ-2) — REQUIRE the bundle's
    ///    `issuer_verifying_key` (which Layer 1 verified the envelope-sig
    ///    under, but which is attacker-controllable and anchored to
    ///    nothing on its own) to equal the authoritative issuer of the
    ///    grant just verified in Layer 2 (`auth_grant.issuer_verifying_key`,
    ///    self-bound via the grant's 7-segment binding-message). On
    ///    mismatch: [`DropBundleError::EnvelopeIssuerMismatch`]. Closes the
    ///    strip attack (fresh-key re-sign of a re-authored header — a
    ///    signature-by-nobody). Verify-time only; no wire-shape change.
    /// 3. **Per-Node decrypt + AEAD authentication** — the inner
    ///    layer. Each [`EncryptedContent`] is decoded then decrypted
    ///    via `benten_graph::aead_wrap::decrypt`; AEAD authentication-
    ///    tag failure surfaces as
    ///    [`DropBundleError::PerNodeAeadAuthenticationFailed`].
    ///
    /// On success returns the recovered plaintext bytes for each
    /// content cell, in bundle order.
    ///
    /// # Errors
    ///
    /// Any failure in any of the three layers surfaces typed; see
    /// the [`DropBundleError`] variants.
    pub fn consume_offline(&self, recipient_kp: &Keypair) -> Result<Vec<Vec<u8>>, DropBundleError> {
        // Layer 1: envelope-sig (BEFORE any decrypt attempt).
        self.verify_envelope_signature()?;

        // Layer 2: grant binding-sig verify.
        let recipient_did_cid = derive_audience_cid_from_keypair(recipient_kp);
        self.auth_grant.verify_binding(recipient_did_cid)?;

        // Layer 2b (F-INJ-2): anchor the envelope-sig to the trusted grant
        // issuer. `verify_envelope_signature` (Layer 1) verifies the header
        // under `self.issuer_verifying_key` — an attacker-controllable field
        // anchored to NOTHING on its own: a strip attacker re-authors the
        // header, mints a fresh keypair, re-signs `build_envelope_message`,
        // and overwrites `issuer_verifying_key`, and Layer 1 passes (a
        // signature-by-nobody). Now that Layer 2 has established the
        // authoritative issuer — `auth_grant.issuer_verifying_key`, which is
        // cryptographically self-bound via the grant's 7-segment
        // binding-message (segment 7) and re-verified by `verify_binding` —
        // REQUIRE the two to match. On mismatch the otherwise-hollow
        // envelope-sig is not anchored to the grant the recipient trusts, so
        // reject typed. This is a verify-time check only; no wire byte /
        // CBOR field / golden vector changes.
        if self.issuer_verifying_key != self.auth_grant.issuer_verifying_key {
            return Err(DropBundleError::EnvelopeIssuerMismatch {
                detail: "bundle envelope-sig verifying key does not match the \
                         authoritative issuer of the trusted AuthorizationGrant \
                         (auth_grant.issuer_verifying_key)"
                    .to_string(),
            });
        }

        // Layer 3: per-Node decode + decrypt under the carried key.
        // R6 R2 fix-pass (Bundle R6-R2-FP-A L4 sibling): use the
        // per-Recipe AAD-binding decrypt path so inter-Recipe
        // truncation attacks (where a relay drops a Recipe from
        // self.content) surface as AEAD authentication failures
        // rather than silent admission of a partial bundle.
        let key_bytes = &self.auth_grant.key_material.bytes;
        let total_recipes_usize = self.content.len();
        let total_recipes = u32::try_from(total_recipes_usize).map_err(|_| {
            DropBundleError::CodecError(format!(
                "bundle recipe count {total_recipes_usize} exceeds u32"
            ))
        })?;
        let mut recovered = Vec::with_capacity(total_recipes_usize);
        for (idx, cell) in self.content.iter().enumerate() {
            let node = cell.to_encrypted_node()?;
            let recipe_index = u32::try_from(idx).map_err(|_| {
                DropBundleError::CodecError(format!("recipe index {idx} exceeds u32"))
            })?;
            let plaintext = benten_graph::aead_wrap::decrypt_recipe_encrypted_node(
                &node,
                key_bytes,
                recipe_index,
                total_recipes,
            )
            .map_err(|e| DropBundleError::PerNodeAeadAuthenticationFailed {
                index: idx,
                detail: format!("{e}"),
            })?;
            recovered.push(plaintext);
        }

        Ok(recovered)
    }

    /// Test-only variant of [`Self::consume_offline`] that returns
    /// the number of per-Node decrypt attempts reached (0 if the
    /// envelope-sig failed BEFORE any decrypt was attempted). Used by
    /// `tf3f_tampered_envelope_sig_fails_before_per_node_decrypt` to
    /// pin the fail-fast-no-wasted-work property.
    ///
    /// # Errors
    ///
    /// On error, the returned tuple's second element carries the
    /// `decrypt_attempt_count` reached BEFORE failure (0 if failure
    /// at envelope-sig layer; ≥1 if the envelope-sig passed and
    /// failure occurred inside the per-Node loop).
    #[cfg(any(test, feature = "testing"))]
    pub fn consume_offline_with_dec_counter_for_test(
        &self,
        recipient_kp: &Keypair,
    ) -> Result<Vec<Vec<u8>>, (DropBundleError, usize)> {
        // Layer 1: envelope-sig (BEFORE any decrypt attempt).
        if let Err(e) = self.verify_envelope_signature() {
            return Err((e, 0));
        }

        // Layer 2: grant binding.
        let recipient_did_cid = derive_audience_cid_from_keypair(recipient_kp);
        if let Err(e) = self.auth_grant.verify_binding(recipient_did_cid) {
            return Err((DropBundleError::AuthorizationGrantFailed(e), 0));
        }

        // Layer 2b (F-INJ-2): anchor the envelope-sig to the trusted grant
        // issuer (mirrors `consume_offline`). Fires before any decrypt, so
        // the reached-decrypt-count is 0.
        if self.issuer_verifying_key != self.auth_grant.issuer_verifying_key {
            return Err((
                DropBundleError::EnvelopeIssuerMismatch {
                    detail: "bundle envelope-sig verifying key does not match the \
                             authoritative issuer of the trusted AuthorizationGrant \
                             (auth_grant.issuer_verifying_key)"
                        .to_string(),
                },
                0,
            ));
        }

        // Layer 3: per-Node decrypt.
        // R6 R2 fix-pass (Bundle R6-R2-FP-A L4 sibling): use the
        // per-Recipe AAD-binding decrypt path matching consume_offline.
        let key_bytes = &self.auth_grant.key_material.bytes;
        let total_recipes_usize = self.content.len();
        let total_recipes = u32::try_from(total_recipes_usize).map_err(|_| {
            (
                DropBundleError::CodecError(format!(
                    "bundle recipe count {total_recipes_usize} exceeds u32"
                )),
                0,
            )
        })?;
        let mut recovered = Vec::with_capacity(total_recipes_usize);
        let mut attempts = 0usize;
        for (idx, cell) in self.content.iter().enumerate() {
            attempts += 1;
            let node = match cell.to_encrypted_node() {
                Ok(n) => n,
                Err(e) => return Err((e, attempts)),
            };
            let recipe_index = match u32::try_from(idx) {
                Ok(v) => v,
                Err(_) => {
                    return Err((
                        DropBundleError::CodecError(format!("recipe index {idx} exceeds u32")),
                        attempts,
                    ));
                }
            };
            match benten_graph::aead_wrap::decrypt_recipe_encrypted_node(
                &node,
                key_bytes,
                recipe_index,
                total_recipes,
            ) {
                Ok(plaintext) => recovered.push(plaintext),
                Err(e) => {
                    return Err((
                        DropBundleError::PerNodeAeadAuthenticationFailed {
                            index: idx,
                            detail: format!("{e}"),
                        },
                        attempts,
                    ));
                }
            }
        }

        Ok(recovered)
    }

    // -----------------------------------------------------------------
    // Test fixtures (Spike G + tf3f pin support)
    // -----------------------------------------------------------------

    /// Build a 5-Recipe DropBundle for testing — uses
    /// `build_5_recipe_bundle_for_recipient` against a fresh
    /// recipient keypair so the round-trip pins have a known fixture.
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn build_5_recipe_bundle_for_test(issuer_kp: &Keypair) -> Self {
        let recipient_kp = Keypair::generate();
        Self::build_5_recipe_bundle_for_recipient(issuer_kp, &recipient_kp)
    }

    /// Build a 5-Recipe DropBundle for a specific recipient. Used by
    /// the offline-consume + tamper pins.
    ///
    /// **R6 R1 FP-A Bundle F1.d:** cfg-gated under
    /// `cfg(any(test, feature = "testing"))` because the body composes
    /// `UcanEnvelope::synthetic_for_test` + `GrantKeyMaterial::synthetic_for_test`
    /// + `AuthorizationGrant::issue_envelopes_for_test`, all of which
    /// became cfg-gated in F1.a. Gating the caller is cleaner than
    /// per-call cascading and matches V1-FROZEN-INTERFACE-DEFERRED.md
    /// Row D-22 sub-task 5's "transitive `_for_test`-consumer" rule.
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn build_5_recipe_bundle_for_recipient(
        issuer_kp: &Keypair,
        recipient_kp: &Keypair,
    ) -> Self {
        build_5_recipe_bundle_impl(
            issuer_kp,
            recipient_kp,
            /* include_per_node_attestation= */ true,
        )
    }

    /// Build a 5-Recipe DropBundle WITHOUT the per-Node attestation
    /// blob (used to measure the placeholder size-reservation overhead
    /// in `tf3f_per_node_attestation_size_overhead_under_12_percent`).
    /// The envelope-sig is still emitted; per-Node integrity reduces
    /// to the AEAD tag layer only.
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn build_5_recipe_bundle_for_recipient_envelope_only_for_test(
        issuer_kp: &Keypair,
        recipient_kp: &Keypair,
    ) -> Self {
        build_5_recipe_bundle_impl(
            issuer_kp,
            recipient_kp,
            /* include_per_node_attestation= */ false,
        )
    }

    /// Test-only: flip one byte in `content[index]`'s ciphertext to
    /// simulate a content-only tamper. The envelope-sig binds the
    /// (un-tampered) header — the tamper happens post-sign — so the
    /// envelope-sig CAN still pass. AEAD-tag catches the tamper at
    /// decrypt time (the defense-in-depth claim).
    ///
    /// Used by
    /// `tf3f_per_node_ciphertext_tamper_detected_envelope_sig_still_valid`.
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn flip_byte_in_content_for_test(&self, index: usize, offset: usize) -> Self {
        let mut clone = self.clone();
        if let Some(cell) = clone.content.get_mut(index) {
            // For small 5-Recipe fixtures the encoded blob carries
            // the AEAD ciphertext + tag inline. The storage envelope
            // format is `[magic(1)][variant(1)][plaintext_cid(36)][aead_envelope(...)]`
            // for the Whole arm; the ciphertext+tag live near the
            // end. Flipping a byte at offset `38 + offset` from the
            // start lands in the AEAD payload area (well past the
            // header) and breaks AEAD authentication.
            let target = 38 + offset;
            if let Some(b) = cell.bytes.get_mut(target) {
                *b ^= 0xFF;
            } else if let Some(last) = cell.bytes.last_mut() {
                // Bundle too small for the requested offset — flip
                // the last byte so the AEAD tag is still corrupted.
                *last ^= 0xFF;
            }
        }
        clone
    }

    /// Test-only: corrupt the envelope-sig field to simulate an
    /// envelope-layer tamper. Used by
    /// `tf3f_tampered_envelope_sig_fails_before_per_node_decrypt`.
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn tamper_envelope_signature_for_test(&self) -> Self {
        let mut clone = self.clone();
        if let Some(b) = clone.envelope_sig.first_mut() {
            *b ^= 0xFF;
        } else {
            // Empty sig (should never happen for a built bundle) —
            // push a non-empty obviously-bogus value so the verify
            // still fails typed.
            clone.envelope_sig = alloc::vec![0xDEu8; 64];
        }
        clone
    }

    /// Test-only: synthesize a bundle carrying an arbitrary version
    /// discriminator (used by the future-version reject pin).
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn synthesize_future_version_for_test(version: DropBundleVersion) -> Self {
        let issuer_kp = Keypair::generate();
        let recipient_kp = Keypair::generate();
        let mut bundle = Self::build_5_recipe_bundle_for_recipient(&issuer_kp, &recipient_kp);
        bundle.version = version;
        // Re-sign so the envelope-sig matches the mutated header.
        // (If we left the old sig, future-version-reject would
        // also be hidden by an envelope-sig-failure. We want the
        // typed UnsupportedDropVersion path to surface.)
        let msg = build_envelope_message(&bundle);
        bundle.envelope_sig = sign_envelope(&issuer_kp, &msg);
        bundle
    }

    /// Test-only: synthesize a CBOR payload that the parser will
    /// reject with the Mode-3-deferred-to-post-v1 typed error.
    ///
    /// Implementation: emits a bundle with a `Synthetic(0xFFFE)`
    /// version + an internal marker requesting Mode-3 rejection
    /// surfacing. The pin
    /// `tf3f_inline_tiny_synthetic_rejected_via_unsupported_version_or_mode_typed` accepts
    /// either `UnsupportedDropMode` OR `UnsupportedDropVersion` — the
    /// defer-to-post-v1 contract is the load-bearing property; the
    /// specific typed code is not.
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn synthesize_inline_tiny_cbor_for_test() -> Vec<u8> {
        let issuer_kp = Keypair::generate();
        let recipient_kp = Keypair::generate();
        let mut bundle = Self::build_5_recipe_bundle_for_recipient(&issuer_kp, &recipient_kp);
        // Synthetic version that is NOT v1 — triggers
        // UnsupportedDropVersion at parse time, satisfying the
        // defer-to-post-v1 contract.
        bundle.version = DropBundleVersion::Synthetic(0xFFFE);
        // Re-sign so the envelope-sig is not the failure point.
        let msg = build_envelope_message(&bundle);
        bundle.envelope_sig = sign_envelope(&issuer_kp, &msg);
        bundle
            .to_cbor_bytes()
            .expect("synthetic bundle encodes for test")
    }

    /// Test-only: synthesize a revocation record for the embedded
    /// UCAN. The record is opaque to this crate (revocation routing
    /// lives in `benten-id` + the G-CORE-3e ALPN path); the test
    /// fixture exists so the `tf3f_drop_bundle_decrypts_after_ucan_revocation_forever_valid`
    /// pin can demonstrate that the revocation record's existence
    /// does NOT prevent the offline decrypt (the R6 reality).
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn synthesize_revocation_for_embedded_ucan(
        _bundle: &Self,
        _issuer_kp: &Keypair,
    ) -> RevocationRecord {
        RevocationRecord {
            // The opaque record content is irrelevant to the R6
            // reality pin — the load-bearing property is that the
            // bundle decrypts WITHOUT consulting this record.
            opaque: alloc::vec![0xDEu8; 16],
        }
    }
}

// ---------------------------------------------------------------------------
// Revocation record (test-only opaque sentinel)
// ---------------------------------------------------------------------------

/// Opaque revocation-record sentinel — see
/// `DropBundle::synthesize_revocation_for_embedded_ucan`.
///
/// `cfg(any(test, feature = "testing"))` because it exists solely to carry the
/// sentinel that fixture returns; it is not part of the frozen v1-beta surface.
#[cfg(any(test, feature = "testing"))]
#[derive(Debug, Clone)]
pub struct RevocationRecord {
    opaque: Vec<u8>,
}

#[cfg(any(test, feature = "testing"))]
impl RevocationRecord {
    /// Number of opaque bytes carried (so the type isn't trivially
    /// optimizable away by clippy::dead_code).
    #[must_use]
    pub fn opaque_len(&self) -> usize {
        self.opaque.len()
    }
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

/// Common 5-Recipe bundle construction. Materializes 5 small
/// plaintext Recipe Node bodies, AEAD-wraps each, builds the
/// `AuthorizationGrant` bound to the recipient's audience, signs
/// the envelope with the issuer's Keypair.
///
/// **R6 R1 FP-A Bundle F1.d:** cfg-gated under
/// `cfg(any(test, feature = "testing"))` because the body composes
/// `UcanEnvelope::synthetic_for_test` + `GrantKeyMaterial::from_bytes_for_test`
/// + `AuthorizationGrant::issue_envelopes_for_test`, all of which became
/// cfg-gated in F1.a. Sole callers
/// (`build_5_recipe_bundle_for_recipient` +
/// `build_5_recipe_bundle_for_recipient_envelope_only_for_test`) are
/// themselves cfg-gated, so this private helper following them is the
/// minimal-blast-radius gate.
#[cfg(any(test, feature = "testing"))]
fn build_5_recipe_bundle_impl(
    issuer_kp: &Keypair,
    recipient_kp: &Keypair,
    include_per_node_attestation: bool,
) -> DropBundle {
    let audience = derive_audience_cid_from_keypair(recipient_kp);

    // 5 Recipe Node plaintexts. Body kept small so the bundle
    // measures < 4 KiB (Spike G target ~2688 bytes).
    let recipes: [&[u8]; 5] = [
        b"Recipe #1: pasta carbonara",
        b"Recipe #2: caesar salad",
        b"Recipe #3: cacio e pepe",
        b"Recipe #4: risotto milanese",
        b"Recipe #5: tiramisu",
    ];

    // Synthetic AEAD key material (must match what the
    // AuthorizationGrant carries). Use 32 bytes of a fixed pattern
    // so the test fixtures are deterministic; the real production
    // path threads a per-recipient wrapped key.
    let key_bytes = alloc::vec![0xAAu8; 32];

    // Encrypt each Recipe under a synthetic plaintext-CID
    // (derived from the recipe label). The AAD binds
    // `(plaintext_cid, recipe_index, total_recipes)` per the
    // R6 R2 fix-pass L4-sibling inter-Recipe truncation defense —
    // dropping a Recipe from the Vec causes consume_offline to
    // reconstruct AAD with a smaller `total_recipes` value and AEAD
    // authentication fails cryptographically.
    let total_recipes_usize = recipes.len();
    let total_recipes =
        u32::try_from(total_recipes_usize).expect("5-Recipe fixture count trivially fits u32");
    let mut content = Vec::with_capacity(total_recipes_usize);
    for (i, body) in recipes.iter().enumerate() {
        let label = format!("recipe-{i}");
        let plaintext_cid = sample_cid_for_label(&label);
        let recipe_index = u32::try_from(i).expect("5-Recipe fixture index trivially fits u32");
        let cell = benten_graph::aead_wrap::EncryptedNode::encrypt_recipe(
            body,
            &plaintext_cid,
            &key_bytes,
            recipe_index,
            total_recipes,
        )
        .expect("AEAD encrypt_recipe of small recipe body must succeed");
        let encoded =
            EncryptedContent::from_encrypted_node(&cell).expect("EncryptedContent wraps OK");
        content.push(encoded);
    }

    // SubgraphSpec selector + canonical CID over its canonical bytes.
    let restricted_spec = RestrictedScope::default();
    let spec_cid_bytes = serde_ipld_dagcbor::to_vec(&restricted_spec)
        .expect("RestrictedScope encodes for spec_cid derivation");
    let spec_cid_digest = blake3_digest_of(&spec_cid_bytes);
    let spec_cid = Cid::from_blake3_digest(spec_cid_digest);

    // AuthorizationGrant — the ONE signed artifact per §R3.
    // Synthetic UCAN + matching synthetic GrantKeyMaterial. The
    // `key_material.bytes` MUST equal the AEAD key bytes above so
    // the recipient's `consume_offline` can decrypt.
    let ucan = UcanEnvelope::synthetic_for_test(audience);
    let key_material =
        benten_caps::authorization_grant::GrantKeyMaterial::from_bytes_for_test(key_bytes.clone());
    // Note: PR #1336 (G-CORE-3e wave) reshaped `AuthorizationGrant::
    // issue_for_test` to a 4-arg keypair+audience-pubkey+scope+expiry
    // signature; the wave-3b 3-arg helper is now `issue_envelopes_for_test`.
    // benten-drop fixture needs the wave-3b shape because it must control
    // `key_material.bytes` (matching the AEAD key_bytes above for
    // consume_offline decryption) — Strategy-C wave-2 batch consolidation.
    // F-INJ-2: issue the grant with the SAME `issuer_kp` that signs the
    // envelope below, so the honest fixture satisfies the consume_offline
    // envelope-issuer anchor (`bundle.issuer_verifying_key ==
    // auth_grant.issuer_verifying_key`). The prior `issue_envelopes_for_test`
    // generated an ephemeral grant-issuer key, decoupling the two — which the
    // anchor check (correctly) rejects.
    let auth_grant = AuthorizationGrant::issue_envelopes_for_test_with_issuer(
        issuer_kp,
        ucan,
        key_material,
        audience,
    )
    .expect("synthetic grant issues for test");

    // G-CORE-3f: this is a SIZED PLACEHOLDER, not real per-Node
    // Ed25519 signatures. The placeholder reserves the wire-shape
    // budget; the G-CORE-9 v1-beta freeze FROZE it as this inert
    // reserved-size blob — the typed `Vec<Signature>` (parallel to
    // `content`) upgrade was DEFERRED past the freeze (Row D-66), so it
    // remains a sized placeholder at v1-beta and the upgrade lands
    // additively. See `per_node_attestation` field docstring + the
    // `tf3f_per_node_attestation_size_overhead_under_12_percent` pin.
    // For Spike G's overhead measurement the load-bearing property is
    // the size differential between with-attestation and envelope-only
    // variants. Sized to stay <12% of base bundle size per the Spike G
    // measurement.
    //
    // Sizing math: the envelope-only bundle measures ~1327 bytes
    // (5 small Recipe cells + grant + header + envelope-sig);
    // 12% = ~159 bytes; we target ~10% headroom = ~130 bytes
    // (26 bytes per cell × 5 cells).
    let per_node_attestation = if include_per_node_attestation {
        // 5 cells × 26-byte synthetic attestation marker. Total
        // ~130 bytes — ~10% of base; well under the 12% ceiling.
        alloc::vec![0xA5u8; content.len() * 26]
    } else {
        Vec::new()
    };

    let mut bundle = DropBundle {
        version: DropBundleVersion::V1,
        mode: DropContentMode::OfflineDrop,
        spec_cid,
        audience,
        auth_grant,
        content,
        restricted_spec,
        per_node_attestation,
        envelope_sig: Vec::new(),
        issuer_verifying_key: issuer_kp.public_key().to_bytes().to_vec(),
    };

    // Compute the envelope-sig over the (final, post-modification)
    // header.
    let msg = build_envelope_message(&bundle);
    bundle.envelope_sig = sign_envelope(issuer_kp, &msg);
    bundle
}

/// Derive a synthetic audience CID from a recipient `Keypair`. The
/// audience routing surface uses CIDs (not DIDs directly) so the
/// `AuthorizationGrant::audience_binding` shape can be threaded.
fn derive_audience_cid_from_keypair(kp: &Keypair) -> Cid {
    let vk_bytes = kp.public_key().to_bytes();
    let digest = blake3_digest_of(&vk_bytes);
    Cid::from_blake3_digest(digest)
}

/// Deterministic per-label test CID derivation. Same shape as
/// `Cid::sample_for_label` but exposed here without depending on the
/// `testing` feature of `benten-core` (which this crate doesn't
/// declare).
fn sample_cid_for_label(label: &str) -> Cid {
    let digest = blake3_digest_of(label.as_bytes());
    Cid::from_blake3_digest(digest)
}

/// 32-byte BLAKE3 digest. Routes via `benten_crypto_suite::primitives::
/// blake3` per the only-call-site rule (CLAUDE.md baked-in #5).
fn blake3_digest_of(bytes: &[u8]) -> [u8; 32] {
    *benten_crypto_suite::primitives::blake3::hash(bytes).as_bytes()
}
