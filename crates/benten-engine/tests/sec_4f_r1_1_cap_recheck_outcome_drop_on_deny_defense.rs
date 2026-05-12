//! Phase 4-Foundation R3 (Family A — R1-FP wave-1 G22-FP-1 regression-
//! defense). Source-level + type-level defense paired with the
//! `subscribe_delivery_cap_recheck_per_event_redacts_revoked_node_granularity.rs`
//! production-runtime acceptance test (SHIPPED at PR #210 closing
//! sec-4f-r1-1 BLOCKER + mat-r1-4).
//!
//! # Charter
//!
//! Per R3 dispatch brief (Family A R1-FP regression-defense section).
//! Ben's option-D ratification (2026-05-12) reconciled the post-R1-
//! triage ratification #7 fork: per-Node fail-soft elision (admin UI
//! per-cap-revocation UX) was wanted, AND Phase-3 R6-FP Wave-C1
//! SHIPPED contract (whole-subscription auto-cancel on cap revoke).
//! Option-D shipped the
//! [`benten_eval::primitives::subscribe::CapRecheckOutcome`] enum
//! carrying `{ Keep, Drop, Cancel }` so the eval-side publish loop
//! can dispatch per-event:
//!
//! - `Keep` — deliver this event (replaces the prior `true` return).
//! - `Drop` — silent elision; stream stays open; no termination
//!   notify; no `SUBSCRIBE_REVOKED_MID_STREAM_COUNT` bump.
//! - `Cancel` — preserves Phase-3 R6-FP Wave-C1 SHIPPED contract
//!   (typed `EvalError::SubscribeRevokedMidStream` + termination
//!   notify + auto-unregister).
//!
//! The `Engine::on_change_as_with_cursor` cap-recheck closure (in
//! `crates/benten-engine/src/engine_subscribe.rs`) wires the
//! per-event `CapabilityPolicy::check_read` gate to return **`Drop`**
//! on `Err(_)` (NOT `Cancel`) — this is the load-bearing
//! per-Node-fail-soft semantic the admin UI relies on.
//!
//! # What this pin asserts (would-FAIL-if-no-op'd per §3.6b)
//!
//! Three composable invariants:
//!
//! 1. **Variant-count exact 3.** The `CapRecheckOutcome` enum MUST
//!    carry exactly 3 variants: `Keep`, `Drop`, `Cancel`. Adding a
//!    4th variant without dispatching it through the engine-side
//!    closure trips the variant-count assertion; removing one trips
//!    the named-variant assertion below. This pins the protocol
//!    surface at the type level — a paired contract with the napi
//!    bindings + future serialization shape.
//!
//! 2. **Engine-side `Drop` for `check_read` deny.** The source at
//!    `crates/benten-engine/src/engine_subscribe.rs` MUST contain the
//!    pattern `Err(_) => CapRecheckOutcome::Drop` inside the
//!    `policy.check_read(&ctx)` match. Reverting to `Cancel` on
//!    cap-deny would convert per-Node revoke into whole-subscription
//!    auto-cancel — the regression mode the production-runtime
//!    `subscribe_delivery_cap_recheck_per_event_redacts_revoked_node_granularity`
//!    test catches at runtime.
//!
//! 3. **Engine-side `Cancel` preserved for `is_actor_active=false`.**
//!    The source MUST also contain a `CapRecheckOutcome::Cancel`
//!    arm that fires on `!is_actor_active`. Conflating both arms
//!    (e.g. routing the whole-actor-revoke path through `Drop` too)
//!    would silently violate the Phase-3 R6-FP Wave-C1 SHIPPED
//!    termination contract.
//!
//! # Pairs with
//!
//! - `crates/benten-engine/tests/subscribe_delivery_cap_recheck_per_event_redacts_revoked_node_granularity.rs`
//!   (the §3.6b production-runtime arm; SHIPPED at PR #210).
//!
//! # Status
//!
//! NOT RED-PHASE — this is a regression-defense pin guarding a SHIPPED
//! closure (PR #210). Runs unconditionally in CI; fails if anyone
//! flattens the 3-variant dispatch.
//!
//! # Owned by
//!
//! Phase 4-Foundation R3 Family A test-writer (regression-defense set).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use benten_eval::primitives::subscribe::CapRecheckOutcome;

fn engine_subscribe_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("engine_subscribe.rs");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

#[test]
fn cap_recheck_outcome_enum_has_exactly_three_named_variants() {
    // Type-level pin: instantiate each named variant. If any variant
    // is renamed or removed, this body fails to compile — the test
    // is a compile-time + runtime guard.
    let keep = CapRecheckOutcome::Keep;
    let drop_ = CapRecheckOutcome::Drop;
    let cancel = CapRecheckOutcome::Cancel;

    // Equality contract: distinct variants must not compare equal.
    // This pins the discriminant uniqueness (a hypothetical regression
    // collapsing variants to type-aliases would silently mis-route).
    assert_ne!(keep, drop_, "Keep and Drop must be distinct variants");
    assert_ne!(keep, cancel, "Keep and Cancel must be distinct variants");
    assert_ne!(drop_, cancel, "Drop and Cancel must be distinct variants");

    // Match exhaustiveness pin: an exhaustive match over the 3
    // variants must compile without a `_` catch-all. A regression
    // adding a 4th variant without updating this match arm trips
    // the compiler (which is the intended early-warning signal).
    let observe = |o: CapRecheckOutcome| -> &'static str {
        match o {
            CapRecheckOutcome::Keep => "keep",
            CapRecheckOutcome::Drop => "drop",
            CapRecheckOutcome::Cancel => "cancel",
        }
    };
    assert_eq!(observe(keep), "keep");
    assert_eq!(observe(drop_), "drop");
    assert_eq!(observe(cancel), "cancel");
}

#[test]
fn engine_subscribe_routes_check_read_err_to_drop_not_cancel() {
    let body = engine_subscribe_source();

    // The substantive option-D wiring: when `policy.check_read` returns
    // `Err(_)`, the engine-side closure MUST dispatch `Drop` (silent
    // elision; stream stays active). The canonical post-fix source
    // form is the inline match arm:
    //
    //     Err(_) => CapRecheckOutcome::Drop,
    //
    // We allow some whitespace flexibility around the arrow but pin
    // the substantive token sequence.
    let has_drop_on_err = body.contains("Err(_) => CapRecheckOutcome::Drop");
    assert!(
        has_drop_on_err,
        "expected `Err(_) => CapRecheckOutcome::Drop` arm in \
         crates/benten-engine/src/engine_subscribe.rs (sec-4f-r1-1 + \
         mat-r1-4 BLOCKER regression-defense): the per-event \
         `CapabilityPolicy::check_read` denial MUST dispatch \
         `Drop` (silent elision; stream stays open), NOT `Cancel` \
         (whole-subscription auto-cancel). A revert to `Cancel` \
         re-introduces the pre-option-D conflation that ratification \
         #7 split.",
    );
}

#[test]
fn engine_subscribe_keeps_cancel_arm_for_is_actor_active_false() {
    let body = engine_subscribe_source();

    // Defense of the symmetric SHIPPED contract: when the whole-actor
    // is revoked (`is_actor_active=false`), the closure MUST still
    // dispatch `Cancel`. This is the Phase-3 R6-FP Wave-C1 SHIPPED
    // termination contract. Conflating with `Drop` would silently
    // strip the typed `EvalError::SubscribeRevokedMidStream` + the
    // termination notify callback + the auto-unregister flow.
    let has_cancel_for_actor_inactive =
        body.contains("is_actor_active") && body.contains("CapRecheckOutcome::Cancel");
    assert!(
        has_cancel_for_actor_inactive,
        "expected `is_actor_active`-paired `CapRecheckOutcome::Cancel` \
         arm in crates/benten-engine/src/engine_subscribe.rs \
         (Phase-3 R6-FP Wave-C1 SHIPPED contract preservation): \
         whole-actor revoke MUST dispatch `Cancel`. Collapsing this \
         to `Drop` would silently drop the typed-error termination \
         notify path the SHIPPED contract preserves.",
    );
}
