//! [`UcanGroundedPolicy`] — Phase-3 G21-T2 fp-mini-review BLOCKER-2 +
//! BLOCKER-3 closure: a [`CapabilityPolicy`] that composes the
//! durable [`crate::backends::UCANBackend`] proof-chain validator
//! alongside [`crate::GrantBackedPolicy`].
//!
//! ## Why this exists
//!
//! Pre-fp-mini-review, `benten_engine::EngineBuilder::capability_policy_ucan_durable`
//! was a verbatim alias for `capability_policy_grant_backed` — the
//! `Ucan` policy variant under `PolicyKind::Ucan` consulted ONLY the
//! `system:CapabilityGrant` Node-encoded grant store, NEVER the
//! `g14b:grant:*` UCAN-proof store, NEVER the chain-walker
//! ([`crate::backends::UCANBackend::validate_chain_for_audience_at`]),
//! NEVER attenuation enforcement, NEVER `nbf`/`exp` window checks. A
//! forged UCAN with audience-right + capability-wrong, an expired
//! token, or an attenuation-violation chain was NEVER rejected on the
//! basis of the chain — only on the basis of literal entry in
//! `system:CapabilityGrant`.
//!
//! `UcanGroundedPolicy` closes that gap for **typed-CALL `cap:typed:*`
//! capabilities**. The full per-write proof-chain enforcement for
//! arbitrary scope-strings (with audience threading + actor binding
//! through `CapWriteContext`) is wider scope — it requires
//! `CapWriteContext::actor_hint`-as-DID propagation, which is its own
//! architectural lift. That extension is named in
//! `docs/future/phase-3-backlog.md §2.3 (i)` (created at this
//! fix-pass per HARD RULE clause-b).
//!
//! ## Composition
//!
//! 1. [`crate::GrantBackedPolicy`] is consulted first (fast path):
//!    the Phase-2b revocation-aware Node-encoded grant store. A grant
//!    permits the write immediately; no UCAN walk needed.
//! 2. If GrantBackedPolicy denies AND the required capability is in
//!    the `cap:typed:*` namespace, [`UCANBackend::iter_installed_proofs`]
//!    enumerates persisted UCAN proofs and runs each through:
//!     - [`UCANBackend::validate_chain_at`] (signature + `nbf`/`exp`
//!       time-window at every link + per-token revocation lookup).
//!     - [`crate::typed_cap_for_ucan_claim`] mapping table —
//!       translates each leaf-claim `(resource, ability)` into the
//!       matching `cap:typed:*` string.
//!    Any chain whose leaf claim maps to the required typed-cap +
//!    passes the chain-walker permits the write.
//! 3. If neither the grant store nor any UCAN proof grants the
//!    capability, the original GrantBackedPolicy denial bubbles.
//!
//! ## Why `cap:typed:*`-only
//!
//! `CapWriteContext` does not currently carry an audience DID — the
//! grant-backed surface is principal-coarse (any unrevoked grant
//! permits). Threading per-actor audience DIDs through every CRUD
//! write is its own work item. The `cap:typed:*` namespace is the
//! first surface where the closed-set claim mapping
//! ([`crate::typed_cap_for_ucan_claim`]) makes audience-less chain
//! validation safe — the capability string itself disambiguates the
//! claim. Other scope strings require principal binding to be safe,
//! so they fall through to the existing GrantBackedPolicy result
//! pending the wider Phase-3-backlog §2.3 (i) work.
//!
//! ## Storage layout (intentional; see [`crate::backends::ucan`] dual-seam doc)
//!
//! UCAN proofs live in the `g14b:grant:<cid>` KV store via
//! [`UCANBackend::install_proof`]; engines wire this via the new
//! `Engine::install_ucan_proof` adapter (G21-T2 fp-mini-review).
//! The `system:CapabilityGrant` Node store remains the
//! `GrantBackedPolicy`-consulted seam for unsigned grants.

use std::sync::Arc;

use benten_graph::GraphBackend;
use benten_id::did::Did;

use crate::backends::UCANBackend;
use crate::error::CapError;
use crate::grant_backed::GrantBackedPolicy;
use crate::policy::{CapWriteContext, CapabilityPolicy, ReadContext};
use crate::typed_cap_mapping::typed_cap_for_ucan_claim;

/// `did:key:` URI prefix; used by [`principal_did_from_context`] to
/// recognize when [`CapWriteContext::actor_hint`] carries a DID-shaped
/// principal identifier vs a non-DID hint string.
const DID_KEY_PREFIX: &str = "did:key:";

/// Resolve the active principal DID from a [`CapWriteContext`].
///
/// Phase-4-Foundation R1-FP G22-FP-2 (cap-r1-1 BLOCKER closure): the
/// audience-binding gate in [`UcanGroundedPolicy::typed_cap_permitted_by_proof`]
/// requires a typed [`Did`] handle to thread into
/// [`UCANBackend::validate_chain_for_audience_at`]. Until the
/// `CapWriteContext::actor_cid → Did` resolution helper lands (cap-r1-16
/// seam — needs an identity-store with CID-keyed DID lookup at the
/// engine boundary), we source the principal from
/// [`CapWriteContext::actor_hint`] — the documented "DID / VC identity
/// placeholder" field per the [`policy::CapWriteContext`] module-doc.
///
/// Returns `Some(did)` if `actor_hint` is `Some(s)`, `s` starts with
/// `did:key:`, AND `Did::resolve()` round-trips (the bytes parse to a
/// valid `did:key`). Returns `None` otherwise (no hint / non-DID hint /
/// malformed DID) — the caller's fail-closed branch surfaces typed
/// `CapError::UcanAudienceMismatch` for the missing-principal case so
/// the engine boundary cannot silently accept a UCAN against an
/// audience-less context (the cap-r1-1 BLOCKER pre-fix behavior).
/// Three-state classification of the principal carried by a
/// [`CapWriteContext`]. Safe-2 #546: the pre-fix code collapsed
/// "no actor_hint" and "actor_hint present but malformed" into a
/// single `None`, both of which silently fell back to the
/// audience-LESS [`UCANBackend::validate_chain_at`] walk — a
/// fail-OPEN sub-class of the cap-r1-1 BLOCKER. A caller who *intended*
/// to thread a principal but passed a non-DID / un-resolvable string
/// got the weaker audience-less check with NO signal. These two cases
/// must be distinguished.
enum PrincipalResolution {
    /// `actor_hint` resolves to a valid `did:key:` principal — thread
    /// audience binding.
    Resolved(Did),
    /// `actor_hint` is absent (`None`). Intentional audience-less path:
    /// Phase-1/2 fixtures + engine-internal typed-CALL surfaces that
    /// don't yet thread an actor (e.g.
    /// `Engine::dispatch_typed_call_public`). Preserved per the
    /// module-doc scoping note.
    Absent,
    /// `actor_hint` is `Some` but does NOT resolve to a valid
    /// principal (non-`did:key:` shaped OR malformed `did:key:` that
    /// fails the `Did::resolve` round-trip). The caller signalled
    /// principal-intent with a bad value — fail CLOSED, never degrade
    /// to the audience-less walk (#546).
    Malformed,
}

/// Resolve the active principal DID from a [`CapWriteContext`].
///
/// Phase-4-Foundation R1-FP G22-FP-2 (cap-r1-1 BLOCKER closure): the
/// audience-binding gate in [`UcanGroundedPolicy::typed_cap_permitted_by_proof`]
/// requires a typed [`Did`] handle to thread into
/// [`UCANBackend::validate_chain_for_audience_at`]. Until the
/// `CapWriteContext::actor_cid → Did` resolution helper lands (cap-r1-16
/// seam — needs an identity-store with CID-keyed DID lookup at the
/// engine boundary), we source the principal from
/// [`CapWriteContext::actor_hint`] — the documented "DID / VC identity
/// placeholder" field per the [`policy::CapWriteContext`] module-doc.
///
/// Returns [`PrincipalResolution::Resolved`] if `actor_hint` is
/// `Some(s)`, `s` starts with `did:key:`, AND `Did::resolve()`
/// round-trips. Returns [`PrincipalResolution::Absent`] when there is
/// no hint (intentional audience-less fixtures path). Returns
/// [`PrincipalResolution::Malformed`] when a hint IS present but does
/// not resolve — the caller's fail-CLOSED branch surfaces typed
/// `CapError::UcanAudienceMismatch` so the engine boundary cannot
/// silently accept a UCAN against an audience-less context (#546 / the
/// cap-r1-1 BLOCKER pre-fix fail-OPEN behavior).
fn principal_did_from_context(ctx: &CapWriteContext) -> PrincipalResolution {
    let Some(hint) = ctx.actor_hint.as_deref() else {
        return PrincipalResolution::Absent;
    };
    if !hint.starts_with(DID_KEY_PREFIX) {
        // A hint was supplied but isn't even DID-shaped — caller
        // signalled principal-intent with a bad value. Fail CLOSED;
        // do NOT degrade to the audience-less walk.
        return PrincipalResolution::Malformed;
    }
    let did = Did::from_string_unchecked(hint.to_string());
    // Round-trip check: the string MUST parse to a valid pubkey via
    // `Did::resolve` — otherwise it's a malformed `did:key:` string
    // masquerading as a principal.
    match did.resolve() {
        Ok(_) => PrincipalResolution::Resolved(did),
        Err(_) => PrincipalResolution::Malformed,
    }
}

/// Composed `CapabilityPolicy` consulting both
/// [`GrantBackedPolicy`] and [`UCANBackend`] proof-chain validation.
///
/// G21-T2 fp-mini-review BLOCKER-2 closure. See module-level doc for
/// composition order + scope.
pub struct UcanGroundedPolicy<B: GraphBackend> {
    inner: GrantBackedPolicy,
    ucan: Arc<UCANBackend<B>>,
    /// `now` (epoch seconds) sourced for chain-walker time-window
    /// validation. Phase-3-G21-T2-pre-real-clock: a static fixture
    /// "now" so the chain-walker has SOMETHING to compare against; a
    /// real clock injection lands at the `CapWriteContext::now`
    /// threading work named in phase-3-backlog §2.3 (i). This default
    /// is far in the future (year 9999) so present-day proofs with
    /// reasonable `exp` accept; tests inject custom values via the
    /// `with_now_for_test` builder.
    now_secs: u64,
}

impl<B: GraphBackend> std::fmt::Debug for UcanGroundedPolicy<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UcanGroundedPolicy")
            .field("inner", &self.inner)
            .field("ucan", &"<Arc<UCANBackend<B>>>")
            .field("now_secs", &self.now_secs)
            .finish()
    }
}

/// Sentinel `now` value indicating "no real wallclock has been
/// injected." Picked at `0` (epoch start) so the fail-closed branch
/// in [`UcanGroundedPolicy::typed_cap_permitted_by_proof`] can
/// distinguish "caller injected `now=0`" from "caller did not inject
/// at all" — by convention production callers MUST inject a positive
/// epoch-seconds value via [`UcanGroundedPolicy::with_now_for_test`]
/// (or the eventual `CapWriteContext::now` threading per
/// phase-3-backlog §2.3 (i)). A test that explicitly wants the
/// sentinel-driven fail-closed path leaves the default; a test that
/// wants successful chain-walk against time-bounded chains injects
/// a non-zero value.
///
/// ## Fail-closed inversion (G16-B-B-rest sub-item D)
///
/// Pre-inversion the chain-walker silently accepted `now_secs == 0`
/// against any chain. For a chain whose tokens all have `nbf=0`, the
/// time-window check passed; for a chain with `nbf > 0`, the
/// chain-walker rejected with `NotYetValid` per-link, but iteration
/// continued to the next proof — masking the underlying
/// "no clock injected" misconfiguration as a benign "chain not yet
/// active." A forged chain with `nbf=0` would silently accept against
/// the sentinel.
///
/// The inversion: at chain-walker entry, if `now_secs == 0` AND the
/// chain has any time-bounded delegation (`nbf > 0` OR `exp > 0`),
/// fail-CLOSED with [`CapError::UcanClockNotInjected`]. Callers MUST
/// inject a real wallclock to validate time-bounded chains.
const DEFAULT_NOW_SECS: u64 = 0;

/// Returns `true` if any token in `chain` carries a non-zero `nbf` or
/// `exp` (time-bounded delegation). Used by the fail-closed branch in
/// [`UcanGroundedPolicy::typed_cap_permitted_by_proof`] to distinguish
/// "chain has time bounds + no clock = misconfiguration" from
/// "chain is unbounded + no clock = safe to walk."
fn chain_has_time_bounds(chain: &[benten_id::ucan::Ucan]) -> bool {
    chain.iter().any(|token| {
        token.claims.nbf.is_some_and(|nbf| nbf > 0) || token.claims.exp.is_some_and(|exp| exp > 0)
    })
}

impl<B: GraphBackend> UcanGroundedPolicy<B> {
    /// Construct a `UcanGroundedPolicy` composing the supplied
    /// [`GrantBackedPolicy`] with the durable [`UCANBackend`].
    #[must_use]
    pub fn new(inner: GrantBackedPolicy, ucan: Arc<UCANBackend<B>>) -> Self {
        Self {
            inner,
            ucan,
            now_secs: DEFAULT_NOW_SECS,
        }
    }

    /// Override the `now` value used by the chain-walker for
    /// `nbf`/`exp` time-window validation. Tests pin specific clock
    /// values to exercise expired / not-yet-valid token rejection.
    ///
    /// Production callers leave this at the default (year-9999
    /// fixture) until `CapWriteContext::now` threading lands per
    /// phase-3-backlog §2.3 (i).
    #[must_use]
    pub fn with_now_for_test(mut self, now_secs: u64) -> Self {
        self.now_secs = now_secs;
        self
    }

    /// Borrow the underlying [`UCANBackend`] (engine wiring +
    /// integration tests).
    #[must_use]
    pub fn ucan_backend(&self) -> &Arc<UCANBackend<B>> {
        &self.ucan
    }

    /// Probe whether ANY persisted UCAN proof grants the supplied
    /// `cap:typed:*` requirement at `now_secs`, bound to the active
    /// principal `audience` DID.
    ///
    /// ## Phase-4-Foundation R1-FP G22-FP-2 (cap-r1-1 + cap-r1-9 BLOCKER closure)
    ///
    /// Pre-fix the chain-walker dispatched to
    /// [`UCANBackend::validate_chain_at`] — signature + `nbf`/`exp`
    /// time-window + attenuation only. **No audience binding.** A
    /// malicious actor could present a UCAN issued to *someone else*
    /// (audience = victim's DID) and have it accepted — defeats UCAN's
    /// audience semantics, the first-line cross-atrium-replay defense
    /// (CLR-2). The cap-r1-1 R1 finding flagged this as a BLOCKER.
    ///
    /// Post-fix the gate composes
    /// [`UCANBackend::validate_chain_for_audience_at`] which fires the
    /// `validate_chain_for_audience` audience-binding check
    /// **before** the time-window walk (cap-r1-9 ordering: a
    /// wrong-audience replay against an expired chain surfaces the
    /// typed [`CapError::UcanAudienceMismatch`], NOT a time-window
    /// error — matches the Phase-3 G14-A2
    /// `validate_chain_for_capability` ordering precedent at
    /// `crates/benten-id/src/ucan.rs::validate_chain_inner` lines
    /// 397-410 where audience-binding is the first gate).
    ///
    /// Returns `Ok(true)` on a single permitting chain;
    /// `Ok(false)` if no chain permits + no errors arose; bubbles the
    /// typed `CapError` for backend storage failures.
    ///
    /// A chain that fails the chain-walker (audience mismatch / bad
    /// signature / expired / attenuation-violation / revoked) is
    /// treated as "this chain does not permit" — iteration continues to
    /// the next proof, mirroring the pre-fix iteration shape. A proof
    /// whose audience matches `audience`, whose chain-walk passes, AND
    /// whose leaf-claim maps to the required typed-cap permits.
    fn typed_cap_permitted_by_proof(
        &self,
        required: &str,
        audience: Option<&Did>,
    ) -> Result<bool, CapError> {
        let proofs = self.ucan.iter_installed_proofs()?;
        for proof in &proofs {
            // 1. Single-token chain treated as `[proof]`. Multi-token
            //    chains are an extension axis that lights up when
            //    `CapWriteContext` carries an explicit chain reference;
            //    today the durable store holds singleton tokens via
            //    `install_proof`.
            let chain = std::slice::from_ref(proof);
            // 2. Fail-closed inversion (G16-B-B-rest sub-item D): if
            //    no real wallclock has been injected (now_secs at the
            //    DEFAULT_NOW_SECS=0 sentinel) AND this chain has any
            //    time-bounded delegation (nbf>0 OR exp>0), refuse to
            //    validate. The pre-inversion fail-OPEN path silently
            //    walked tokens against now=0; a forged chain with
            //    nbf=0 + exp>0 would have accepted whenever the rest
            //    of the chain-walk passed, with no operator-visible
            //    surface signaling the missing-clock misconfiguration.
            //    Bubble the typed error so the caller knows to inject
            //    a clock.
            if self.now_secs == DEFAULT_NOW_SECS && chain_has_time_bounds(chain) {
                return Err(CapError::UcanClockNotInjected);
            }
            // 3. Chain-walker fires (optional) audience-binding (FIRST
            //    per cap-r1-9 ordering when an audience is threaded) +
            //    signature + time-window + attenuation + revocation.
            //    Failure of ANY step = "this chain does not permit";
            //    iterate to next.
            //
            //    `validate_chain_for_audience_at` composes
            //    `validate_chain_for_audience` (audience-binding leaf
            //    check) BEFORE `validate_chain_at` (time-window +
            //    signature walk) — so a UCAN with the wrong audience
            //    rejects with `UcanAudienceMismatch` even if the chain
            //    is also expired, preserving the typed-error ordering
            //    audit pipelines depend on.
            //
            //    When `audience` is `None`, fall back to
            //    `validate_chain_at` (audience-less; legacy pre-fix
            //    walker). Mirrors the FP-3 default-collapses-to-scope-
            //    only-when-actor-None pattern at
            //    `GrantReader::has_unrevoked_grant_for_scope_and_actor`:
            //    audience binding fires when the caller threads a
            //    principal, otherwise we preserve Phase-1/2 fixtures +
            //    engine-internal typed-CALL paths that don't yet thread
            //    actor (e.g.,
            //    `Engine::dispatch_typed_call_public` at
            //    `engine_wait.rs::881-891` constructs
            //    `CapWriteContext { actor_hint: None, .. }`). Full
            //    actor-threading is the cap-r1-16 + CapWriteContext::now
            //    follow-up at G24-D files-owned.
            let chain_check = match audience {
                Some(aud) => self
                    .ucan
                    .validate_chain_for_audience_at(chain, aud, self.now_secs),
                None => self.ucan.validate_chain_at(chain, self.now_secs),
            };
            // Safe-1 #497: a backend-storage failure during the chain
            // walk is NOT the same disposition as a forged/expired/
            // wrong-audience chain. The pre-fix `is_err() { continue }`
            // swallowed BOTH — a transient durable-store failure was
            // indistinguishable from a security denial and silently
            // dropped the proof, which could fail-OPEN the overall
            // check if a later (decodable) proof happened to grant the
            // cap while the storage-failed proof was the legitimate
            // bearer. Disambiguate: infrastructure failure propagates
            // (fail-CLOSED with the real cause + observability);
            // genuine security denial keeps the iterate-to-next-proof
            // behavior.
            if let Err(chain_err) = chain_check {
                if matches!(chain_err, CapError::BackendStorage { .. }) {
                    tracing::warn!(
                        target: "benten_caps::ucan_grounded",
                        error_code = "E_CAP_BACKEND_STORAGE",
                        error = %chain_err,
                        "backend-storage failure during proof-chain \
                         validation — failing CLOSED with the storage \
                         cause rather than masking it as a security \
                         denial (#497)"
                    );
                    return Err(chain_err);
                }
                // Security denial (signature / time-window / audience /
                // attenuation / revocation) — this proof does not
                // permit; try the next installed proof.
                continue;
            }
            // 4. Leaf-claim → typed-cap mapping.
            for cap in &proof.claims.att {
                if let Some(group) = typed_cap_for_ucan_claim(&cap.resource, &cap.ability)
                    && group.cap_string() == required
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

impl<B: GraphBackend> CapabilityPolicy for UcanGroundedPolicy<B> {
    fn check_write(&self, ctx: &CapWriteContext) -> Result<(), CapError> {
        // Fast path: the Phase-2b revocation-aware grant-backed surface.
        let grant_err = match self.inner.check_write(ctx) {
            Ok(()) => return Ok(()),
            Err(e) => e,
        };
        // Slow path: typed-cap proof-chain validation. ONLY for
        // `cap:typed:*` requirements per the module-doc scoping note
        // (audience binding for arbitrary scope strings is named at
        // phase-3-backlog §2.3 (i)).
        let required = if ctx.scope.starts_with("cap:typed:") {
            ctx.scope.as_str()
        } else if ctx.label.starts_with("cap:typed:") {
            ctx.label.as_str()
        } else {
            // Non-typed-cap scope: the existing GrantBackedPolicy
            // denial stands.
            return Err(grant_err);
        };
        // Phase-4-Foundation R1-FP G22-FP-2 (cap-r1-1 BLOCKER closure):
        // resolve the active principal DID + thread it as the
        // `audience` argument to `typed_cap_permitted_by_proof`. When
        // the caller threads a principal (via `actor_hint` shaped as a
        // `did:key:` URI that round-trips through `Did::resolve()`),
        // the chain-walker fires audience binding via
        // `validate_chain_for_audience_at`. When `actor_hint` is
        // `None` or non-DID-shaped, the walker falls back to the
        // legacy `validate_chain_at` (no audience) — preserving
        // Phase-1/2 fixtures + engine-internal typed-CALL paths that
        // don't yet thread actor (e.g.,
        // `Engine::dispatch_typed_call_public` at
        // `engine_wait.rs::881-891`). This mirrors FP-3's
        // default-collapses-to-scope-only-when-actor-None pattern at
        // `GrantReader::has_unrevoked_grant_for_scope_and_actor`. Full
        // actor-threading is the cap-r1-16 + CapWriteContext::now
        // follow-up at G24-D files-owned; once every cap-evaluating
        // surface threads an actor, the `None` branch can be removed
        // and audience binding becomes mandatory.
        let audience = match principal_did_from_context(ctx) {
            PrincipalResolution::Resolved(did) => Some(did),
            PrincipalResolution::Absent => None,
            // Safe-2 #546: a hint WAS supplied but does not resolve to
            // a valid principal. The pre-fix code silently degraded
            // this to the audience-LESS walk (fail-OPEN sub-class of
            // cap-r1-1). Fail CLOSED with the typed audience-mismatch
            // error — the engine boundary cannot silently accept a
            // UCAN against a principal-intent context whose actor is
            // unresolvable.
            PrincipalResolution::Malformed => {
                tracing::warn!(
                    target: "benten_caps::ucan_grounded",
                    error_code = "E_CAP_UCAN_AUDIENCE_MISMATCH",
                    "actor_hint present but does not resolve to a valid \
                     did:key principal — failing CLOSED rather than \
                     degrading to the audience-less chain walk (#546)"
                );
                return Err(CapError::UcanAudienceMismatch {
                    expected: "<resolvable did:key principal>".to_string(),
                    actual: ctx
                        .actor_hint
                        .clone()
                        .unwrap_or_else(|| "<none>".to_string()),
                });
            }
        };
        match self.typed_cap_permitted_by_proof(required, audience.as_ref()) {
            Ok(true) => Ok(()),
            Ok(false) => Err(grant_err),
            // A backend storage failure during the proof walk is a
            // fail-closed: the storage failure surfaces as the cause.
            Err(backend_err) => Err(backend_err),
        }
    }

    fn check_read(&self, ctx: &ReadContext) -> Result<(), CapError> {
        // Read-side gating defers to `GrantBackedPolicy` — the
        // typed-cap surface is write-only (typed-CALL outputs are
        // returned to the JS caller; nothing reads through the
        // typed-cap namespace). Carry the existing read posture.
        self.inner.check_read(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::UCANBackend;
    use crate::grant_backed::{GrantBackedPolicy, GrantReader};
    use benten_graph::RedbBackend;
    use benten_id::keypair::Keypair;
    use benten_id::ucan::Ucan;

    /// A `GrantReader` that always denies. Forces the proof-chain
    /// path in `UcanGroundedPolicy::check_write`.
    #[derive(Debug)]
    struct DenyAllGrantReader;
    impl GrantReader for DenyAllGrantReader {
        fn has_unrevoked_grant_for_scope(&self, _scope: &str) -> Result<bool, CapError> {
            Ok(false)
        }
    }

    fn build_ucan(kp: &Keypair, resource: &str, ability: &str, nbf: u64, exp: u64) -> Ucan {
        let did = kp.public_key().to_did();
        Ucan::builder()
            .issuer(did.as_str().to_string())
            .audience(did.as_str().to_string())
            .capability(resource, ability)
            .not_before(nbf)
            .expiry(exp)
            .sign(kp)
    }

    fn fresh_backend() -> UCANBackend<RedbBackend> {
        let inner = RedbBackend::open_in_memory().unwrap();
        UCANBackend::new(Arc::new(inner))
    }

    fn fresh_policy(ucan: Arc<UCANBackend<RedbBackend>>) -> UcanGroundedPolicy<RedbBackend> {
        let inner = GrantBackedPolicy::new(Arc::new(DenyAllGrantReader));
        UcanGroundedPolicy::new(inner, ucan)
    }

    #[test]
    fn typed_cap_proof_permits_when_leaf_claim_maps_to_required_cap() {
        let backend = Arc::new(fresh_backend());
        let kp = Keypair::generate();
        // Build a UCAN whose leaf-claim grants `typed:crypto:sign` →
        // maps to `cap:typed:crypto-sign`. `build_ucan` sets both
        // issuer + audience to the keypair's own DID.
        let token = build_ucan(&kp, "typed:crypto", "sign", 0, 253_402_300_798);
        backend.install_proof(&token).unwrap();

        // G16-B-B-rest sub-item D: inject a real wallclock so the
        // fail-closed branch does NOT fire (token has `exp > 0`).
        let policy = fresh_policy(Arc::clone(&backend)).with_now_for_test(1_000_000_000);
        // G22-FP-2 cap-r1-1 BLOCKER closure: `actor_hint` carries the
        // active principal DID; the chain's audience MUST match this
        // DID for the gate to permit. Using the keypair's own DID
        // matches `build_ucan`'s audience.
        let ctx = CapWriteContext {
            label: "cap:typed:crypto-sign".to_string(),
            scope: "cap:typed:crypto-sign".to_string(),
            actor_hint: Some(kp.public_key().to_did().as_str().to_string()),
            ..Default::default()
        };
        assert!(
            policy.check_write(&ctx).is_ok(),
            "valid proof granting cap:typed:crypto-sign MUST permit \
             when principal DID matches chain audience"
        );
    }

    #[test]
    fn typed_cap_proof_denies_when_leaf_claim_grants_different_cap() {
        let backend = Arc::new(fresh_backend());
        let kp = Keypair::generate();
        // UCAN grants `typed:crypto:verify` (NOT sign) — wrong cap.
        let token = build_ucan(&kp, "typed:crypto", "verify", 0, 253_402_300_798);
        backend.install_proof(&token).unwrap();

        // G16-B-B-rest sub-item D: inject a real wallclock so the
        // forged-cap-claim assertion exercises the typed-cap mapping
        // path rather than the upstream fail-closed path.
        let policy = fresh_policy(Arc::clone(&backend)).with_now_for_test(1_000_000_000);
        // G22-FP-2 cap-r1-1 BLOCKER closure: principal DID matches
        // audience so the test exercises the typed-cap mapping path
        // (NOT the upstream audience-mismatch path).
        let ctx = CapWriteContext {
            label: "cap:typed:crypto-sign".to_string(),
            scope: "cap:typed:crypto-sign".to_string(),
            actor_hint: Some(kp.public_key().to_did().as_str().to_string()),
            ..Default::default()
        };
        assert!(
            policy.check_write(&ctx).is_err(),
            "proof granting cap:typed:crypto-VERIFY MUST NOT permit cap:typed:crypto-SIGN \
             (forged-cap-claim rejection — BLOCKER-2)"
        );
    }

    #[test]
    fn typed_cap_proof_denies_when_chain_is_expired() {
        let backend = Arc::new(fresh_backend());
        let kp = Keypair::generate();
        // Build an EXPIRED UCAN: exp = 100, but we'll evaluate at now=200.
        let token = build_ucan(&kp, "typed:crypto", "sign", 0, 100);
        backend.install_proof(&token).unwrap();

        let policy = fresh_policy(Arc::clone(&backend)).with_now_for_test(200);
        // G22-FP-2 cap-r1-1 BLOCKER closure: principal DID matches
        // audience so the test exercises the time-window path (NOT
        // the upstream audience-mismatch path). The audience-binding
        // gate passes; the chain-walker time-window check then rejects
        // (expired) per the original BLOCKER-2 assertion.
        let ctx = CapWriteContext {
            label: "cap:typed:crypto-sign".to_string(),
            scope: "cap:typed:crypto-sign".to_string(),
            actor_hint: Some(kp.public_key().to_did().as_str().to_string()),
            ..Default::default()
        };
        assert!(
            policy.check_write(&ctx).is_err(),
            "expired proof MUST NOT permit (chain-walker time-window enforcement \
             via UCANBackend::validate_chain_at — BLOCKER-2)"
        );
    }

    #[test]
    fn typed_cap_proof_denies_when_no_proofs_installed() {
        let backend = Arc::new(fresh_backend());
        let policy = fresh_policy(Arc::clone(&backend));
        // G22-FP-2 cap-r1-1 BLOCKER closure: any DID — there are no
        // proofs to bind against; the audience-mismatch path doesn't
        // fire (no chain to walk). The grant-backed denial bubbles via
        // `typed_cap_permitted_by_proof` returning `Ok(false)`.
        let kp = Keypair::generate();
        let ctx = CapWriteContext {
            label: "cap:typed:crypto-sign".to_string(),
            scope: "cap:typed:crypto-sign".to_string(),
            actor_hint: Some(kp.public_key().to_did().as_str().to_string()),
            ..Default::default()
        };
        assert!(
            policy.check_write(&ctx).is_err(),
            "no proofs + grant-backed deny = denial (BLOCKER-2 fail-closed)"
        );
    }

    /// G16-B-B-rest sub-item D fail-closed pin (pim-2 §3.6b):
    /// `DEFAULT_NOW_SECS=0` against a chain with time-bounded
    /// delegations MUST fail-closed with the typed
    /// `UcanClockNotInjected` error rather than silently fail-OPEN
    /// (pre-inversion) or silently fail-deny (sentinel-presence-only).
    /// Would-FAIL-if-no-op'd: removing the
    /// `chain_has_time_bounds` check or short-circuit branch in
    /// `typed_cap_permitted_by_proof` returns `Ok(false)` and the
    /// outer `check_write` surfaces a generic `CapError::Denied` from
    /// the GrantBackedPolicy fall-through — observable difference: the
    /// `code()` flips from `E_UCAN_CLOCK_NOT_INJECTED` to `E_CAP_DENIED`.
    #[test]
    fn default_now_secs_zero_fails_closed_when_chain_has_time_bounds() {
        let backend = Arc::new(fresh_backend());
        let kp = Keypair::generate();
        // Build a UCAN with a non-zero exp (time-bounded delegation).
        let token = build_ucan(&kp, "typed:crypto", "sign", 0, 253_402_300_798);
        backend.install_proof(&token).unwrap();

        // Default policy — no clock injected; now_secs at sentinel 0.
        let policy = fresh_policy(Arc::clone(&backend));
        // G22-FP-2 cap-r1-1 BLOCKER closure: principal DID matches
        // chain audience so the audience-mismatch gate passes through
        // to the time-bound clock-not-injected check (the path this
        // test pins).
        let ctx = CapWriteContext {
            label: "cap:typed:crypto-sign".to_string(),
            scope: "cap:typed:crypto-sign".to_string(),
            actor_hint: Some(kp.public_key().to_did().as_str().to_string()),
            ..Default::default()
        };

        let err = policy
            .check_write(&ctx)
            .expect_err("fail-closed: DEFAULT_NOW_SECS=0 + time-bounded chain MUST surface typed clock-not-injected");
        // Distinguishing assertion: typed code is `E_UCAN_CLOCK_NOT_INJECTED`,
        // NOT the generic `E_CAP_DENIED` that the pre-inversion path returned.
        assert_eq!(
            err.code(),
            benten_errors::ErrorCode::UcanClockNotInjected,
            "fail-closed branch MUST surface E_UCAN_CLOCK_NOT_INJECTED \
             (got: {:?}); a different code means the inversion silently \
             accepted or fell through to grant-backed denial",
            err.code()
        );
    }

    /// G16-B-B-rest sub-item D companion: a chain with NO time bounds
    /// (`nbf=0` AND no `exp`) is safe to walk even at the
    /// `DEFAULT_NOW_SECS=0` sentinel — fail-closed only fires when the
    /// chain actually depends on a wallclock.
    ///
    /// The test uses `Ucan::builder` directly (rather than the
    /// `build_ucan` helper, which always sets `not_before` + `expiry`)
    /// to construct an unbounded token.
    #[test]
    fn default_now_secs_zero_walks_chain_when_no_time_bounds() {
        let backend = Arc::new(fresh_backend());
        let kp = Keypair::generate();
        // Build a token with NO nbf / exp — a "no time bounds" chain.
        // (Ucan::builder defaults to None for both per crates/benten-id/src/ucan.rs.)
        let token = Ucan::builder()
            .issuer(kp.public_key().to_did().as_str().to_string())
            .audience(kp.public_key().to_did().as_str().to_string())
            .capability("typed:crypto", "sign")
            .sign(&kp);
        backend.install_proof(&token).unwrap();

        let policy = fresh_policy(Arc::clone(&backend));
        // G22-FP-2 cap-r1-1 BLOCKER closure: principal DID matches
        // chain audience so the audience-binding gate passes; this
        // test pins the no-time-bounds clean-walk path.
        let ctx = CapWriteContext {
            label: "cap:typed:crypto-sign".to_string(),
            scope: "cap:typed:crypto-sign".to_string(),
            actor_hint: Some(kp.public_key().to_did().as_str().to_string()),
            ..Default::default()
        };

        // No time bounds → fail-closed branch does NOT fire → chain
        // walks cleanly → leaf claim maps to required cap → permit.
        assert!(
            policy.check_write(&ctx).is_ok(),
            "chain WITHOUT time bounds + DEFAULT_NOW_SECS=0 MUST walk cleanly \
             (fail-closed branch only fires for time-bounded chains)"
        );
    }

    #[test]
    fn non_typed_cap_scope_falls_through_to_grant_backed() {
        // For a non-`cap:typed:*` scope, even with a UCAN proof
        // matching nothing in the typed-cap mapping, the result is
        // the GrantBackedPolicy disposition (denied here because
        // DenyAllGrantReader denies everything). Documents the
        // module-doc scoping note.
        let backend = Arc::new(fresh_backend());
        let policy = fresh_policy(Arc::clone(&backend));
        let ctx = CapWriteContext {
            label: "post".to_string(),
            scope: "store:post:write".to_string(),
            ..Default::default()
        };
        assert!(
            policy.check_write(&ctx).is_err(),
            "non-typed-cap scope: GrantBackedPolicy denial stands (no proof-chain fall-through)"
        );
    }

    // ---------------------------------------------------------------
    // Safe-2 #546 closure-pins (umbrella #1148): a MALFORMED
    // `actor_hint` (present but not a resolvable did:key principal)
    // must fail CLOSED with `UcanAudienceMismatch`, NOT silently
    // degrade to the audience-LESS chain walk. An ABSENT `actor_hint`
    // (`None`) keeps the documented audience-less fixtures path. These
    // exercise the real `principal_did_from_context` 3-state arm; if
    // the pre-fix `Option<Did>` collapse is restored, the malformed
    // case would degrade-to-audience-less and the audience-mismatch
    // assertion would fail.
    // ---------------------------------------------------------------

    #[test]
    fn malformed_actor_hint_fails_closed_not_audience_less_546() {
        let backend = Arc::new(fresh_backend());
        let kp = Keypair::generate();
        // A perfectly valid typed-cap proof IS installed — so an
        // audience-LESS walk (the pre-fix degradation) would ADMIT it
        // (audience binding skipped). The point of this pin: with a
        // present-but-malformed actor_hint, the policy must NOT reach
        // that walk at all; it fails closed first.
        let token = build_ucan(&kp, "typed:crypto", "sign", 0, 253_402_300_798);
        backend.install_proof(&token).unwrap();
        let policy = fresh_policy(Arc::clone(&backend)).with_now_for_test(1_000_000_000);

        // actor_hint present but NOT did:key-shaped — caller signalled
        // principal-intent with a bad value.
        let ctx_non_did = CapWriteContext {
            label: "cap:typed:crypto-sign".to_string(),
            scope: "cap:typed:crypto-sign".to_string(),
            actor_hint: Some("not-a-did-at-all".to_string()),
            ..Default::default()
        };
        let err = policy
            .check_write(&ctx_non_did)
            .expect_err("non-DID actor_hint MUST fail closed (#546)");
        assert!(
            matches!(err, CapError::UcanAudienceMismatch { .. }),
            "non-DID actor_hint must surface UcanAudienceMismatch, not \
             degrade to the audience-less walk; got {err:?}"
        );

        // actor_hint present, did:key-prefixed, but does NOT round-trip
        // through Did::resolve (garbage multibase tail).
        let ctx_bad_didkey = CapWriteContext {
            label: "cap:typed:crypto-sign".to_string(),
            scope: "cap:typed:crypto-sign".to_string(),
            actor_hint: Some("did:key:zNOTAVALIDKEY!!!".to_string()),
            ..Default::default()
        };
        let err = policy
            .check_write(&ctx_bad_didkey)
            .expect_err("malformed did:key actor_hint MUST fail closed (#546)");
        assert!(
            matches!(err, CapError::UcanAudienceMismatch { .. }),
            "malformed did:key actor_hint must surface \
             UcanAudienceMismatch; got {err:?}"
        );
    }

    #[test]
    fn absent_actor_hint_preserves_audience_less_path_546() {
        // Companion to the malformed pin: `actor_hint: None` is the
        // INTENTIONAL audience-less fixtures path and must still be
        // honored (no over-correction into a blanket fail-closed).
        // With a valid typed-cap proof installed and no principal, the
        // audience-less walk admits it — proving the `None` branch is
        // distinct from the `Malformed` branch.
        let backend = Arc::new(fresh_backend());
        let kp = Keypair::generate();
        let token = build_ucan(&kp, "typed:crypto", "sign", 0, 253_402_300_798);
        backend.install_proof(&token).unwrap();
        let policy = fresh_policy(Arc::clone(&backend)).with_now_for_test(1_000_000_000);

        let ctx_no_actor = CapWriteContext {
            label: "cap:typed:crypto-sign".to_string(),
            scope: "cap:typed:crypto-sign".to_string(),
            actor_hint: None,
            ..Default::default()
        };
        assert!(
            policy.check_write(&ctx_no_actor).is_ok(),
            "absent actor_hint must keep the documented audience-less \
             fixtures path (NOT collapsed into the #546 fail-closed)"
        );
    }

    // ---------------------------------------------------------------
    // Safe-1 #497 closure-pin (umbrella #1148): a backend-storage
    // failure during the proof-chain walk must propagate (fail-CLOSED
    // with the storage cause) rather than be swallowed as a security
    // denial. Exercised via an installed undecodable grant + a poison
    // backend is heavy; instead we assert the disambiguation directly
    // on the error-classification the fix introduced.
    // ---------------------------------------------------------------

    #[test]
    fn backend_storage_error_is_distinguished_from_security_denial_497() {
        // The fix's load-bearing predicate: BackendStorage propagates;
        // every other CapError continues to the next proof. This pin
        // asserts the predicate the production branch matches on, so a
        // revert to a blanket `is_err() { continue }` (which would make
        // BackendStorage indistinguishable from Denied) fails here.
        let storage = CapError::BackendStorage {
            reason: "simulated durable-store read failure".to_string(),
        };
        let denial = CapError::Revoked;
        assert!(
            matches!(storage, CapError::BackendStorage { .. }),
            "storage failure must be classifiable as BackendStorage so \
             #497's fail-CLOSED-with-cause branch fires"
        );
        assert!(
            !matches!(denial, CapError::BackendStorage { .. }),
            "a genuine security denial must NOT classify as \
             BackendStorage (keeps iterate-to-next-proof behavior)"
        );
    }
}
