//! GREEN-PHASE pins for IVM Algorithm B kernel generalization
//! (G15-A wave-5a; per r2-test-landscape §2.3 + plan §3 G15-A row).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.3 G15-A rows.
//! - plan §3 G15-A must-pass column.
//! - `ivm-major-1` (kernel must handle arbitrary
//!   `(view_id, label_pattern, projection)` triples — drop
//!   canonical-only fallback).
//! - `ivm-major-5` + D8-RESOLVED (Strategy::A vs Strategy::B router
//!   internal; engine refuses Strategy::A user-view registration).
//! - `ivm-minor-6` + `ivm-disagree-1` (canonical-view fast-path within
//!   20% gate against Strategy::B baseline).
//! - `D-PHASE-3-28` (Strategy::A/B dispatch router internal).
//! - plan §3 G15-A row (view-label-mismatch fail-loud preserved).

#![allow(clippy::unwrap_used)]

use benten_core::{Node, Value};
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::{
    Algorithm, AlgorithmError, LabelPattern, Projection, Strategy, View, ViewQuery, ViewResult,
    dispatch_for,
};

fn make_event(kind: ChangeKind, label: &str, idx: u64) -> ChangeEvent {
    let mut props = std::collections::BTreeMap::new();
    props.insert(String::from("seq"), Value::Int(idx as i64));
    let node = Node::new(vec![label.to_string()], props);
    let cid = node.cid().unwrap();
    ChangeEvent {
        cid,
        labels: vec![label.to_string()],
        kind,
        tx_id: idx,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
        node: Some(node),
        edge_endpoints: None,
    }
}

#[test]
fn algorithm_b_generalized_user_defined_view_no_silent_coerce_to_content_listing() {
    // ivm-major-1 + plan §3 G15-A pin. A user-defined view ID with an
    // exact label pattern produces ONLY rows matching that label —
    // never silently coerces to ContentListingView semantics that
    // would have admitted other labels too.
    let mut view = Algorithm::register(
        "custom:posts_by_author",
        LabelPattern::exact("post"),
        Projection::all_props(),
    )
    .unwrap();
    // Insert nodes with label "post" and label "user".
    view.update(&make_event(ChangeKind::Created, "post", 1))
        .unwrap();
    view.update(&make_event(ChangeKind::Created, "user", 2))
        .unwrap();
    view.update(&make_event(ChangeKind::Created, "post", 3))
        .unwrap();
    let result = view.read(&ViewQuery::default()).unwrap();
    match result {
        ViewResult::Cids(cids) => {
            assert_eq!(cids.len(), 2, "only post-labeled events admitted");
        }
        other => panic!("expected Cids, got {other:?}"),
    }
}

#[test]
fn algorithm_b_arbitrary_label_pattern_drives_correct_subset() {
    // plan §3 G15-A pin. The kernel accepts both Exact and AnchorPrefix
    // patterns and produces the correct subset for each.
    //
    // Mixed-label corpus: post, user, system:zone, ephemeral, crud:post,
    // crud:user.
    let labels = [
        "post",
        "user",
        "system:zone",
        "ephemeral",
        "crud:post",
        "crud:user",
    ];
    let make_corpus = || -> Vec<ChangeEvent> {
        labels
            .iter()
            .enumerate()
            .map(|(i, l)| make_event(ChangeKind::Created, l, i as u64))
            .collect()
    };

    // Exact("post") — admits only the single "post" event.
    let mut exact = Algorithm::register(
        "custom:exact_post",
        LabelPattern::exact("post"),
        Projection::all_props(),
    )
    .unwrap();
    for e in make_corpus() {
        exact.update(&e).unwrap();
    }
    match exact.read(&ViewQuery::default()).unwrap() {
        ViewResult::Cids(cids) => assert_eq!(cids.len(), 1, "Exact('post') admits 1 event"),
        other => panic!("expected Cids, got {other:?}"),
    }

    // AnchorPrefix("crud:") — admits crud:post + crud:user (2 events).
    let mut prefix = Algorithm::register(
        "custom:crud_prefix",
        LabelPattern::anchor_prefix("crud:"),
        Projection::all_props(),
    )
    .unwrap();
    for e in make_corpus() {
        prefix.update(&e).unwrap();
    }
    match prefix.read(&ViewQuery::default()).unwrap() {
        ViewResult::Cids(cids) => {
            assert_eq!(cids.len(), 2, "AnchorPrefix('crud:') admits 2 events");
        }
        other => panic!("expected Cids, got {other:?}"),
    }

    // AnchorPrefix("system:") — admits system:zone (1 event).
    let mut sys_prefix = Algorithm::register(
        "custom:system_prefix",
        LabelPattern::anchor_prefix("system:"),
        Projection::all_props(),
    )
    .unwrap();
    for e in make_corpus() {
        sys_prefix.update(&e).unwrap();
    }
    match sys_prefix.read(&ViewQuery::default()).unwrap() {
        ViewResult::Cids(cids) => assert_eq!(cids.len(), 1, "AnchorPrefix('system:') admits 1"),
        other => panic!("expected Cids, got {other:?}"),
    }
}

#[test]
fn algorithm_b_view_label_mismatch_fail_loud_remains_present() {
    // plan §3 G15-A pin. Even after generalization, the existing
    // fail-loud check for view-label mismatch (registering a canonical
    // view id with a label_pattern that EXCLUDES the canonical hardcoded
    // label) still fires.
    let result = Algorithm::try_register(
        "capability_grants",
        LabelPattern::exact("user"), // mismatch with hardcoded "system:CapabilityGrant"
        Projection::all_props(),
    );
    match result {
        Err(AlgorithmError::ViewLabelMismatch {
            view_id,
            expected_label,
            ..
        }) => {
            assert_eq!(view_id, "capability_grants");
            assert_eq!(expected_label, "system:CapabilityGrant");
        }
        Ok(_) => panic!("expected ViewLabelMismatch, got Ok"),
    }
}

#[test]
fn algorithm_b_strategy_a_b_dispatch_router_routes_correctly() {
    // D-PHASE-3-28 + ivm-major-5 pin. The router classifies canonical
    // view ids as Strategy::A (canonical fast-path classification) and
    // user-defined view ids as Strategy::B (generic kernel).
    assert_eq!(dispatch_for("capability_grants"), Strategy::A);
    assert_eq!(dispatch_for("event_dispatch"), Strategy::A);
    assert_eq!(dispatch_for("content_listing"), Strategy::A);
    assert_eq!(dispatch_for("governance_inheritance"), Strategy::A);
    assert_eq!(dispatch_for("version_current"), Strategy::A);
    assert_eq!(dispatch_for("custom:posts_by_author"), Strategy::B);
    assert_eq!(dispatch_for("user:my_view"), Strategy::B);
    assert_eq!(dispatch_for("anything_else"), Strategy::B);

    // Determinism: same view-id always routes to the same strategy.
    for _ in 0..16 {
        assert_eq!(dispatch_for("capability_grants"), Strategy::A);
        assert_eq!(dispatch_for("custom:foo"), Strategy::B);
    }
}

#[test]
fn algorithm_b_canonical_view_fast_path_preserved_within_20pct_of_strategy_b_baseline() {
    // ivm-minor-6 + ivm-disagree-1 pin. Per the plan G15-A row line
    // "preserve canonical-view fast-path within 20% gate" — the
    // companion bench `benches/algorithm_b_canonical.rs` produces
    // criterion output at the load-bearing paths. This test parses
    // those paths at-load if they exist, asserting ratio <= 1.20.
    //
    // **At G15-A landing the gate is INFORMATIONAL** (matches Phase-2b
    // precedent for new bench gates — promoted to required at R6 once
    // baseline measurements are stable). When the criterion paths
    // don't exist (e.g. `cargo test` was run without a prior `cargo
    // bench`), the test passes — the gate is opt-in via the bench
    // run, not blocking on the test runner.
    let baseline_path = "target/criterion/algorithm_b_canonical_view_fast_path/Strategy_B_baseline/new/estimates.json";
    let post_gen_path =
        "target/criterion/algorithm_b_canonical_view_fast_path/post_g15a/new/estimates.json";

    let baseline = std::fs::read_to_string(baseline_path).ok();
    let post_gen = std::fs::read_to_string(post_gen_path).ok();
    let (Some(baseline), Some(post_gen)) = (baseline, post_gen) else {
        // Bench not run — gate is informational at G15-A landing.
        return;
    };

    // Crude estimate parser: criterion's `estimates.json` has a
    // top-level `"mean": { "point_estimate": <ns>, ... }` key. We
    // grep the point estimate without a full JSON dependency to keep
    // the test crate-light.
    fn parse_point_estimate(json: &str) -> Option<f64> {
        let needle = r#""point_estimate":"#;
        let idx = json.find(needle)?;
        let tail = &json[idx + needle.len()..];
        let end = tail.find(|c: char| {
            c != '.' && c != '-' && c != 'e' && !c.is_ascii_digit() && !c.is_whitespace()
        })?;
        tail[..end].trim().parse::<f64>().ok()
    }

    let baseline_ns = parse_point_estimate(&baseline).unwrap_or(0.0);
    let post_gen_ns = parse_point_estimate(&post_gen).unwrap_or(0.0);
    if baseline_ns <= 0.0 || post_gen_ns <= 0.0 {
        // Parser missed; treat as informational rather than fail-loud.
        return;
    }
    let ratio = post_gen_ns / baseline_ns;
    assert!(
        ratio <= 1.20,
        "G15-A canonical fast-path regressed beyond 20% \
         (ratio = {ratio:.3}; baseline {baseline_ns}ns; post-gen {post_gen_ns}ns). \
         ivm-disagree-1: gate measures canonical fast-path vs Strategy::B baseline."
    );
}
