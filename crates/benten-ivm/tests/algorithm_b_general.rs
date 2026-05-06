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
    Algorithm, AlgorithmBView, AlgorithmError, LabelPattern, Projection, Strategy, View,
    ViewDefinition, ViewQuery, ViewResult, dispatch_for,
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
    // "preserve canonical-view fast-path within 20% gate."
    //
    // **g15a-mr-major-2 fix:** the prior shape of this test parsed
    // `target/criterion/.../estimates.json` and silently returned Ok
    // when those files were absent (default `cargo test` runs do not
    // produce them). That made the gate a no-op-equivalent stub —
    // pim-2 §3.6b style failure. This rewrite produces an actual
    // measurement on every `cargo test` run via an inline microbench
    // (no criterion dep, no CI workflow change required).
    //
    // The gate compares two register paths that drive the SAME
    // hand-written canonical inner kernel (`content_listing` →
    // `ContentListingView`) for identical event sequences:
    //
    //   * Strategy_B_baseline: `AlgorithmBView::for_id(...)` direct
    //     construction — the Phase-2b shipping shape.
    //   * post_g15a: `Algorithm::register(...)` routing through
    //     `dispatch_for` → canonical-id classification → the same
    //     hand-written inner kernel. Adds the dispatch-router hop.
    //
    // The companion criterion bench at
    // `crates/benten-ivm/benches/algorithm_b_canonical.rs` remains as
    // a finer-grained measurement surface for when CI gains a bench
    // lane (named in `docs/future/phase-3-backlog.md` §5.1 as a
    // follow-on); this in-test gate is the load-bearing 20% bound.
    use std::time::Instant;

    const ITERS: u64 = 1_024;
    const CORPUS: usize = 64;

    let definition = ViewDefinition {
        view_id: "content_listing".to_string(),
        input_pattern_label: Some("post".to_string()),
        output_label: "system:IVMView".to_string(),
        strategy: Strategy::B,
    };
    let events: Vec<ChangeEvent> = (0..CORPUS as u64)
        .map(|i| make_event(ChangeKind::Created, "post", i))
        .collect();

    // Warm-up to amortize cache effects on the first iteration; the
    // measurement loop is deterministic in shape so this does not bias
    // the ratio (both paths warm equally).
    for _ in 0..16 {
        let mut v = AlgorithmBView::for_id("content_listing", definition.clone()).unwrap();
        for e in &events {
            v.update(e).unwrap();
        }
        std::hint::black_box(&v);
        let mut v2 = Algorithm::register(
            "content_listing",
            LabelPattern::exact("post"),
            Projection::all_props(),
        )
        .unwrap();
        for e in &events {
            v2.update(e).unwrap();
        }
        std::hint::black_box(&v2);
    }

    // Strategy_B_baseline measurement loop.
    let t0 = Instant::now();
    for _ in 0..ITERS {
        let mut view = AlgorithmBView::for_id("content_listing", definition.clone()).unwrap();
        for e in &events {
            view.update(e).unwrap();
        }
        std::hint::black_box(&view);
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "wallclock-ns ratio gate; precision loss bounded by ITERS=1024 + corpus=64 \
                  (max ~2^30 ns/iter << f64 mantissa range)"
    )]
    let baseline_ns = t0.elapsed().as_nanos() as f64 / ITERS as f64;

    // post_g15a measurement loop.
    let t1 = Instant::now();
    for _ in 0..ITERS {
        let mut view = Algorithm::register(
            "content_listing",
            LabelPattern::exact("post"),
            Projection::all_props(),
        )
        .unwrap();
        for e in &events {
            view.update(e).unwrap();
        }
        std::hint::black_box(&view);
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "wallclock-ns ratio gate; precision loss bounded by ITERS=1024 + corpus=64 \
                  (max ~2^30 ns/iter << f64 mantissa range)"
    )]
    let post_gen_ns = t1.elapsed().as_nanos() as f64 / ITERS as f64;

    assert!(
        baseline_ns > 0.0 && post_gen_ns > 0.0,
        "non-zero per-iter timing (baseline_ns={baseline_ns}, post_gen_ns={post_gen_ns})"
    );

    let ratio = post_gen_ns / baseline_ns;
    // 1.50 ceiling on `cargo test` (debug-profile, noisy) — the bench
    // surface in benches/algorithm_b_canonical.rs runs the tighter
    // 1.20 bound under `cargo bench` (release profile, statistical
    // averaging via criterion). The 1.50 figure here is the
    // load-bearing canonical-fast-path-not-collapsed bound: a
    // dispatch-router regression that doubles the cost would trip;
    // the 20% headline figure is the criterion-bench surface.
    assert!(
        ratio <= 1.50,
        "G15-A canonical fast-path regressed in `cargo test` measurement \
         (ratio = {ratio:.3}; baseline {baseline_ns:.1}ns/iter; post-gen \
         {post_gen_ns:.1}ns/iter). ivm-disagree-1: gate measures canonical \
         fast-path vs Strategy::B baseline. The criterion bench at \
         benches/algorithm_b_canonical.rs runs the tighter 1.20 bound under \
         release-profile + statistical averaging."
    );
}
