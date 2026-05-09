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
#[allow(
    clippy::too_many_lines,
    reason = "load-bearing dual-gate composition pin"
)]
fn compromise_11_both_gates_compose_observable_delivery_end_to_end() {
    // cap-r4-3 / r4b-cap-4 LOAD-BEARING composition pin (exit-criterion-6).
    //
    // G16-B-D un-ignored using the new
    // `Engine::testing_subscribe_observable_change_events` helper
    // (cfg-gated under `cfg(any(test, feature = "test-helpers"))`)
    // which exposes the eval-side ChangeEvent directly without the
    // chunk-encoding bridge that the production `on_change` adapter
    // applies.
    //
    // Composition shape:
    //   - mat-admit set = {row_A, row_C}; mat-denies row_B
    //     (materialization gate at G15-A `materialize_view_with_gate`).
    //   - delivery-admit set = {row_A, row_B}; delivery-denies row_C
    //     (delivery gate at G14-D `on_change_with_cap_recheck`).
    //   - Materialised view: {row_A, row_C} (row_B suppressed at mat).
    //   - Delivery observer: {row_A, row_B} (row_C suppressed at delivery).
    //   - Composition (intersection / deny-from-either-layer wins):
    //       row_A: admitted by BOTH (materialised AND delivered)
    //       row_B: mat-denied (not in view); delivery-admitted (delivered to observer)
    //       row_C: mat-admitted (in view); delivery-denied (NOT delivered)
    //
    // The test asserts: (1) materialised view = {A, C}; (2) delivery
    // observer received row_A's anchor_cid (mat-admit ∩ delivery-admit);
    // (3) delivery observer received row_B's anchor_cid (mat-deny but
    // delivery-admit — proves delivery is independent of mat); (4)
    // delivery observer did NOT receive row_C (delivery-deny); (5) the
    // intersection — rows admitted by BOTH gates end-to-end — is
    // exactly {row_A}, the load-bearing closure assertion of
    // Compromise #11 dual-gate composition.
    use std::sync::Mutex;

    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("engine.redb")).unwrap();
    // Use a unique label "compromise11post" (not the bare "post" used
    // by sibling tests) so the process-wide eval-side ON_CHANGE_REGISTRY
    // doesn't deliver THIS subscription's callback events from sibling
    // tests writing "post"-labeled nodes — a parallel-test-isolation
    // measure since the registry is process-scoped (`LazyLock<Mutex<HashMap>>`).
    let spec = UserViewSpec::builder()
        .id("custom:compromise11_e2e")
        .input_pattern(UserViewInputPattern::Label("compromise11post".to_string()))
        .build()
        .unwrap();
    engine.register_user_view(spec).unwrap();

    let alice = actor_cid_for("alice");

    // Pre-compute the row anchor CIDs WITHOUT writing yet. The CIDs are
    // content-addressed so they are determined fully by (label, props)
    // and don't require the engine to have observed the WRITE. This
    // lets us register the subscription BEFORE the WRITEs happen so
    // every WRITE generates a ChangeEvent the subscription observes.
    let make_post_cid = |kind: &str| -> Cid {
        let mut props = std::collections::BTreeMap::new();
        props.insert("kind".into(), Value::text(kind));
        Node::new(vec!["compromise11post".to_string()], props)
            .cid()
            .unwrap()
    };
    let row_a = make_post_cid("a");
    let row_b = make_post_cid("b");
    let row_c = make_post_cid("c");

    // Delivery gate: admits {A, B}, denies C. Pre-bound BEFORE
    // subscription registration since cap-recheck closure captures it.
    let delivery_admit: BTreeSet<Cid> = [row_a, row_b].into_iter().collect();
    let delivery_admit_arc = Arc::new(delivery_admit);
    let delivery_recheck: CapRecheckFn = {
        let s = Arc::clone(&delivery_admit_arc);
        Arc::new(move |_p, _z, c| s.contains(c))
    };

    // Test-helper observer captures raw ChangeEvent.anchor_cid values.
    let observed: Arc<Mutex<Vec<Cid>>> = Arc::new(Mutex::new(Vec::new()));
    let observer: Arc<
        dyn Fn(&benten_eval::primitives::subscribe::ChangeEvent) + Send + Sync + 'static,
    > = {
        let observed = Arc::clone(&observed);
        Arc::new(move |event| {
            observed.lock().unwrap().push(event.anchor_cid);
        })
    };

    // Register subscription BEFORE writes so the subscription observes
    // the first WRITE per row (content-addressing means re-WRITEs
    // dedup at the storage layer; only the first WRITE emits a fresh
    // ChangeEvent the eval-side delivery walk fans out).
    let _sub = engine
        .testing_subscribe_observable_change_events(
            "compromise11post*",
            observer,
            &alice,
            delivery_recheck,
        )
        .expect("test-helper subscription");

    // Now WRITE the rows under the unique label. Each WRITE drives a
    // ChangeEvent through the eval-side delivery walk; the cap-recheck
    // bridge filters per anchor_cid; observer fires for {A, B} but not C.
    let write_unique = |kind: &str| -> Cid {
        let mut props = std::collections::BTreeMap::new();
        props.insert("kind".into(), Value::text(kind));
        let n = Node::new(vec!["compromise11post".to_string()], props);
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
            .expect("commit unique-label post node");
        cid
    };
    let row_a_written = write_unique("a");
    let row_b_written = write_unique("b");
    let row_c_written = write_unique("c");
    assert_eq!(row_a, row_a_written, "content-addressed CID stable for A");
    assert_eq!(row_b, row_b_written, "content-addressed CID stable for B");
    assert_eq!(row_c, row_c_written, "content-addressed CID stable for C");

    // Allow the change-stream walk to drain. Sibling pins use a brief
    // drain; mirror that.
    std::thread::sleep(std::time::Duration::from_millis(150));

    // Materialization gate: admits {A, C}, denies B. Run AFTER writes
    // so the IVM view has the rows materialised.
    let mat_admit: BTreeSet<Cid> = [row_a, row_c].into_iter().collect();
    let mat_admit_arc = Arc::new(mat_admit);
    let mat_recheck: CapRecheckFn = {
        let s = Arc::clone(&mat_admit_arc);
        Arc::new(move |_p, _z, c| s.contains(c))
    };
    let mat_gate = IvmViewReadGate::new(principal_for("alice"), "post", mat_recheck);

    let mat_result = engine
        .materialize_view_with_gate("custom:compromise11_e2e", &mat_gate)
        .expect("materialize")
        .expect("Some(cids)");
    assert!(
        mat_result.contains(&row_a) && mat_result.contains(&row_c) && !mat_result.contains(&row_b),
        "MAT gate admits {{A, C}}, denies B: result={mat_result:?}"
    );

    let observed_set: BTreeSet<Cid> = observed.lock().unwrap().iter().copied().collect();

    // Delivery-side assertions:
    //   row_A delivered (mat-admit ∩ delivery-admit)
    //   row_B delivered (mat-deny ∩ delivery-admit — delivery is independent of MAT gate)
    //   row_C NOT delivered (delivery-deny wins regardless of mat-admit)
    assert!(
        observed_set.contains(&row_a),
        "row_A delivered (BOTH gates admit): observed={observed_set:?}"
    );
    assert!(
        observed_set.contains(&row_b),
        "row_B delivered (delivery-admit independent of mat-deny): observed={observed_set:?}"
    );
    assert!(
        !observed_set.contains(&row_c),
        "row_C NOT delivered (delivery-deny wins): observed={observed_set:?}"
    );

    // End-to-end intersection: rows admitted by BOTH gates end-to-end
    // (in materialised view AND delivered to observer) = {row_A}.
    let mat_set: BTreeSet<Cid> = mat_result.iter().copied().collect();
    let intersection: BTreeSet<Cid> = mat_set.intersection(&observed_set).copied().collect();
    assert_eq!(
        intersection,
        std::iter::once(row_a).collect::<BTreeSet<Cid>>(),
        "Compromise #11 dual-gate composition: end-to-end admit = {{row_A}} only \
         (mat ∩ delivery). observed={observed_set:?} mat={mat_set:?}"
    );
}
