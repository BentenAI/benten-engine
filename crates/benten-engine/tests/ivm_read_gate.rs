//! GREEN-PHASE pins for IVM per-row read-gate at materialization
//! (G15-A wave-5a; closes Compromise #11 in coordination with G14-D).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.3 G15-A rows
//!   `ivm_view_read_gate_fires_at_materialization_separately_from_g14_d_delivery_gate`
//!   + `ivm_view_per_row_read_gate_against_actor_cap_set`.
//! - plan §3 G15-A row + plan §1 deliverable 6 (Compromise #11
//!   per-row read-gate closure).
//! - `ivm-major-2` (gate fires AT MATERIALIZATION TIME, separately
//!   from G14-D delivery-time gate).
//! - LOAD-BEARING #11 closure pin per plan §1 line "Compromise #11
//!   ... closed end-to-end".

#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;
use std::sync::Arc;

use benten_core::{Cid, Node, Value};
use benten_engine::cap_recheck::{CapRecheckFn, PrincipalId};
use benten_engine::ivm_view_read_gate::IvmViewReadGate;

fn principal_for(label: &str) -> PrincipalId {
    let mut props = std::collections::BTreeMap::new();
    props.insert(String::from("name"), Value::text(label));
    let node = Node::new(vec!["actor".to_string()], props);
    PrincipalId::from_actor_cid(node.cid().unwrap())
}

fn cid_for_label(label: &str, idx: u64) -> Cid {
    let mut props = std::collections::BTreeMap::new();
    props.insert(String::from("seq"), Value::Int(idx as i64));
    let node = Node::new(vec![label.to_string()], props);
    node.cid().unwrap()
}

#[test]
fn ivm_view_read_gate_fires_at_materialization_separately_from_g14_d_delivery_gate() {
    // ivm-major-2 pin. The per-row READ gate fires at MATERIALIZATION
    // TIME — not at delivery time (which is G14-D's job). Concrete
    // shape: instantiating an IvmViewReadGate with a custom
    // CapRecheckFn produces row-level filtering that is independent
    // of the G14-D SUBSCRIBE delivery layer (which is wired in
    // engine_subscribe.rs).
    //
    // The gate fires per-row at materialization: a cap-recheck closure
    // that denies CIDs in `/zone/posts/private/...` (encoded here as
    // CIDs minted from `post:private`-labeled Nodes) yields ONLY the
    // public rows from the materialised list, NOT the private ones.
    // Crucially this happens WITHOUT any SUBSCRIBE channel — the gate
    // is part of the materialization path.
    let public_rows: Vec<Cid> = (0..5).map(|i| cid_for_label("post:public", i)).collect();
    let private_rows: Vec<Cid> = (0..5).map(|i| cid_for_label("post:private", i)).collect();
    let public_set: BTreeSet<Cid> = public_rows.iter().copied().collect();
    let public_set_arc = Arc::new(public_set);
    let cap_recheck: CapRecheckFn = {
        let set = Arc::clone(&public_set_arc);
        Arc::new(move |_p: &PrincipalId, _zone: &str, cid: &Cid| set.contains(cid))
    };
    let gate = IvmViewReadGate::new(principal_for("alice"), "post", cap_recheck);

    let mut all_rows = Vec::with_capacity(10);
    all_rows.extend(public_rows.iter().copied());
    all_rows.extend(private_rows.iter().copied());

    let admitted = gate.filter_rows(all_rows);
    assert_eq!(admitted.len(), 5, "gate fires per-row at materialization");
    for cid in &admitted {
        assert!(
            public_set_arc.contains(cid),
            "every admitted row in public set"
        );
    }

    // Independence from G14-D: the gate produces deterministic results
    // without invoking any subscription path (no Engine instance, no
    // SUBSCRIBE, no ChangeEvent stream). The same fixture would
    // produce identical filtering at materialization regardless of
    // SUBSCRIBE state.
    let admitted_again = gate.filter_rows(public_rows.iter().copied().chain(private_rows));
    assert_eq!(admitted_again.len(), 5, "deterministic across calls");
}

#[test]
fn ivm_view_per_row_read_gate_against_actor_cap_set() {
    // LOAD-BEARING per plan §1 deliverable 6: Compromise #11 closes
    // end-to-end. Under a 100-row fixture split 50/50 public vs
    // private, an actor with READ caps only on public sees EXACTLY
    // 50 rows in their materialised view. The pre-G15-A coarse gate
    // would have answered 0 (deny entire view) or 100 (admit entire
    // view) — never 50.
    let public_rows: Vec<Cid> = (0..50).map(|i| cid_for_label("post:public", i)).collect();
    let private_rows: Vec<Cid> = (0..50).map(|i| cid_for_label("post:private", i)).collect();
    let public_set: BTreeSet<Cid> = public_rows.iter().copied().collect();
    let public_set_arc = Arc::new(public_set);

    // Per-row check: admit iff CID is in the public set. Models a
    // fixture grant that scopes Alice to /zone/posts/public/*.
    let cap_recheck: CapRecheckFn = {
        let set = Arc::clone(&public_set_arc);
        Arc::new(move |_p: &PrincipalId, _zone: &str, cid: &Cid| set.contains(cid))
    };

    let gate = IvmViewReadGate::new(principal_for("alice"), "post", cap_recheck);

    let mut all_rows = Vec::with_capacity(100);
    all_rows.extend(public_rows.iter().copied());
    all_rows.extend(private_rows.iter().copied());

    let admitted = gate.filter_rows(all_rows);
    assert_eq!(
        admitted.len(),
        50,
        "per-row gate yields exactly 50 (not 0, not 100) per Compromise #11 closure"
    );
    for cid in &admitted {
        assert!(public_set_arc.contains(cid));
    }
}

#[test]
fn materialize_view_with_gate_filters_rows_per_actor_cap_set_at_engine_entry_point_e2e() {
    // LOAD-BEARING pim-2 §3.6b end-to-end pin (g15a-mr-blocker-3
    // closure). Drives the production `Engine::materialize_view_with_gate`
    // entry point with Nodes WRITTEN through the engine's normal
    // transaction surface (so the IVM subscriber materialises them via
    // ChangeEvents) + a CapRecheckFn that admits some CIDs and denies
    // others. Asserts row-level filtering: the result contains EXACTLY
    // the admitted CIDs — not all of them (would fail if the gate were
    // silently bypassed) and not none of them (would fail if the arm
    // returned `Ok(Some(Vec::new()))` unconditionally).
    use benten_core::{Node, Value};
    use benten_engine::{Engine, UserViewInputPattern, UserViewSpec};

    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let spec = UserViewSpec::builder()
        .id("custom:e2e_gate")
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        .build()
        .unwrap();
    engine.register_user_view(spec).unwrap();

    // Write 4 Nodes through the engine's transaction surface: 2 'post'
    // (matching the registered view's label pattern) + 2 'user'
    // (non-matching, won't enter the materialised list anyway). Of the
    // 2 'post' Nodes the gate admits ONE and denies the OTHER.
    let mut admitted_props = std::collections::BTreeMap::new();
    admitted_props.insert("kind".into(), Value::text("admitted"));
    let admitted_node = Node::new(vec!["post".to_string()], admitted_props);
    let admitted_cid = admitted_node.cid().unwrap();

    let mut denied_props = std::collections::BTreeMap::new();
    denied_props.insert("kind".into(), Value::text("denied"));
    let denied_node = Node::new(vec!["post".to_string()], denied_props);
    let denied_cid = denied_node.cid().unwrap();

    let mut user_a_props = std::collections::BTreeMap::new();
    user_a_props.insert("name".into(), Value::text("a"));
    let user_a = Node::new(vec!["user".to_string()], user_a_props);

    let mut user_b_props = std::collections::BTreeMap::new();
    user_b_props.insert("name".into(), Value::text("b"));
    let user_b = Node::new(vec!["user".to_string()], user_b_props);

    engine
        .transaction(|tx| {
            for n in [&admitted_node, &denied_node, &user_a, &user_b] {
                tx.put_node(n)
                    .map_err(|e| benten_engine::EngineError::Other {
                        code: benten_errors::ErrorCode::Unknown("E_TEST_HARNESS".into()),
                        message: format!("put_node: {e:?}"),
                    })?;
            }
            Ok(())
        })
        .expect("commit four mixed-label Nodes");

    // Construct a gate that admits ONLY `admitted_cid`.
    let admitted_set: BTreeSet<Cid> = std::iter::once(admitted_cid).collect();
    let admitted_set_arc = Arc::new(admitted_set);
    let cap_recheck: CapRecheckFn = {
        let set = Arc::clone(&admitted_set_arc);
        Arc::new(move |_p: &PrincipalId, _zone: &str, cid: &Cid| set.contains(cid))
    };
    let gate = IvmViewReadGate::new(principal_for("alice"), "post", cap_recheck);

    let result = engine
        .materialize_view_with_gate("custom:e2e_gate", &gate)
        .expect("materialize_view_with_gate succeeds");
    let cids = result.expect("Some(cids) for a registered view");
    assert_eq!(
        cids.len(),
        1,
        "exactly one row admitted (not 2 = gate-bypass; not 0 = arm-no-op); \
         pim-2 §3.6b end-to-end behavior: gate filters rows at the engine \
         entry point. cids = {cids:?}"
    );
    assert_eq!(
        cids[0], admitted_cid,
        "the admitted CID is the one the cap-recheck closure permits"
    );
    assert!(
        !cids.contains(&denied_cid),
        "the denied CID is suppressed at materialization time"
    );

    // Smoke-check: an allow-all gate against the same view sees BOTH
    // 'post' Nodes (further proves the 1-row count above is gate-driven,
    // not view-empty).
    let allow_gate = IvmViewReadGate::allow_all_for(principal_for("alice"), "post");
    let allowed = engine
        .materialize_view_with_gate("custom:e2e_gate", &allow_gate)
        .unwrap()
        .expect("Some(cids)");
    assert_eq!(
        allowed.len(),
        2,
        "allow-all gate observes both 'post' Nodes (admitted + denied); \
         gate is the load-bearing filter, not the view's content."
    );

    // Unknown view -> Ok(None) (existing contract preserved).
    let unknown = engine
        .materialize_view_with_gate("custom:no_such_view", &allow_gate)
        .unwrap();
    assert!(unknown.is_none(), "unknown view-id yields Ok(None)");
}
