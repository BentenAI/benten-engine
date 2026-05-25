//! G-CORE-3e — UCAN-gated iroh-blobs custom-ALPN handler
//! (Flavor B per-request UCAN check).
//!
//! ## Wave contract
//!
//! Per `RATIFIED-sharing-and-confidentiality-2026-05-21.md` §R2 +
//! Spike A2 finding: the wave-3e handler validates the requester's
//! [`AuthorizationGrant`] per request, then hands the same iroh
//! `Connection` to upstream
//! `iroh_blobs::provider::handle_connection` to serve bytes by hash
//! (the `ciphertext_hash` from the two-CID mapping).
//!
//! **Zero-conversion plumbing (Spike A2):** iroh `EndpointId` IS
//! `ed25519_dalek::VerifyingKey` — the requester's iroh peer ID IS
//! their UCAN audience pubkey. No parsing, no `VerifyingKey::from_bytes`
//! round-trip in the per-request hot path. [`endpoint_id_to_verifying_key`]
//! + [`verifying_key_to_endpoint_id`] are the named identity-cast
//! seam (with a compile-time round-trip pin at
//! `tests/tf3e_zero_conversion_endpoint_id_is_verifying_key.rs`).
//!
//! ## Per-request validation pipeline
//!
//! The handler runs validation arms IN ORDER (fail-closed at the
//! first reject; the typed reject IS the defense per §R3 + §R2):
//!
//! 1. **Unresolvable peer-DID short-circuit.** If the grant's UCAN
//!    is flagged as referencing an unresolvable peer (the
//!    `<unresolved-peer>` sentinel from `security-r1-2`), return
//!    typed [`UcanBlobsHandlerError::UnresolvedDeny`] BEFORE any
//!    cryptographic work. Couples §4.36 recheck + §4.25 sync-hydrate
//!    denial.
//! 2. **Audience binding (connection EndpointId == UCAN audience).**
//!    The connection's verified `EndpointId` (provided by the caller
//!    via [`UcanBlobsHandler::validate_request_for_connection`]'s
//!    second argument) must equal the grant's `audience_pubkey`
//!    bytes. Mismatch → typed
//!    [`UcanBlobsHandlerError::UcanAudienceMismatch`]. This is the
//!    A-2 audience-substitution defense.
//! 3. **Time-bounded UCAN validity.** `nbf <= now < exp` per
//!    injected clock. Past-exp →
//!    [`UcanBlobsHandlerError::UcanExpired`]; future-nbf →
//!    [`UcanBlobsHandlerError::UcanNotYetValid`].
//! 4. **Revocation observance.** If `grant.grant_cid_for_test()` is
//!    in the handler's revocation store, → typed
//!    [`UcanBlobsHandlerError::GrantRevoked`]. (Couples Phase-3
//!    revocation infrastructure per §R6 reach.)
//! 5. **Binding-sig verification.** Delegate to
//!    [`AuthorizationGrant::verify_binding`] — covers A-1
//!    stolen-UCAN-without-keys, A-2 stolen-keys-without-UCAN, A-3
//!    wrong-audience-swap. A failure here surfaces as
//!    [`UcanBlobsHandlerError::BindingSigInvalid`] (with the original
//!    [`AuthorizationGrantError`] embedded for diagnostic clarity).
//! 6. **Scope check (requested ciphertext_hash in granted
//!    `RestrictedScope`).** The handler resolves the requested hash
//!    against the grant's `RestrictedScope::roots` allowlist (per
//!    the `with_hashes` constructor pattern). Not-in-scope → typed
//!    [`UcanBlobsHandlerError::NotInScope`].
//!
//! ONLY after all six arms pass does the handler dispatch to
//! upstream `iroh_blobs::provider::handle_connection` (the
//! `serve_request_for_test` arm increments the test-instrumented
//! dispatch counter; the production wire-up performs the actual
//! iroh-blobs call).
//!
//! ## What is NOT in this wave's scope
//!
//! - The actual `iroh_blobs::provider::handle_connection` call: the
//!   brief explicitly says the handler "reuses" the upstream call;
//!   landing iroh-blobs as a workspace dependency is OUT-OF-SCOPE
//!   for G-CORE-3e (the named integration wave at G-CORE-3-real-iroh-
//!   blobs swaps the test-instrumented dispatch counter for the real
//!   upstream call). The wave-3e proof: per-request UCAN validation +
//!   scope check + identity-cast plumbing all work end-to-end against
//!   the test seam.
//! - Real RotationLog peer-resolution: the wave-3e
//!   `unresolved_peer` flag is a sentinel; the production wire-up
//!   replaces it with a real `benten-id::RotationLog::resolve` call.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::sync::Mutex;

use benten_caps::authorization_grant::{AuthorizationGrant, AuthorizationGrantError};
use benten_core::Cid;
use benten_crypto_suite::primitives::ed25519_dalek::VerifyingKey;
use benten_id::keypair::{Keypair, PublicKey};

/// ALPN identifier for the UCAN-gated iroh-blobs protocol per
/// G-CORE-3e (Phase-4-Meta-Core). Distinct from
/// [`crate::transport::ATRIUM_ALPN`] so the iroh endpoint can dispatch
/// inbound connections by ALPN to the right handler — Atrium peer-mesh
/// connections go to [`crate::transport::ATRIUM_ALPN`]; UCAN-gated
/// blobs requests go here.
///
/// Version-suffixed (`/1`) per the workspace ALPN-versioning
/// convention so a future protocol revision is a non-breaking
/// additive ALPN (the older ALPN keeps working for legacy peers).
pub const UCAN_BLOBS_ALPN: &[u8] = b"benten/ucan-blobs/1";

/// A request to the UCAN-gated iroh-blobs ALPN handler.
///
/// The requester opens a connection (verified at the iroh QUIC layer
/// to bind a specific `EndpointId`, which IS the requester's
/// Ed25519 pubkey per Spike A2), presents this request structure,
/// and the handler validates BEFORE dispatching to
/// `iroh_blobs::provider::handle_connection`.
#[derive(Debug, Clone)]
pub struct UcanBlobsRequest {
    /// The [`AuthorizationGrant`] authorising this request. The
    /// handler validates `grant.verify_binding(audience_cid)` per
    /// request — never trust a previously-validated grant across
    /// requests (the wave-3e Flavor B contract).
    pub grant: AuthorizationGrant,
    /// The ciphertext_hash the requester wants. The handler resolves
    /// this against the grant's [`benten_caps::restricted_spec::RestrictedScope`]
    /// scope; out-of-scope hashes are rejected typed-`NotInScope`.
    pub ciphertext_hash: Cid,
}

/// Response from the UCAN-gated iroh-blobs ALPN handler — emitted by
/// the test-instrumented `UcanBlobsHandler::serve_request_for_test`
/// arm. The production wire-up delegates the bytes response to
/// `iroh_blobs::provider::handle_connection`.
#[derive(Debug, Clone)]
pub struct UcanBlobsResponse {
    /// The ciphertext_hash that was authorised + dispatched.
    pub ciphertext_hash: Cid,
}

/// Typed error envelope for the UCAN-gated iroh-blobs ALPN handler.
///
/// Each arm has a documented audit semantics + an explicit mapping
/// to the corresponding stable [`benten_errors::ErrorCode`] catalog
/// code (the wave-3e per-request rejection arms route through the
/// `E_UCAN_BLOBS_*` / `E_UNRESOLVED_PEER_DENY` / `E_CAP_UCAN_*`
/// surfaces; see `docs/ERROR-CATALOG.md`).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UcanBlobsHandlerError {
    /// Grant validation failed at the umbrella "request rejected"
    /// arm — the malformed-grant fixture path. Distinct from
    /// [`Self::BindingSigInvalid`] (specific binding-sig failure) so
    /// the wave-3e tests can disambiguate "we never got to
    /// binding-sig verification" from "binding-sig verification
    /// itself rejected." Maps to
    /// [`benten_errors::ErrorCode::UcanBlobsRequestRejected`].
    #[error("UCAN-blobs request rejected at grant validation: {reason}")]
    GrantValidation {
        /// Diagnostic note describing the rejection class.
        reason: String,
    },
    /// Binding-sig verification failed — covers A-1
    /// stolen-UCAN-without-keys, A-2 stolen-keys-without-UCAN, A-3
    /// wrong-audience-swap (the [`AuthorizationGrantError`] embedded
    /// names which arm). Maps to
    /// [`benten_errors::ErrorCode::AuthorizationGrantBindingSigInvalid`].
    #[error("binding signature invalid: {source}")]
    BindingSigInvalid {
        /// The underlying [`AuthorizationGrantError`].
        #[source]
        source: AuthorizationGrantError,
    },
    /// The connection's verified `EndpointId` (the requester's iroh
    /// peer ID = Ed25519 pubkey per Spike A2) does NOT match the
    /// UCAN audience. Covers the F-1 unauthorised-requester arm +
    /// A-2 audience-substitution attack. Maps to
    /// [`benten_errors::ErrorCode::CapUcanAudienceMismatch`].
    #[error("UCAN audience mismatch: connection EndpointId does not match UCAN audience")]
    UcanAudienceMismatch {
        /// Connection EndpointId bytes (the requester's pubkey).
        connection_endpoint_id: [u8; 32],
        /// UCAN audience pubkey bytes (the grant's bound audience).
        ucan_audience_pubkey: [u8; 32],
    },
    /// The grant's UCAN `exp` has passed relative to the injected
    /// clock. Covers the A-1 replay-attack-with-expired-UCAN arm.
    /// Maps to [`benten_errors::ErrorCode::CapUcanExpired`].
    #[error("UCAN expired: exp={exp_secs} < now={now_secs}")]
    UcanExpired {
        /// The grant's `exp` value (seconds-since-epoch).
        exp_secs: u64,
        /// The handler's injected `now` (seconds-since-epoch).
        now_secs: u64,
    },
    /// The grant's UCAN `nbf` is in the future relative to the
    /// injected clock. Defense-in-depth arm — symmetric mirror of
    /// [`Self::UcanExpired`]. Maps to
    /// [`benten_errors::ErrorCode::CapUcanNotYetValid`].
    #[error("UCAN not yet valid: nbf={nbf_secs} > now={now_secs}")]
    UcanNotYetValid {
        /// The grant's `nbf` value.
        nbf_secs: u64,
        /// The handler's injected `now`.
        now_secs: u64,
    },
    /// The grant has been revoked (recorded in the handler's
    /// revocation store via `UcanBlobsHandler::record_revocation_for_test`).
    /// Couples Phase-3 revocation infrastructure per §R6 reach
    /// (already-decrypted plaintext is NOT revoked; that's the
    /// documented cryptographic limit). Maps to
    /// [`benten_errors::ErrorCode::CapRevoked`].
    #[error("grant revoked: grant_cid={grant_cid}")]
    GrantRevoked {
        /// The revoked grant's CID.
        grant_cid: String,
    },
    /// The requested ciphertext_hash is NOT in the grant's
    /// [`benten_caps::restricted_spec::RestrictedScope`]'s `roots`
    /// allowlist. F-2 scope-check arm. Maps to
    /// [`benten_errors::ErrorCode::UcanBlobsRequestNotInScope`].
    #[error("requested ciphertext_hash {ciphertext_hash} NOT in granted RestrictedScope scope")]
    NotInScope {
        /// The requested hash that was out-of-scope.
        ciphertext_hash: Cid,
    },
    /// The grant references an unresolvable peer-DID — the
    /// sentinel pattern from `security-r1-2`. Couples §4.36 recheck +
    /// §4.25 sync-hydrate denial. Maps to
    /// [`benten_errors::ErrorCode::UnresolvedPeerDeny`]. NEVER admit;
    /// the sentinel exists to assert the handler short-circuits.
    #[error("grant references unresolvable peer-DID; refusing admission (security-r1-2)")]
    UnresolvedDeny {
        /// Sentinel marker carried through for diagnostic clarity.
        sentinel: &'static str,
    },
}

impl UcanBlobsHandlerError {
    /// Map to the corresponding stable [`benten_errors::ErrorCode`].
    /// Used at the engine-side fan-out boundary where the handler's
    /// typed error flows out as a catalog-stable code.
    ///
    /// The match arms below name `UcanBlobsHandlerError::Variant`
    /// explicitly (rather than the `Self::Variant` shorthand) so the
    /// `scripts/drift-detect.ts` reachability scanner records this
    /// function as the construction site for the named
    /// `ErrorCode::*` variants. The scanner's regex pattern requires
    /// `TypeName::Variant => ErrorCode::V` shape.
    #[must_use]
    pub fn error_code(&self) -> benten_errors::ErrorCode {
        match self {
            UcanBlobsHandlerError::GrantValidation { .. } => {
                benten_errors::ErrorCode::UcanBlobsRequestRejected
            }
            UcanBlobsHandlerError::BindingSigInvalid { .. } => {
                benten_errors::ErrorCode::AuthorizationGrantBindingSigInvalid
            }
            UcanBlobsHandlerError::UcanAudienceMismatch { .. } => {
                benten_errors::ErrorCode::CapUcanAudienceMismatch
            }
            UcanBlobsHandlerError::UcanExpired { .. } => benten_errors::ErrorCode::CapUcanExpired,
            UcanBlobsHandlerError::UcanNotYetValid { .. } => {
                benten_errors::ErrorCode::CapUcanNotYetValid
            }
            UcanBlobsHandlerError::GrantRevoked { .. } => benten_errors::ErrorCode::CapRevoked,
            UcanBlobsHandlerError::NotInScope { .. } => {
                benten_errors::ErrorCode::UcanBlobsRequestNotInScope
            }
            UcanBlobsHandlerError::UnresolvedDeny { .. } => {
                benten_errors::ErrorCode::UnresolvedPeerDeny
            }
        }
    }
}

/// The wave-3e UCAN-gated iroh-blobs ALPN handler.
///
/// Owns:
/// - The handler's own keypair (the Atrium identity that ISSUED the
///   grants this handler will validate — used as the audience-pubkey
///   the binding-sig was signed against). At the wave-3e fixtures
///   the handler-keypair is the grant issuer; the production wire-up
///   accepts grants issued by other principals through the chain
///   validator at the cap-policy hook.
/// - The injected wall-clock (`now_secs`) — handler-local so the
///   wave-3e expiry/nbf pins can inject arbitrary times without
///   wall-clock dependencies. The production wire-up reads
///   `SystemTime::now()` at handler-construction time.
/// - The revocation store — set of grant CIDs that have been
///   explicitly revoked. The production wire-up plumbs through the
///   `benten-id::ucan` revocation surface.
/// - A test-instrumented dispatch counter so the wave-3e pins can
///   assert the handler dispatches to iroh-blobs ONLY after positive
///   validation.
#[derive(Debug)]
pub struct UcanBlobsHandler {
    /// The handler's keypair (NOT used for signing — the handler
    /// validates grants signed by OTHER issuers via the embedded
    /// `issuer_verifying_key` on the grant. Carried here so the
    /// handler has a stable identity for the iroh peer layer.)
    #[allow(dead_code)]
    handler_pubkey: PublicKey,
    /// The injected wall-clock — seconds-since-epoch. `u64::MAX`
    /// effectively disables the expiry check (matches the wave-3b
    /// default behaviour where grants have `exp = u64::MAX`).
    now_secs: u64,
    /// Revocation store — grant CIDs that have been explicitly
    /// revoked. Wave-3e in-memory; production swap-point to a
    /// proper revocation backend.
    revocations: Mutex<BTreeSet<Cid>>,
    /// Test-instrumented counter — increments only on positive
    /// dispatch paths so the wave-3e pin
    /// `tf3e_dispatch_to_iroh_blobs_only_after_positive_validation`
    /// can assert the seam fires only after validation passes.
    dispatch_count: Mutex<usize>,
}

impl UcanBlobsHandler {
    /// Construct a handler bound to the supplied keypair, with the
    /// wall-clock set to epoch (`now = 0`). Wave-3e fixtures use
    /// `exp = u64::MAX` + `nbf = 0` defaults so the `nbf <= now < exp`
    /// time-bound check passes trivially under `now = 0`. Tests that
    /// exercise the expiry/nbf arms use [`Self::new_with_clock`]
    /// with explicit injected times.
    ///
    /// `now = 0` rather than `u64::MAX`: the strict-less-than
    /// expiry check (`now < exp`) requires `now < exp` to pass; with
    /// `now = u64::MAX` AND `exp = u64::MAX` the check would
    /// FALSELY fire `UcanExpired` (saturating overflow into the
    /// expired arm). The epoch default safely says "no production
    /// clock injected"; production wire-up calls
    /// [`Self::new_with_clock`] with a real `SystemTime::now()`-
    /// derived value.
    #[must_use]
    pub fn new(handler_keypair: &Keypair) -> Self {
        Self::new_with_clock(handler_keypair, 0)
    }

    /// Construct a handler with an explicit `now_secs` (seconds-
    /// since-epoch). Used by the wave-3e replay-attack pins to
    /// inject past/future clocks for the expiry/nbf arms.
    #[must_use]
    pub fn new_with_clock(handler_keypair: &Keypair, now_secs: u64) -> Self {
        Self {
            handler_pubkey: handler_keypair.public_key().clone(),
            now_secs,
            revocations: Mutex::new(BTreeSet::new()),
            dispatch_count: Mutex::new(0),
        }
    }

    /// G-CORE-3e helper — record a grant's revocation. The handler
    /// observes this via [`Self::validate_request_for_connection`]'s
    /// revocation arm. Couples Phase-3 revocation infrastructure
    /// per §R6 reach (cuts FUTURE serves; already-derived plaintext
    /// remains decryptable — that's the documented cryptographic
    /// limit at the recipient side).
    #[cfg(any(test, feature = "testing"))]
    pub fn record_revocation_for_test(&self, grant_cid: &Cid) {
        self.revocations
            .lock()
            .expect("UcanBlobsHandler revocations mutex poisoned")
            .insert(*grant_cid);
    }

    /// G-CORE-3e test seam — read the dispatch counter.
    #[doc(hidden)]
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub fn dispatch_count_for_test(&self) -> usize {
        *self
            .dispatch_count
            .lock()
            .expect("UcanBlobsHandler dispatch_count mutex poisoned")
    }

    /// G-CORE-3e: validate a request WITHOUT a connection-EndpointId
    /// constraint. Used by tests that exercise the
    /// not-yet-bound-to-a-specific-connection arm (e.g. the
    /// per-request validation pin that rejects a malformed grant
    /// before any audience check). Production callers should use
    /// [`Self::validate_request_for_connection`] (audience-binding
    /// is the §R3 contract; skipping it is a confidentiality hazard).
    ///
    /// Internally delegates to
    /// [`Self::validate_request_for_connection`] using the grant's
    /// own `audience_pubkey` as the connection EndpointId — this
    /// short-circuits the audience-mismatch arm but still exercises
    /// binding-sig + expiry + revocation + scope.
    ///
    /// # Errors
    ///
    /// Returns any [`UcanBlobsHandlerError`] variant per the
    /// validation pipeline.
    pub fn validate_request(
        &self,
        request: &UcanBlobsRequest,
    ) -> Result<(), UcanBlobsHandlerError> {
        // Use the grant's own audience_pubkey as the connection
        // EndpointId for this validation pass. This is the
        // wave-3e "no connection context" arm — production callers
        // ALWAYS use validate_request_for_connection with the real
        // EndpointId from iroh.
        let conn_pubkey_bytes: [u8; 32] = request
            .grant
            .audience_pubkey
            .as_deref()
            .and_then(|b| <[u8; 32]>::try_from(b).ok())
            .unwrap_or([0u8; 32]);
        let conn_pubkey = PublicKey::from_bytes(&conn_pubkey_bytes).ok_or_else(|| {
            UcanBlobsHandlerError::GrantValidation {
                reason: "grant audience_pubkey absent or malformed (32-byte Ed25519 required)"
                    .to_string(),
            }
        })?;
        self.validate_request_for_connection(request, &conn_pubkey)
    }

    /// G-CORE-3e: validate a request bound to a specific connection
    /// EndpointId. The verified `EndpointId` (from the iroh QUIC
    /// layer) IS the requester's Ed25519 pubkey per Spike A2's
    /// zero-conversion plumbing — no parsing, no conversion.
    ///
    /// # Errors
    ///
    /// Returns the appropriate [`UcanBlobsHandlerError`] variant on
    /// any per-request validation failure (see module-level
    /// pipeline documentation for the ordered arms).
    pub fn validate_request_for_connection(
        &self,
        request: &UcanBlobsRequest,
        connection_endpoint_id: &PublicKey,
    ) -> Result<(), UcanBlobsHandlerError> {
        // ARM 1 — unresolvable peer-DID short-circuit. Couples
        // security-r1-2 + §4.36 recheck + §4.25 sync-hydrate denial.
        // FIRST because if we can't even resolve who's asking we
        // can't validate ANY subsequent arm; silent admission
        // through any arm would defeat the audience-binding (§R3)
        // defense.
        if request.grant.ucan.unresolved_peer {
            return Err(UcanBlobsHandlerError::UnresolvedDeny {
                sentinel: "<unresolved-peer>",
            });
        }

        // ARM 2 — audience binding: connection EndpointId == UCAN
        // audience pubkey. THIS IS the Spike A2 zero-conversion
        // arm: both sides are 32-byte Ed25519 pubkey
        // representations; the comparison is a direct byte-compare.
        let conn_bytes = connection_endpoint_id.to_bytes();
        let audience_bytes_vec = request.grant.audience_pubkey.as_ref().ok_or_else(|| {
            UcanBlobsHandlerError::GrantValidation {
                reason: "grant has no audience_pubkey — wave-3e grants must carry the audience \
                         pubkey for connection-EndpointId comparison"
                    .to_string(),
            }
        })?;
        let audience_bytes: [u8; 32] = audience_bytes_vec.as_slice().try_into().map_err(|_| {
            UcanBlobsHandlerError::GrantValidation {
                reason: "grant audience_pubkey has non-Ed25519 length (must be 32 bytes)"
                    .to_string(),
            }
        })?;
        if conn_bytes != audience_bytes {
            return Err(UcanBlobsHandlerError::UcanAudienceMismatch {
                connection_endpoint_id: conn_bytes,
                ucan_audience_pubkey: audience_bytes,
            });
        }

        // ARM 3 — UCAN time-bound validity (nbf <= now < exp).
        if request.grant.ucan.nbf_secs > self.now_secs {
            return Err(UcanBlobsHandlerError::UcanNotYetValid {
                nbf_secs: request.grant.ucan.nbf_secs,
                now_secs: self.now_secs,
            });
        }
        if request.grant.ucan.exp_secs <= self.now_secs {
            return Err(UcanBlobsHandlerError::UcanExpired {
                exp_secs: request.grant.ucan.exp_secs,
                now_secs: self.now_secs,
            });
        }

        // ARM 4 — revocation observance.
        let grant_cid = request.grant.grant_cid_for_test();
        if self
            .revocations
            .lock()
            .expect("UcanBlobsHandler revocations mutex poisoned")
            .contains(&grant_cid)
        {
            return Err(UcanBlobsHandlerError::GrantRevoked {
                grant_cid: format!("{grant_cid}"),
            });
        }

        // ARM 5 — binding-sig verification (delegates to wave-3b's
        // AuthorizationGrant::verify_binding which covers A-1/A-2/A-3
        // uniformly). The audience check inside verify_binding is
        // against the grant's `audience_binding` Cid — we use the
        // grant's own audience_binding so the binding-sig check
        // doesn't conflate with the EndpointId check (which is
        // ARM 2 above).
        request
            .grant
            .verify_binding(request.grant.audience_binding)
            .map_err(|source| UcanBlobsHandlerError::BindingSigInvalid { source })?;

        // ARM 6 — scope check. The grant MUST authorise the
        // requested ciphertext_hash via the carried RestrictedScope's
        // roots allowlist (per `with_hashes` constructor).
        let scope =
            request
                .grant
                .scope
                .as_ref()
                .ok_or_else(|| UcanBlobsHandlerError::GrantValidation {
                    reason: "grant has no scope — wave-3e grants must carry a RestrictedScope"
                        .to_string(),
                })?;
        let in_scope = match scope.roots.as_deref() {
            // No roots constraint = no scope-allowlist set; treat as
            // fail-closed (the wave-3e contract is "explicit hash
            // allowlist"; a `roots: None` would semantically mean
            // "any hash" which is wrong for the per-recipient
            // bytes-serving arm).
            None => false,
            Some(allowed) => allowed.contains(&request.ciphertext_hash),
        };
        if !in_scope {
            return Err(UcanBlobsHandlerError::NotInScope {
                ciphertext_hash: request.ciphertext_hash,
            });
        }

        // All six arms passed — caller may now dispatch to
        // iroh-blobs. The dispatch itself is NOT performed here
        // (this is a validate-only surface); callers use
        // `serve_request_for_test` (test) / the production
        // dispatch wire-up (real iroh-blobs) which delegate to
        // `iroh_blobs::provider::handle_connection`.
        Ok(())
    }

    /// G-CORE-3e test seam — validate + dispatch. Increments the
    /// test-instrumented dispatch counter on positive paths so the
    /// wave-3e pins can assert the iroh-blobs dispatch fires only
    /// after positive validation. The production wire-up calls
    /// `iroh_blobs::provider::handle_connection` after a positive
    /// `validate_request_for_connection`.
    ///
    /// # Errors
    ///
    /// Returns the validation error if any arm of the pipeline
    /// rejects; otherwise increments the dispatch counter +
    /// returns a synthetic [`UcanBlobsResponse`].
    #[cfg(any(test, feature = "testing"))]
    pub fn serve_request_for_test(
        &self,
        request: UcanBlobsRequest,
    ) -> Result<UcanBlobsResponse, UcanBlobsHandlerError> {
        self.validate_request(&request)?;
        // Positive validation — increment the dispatch counter +
        // synthesize a response. Production wire-up dispatches to
        // `iroh_blobs::provider::handle_connection` here instead.
        let mut count = self
            .dispatch_count
            .lock()
            .expect("UcanBlobsHandler dispatch_count mutex poisoned");
        *count = count
            .checked_add(1)
            .expect("UcanBlobsHandler dispatch_count overflow (>2^64 requests is implausible)");
        Ok(UcanBlobsResponse {
            ciphertext_hash: request.ciphertext_hash,
        })
    }

    /// G-CORE-3e: derive an iroh `EndpointId` from a `PublicKey`
    /// WITHOUT parsing — Spike A2's zero-conversion identity-cast
    /// (`PublicKey` is also a 32-byte Ed25519 encoding; iroh's
    /// `EndpointId = iroh_base::PublicKey` is byte-identical).
    /// Returns the byte-identical [`EndpointIdBytes`] handle.
    #[must_use]
    pub fn endpoint_id_from_public_key_no_parsing(public_key: &PublicKey) -> EndpointIdBytes {
        EndpointIdBytes(public_key.to_bytes())
    }
}

/// G-CORE-3e: byte-identical handle to an iroh `EndpointId`.
///
/// Per Spike A2: iroh's `EndpointId` IS `ed25519_dalek::VerifyingKey`
/// at the byte-encoding level (both are 32-byte Ed25519 public-key
/// representations). This newtype carries the 32 bytes plus the
/// `to_bytes_for_test` accessor the wave-3e zero-conversion pin
/// uses to assert the bytes are byte-identical.
///
/// Production wire-up reaches the iroh API directly through
/// [`verifying_key_to_endpoint_id`] / [`endpoint_id_to_verifying_key`].
#[derive(Debug, Clone, Copy)]
pub struct EndpointIdBytes(pub [u8; 32]);

impl EndpointIdBytes {
    /// G-CORE-3e test helper — return the 32-byte form for
    /// byte-identity comparison.
    #[doc(hidden)]
    #[must_use]
    #[cfg(any(test, feature = "testing"))]
    pub const fn to_bytes_for_test(&self) -> [u8; 32] {
        self.0
    }
}

/// G-CORE-3e: identity-cast from `VerifyingKey` to iroh `EndpointId`-
/// shaped bytes. Spike A2 zero-conversion contract: both encode the
/// same 32 bytes; the cast is a `.to_bytes()` call, NOT a parse.
#[must_use]
pub fn verifying_key_to_endpoint_id(vk: &VerifyingKey) -> EndpointIdBytes {
    EndpointIdBytes(vk.to_bytes())
}

/// G-CORE-3e: identity-cast from iroh `EndpointId`-shaped bytes back
/// to `VerifyingKey`. Round-trip byte-identical with
/// [`verifying_key_to_endpoint_id`] (the round-trip pin asserts the
/// contract).
///
/// # Errors
///
/// Returns `None` if the 32 bytes do not form a valid Edwards point
/// (the underlying `VerifyingKey::from_bytes` rejects malformed
/// keys — distinct from "parsing failure" since the bytes are
/// SUPPOSED to come from a real iroh endpoint that already
/// validated them; this fallback is for the round-trip test +
/// future production callers that receive raw bytes off the wire).
#[must_use]
pub fn endpoint_id_to_verifying_key(endpoint_id: &EndpointIdBytes) -> Option<VerifyingKey> {
    VerifyingKey::from_bytes(&endpoint_id.0).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_id_round_trip_is_byte_identical() {
        let kp = Keypair::generate();
        let vk = kp.public_key_verifying_key_for_test();
        let eid = verifying_key_to_endpoint_id(&vk);
        let recovered = endpoint_id_to_verifying_key(&eid).expect("round-trip ok");
        assert_eq!(vk.to_bytes(), recovered.to_bytes());
    }

    #[test]
    fn audience_bytes_via_handler_helper_match_pubkey() {
        let kp = Keypair::generate();
        let pk = kp.public_key();
        let eid = UcanBlobsHandler::endpoint_id_from_public_key_no_parsing(pk);
        assert_eq!(eid.to_bytes_for_test(), pk.to_bytes());
    }
}
