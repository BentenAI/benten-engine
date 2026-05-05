//! Architectural pin: the G13-pre-C `cap_recheck` helper must be
//! consumed AS-IS by both G14-D F6 SUBSCRIBE filtering (wave-5a) and
//! G17-A1 ESC-9 `live_cap_check` (wave-5b).
//!
//! ## What this test asserts
//!
//! - The helper module `benten_engine::cap_recheck` is publicly
//!   reachable.
//! - The pinned public surface (`CapRecheckFn` type alias +
//!   `PrincipalId` placeholder + `allow_all` + `deny_all`
//!   constructors) is the consumer-surface lock per `seq-minor-6`.
//! - The closure shape composes both layers of the dual cap-recheck
//!   per CLR-2 / `cap-major-2`: subscribe-time gate + per-event
//!   delivery-time gate (G14-D) AND host-fn-boundary gate (G17-A1).
//! - Any FUTURE signature change MUST land in
//!   `crates/benten-engine/src/cap_recheck.rs` — the consumer sites
//!   (`engine_subscribe.rs` at G14-D + `sandbox/host_fns.rs` at
//!   G17-A1) cite this module and inherit the change.
//!
//! ## Why this matters (per `seq-minor-6`)
//!
//! Both consumer waves ship in parallel — wave-5a (G14-D F6
//! filtering) and wave-5b (G17-A1 ESC-9 live_cap_check). If either
//! wave landed an inline ad-hoc shape, we'd pay refactor cost when
//! the second wave consolidated against it. Extracting the helper
//! FIRST at G13-pre-C means both waves cite the same module and
//! the no-refactor contract is observable.
//!
//! ## Forward-looking simulation
//!
//! The test bodies below simulate the wave-5a + wave-5b consumption
//! patterns: each constructs a closure that the actual consumer site
//! will construct in production code, and asserts the constructed
//! closure round-trips through the pinned `CapRecheckFn` alias
//! without coercion / refactor. If the pinned alias drifts, the
//! simulation fails to compile — exactly the loud signal we want.

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

/// Simulates the G14-D F6 SUBSCRIBE delivery-time filtering site.
///
/// At G14-D wave-5a, `engine_subscribe.rs` builds a
/// per-subscriber `CapRecheckFn` closing over the durable grant
/// store. On every delivered ChangeEvent the helper is consulted
/// for `(subscriber_principal, event_zone, event_node_cid)`; deny
/// fires `E_SUBSCRIBE_REVOKED_MID_STREAM` and cancels the path.
///
/// This simulation pins the SHAPE that wave-5a consumes — a deny
/// for a specific zone returns `false`, allow otherwise.
#[test]
fn g14d_f6_subscribe_per_event_delivery_gate_consumes_helper_as_is() {
    let revoked_zone = String::from("user:revoked-posts");

    // Production-shape closure construction — wave-5a's
    // `engine_subscribe.rs` builds an analogous closure closing over
    // the durable grant store. If the pinned alias changes, this
    // line stops compiling.
    let f: CapRecheckFn = {
        let revoked_zone = revoked_zone.clone();
        Arc::new(move |_p: &PrincipalId, zone: &str, _cid: &Cid| zone != revoked_zone)
    };

    let p = sample_principal();
    let cid = sample_cid();
    assert!(
        f(&p, "user:posts", &cid),
        "non-revoked zone delivery permitted"
    );
    assert!(
        !f(&p, &revoked_zone, &cid),
        "revoked zone fires deny — wave-5a will translate to E_SUBSCRIBE_REVOKED_MID_STREAM"
    );
}

/// Simulates the G17-A1 ESC-9 `live_cap_check` host-fn boundary
/// site.
///
/// At G17-A1 wave-5b, `sandbox/host_fns.rs` threads a
/// `CapRecheckFn` into the host-fn dispatch. On every host-fn
/// invocation the helper is consulted for `(call_principal,
/// host_fn_target_zone, host_fn_target_cid)`; deny fires
/// `E_CAP_DENIED_READ` and traps the guest.
///
/// Per `r1-wsa-3`: fires at every host-fn boundary, NOT cached. The
/// closure can mutate its closed-over state (e.g. tick a budget,
/// observe a revocation flag) without re-keying the `CapRecheckFn`
/// itself.
#[test]
fn g17a1_esc_9_host_fn_boundary_gate_consumes_helper_as_is() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let revoked = Arc::new(AtomicBool::new(false));

    // Production-shape closure construction — wave-5b's
    // `sandbox/host_fns.rs` builds an analogous closure closing over
    // the live cap store / revocation flag. If the pinned alias
    // changes, this line stops compiling.
    let f: CapRecheckFn = {
        let revoked = Arc::clone(&revoked);
        Arc::new(move |_p: &PrincipalId, _zone: &str, _cid: &Cid| !revoked.load(Ordering::SeqCst))
    };

    let p = sample_principal();
    let cid = sample_cid();

    // First host-fn boundary: not yet revoked, permitted.
    assert!(f(&p, "host:kv:read", &cid));

    // Mid-call revocation observable BEFORE the next sensitive
    // host-fn fires — this is the `r1-wsa-3` no-caching-window
    // contract.
    revoked.store(true, Ordering::SeqCst);
    assert!(
        !f(&p, "host:kv:read", &cid),
        "revocation observable at next boundary — wave-5b will translate to E_CAP_DENIED_READ"
    );
}

/// Simulates the dual-layer cap-recheck per CLR-2 / `cap-major-2`.
///
/// Subscribe-time gate (does the subscriber have read coverage right
/// now?) PLUS per-event delivery-time gate (is the coverage still
/// live for THIS event?). Both legs use the SAME `CapRecheckFn`
/// shape — the pin asserts a single helper composes both layers.
#[test]
fn dual_layer_cap_recheck_clr_2_composes_with_single_helper_shape() {
    let f: CapRecheckFn = allow_all();

    // Subscribe-time leg — a one-shot up-front check before
    // attaching the subscriber.
    let p = sample_principal();
    let cid = sample_cid();
    let subscribe_time_ok = f(&p, "user:posts", &cid);
    assert!(subscribe_time_ok, "subscribe-time gate: allow_all permits");

    // Delivery-time leg — same `f` consulted on each event.
    for _ in 0..3 {
        assert!(
            f(&p, "user:posts", &cid),
            "delivery-time gate: same helper, same shape, no refactor"
        );
    }
}

/// Simulates Compromise #11 per-row read-gate composition (G15-A
/// label-hint extraction + G14-D per-subscriber filtering).
///
/// Per the brief: helper signature must accommodate per-row check
/// `(zone + node_cid + principal)`. This test asserts the alias
/// supports row-grain dispatch — the closure body inspects every
/// row's zone + cid before deciding.
#[test]
fn compromise_11_per_row_read_gate_composes_via_helper() {
    let f: CapRecheckFn = Arc::new(|_p: &PrincipalId, zone: &str, _cid: &Cid| {
        // Per-row grain: only `user:` zones are visible to the
        // subscriber.
        zone.starts_with("user:")
    });

    let p = sample_principal();
    let cid = sample_cid();
    let rows = [
        ("user:posts", true),
        ("user:comments", true),
        ("system:ModuleManifest", false),
        ("system:zones", false),
    ];
    for (zone, expected) in rows {
        assert_eq!(
            f(&p, zone, &cid),
            expected,
            "per-row gate decision for zone {zone:?}"
        );
    }
}

/// Asserts the placeholder `PrincipalId` survives the type-alias
/// boundary into a closure — i.e. the pin is observable from a
/// consumer module's perspective.
///
/// At G14-A1 the placeholder is replaced by `pub use
/// benten_id::PrincipalId;` and this test continues to pass without
/// modification (the surface name + arity are identical).
#[test]
fn principal_id_placeholder_is_consumer_visible() {
    let p = PrincipalId::from_actor_cid(sample_cid());
    let f: CapRecheckFn = Arc::new(move |observed: &PrincipalId, _zone: &str, _cid: &Cid| {
        // Pin: the closure can structurally inspect the principal —
        // the G14-A1 typed `benten_id::PrincipalId` substitution must
        // also expose `actor_cid` (per the placeholder migration
        // note in `cap_recheck.rs`).
        observed.actor_cid == p.actor_cid
    });
    let same = sample_principal();
    assert!(f(&same, "user:posts", &sample_cid()));
}

/// Forward-asserts the no-refactor contract: the brief specifies
/// `allow_all()` + `deny_all()` constructors. Both must remain
/// importable from the public path `benten_engine::cap_recheck::`
/// across the wave-5a + wave-5b landings.
#[test]
fn allow_all_and_deny_all_remain_pub_at_module_root() {
    // If either constructor were renamed / scoped narrower, this
    // import would fail at compile time — caught HERE, before
    // wave-5a + wave-5b consume them.
    let _allow: CapRecheckFn = allow_all();
    let _deny: CapRecheckFn = deny_all();
}
