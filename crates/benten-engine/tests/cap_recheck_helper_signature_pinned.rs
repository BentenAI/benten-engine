//! Surface-pin for the G13-pre-C `cap_recheck` helper.
//!
//! This test asserts the public type signature is stable so that both
//! G14-D F6 SUBSCRIBE filtering (wave 5a) and G17-A1 ESC-9
//! `live_cap_check` (wave 5b) consume the same shape without
//! refactoring. Any signature change MUST update this test FIRST —
//! the test failing is the loud signal that downstream consumers will
//! need to be re-pointed.
//!
//! Pinned shape (per the G13-pre-C brief):
//!
//! ```text
//! pub type CapRecheckFn =
//!     std::sync::Arc<dyn Fn(&PrincipalId, &str /* zone */, &Cid /* node_cid */)
//!         -> bool + Send + Sync + 'static>;
//! ```
//!
//! Plus the two default constructors `allow_all()` + `deny_all()`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests and benches may use unwrap/expect per workspace policy"
)]

use std::sync::Arc;

use benten_core::Cid;
use benten_core::testing::canonical_test_node;
use benten_engine::cap_recheck::{CapRecheckFn, PrincipalId, allow_all, deny_all};

fn sample_principal() -> PrincipalId {
    PrincipalId::from_actor_cid(canonical_test_node().cid().unwrap())
}

fn sample_cid() -> Cid {
    canonical_test_node().cid().unwrap()
}

#[test]
fn cap_recheck_fn_pub_type_is_arc_dyn_fn_principal_str_cid_to_bool() {
    // Compile-time assertion: a fresh `Arc<dyn Fn(&PrincipalId, &str,
    // &Cid) -> bool + Send + Sync + 'static>` is assignable to
    // `CapRecheckFn`. If the type alias drifts, this fails to compile.
    let _: CapRecheckFn = Arc::new(|_p: &PrincipalId, _zone: &str, _cid: &Cid| -> bool { true });
    let _: CapRecheckFn = Arc::new(|_p: &PrincipalId, _zone: &str, _cid: &Cid| -> bool { false });
}

#[test]
fn cap_recheck_fn_satisfies_send_sync_static() {
    // Compile-time pin matching the consumer requirements:
    //  - G14-D F6 fan-out across subscriber threads needs Send + Sync.
    //  - G17-A1 host-fn-boundary thread-through needs Send + Sync.
    //  - G14-D resume_from_bytes envelope handoff needs 'static.
    fn require_send_sync_static<T: Send + Sync + 'static>(_: &T) {}
    let f: CapRecheckFn = allow_all();
    require_send_sync_static(&f);
}

#[test]
fn cap_recheck_fn_is_arc_clone_able_for_fan_out() {
    // G14-D per-subscriber filtering fans out a single closure across
    // many subscribers via `Arc::clone` rather than re-constructing.
    // Pin Clone-ability on the alias.
    let f: CapRecheckFn = allow_all();
    let f2 = f.clone();
    assert!(f(&sample_principal(), "user:posts", &sample_cid()));
    assert!(f2(&sample_principal(), "user:posts", &sample_cid()));
}

#[test]
fn allow_all_constructor_is_pub_and_returns_cap_recheck_fn() {
    let f: CapRecheckFn = allow_all();
    assert!(f(&sample_principal(), "user:posts", &sample_cid()));
    assert!(f(&sample_principal(), "", &sample_cid()));
    assert!(f(
        &sample_principal(),
        "system:ModuleManifest",
        &sample_cid()
    ));
}

#[test]
fn deny_all_constructor_is_pub_and_returns_cap_recheck_fn() {
    let f: CapRecheckFn = deny_all();
    assert!(!f(&sample_principal(), "user:posts", &sample_cid()));
    assert!(!f(&sample_principal(), "", &sample_cid()));
    assert!(!f(
        &sample_principal(),
        "system:ModuleManifest",
        &sample_cid()
    ));
}

#[test]
fn principal_id_constructor_round_trips_actor_cid() {
    let cid = sample_cid();
    let p = PrincipalId::from_actor_cid(cid);
    assert_eq!(p.actor_cid, cid);
}

#[test]
fn closure_argument_order_is_principal_zone_cid_per_brief() {
    // The brief pins the argument order as `(&PrincipalId, &str /* zone
    // */, &Cid /* node_cid */)`. A consumer that mistakenly swapped
    // zone + cid would still type-check IFF zone were `&Cid` — but the
    // pinned alias is `&str`. This compile-time assertion catches that
    // shape drift.
    let f: CapRecheckFn = Arc::new(|p: &PrincipalId, zone: &str, cid: &Cid| {
        // The closure body confirms the parameter types via use:
        // `zone` is treated as a string slice (`is_empty()`),
        // `cid` is treated as a `Cid` (passed by reference).
        !zone.is_empty() && p.actor_cid == *cid
    });
    let p = sample_principal();
    let cid = sample_cid();
    assert!(f(&p, "user:posts", &cid));
    assert!(!f(&p, "", &cid));
}
