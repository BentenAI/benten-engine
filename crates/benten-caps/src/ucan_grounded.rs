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
//! through `WriteContext`) is wider scope — it requires
//! `WriteContext::actor_hint`-as-DID propagation, which is its own
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
//! `WriteContext` does not currently carry an audience DID — the
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

use crate::backends::UCANBackend;
use crate::error::CapError;
use crate::grant_backed::GrantBackedPolicy;
use crate::policy::{CapabilityPolicy, ReadContext, WriteContext};
use crate::typed_cap_mapping::typed_cap_for_ucan_claim;

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
    /// real clock injection lands at the `WriteContext::now`
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
/// (or the eventual `WriteContext::now` threading per
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
    /// fixture) until `WriteContext::now` threading lands per
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
    /// `cap:typed:*` requirement at `now_secs`. Returns `Ok(true)` on
    /// a single permitting chain; `Ok(false)` if no chain permits +
    /// no errors arose; bubbles the typed `CapError` for backend
    /// storage failures.
    ///
    /// A chain that fails the chain-walker (bad signature / expired /
    /// attenuation-violation / revoked) is treated as "this chain
    /// does not permit" — iteration continues to the next proof. A
    /// proof whose leaf-claim maps to the required typed-cap AND
    /// passes `validate_chain_at` permits.
    fn typed_cap_permitted_by_proof(&self, required: &str) -> Result<bool, CapError> {
        let proofs = self.ucan.iter_installed_proofs()?;
        for proof in &proofs {
            // 1. Single-token chain treated as `[proof]`. Multi-token
            //    chains are an extension axis that lights up when
            //    `WriteContext` carries an explicit chain reference;
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
            // 3. Chain-walker fires signature + time-window +
            //    attenuation + revocation. Failure of ANY step =
            //    "this chain does not permit"; iterate to next.
            if self.ucan.validate_chain_at(chain, self.now_secs).is_err() {
                continue;
            }
            // 3. Leaf-claim → typed-cap mapping.
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
    fn check_write(&self, ctx: &WriteContext) -> Result<(), CapError> {
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
        match self.typed_cap_permitted_by_proof(required) {
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
        // maps to `cap:typed:crypto-sign`.
        let token = build_ucan(&kp, "typed:crypto", "sign", 0, 253_402_300_798);
        backend.install_proof(&token).unwrap();

        // G16-B-B-rest sub-item D: inject a real wallclock so the
        // fail-closed branch does NOT fire (token has `exp > 0`).
        let policy = fresh_policy(Arc::clone(&backend)).with_now_for_test(1_000_000_000);
        let ctx = WriteContext {
            label: "cap:typed:crypto-sign".to_string(),
            scope: "cap:typed:crypto-sign".to_string(),
            ..Default::default()
        };
        assert!(
            policy.check_write(&ctx).is_ok(),
            "valid proof granting cap:typed:crypto-sign MUST permit"
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
        let ctx = WriteContext {
            label: "cap:typed:crypto-sign".to_string(),
            scope: "cap:typed:crypto-sign".to_string(),
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
        let ctx = WriteContext {
            label: "cap:typed:crypto-sign".to_string(),
            scope: "cap:typed:crypto-sign".to_string(),
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
        let ctx = WriteContext {
            label: "cap:typed:crypto-sign".to_string(),
            scope: "cap:typed:crypto-sign".to_string(),
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
        let ctx = WriteContext {
            label: "cap:typed:crypto-sign".to_string(),
            scope: "cap:typed:crypto-sign".to_string(),
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
        let ctx = WriteContext {
            label: "cap:typed:crypto-sign".to_string(),
            scope: "cap:typed:crypto-sign".to_string(),
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
        let ctx = WriteContext {
            label: "post".to_string(),
            scope: "store:post:write".to_string(),
            ..Default::default()
        };
        assert!(
            policy.check_write(&ctx).is_err(),
            "non-typed-cap scope: GrantBackedPolicy denial stands (no proof-chain fall-through)"
        );
    }
}
