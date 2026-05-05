//! Shared per-event read-cap-coverage helper (G13-pre-C scaffold).
//!
//! This module is the SINGLE consumer surface for the dual-layer capability
//! recheck pattern that lands at:
//!
//! - **G14-D F6 SUBSCRIBE filtering** (wave 5a) — per-subscriber durable grant
//!   store; closure consults this helper at every delivered ChangeEvent so a
//!   partial-revoke cancels the affected subscription path mid-stream.
//! - **G17-A1 ESC-9 `live_cap_check`** (wave 5b) — host-fn boundary check;
//!   closure consults this helper at every host-fn invocation so a mid-call
//!   revocation is observable BEFORE the next sensitive host-fn fires
//!   (no caching window per `r1-wsa-3`).
//!
//! ## Why extract FIRST (per `seq-minor-6`)
//!
//! Both wave-5a and wave-5b consume this helper from day one. Extracting
//! the type alias + constructors at G13-pre-C means neither wave writes
//! inline-then-refactor code; both waves cite this module directly. Any
//! future signature change MUST land HERE, not at consumer sites — see
//! `tests/cap_recheck_helper_no_refactor_on_g14d_or_g17a1_landing.rs`
//! for the architectural pin asserting the no-refactor contract.
//!
//! ## Design pins (apply, do not re-debate)
//!
//! - **Closure-shaped per `stream-r1-11`.** `Arc<dyn Fn(...) -> bool + Send +
//!   Sync + 'static>` for max flexibility. Implementer wires concrete cap
//!   store at G14-D + G17-A1.
//! - **Dual-layer cap-recheck per CLR-2 / `cap-major-2`.** Subscribe-time
//!   gate (does the subscriber have read coverage right now?) PLUS per-event
//!   delivery-time gate (is the coverage still live for THIS event?). The
//!   helper is consulted at the delivery-time leg.
//! - **D-PHASE-3-5 RECOMMEND: sync-per-event, partial-revoke cancels
//!   affected subset.** Synchronous closure consultation at delivery
//!   boundary; partial-revoke fires `E_SUBSCRIBE_REVOKED_MID_STREAM` at
//!   G14-D consumer site.
//! - **Compromise #11 per-row read-gate composition.** Helper signature
//!   accommodates per-row check `(principal, zone, node_cid)`; G15-A
//!   label-hint extraction + G14-D per-subscriber filtering both compose
//!   on this shape.
//! - **Shared with ESC-9 (G17-A1) at host-fn boundary per `r1-wsa-3`.**
//!   Fires at every host-fn boundary, NOT cached. Same closure shape so
//!   both waves use the same dispatch type.
//!
//! ## `PrincipalId` placeholder
//!
//! `PrincipalId` is the typed actor identity that lands in the new
//! `benten-id` crate at G14-A1 (wave 4a, native + browser). Until that
//! crate exists, this module ships a thin placeholder so G13-pre-C can
//! land a stable surface NOW for the wave-5a + wave-5b consumers. At
//! G14-A1 the placeholder is replaced by `pub use benten_id::PrincipalId;`
//! — the module-public type name does not change, so no consumer needs
//! to refactor (this is the "no-refactor on G14-D or G17-A1 landing"
//! contract pinned by `cap_recheck_helper_no_refactor_on_g14d_or_g17a1_landing`).
//!
//! ## Default constructors
//!
//! - [`allow_all`] — match the `NoAuthBackend` default; permits every check.
//! - [`deny_all`] — fail-closed default for tests + fail-closed deployments.
//!
//! Production policies (e.g. the durable UCAN backend at G14-B) construct
//! their own `CapRecheckFn` closing over the cap store; the
//! `allow_all` / `deny_all` constructors are for tests + the
//! `NoAuthBackend` default.

use std::sync::Arc;

use benten_core::Cid;

/// Placeholder for the typed actor identity that lands in `benten-id`
/// at G14-A1 (wave 4a per the Phase-3 plan).
///
/// At G14-A1 this declaration is replaced by `pub use
/// benten_id::PrincipalId;`. The placeholder shape is intentionally
/// minimal — it carries the actor's CID (matching the `actor_cid`
/// field on `benten_caps::ReadContext`) so the helper signature can
/// be pinned NOW without depending on the not-yet-existing crate.
///
/// **G14-A1 migration:** drop this struct, add `pub use
/// benten_id::PrincipalId;`. Consumer cite paths
/// (`crate::cap_recheck::PrincipalId`) remain stable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrincipalId {
    /// CID-shaped actor identity — re-uses [`benten_core::Cid`] until
    /// G14-A1 lands the typed `benten_id::PrincipalId`.
    pub actor_cid: Cid,
}

impl PrincipalId {
    /// Construct a placeholder principal from an actor CID.
    ///
    /// G14-A1 swaps this for the `benten-id` crate's typed constructor;
    /// the surface name + arity stay stable.
    #[must_use]
    pub fn from_actor_cid(actor_cid: Cid) -> Self {
        Self { actor_cid }
    }
}

/// Per-event read-cap-coverage closure.
///
/// Consulted at:
/// - **G14-D F6 SUBSCRIBE** delivery boundary, per-event.
/// - **G17-A1 ESC-9** host-fn boundary, per-call (no caching window per
///   `r1-wsa-3`).
///
/// Arguments:
/// - `&PrincipalId` — the calling actor.
/// - `&str` — the zone label (e.g. `"system:ModuleManifest"`,
///   `"user:posts"`); empty for CID-only reads.
/// - `&Cid` — the CID of the Node / view-row / event payload being
///   inspected.
///
/// Returns `true` to permit, `false` to deny. Denial at G14-D cancels
/// the subscription path (`E_SUBSCRIBE_REVOKED_MID_STREAM` at the
/// consumer site); denial at G17-A1 fires `E_CAP_DENIED_READ` from
/// the host-fn boundary.
///
/// **No refactor at G14-D or G17-A1.** The signature is the consumer
/// surface lock per `seq-minor-6`. Any change to this shape MUST land
/// HERE, not at consumer sites — see
/// `cap_recheck_helper_no_refactor_on_g14d_or_g17a1_landing` for the
/// architectural pin.
pub type CapRecheckFn = Arc<dyn Fn(&PrincipalId, &str, &Cid) -> bool + Send + Sync + 'static>;

/// Default `CapRecheckFn` matching the `NoAuthBackend` default — permits
/// every check.
///
/// Use at:
/// - Tests that explicitly opt out of cap enforcement.
/// - Embedded / local-only deployments that ship `NoAuthBackend`.
///
/// Production deployments using `GrantBackedPolicy` / the durable UCAN
/// backend (G14-B) construct their own closure closing over the cap
/// store.
#[must_use]
pub fn allow_all() -> CapRecheckFn {
    Arc::new(|_principal: &PrincipalId, _zone: &str, _node_cid: &Cid| true)
}

/// Fail-closed `CapRecheckFn` — denies every check.
///
/// Use at:
/// - Fail-closed deployments (defense-in-depth: any cap-check that
///   somehow loses its closing scope reverts to denial, not permission).
/// - Tests that pin the deny-path observable behavior at G14-D
///   (`E_SUBSCRIBE_REVOKED_MID_STREAM`) + G17-A1 (`E_CAP_DENIED_READ`).
#[must_use]
pub fn deny_all() -> CapRecheckFn {
    Arc::new(|_principal: &PrincipalId, _zone: &str, _node_cid: &Cid| false)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests and benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;
    use benten_core::testing::canonical_test_node;

    fn sample_principal() -> PrincipalId {
        PrincipalId::from_actor_cid(canonical_test_node().cid().unwrap())
    }

    fn sample_node_cid() -> Cid {
        canonical_test_node().cid().unwrap()
    }

    #[test]
    fn allow_all_permits_every_call() {
        let f = allow_all();
        assert!(f(&sample_principal(), "user:posts", &sample_node_cid()));
        assert!(f(&sample_principal(), "", &sample_node_cid()));
        assert!(f(
            &sample_principal(),
            "system:ModuleManifest",
            &sample_node_cid()
        ));
    }

    #[test]
    fn deny_all_denies_every_call() {
        let f = deny_all();
        assert!(!f(&sample_principal(), "user:posts", &sample_node_cid()));
        assert!(!f(&sample_principal(), "", &sample_node_cid()));
        assert!(!f(
            &sample_principal(),
            "system:ModuleManifest",
            &sample_node_cid()
        ));
    }

    #[test]
    fn cap_recheck_fn_is_clone_send_sync_static() {
        // Compile-time pin: the closure type satisfies the bounds
        // every consumer needs (Arc clone for fan-out at G14-D
        // per-subscriber filtering; Send + Sync for the
        // host-fn-boundary thread-through at G17-A1; 'static for
        // suspended-WAIT envelope handoff at G14-D resume_from_bytes).
        fn assert_bounds<T: Clone + Send + Sync + 'static>(_: &T) {}
        let f: CapRecheckFn = allow_all();
        assert_bounds(&f);
        let _f2 = f.clone();
    }

    #[test]
    fn principal_id_from_actor_cid_round_trip() {
        let cid = sample_node_cid();
        let p = PrincipalId::from_actor_cid(cid);
        assert_eq!(p.actor_cid, cid);
    }
}
