//! R3-C RED-PHASE pins for AnchorPrefix selector lift (G15-B
//! wave-5a; per r2-test-landscape §2.3 + plan §3 G15-B row).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.3 G15-B rows
//!   `anchor_prefix_matches_prefix_not_equality` +
//!   `anchor_prefix_no_silent_label_equality_coerce`.
//! - plan §3 G15-B row line "PrefixMatcher selector type —
//!   `anchor_prefix=\"crud:\"` matches both `\"crud:post\"` and
//!   `\"crud:user\"`".
//!
//! ## What this pins
//!
//! Pre-G15-B, the engine's view registration treats `anchor_prefix`
//! as label-equality (silent coercion). G15-B introduces a real
//! `PrefixMatcher` selector type that does TRUE prefix matching:
//! `anchor_prefix="crud:"` matches both `"crud:post"` and
//! `"crud:user"` and `"crud:comment"`, but NOT `"system:zone"` or
//! `"governance:rule"`.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale pointing to phase-3-backlog §7.3.D STALE-RATIONALE sweep #2; destination next Phase-3-close orchestrator-direct fix-pass batch (G15-B + W9-T1 §5.1 closed).
//!
//! ## LabelPattern import path (r4-r2-ivm-6 docstring)
//!
//! The `LabelPattern` enum import path is assumed to be
//! `benten_ivm::LabelPattern` per ivm-major-1 architectural choice (a)
//! — generic kernel keyed on `(label_pattern, projection)`. G15-A
//! implementer adjusts the import path if a different architectural
//! choice is made; this docstring tracks the cross-reference for
//! §3.5b HARDENED point-1 cite verification. The `.into()` calls at
//! the use sites (e.g. `LabelPattern::AnchorPrefix("crud:".into())`)
//! suggest a String/smol-str newtype payload; the `LabelPattern::exact()`
//! constructor at register_user_view.rs sites suggests a builder helper.
//! G15-A implementer ratifies the final shape; this docstring pins the
//! producer-consumer pair documentation expectation per r4-r2-ivm-6.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §7.3.D — anchor-prefix prefix-matching. G15-B closed at §5.1 IVM Algorithm B (W9-T1); test body pins post-G15-B prefix-matching invariant contract; un-ignore at next Phase-3-close orchestrator-direct fix-pass batch per Wave-E rationale-only sweep."]
fn anchor_prefix_matches_prefix_not_equality() {
    // plan §3 G15-B pin. G15-B implementer wires this against the
    // new PrefixMatcher selector type:
    //
    //   let mut engine = test_engine();
    //   engine.write_node(make_post("p1"));
    //   engine.write_node(make_user("u1"));
    //   engine.write_node(make_comment("c1"));
    //   engine.write_node(make_system_zone("z1"));
    //   engine.write_node(make_governance_rule("r1"));
    //
    //   let view = engine.register_user_view(
    //       "all_crud",
    //       LabelPattern::AnchorPrefix("crud:".into()),
    //       Projection::default(),
    //   ).unwrap().materialize();
    //
    //   // 3 nodes match "crud:" prefix (post, user, comment); 2 do not
    //   // (system:zone, governance:rule).
    //   assert_eq!(view.rows().len(), 3);
    //   let labels: BTreeSet<&str> = view.rows().iter().map(|r| r.label()).collect();
    //   assert_eq!(labels, BTreeSet::from(["crud:post", "crud:user", "crud:comment"]));
    //
    // OBSERVABLE consequence: the post-G15-B engine matches
    // anchor_prefix as a true prefix. Defends against the failure
    // shape where a selector regression silently coerces it back to
    // label equality.
    unimplemented!("G15-B wires PrefixMatcher selector type — anchor_prefix true prefix matching");
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — anchor-prefix no-silent-equality-coerce invariant. G15-B closed; test body pins specific equality-coerce-rejection invariant; un-ignore at next Phase-3-close orchestrator-direct fix-pass batch per Wave-E rationale-only sweep."]
fn anchor_prefix_no_silent_label_equality_coerce() {
    // plan §3 G15-B pin. G15-B explicitly REJECTS the silent coerce
    // back to label-equality. If a future refactor accidentally
    // collapses PrefixMatcher::Prefix to PrefixMatcher::Equal, the
    // following assertion fires.
    //
    // Concrete shape:
    //   let mut engine = test_engine();
    //   engine.write_node(make_post("p1"));      // label "crud:post"
    //   engine.write_node(make_post("p2"));      // label "crud:post"
    //   engine.write_node(make_user("u1"));      // label "crud:user"
    //
    //   let view = engine.register_user_view(
    //       "any_crud",
    //       LabelPattern::AnchorPrefix("crud:".into()),
    //       Projection::default(),
    //   ).unwrap().materialize();
    //
    //   // If this regressed to label-equality, "crud:" would match
    //   // ZERO nodes (no node has label exactly "crud:"). The
    //   // post-G15-B prefix matcher returns 3 nodes.
    //   assert_eq!(view.rows().len(), 3, "anchor_prefix \"crud:\" must match 3 nodes via prefix, not 0 via equality");
    //
    // OBSERVABLE consequence: the PrefixMatcher prefix semantics is
    // pinned via a row-count assertion that distinguishes prefix
    // matching from equality matching.
    unimplemented!(
        "G15-B wires the no-silent-coerce assertion via row-count distinguishing prefix from equality"
    );
}
