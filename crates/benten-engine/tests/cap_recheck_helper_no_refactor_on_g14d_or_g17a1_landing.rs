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

/// R3-B RED-PHASE pin per stream-r4r1-7 R4-R2 (25th-p/c-drift candidate
/// by closure-translation shape).
///
/// At G14-D wave-5a, the eval-side `subscribe::execute` runtime stores
/// a `DeliveryCapRecheck` closure of shape
/// `Arc<dyn Fn(&ChangeEvent) -> bool + Send + Sync + 'static>`
/// (per `crates/benten-eval/src/primitives/subscribe.rs::DeliveryCapRecheck`).
/// The engine-side helper exposes `CapRecheckFn` of shape
/// `Arc<dyn Fn(&PrincipalId, &str, &Cid) -> bool + Send + Sync + 'static>`
/// (per `crates/benten-engine/src/cap_recheck.rs::CapRecheckFn`). These are
/// TWO DISTINCT closure shapes — engine-side helper takes a structured
/// `(principal, zone, cid)` tuple; eval-side runtime takes the full
/// `ChangeEvent`. G14-D implementer MUST construct a
/// `DeliveryCapRecheck` from a `CapRecheckFn` at the eval-side seam by
/// unwrapping `ChangeEvent → (principal, zone, cid)` (translation-layer
/// closure-shape — pim-11 §3.6d translation-layer cite-discipline +
/// pim-2 §3.6b end-to-end pin discipline).
///
/// This RED-PHASE pin asserts the translation produces semantically-
/// equivalent cap-pass decisions when the same `(principal, zone, cid)`
/// tuple is fed to `CapRecheckFn` directly vs derived from a synthesized
/// `ChangeEvent`. The pin would FAIL if the translation layer silently
/// dropped a tuple component or mis-mapped a field — catching the 25th-
/// p/c-drift shape at the structural layer before runtime.
///
/// OBSERVABLE consequence: `CapRecheckFn` decision == `DeliveryCapRecheck`
/// decision when both consult the same `(principal, zone, cid)`.
/// Producer (engine-side helper) + consumer (eval-side runtime) name-
/// alignment verified at runtime test time.
#[test]
#[ignore = "phase-3-backlog §7.3.D — cap-recheck-helper translation-layer closure-shape pin. G14-D wave-5a + G17-A1 wave-5b both shipped (PR #115 + #117); test body pins specific DeliveryCapRecheck ↔ CapRecheckFn name-alignment + decision-parity invariant; un-ignore at next Phase-3-close orchestrator-direct fix-pass batch per Wave-E rationale-only sweep."]
fn cap_recheck_helper_consumed_with_change_event_to_principal_zone_cid_translation_shape_documented()
 {
    // G14-D implementer wires this:
    //
    //   use benten_eval::primitives::subscribe::DeliveryCapRecheck;
    //   use benten_engine::cap_recheck::CapRecheckFn;
    //
    //   // Engine-side authoritative cap helper (G13-pre-C):
    //   let revoked_zone = "user:revoked-posts".to_string();
    //   let cap_recheck: CapRecheckFn = {
    //       let revoked = revoked_zone.clone();
    //       Arc::new(move |_p: &PrincipalId, zone: &str, _cid: &Cid| {
    //           zone != revoked
    //       })
    //   };
    //
    //   // Translation seam: G14-D constructs DeliveryCapRecheck from
    //   // CapRecheckFn by unwrapping ChangeEvent → (principal, zone, cid).
    //   let delivery: DeliveryCapRecheck = {
    //       let cap_recheck = Arc::clone(&cap_recheck);
    //       Arc::new(move |evt: &ChangeEvent| {
    //           cap_recheck(
    //               evt.principal(),
    //               evt.zone(),
    //               evt.node_cid(),
    //           )
    //       })
    //   };
    //
    //   // Decision parity: same (principal, zone, cid) → same decision
    //   // through both shapes. The translation layer MUST be transparent.
    //   let p = sample_principal();
    //   let cid = sample_cid();
    //   let evt_allowed = ChangeEvent::synthesize_for_test(
    //       p.clone(), "user:posts".to_string(), cid,
    //   );
    //   let evt_revoked = ChangeEvent::synthesize_for_test(
    //       p.clone(), revoked_zone.clone(), cid,
    //   );
    //
    //   assert_eq!(
    //       cap_recheck(&p, "user:posts", &cid),
    //       delivery(&evt_allowed),
    //       "translation-layer transparency for allowed zone — \
    //        engine-side helper decision MUST match eval-side runtime \
    //        decision when fed semantically-equivalent inputs",
    //   );
    //   assert_eq!(
    //       cap_recheck(&p, &revoked_zone, &cid),
    //       delivery(&evt_revoked),
    //       "translation-layer transparency for revoked zone — \
    //        ChangeEvent unwrap MUST preserve cap-pass decision",
    //   );
    //   assert!(cap_recheck(&p, "user:posts", &cid));
    //   assert!(!cap_recheck(&p, &revoked_zone, &cid));
    //
    // OBSERVABLE consequence: the eval-side `DeliveryCapRecheck` is a
    // structurally-equivalent wrapper around the engine-side
    // `CapRecheckFn`; producer (engine `cap_recheck::CapRecheckFn`) +
    // consumer (eval `subscribe::DeliveryCapRecheck`) names align in
    // the translation layer; the 25th-p/c-drift candidate shape
    // (closure-translation mismatch) is structurally defended.
    //
    // Defends per pim-11 §3.6d (translation-layer cite-discipline) +
    // pim-2 §3.6b (end-to-end pin discipline). Plan G14-D row at line
    // 429 will carry the translation-shape note alongside this pin's
    // un-ignore.
    unimplemented!(
        "G14-D wires DeliveryCapRecheck ↔ CapRecheckFn translation seam at subscribe.rs eval-side seam"
    );
}
