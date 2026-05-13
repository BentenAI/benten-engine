//! Phase-4-Foundation R4-FP-1 — T1 negative pin: materializer rejects
//! subgraph with cap-scope mismatch (declared envelope ≠ runtime
//! composition).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-2 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T1
//! defense (cap-scope derivation refuses any schema whose READ+EMIT
//! composition exceeds declared `requires` envelope) + arch-r1-3
//! `E_MATERIALIZER_SCHEMA_MISMATCH` mint.
//!
//! ## What this pin establishes
//!
//! The negative arm of the T1 defense at materializer entry. A subgraph
//! whose declared `requires` envelope is narrower than what its inner
//! composition actually needs MUST be REJECTED with typed
//! `E_MATERIALIZER_SCHEMA_MISMATCH`.
//!
//! Pair with the positive arm (benign-schema renders) — together they
//! establish the boundary condition per pim-2 §3.6b sub-rule 4 per-
//! finding granularity.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires schema-compiler validation at compile-time only.
//! At runtime, materializer walks the subgraph trustingly — exceeding-
//! envelope composition succeeds. T1 attack class regression.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "RED-PHASE: G23-B wires materializer cap-scope mismatch defense; un-ignore at G23-B landing. Pin source: r4-triage §1 r4-tc-2 + threat-model §T1 negative."]
fn materializer_rejects_subgraph_whose_runtime_composition_exceeds_declared_cap_scope() {
    // G23-B wave wires this. Substantive shape:
    //
    //   use benten_platform_foundation::materializer::Materializer;
    //
    //   // Schema declares narrow `requires`:
    //   let schema_spec = common::schema_fixtures::schema_spec_with_requires(
    //       vec!["store:notes:read"],
    //   );
    //
    //   // BUT the inner subgraph (handler) contains a READ for an
    //   // out-of-envelope scope (e.g., store:secrets:read):
    //   let subgraph_with_hidden_read = common::materializer_fixtures::
    //       subgraph_with_inner_read_scope(
    //           &schema_spec,
    //           /* hidden_read */ "store:secrets:read",
    //       );
    //
    //   let materializer = Materializer::new_for_test();
    //   let result = materializer.materialize_with_gate(
    //       &subgraph_with_hidden_read,
    //       common::manifest_fixtures::stub_user_rooted_chain(),
    //   );
    //
    //   let err = result.expect_err(
    //       "T1 negative: subgraph whose runtime composition exceeds \
    //        declared cap-scope MUST be REJECTED at materializer entry"
    //   );
    //   assert!(
    //       matches!(err.code(), ErrorCode::E_MATERIALIZER_SCHEMA_MISMATCH),
    //       "T1 negative: must surface typed E_MATERIALIZER_SCHEMA_MISMATCH \
    //        per arch-r1-3 mint; got {:?}", err.code()
    //   );
    //
    //   // Defense-in-depth: confirm the rejection happened BEFORE any
    //   // READ for the out-of-envelope scope was issued (no side-effect
    //   // leak):
    //   let read_log = materializer.captured_read_calls();
    //   assert!(
    //       !read_log.iter().any(|r| r.scope == "store:secrets:read"),
    //       "T1 negative: defense MUST refuse BEFORE the hidden READ \
    //        fires; observed READ to store:secrets:read indicates \
    //        side-effect leak"
    //   );
    //
    // OBSERVABLE consequence: cap-scope mismatch surface fires at the
    // composition boundary; no side-effect leak.
    panic!(
        "RED-PHASE: G23-B must wire materializer cap-scope mismatch \
         defense (T1 negative). Substantive: real subgraph + scope-\
         derivation check + typed E_MATERIALIZER_SCHEMA_MISMATCH + \
         no-side-effect assertion."
    );
}
