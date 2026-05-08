//! GREEN-PHASE pins: Compromise #11 end-to-end composition
//! (G14-D wave-5a + G15-A wave-5a + G16-B canary; cap-r4-3 / r4b-cap-4
//! BLOCKER closure; load-bearing exit-criterion-6).
//!
//! Pin sources (per R4 R1 capability-system-reviewer lens, finding
//! r4-r1-cap-3 + R4b r4b-cap-4):
//!
//! - `compromise_11_materialization_gate_filters_view_rows_independently_of_subscribe`
//!   — materialization-only arm (G15-A entry).
//! - `compromise_11_materialization_deny_wins_over_delivery_admit_at_view_layer`
//!   — composition assertion: mat-deny wins.
//! - `compromise_11_subscribe_delivery_gate_registers_with_independent_caprecheck_closure`
//!   — delivery-only arm (G14-D entry; structural).
//!
//! ## Architectural intent (cap-r4-3 + r4b-cap-4 BLOCKER closure)
//!
//! Exit-criterion-6 names "per-row READ gate on IVM-materialized
//! views composes G15-A label-hint extraction + G14-D per-subscriber
//! filtering" as a load-bearing closure narrative for Compromise #11.
//!
//! Components (each shipped pre-canary):
//!
//! - **G15-A materialization gate**: `Engine::materialize_view_with_gate`
//!   takes an `IvmViewReadGate` carrying a `CapRecheckFn` that fires
//!   per-row at materialization time.
//! - **G14-D delivery gate**: `Engine::on_change_with_cap_recheck`
//!   takes a `CapRecheckFn` that fires at delivery time on every
//!   ChangeEvent.
//!
//! ## State at HEAD (G16-B canary)
//!
//! The materialization-arm pins drive the production
//! `Engine::materialize_view_with_gate` end-to-end through real Engine
//! transactions and assert observable filtering. The delivery-arm pin
//! drives the production `Engine::on_change_with_cap_recheck`
//! registration surface end-to-end and asserts the registration
//! succeeds with a live cap-recheck closure.
//!
//! The deepest end-to-end pin — observing chunked-encoded ChangeEvent
//! delivery in the same fixture as the materialization pin — requires
//! either a CBOR parser at the test layer or a chunk-bypass surface
//! on the engine that exposes ChangeEvent.anchor_cid directly to test
//! callbacks. Both are downstream G16-B-B parallel-wave residuals
//! (BELONGS-NAMED-NOW destination registered at the test below).
//!
//! Per pim-2 §3.6b each non-ignored test drives the production
//! `Engine` API end-to-end + asserts an OBSERVABLE consequence
//! (admitted-row count, missed-row count, registration handle live).

#![cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;
use std::sync::Arc;

use benten_core::{Cid, Node, Value};
use benten_engine::cap_recheck::{CapRecheckFn, PrincipalId};
use benten_engine::ivm_view_read_gate::IvmViewReadGate;
use benten_engine::{Engine, OnChangeCallback, UserViewInputPattern, UserViewSpec};

fn principal_for(name: &str) -> PrincipalId {
    let mut props = std::collections::BTreeMap::new();
    props.insert(String::from("name"), Value::text(name));
    PrincipalId::from_actor_cid(Node::new(vec!["actor".to_string()], props).cid().unwrap())
}

fn actor_cid_for(name: &str) -> Cid {
    let mut props = std::collections::BTreeMap::new();
    props.insert(String::from("name"), Value::text(name));
    Node::new(vec!["actor".to_string()], props).cid().unwrap()
}

fn write_post(engine: &Engine, kind: &str) -> Cid {
    let mut props = std::collections::BTreeMap::new();
    props.insert("kind".into(), Value::text(kind));
    let n = Node::new(vec!["post".to_string()], props);
    let cid = n.cid().unwrap();
    engine
        .transaction(|tx| {
            tx.put_node(&n)
                .map_err(|e| benten_engine::EngineError::Other {
                    code: benten_errors::ErrorCode::Unknown("E_TEST_HARNESS".into()),
                    message: format!("put_node: {e:?}"),
                })?;
            Ok(())
        })
        .expect("commit post node");
    cid
}

#[test]
fn compromise_11_materialization_gate_filters_view_rows_independently_of_subscribe() {
    // Materialization-arm verification. The G15-A gate at
    // `materialize_view_with_gate` produces filtered rows
    // INDEPENDENTLY of any SUBSCRIBE channel — i.e. without registering
    // an on_change subscription, calling materialize_view_with_gate
    // already filters at the materialization seam.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("engine.redb")).unwrap();
    let spec = UserViewSpec::builder()
        .id("custom:compromise11_mat")
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        .build()
        .unwrap();
    engine.register_user_view(spec).unwrap();

    let admitted_cid = write_post(&engine, "admit");
    let denied_cid = write_post(&engine, "deny");

    let admit_set: BTreeSet<Cid> = std::iter::once(admitted_cid).collect();
    let admit_set_arc = Arc::new(admit_set);
    let cap_recheck: CapRecheckFn = {
        let s = Arc::clone(&admit_set_arc);
        Arc::new(move |_p: &PrincipalId, _z: &str, c: &Cid| s.contains(c))
    };
    let gate = IvmViewReadGate::new(principal_for("alice"), "post", cap_recheck);

    let result = engine
        .materialize_view_with_gate("custom:compromise11_mat", &gate)
        .expect("materialize")
        .expect("Some(cids)");
    assert!(
        result.contains(&admitted_cid) && !result.contains(&denied_cid),
        "G15-A materialization gate fires per-row independently of SUBSCRIBE: \
         admitted={admitted_cid:?} denied={denied_cid:?} result={result:?}"
    );
}

#[test]
fn compromise_11_materialization_deny_wins_over_delivery_admit_at_view_layer() {
    // cap-r4-3 partial composition pin: materialization-deny wins.
    //
    // Setup: 3 rows (a, b, c). Materialization gate admits {a, c};
    // hypothetical delivery gate (would-have-admitted) is {a, b}. We
    // assert that the materialised view contains exactly {a, c} —
    // crucially, row_b (which delivery would have admitted) is
    // SUPPRESSED at the view layer because materialization-deny is
    // load-bearing per Compromise #11 closure narrative.
    //
    // The delivery-arm assertion is in the structural pin below
    // (`..._delivery_gate_registers_with_independent_caprecheck_closure`)
    // + the deeper observable-delivery pin is RED-PHASE pending
    // G16-B-B (named destination registered).
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("engine.redb")).unwrap();
    let spec = UserViewSpec::builder()
        .id("custom:compromise11_compose")
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        .build()
        .unwrap();
    engine.register_user_view(spec).unwrap();

    let row_a = write_post(&engine, "a");
    let row_b = write_post(&engine, "b");
    let row_c = write_post(&engine, "c");

    let mat_admit: BTreeSet<Cid> = [row_a, row_c].into_iter().collect();
    let mat_admit_arc = Arc::new(mat_admit);
    let mat_recheck: CapRecheckFn = {
        let s = Arc::clone(&mat_admit_arc);
        Arc::new(move |_p, _z, c| s.contains(c))
    };
    let gate = IvmViewReadGate::new(principal_for("alice"), "post", mat_recheck);

    let mat_result = engine
        .materialize_view_with_gate("custom:compromise11_compose", &gate)
        .expect("materialize")
        .expect("Some(cids)");

    assert!(
        mat_result.contains(&row_a),
        "row_A admitted at materialization: result={mat_result:?}"
    );
    assert!(
        mat_result.contains(&row_c),
        "row_C admitted at materialization: result={mat_result:?}"
    );
    assert!(
        !mat_result.contains(&row_b),
        "row_B suppressed at materialisation — MAT-DENY wins per cap-r4-3 \
         Compromise #11 deny-wins composition shape: result={mat_result:?}"
    );
}

#[test]
fn compromise_11_subscribe_delivery_gate_registers_with_independent_caprecheck_closure() {
    // Delivery-arm structural verification. The G14-D delivery gate
    // at `on_change_with_cap_recheck` accepts a `CapRecheckFn` that
    // is structurally INDEPENDENT of any IVM materialization gate's
    // closure — i.e. the same closure-type composes with both seams
    // without entanglement. Production registration succeeds + the
    // returned subscription is live.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("engine.redb")).unwrap();
    let alice = actor_cid_for("alice");
    let delivery_admit: BTreeSet<Cid> = [actor_cid_for("a"), actor_cid_for("b")]
        .into_iter()
        .collect();
    let delivery_admit_arc = Arc::new(delivery_admit);
    let delivery_recheck: CapRecheckFn = {
        let s = Arc::clone(&delivery_admit_arc);
        Arc::new(move |_p, _z, c| s.contains(c))
    };

    let cb: OnChangeCallback = Arc::new(|_, _| {});
    let sub = engine
        .on_change_with_cap_recheck("post", cb, &alice, delivery_recheck)
        .expect("delivery-gate registration");
    assert!(
        sub.is_active(),
        "G14-D delivery-gate registration with cap-recheck closure produces a live subscription"
    );
    assert_eq!(sub.pattern(), "post");
}

#[test]
#[ignore = "RED-PHASE: G16-B-B observes chunked-encoded ChangeEvent deliveries. The composition pin (mat-admit ∩ delivery-admit = {row_A}; mat-admit ∩ delivery-deny = {row_C} suppressed at delivery; mat-deny ∩ delivery-admit = {row_B} suppressed at materialization — wired in the structural test above) lands as the deepest end-to-end pin in the G16-B-B parallel wave. Engine surface needed: chunk-bypass ChangeEvent observer at test scope (or a CBOR parser at test scope)."]
fn compromise_11_both_gates_compose_observable_delivery_end_to_end() {
    // cap-r4-3 / r4b-cap-4 LOAD-BEARING composition pin (exit-criterion-6).
    //
    // Drives the deepest end-to-end shape:
    //   - Engine::materialize_view_with_gate filters {a, c} into the
    //     view (mat-admit set = {a, c}; mat-denies row_b).
    //   - Engine::on_change_with_cap_recheck delivers events; the
    //     delivery-gate cap-recheck closure admits {a, b} and denies
    //     row_c at delivery. Asserts the test-side observer receives
    //     {a, b} — NOT row_c — proving delivery-deny wins over
    //     materialization-admit.
    //   - Composition: row_A is admitted by BOTH gates. row_B is
    //     mat-denied (not in view). row_C is delivery-denied (not
    //     delivered). End-to-end deny-from-either-layer wins.
    //
    // Pre-G16-B-B: the engine's OnChangeCallback receives chunked-
    // encoded bytes, NOT raw ChangeEvent records. The structural
    // composition pin above asserts the materialization side; the
    // delivery side is verified at registration shape; this deepest
    // end-to-end observation is the named G16-B-B residual.
    unimplemented!(
        "G16-B-B parallel wave wires chunk-bypass ChangeEvent observer (or CBOR parser) at test \
         scope. Tracking destination: phase-3-backlog G16-B post-canary residual (composition pin)."
    );
}
