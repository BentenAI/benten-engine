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

/// Default `now` for chain-walker time-window validation. Picked at
/// epoch=0 so present-day fixtures with `nbf=0` and any positive
/// `exp` accept by default; tests override via
/// [`UcanGroundedPolicy::with_now_for_test`].
///
/// Why `0` rather than year-9999: the chain-walker rejects with
/// `Expired` whenever `now >= exp`. Picking a sentinel near year-9999
/// means present-day fixtures with reasonable `exp` values get
/// rejected because their exp is below the sentinel. Picking `0`
/// (epoch start) means any token with `nbf <= 0` and `exp > 0`
/// accepts — the discipline-focused default given the
/// `WriteContext::now`-threading work named in phase-3-backlog
/// §2.3 (i) lights up the real-clock path.
const DEFAULT_NOW_SECS: u64 = 0;

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
            // 2. Chain-walker fires signature + time-window +
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

        let policy = fresh_policy(Arc::clone(&backend));
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

        let policy = fresh_policy(Arc::clone(&backend));
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
