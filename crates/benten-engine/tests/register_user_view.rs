//! GREEN-PHASE pins for `register_user_view` post-G15-A generalization
//! (G15-A wave-5a).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.3 G15-A rows
//!   `register_user_view_canonical_id_with_mismatched_label_returns_e_view_label_mismatch_post_g15_a_generalization`
//!   + `register_user_view_with_label_pattern_succeeds_under_strategy_b`.
//! - plan §3 G15-A row.
//! - `ivm-major-5` (engine refuses Strategy::A user-view registration;
//!   user views always run under Strategy::B post-G15-A).
//! - `D-PHASE-3-28` RESOLVED (non-canonical view IDs maintained via
//!   generic kernel under Strategy::B).

#![allow(clippy::unwrap_used)]

use benten_engine::{Engine, EngineError, ErrorCode, UserViewInputPattern, UserViewSpec};

#[test]
fn register_user_view_canonical_id_with_mismatched_label_returns_e_view_label_mismatch_post_g15_a_generalization()
 {
    // ivm-major-5 pin. Even after G15-A generalizes the kernel, the
    // engine's `register_user_view` still REJECTS a (canonical view
    // ID, mismatched label) registration with `E_VIEW_LABEL_MISMATCH`.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let spec = UserViewSpec::builder()
        .id("capability_grants")
        .input_pattern(UserViewInputPattern::Label("user".to_string())) // mismatch
        .build()
        .unwrap();
    let result = engine.register_user_view(spec);
    match result {
        Err(EngineError::ViewLabelMismatch {
            view_id,
            expected_label,
            got_label,
        }) => {
            assert_eq!(view_id, "capability_grants");
            assert_eq!(expected_label, "system:CapabilityGrant");
            assert_eq!(got_label, "user");
            // Catalog code is stable across the generalization.
            let err = EngineError::ViewLabelMismatch {
                view_id: "capability_grants".into(),
                expected_label: "system:CapabilityGrant".into(),
                got_label: "user".into(),
            };
            assert_eq!(err.code(), ErrorCode::ViewLabelMismatch);
        }
        other => panic!("expected ViewLabelMismatch, got {other:?}"),
    }
}

#[test]
fn register_user_view_with_label_pattern_succeeds_under_strategy_b() {
    // ivm-major-5 + D-PHASE-3-28 pin. User-defined view IDs MAY be
    // registered with arbitrary label patterns under Strategy::B
    // (the generalized Algorithm B kernel). The engine no longer
    // forces these registrations through a ContentListingView shim.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let spec = UserViewSpec::builder()
        .id("custom:posts_by_author")
        .input_pattern(UserViewInputPattern::Label("post".to_string()))
        .build()
        .unwrap();
    let _cid = engine
        .register_user_view(spec)
        .expect("user view + matching label pattern succeeds");
    // Internal: the registered view runs under Strategy::B at the
    // engine boundary regardless of which inner kernel the dispatch
    // router selected.
    let strategy = engine
        .view_strategy("custom:posts_by_author")
        .expect("view registered + queryable strategy");
    assert_eq!(strategy, benten_ivm::Strategy::B);
}

#[test]
fn register_user_view_with_anchor_prefix_pattern_no_silent_label_equality_coerce() {
    // Phase-3 G15-A specifically retires the Phase-2b
    // "AnchorPrefix is silently coerced to a Label-equality match
    // against the prefix string" stub. AnchorPrefix("crud:") must
    // genuinely prefix-match; it must NOT match label == "crud:"
    // exclusively. g15a-mr-major-1 strengthening: assert the
    // prefix-vs-equality distinction is observable at the ENGINE
    // boundary (not just at the kernel boundary in
    // benten-ivm::tests::algorithm_b_general). Pre-strengthening this
    // test only asserted Strategy::B, which would PASS even if the
    // engine path silently coerced AnchorPrefix to Label-equality
    // against the prefix string itself.
    use benten_core::{Node, Value};
    use benten_engine::cap_recheck::PrincipalId;
    use benten_engine::cap_recheck::allow_all;
    use benten_engine::ivm_view_read_gate::IvmViewReadGate;

    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let spec = UserViewSpec::builder()
        .id("custom:by_prefix")
        .input_pattern(UserViewInputPattern::AnchorPrefix("crud:".to_string()))
        .build()
        .unwrap();
    let _ = engine
        .register_user_view(spec)
        .expect("AnchorPrefix registers under Strategy::B");
    let strategy = engine.view_strategy("custom:by_prefix").unwrap();
    assert_eq!(strategy, benten_ivm::Strategy::B);

    // Write three Nodes through the engine: two with `crud:*` labels
    // (should match the AnchorPrefix("crud:") view) + one with the
    // bare label "crud:" itself (which under the retired Phase-2b
    // silent-coerce-to-Label-equality stub would have been the ONLY
    // matching row). Genuine prefix matching admits the two `crud:*`
    // Nodes; under the retired stub a bare `crud:` Node would match
    // and the two `crud:*` Nodes would not.
    let mk_node = |label: &str, name: &str| -> (Node, _) {
        let mut props = std::collections::BTreeMap::new();
        props.insert("name".into(), Value::text(name));
        let node = Node::new(vec![label.to_string()], props);
        let cid = node.cid().unwrap();
        (node, cid)
    };
    let (post_node, post_cid) = mk_node("crud:post", "p");
    let (user_node, user_cid) = mk_node("crud:user", "u");
    // The "ambiguous" Node — bare label exactly equal to the prefix
    // string. Under the retired silent-coerce stub THIS would have
    // been the only match.
    let (bare_node, bare_cid) = mk_node("crud:", "b");

    engine
        .transaction(|tx| {
            for n in [&post_node, &user_node, &bare_node] {
                tx.put_node(n)
                    .map_err(|e| benten_engine::EngineError::Other {
                        code: benten_errors::ErrorCode::Unknown("E_TEST_HARNESS".into()),
                        message: format!("put_node: {e:?}"),
                    })?;
            }
            Ok(())
        })
        .expect("commit three Nodes");

    // Read via the production engine entry point with an allow-all
    // gate so the assertion observes the kernel's pattern matching
    // unfiltered.
    let actor = {
        let mut props = std::collections::BTreeMap::new();
        props.insert("name".into(), Value::text("alice"));
        let node = Node::new(vec!["actor".to_string()], props);
        PrincipalId::from_actor_cid(node.cid().unwrap())
    };
    let allow_gate = IvmViewReadGate::new(actor, "crud:", allow_all());
    let cids = engine
        .materialize_view_with_gate("custom:by_prefix", &allow_gate)
        .expect("materialize succeeds")
        .expect("Some(cids) for registered view");

    // Genuine prefix matching admits BOTH `crud:post` + `crud:user`.
    assert!(
        cids.contains(&post_cid),
        "AnchorPrefix(\"crud:\") admits crud:post; cids = {cids:?}"
    );
    assert!(
        cids.contains(&user_cid),
        "AnchorPrefix(\"crud:\") admits crud:user; cids = {cids:?}"
    );
    // The bare `crud:` Node — which under the retired silent-coerce
    // stub would have been the ONLY match — also begins with the
    // prefix and SHOULD match too (a string prefix-matches itself).
    // The load-bearing assertion: prefix matching is NOT
    // label-equality, evidenced by the two `crud:*` Nodes appearing.
    assert!(
        cids.contains(&bare_cid),
        "AnchorPrefix(\"crud:\") also admits the bare label \"crud:\" (every \
         string is its own prefix); cids = {cids:?}"
    );
    assert_eq!(
        cids.len(),
        3,
        "all three Nodes match AnchorPrefix(\"crud:\"); under the retired \
         silent-coerce-to-Label-equality stub the 'crud:post' + 'crud:user' \
         Nodes would have been MISSING (only bare 'crud:' would match). \
         cids = {cids:?}"
    );
}
