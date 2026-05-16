//! Phase-4-Foundation R5 G24-B-FP-1 — T1 regression-guard: benign
//! schema renders correctly (defense isn't over-strict).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-5 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T1
//! ("Subgraph injection" — regression-guard arm).
//! Closure-destination: `docs/future/phase-4-backlog.md §4.14`
//! (G24-A mini-review g24a-mr-3 MAJOR, paired with g24a-mr-1
//! per pim-2 §3.6b sub-rule 4 per-finding granularity).
//!
//! ## What this pin establishes
//!
//! Pair with `admin_ui_v0_hostile_schema_read_emit_chain_denied.rs`
//! per pim-2 §3.6b sub-rule 4 per-finding granularity. The T1 defense
//! must REJECT hostile schemas; this pin asserts the same flow with a
//! BENIGN schema (declared envelope MATCHES actual composition)
//! SUCCEEDS — defense isn't over-strict.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer makes the materializer-entry cap-recheck over-strict
//! (e.g., refuses ALL schemas without a fail-OK opt-in). Admin UI v0
//! cannot render legitimate workflow forms; user can't use the
//! feature. This regression-guard pins the OK arm.

#![allow(clippy::unwrap_used)]

mod common;

use std::sync::atomic::Ordering;

use benten_platform_foundation::{allow_all_cap_recheck, compile_schema};

use common::admin_ui_v0_harness::{AdminUiV0TestHarness, make_note_node};

/// Minimal benign schema (canonical Note shape with body + created_at
/// + optional author FieldRef). Compiles via `schema_compiler::compile`
/// — the schema-derived cap-scope on the emitted READ primitive is
/// `"read:Note.body"` per `derive_scope(action, schema_name, field_path)`.
fn canonical_note_schema_bytes() -> &'static [u8] {
    br#"{
  "label": "SchemaRoot",
  "name": "Note",
  "fields": [
    { "label": "FieldScalar", "name": "body", "scalar": "text", "required": true,  "scope": ["read:note", "write:note"] },
    { "label": "FieldScalar", "name": "created_at", "scalar": "timestamp-hlc", "required": true, "scope": ["read:note"] }
  ]
}"#
}

#[test]
fn admin_ui_v0_benign_schema_renders_correctly() {
    // Substantive arm per pim-2 §3.6b — PRODUCTION-ARM (real Engine +
    // schema_compile + materializer pipeline through the same
    // HarnessEngineAdapter the hostile-pin uses), OBSERVABLE-
    // CONSEQUENCE (rendered HTML reflects the engine-sourced Node
    // bytes + structural cap-recheck trace fires), WOULD-FAIL-IF-NO-OP
    // (an over-strict envelope-recheck that refuses all schemas would
    // surface SchemaMismatch here too, breaking legitimate admin-UI
    // rendering).
    let harness = AdminUiV0TestHarness::new();

    // ------------------------------------------------------------------
    // (1) Real schema_compile — produces a real SchemaSubgraphSpec
    //     whose primitives carry schema-derived cap-scopes (e.g.
    //     `"read:Note.body"`). The benign envelope DECLARES those
    //     exact scopes so the envelope-recheck admits the walk.
    // ------------------------------------------------------------------
    let spec =
        compile_schema(canonical_note_schema_bytes()).expect("benign schema compiles cleanly");

    // Enumerate every cap_scope the spec's primitives carry — exactly
    // the set the schema author would declare as `requires` at the
    // manifest layer for a faithful admin-UI render.
    let declared_requires: Vec<String> = spec
        .as_subgraph()
        .nodes()
        .iter()
        .filter_map(|op| match op.property("cap_scope") {
            Some(benten_core::Value::Text(s)) => Some(s.clone()),
            _ => None,
        })
        .collect();
    assert!(
        !declared_requires.is_empty(),
        "benign schema MUST emit at least one cap_scope-carrying \
         primitive; the envelope-recheck has nothing to admit otherwise"
    );

    // ------------------------------------------------------------------
    // (2) Persist a benign Node + grant the admin-UI principal the
    //     read scope. The Node's label is `"Note"`; the
    //     GrantBackedPolicy::check_read derives `store:Note:read`,
    //     which the admin-UI plugin holds.
    // ------------------------------------------------------------------
    let content_cid = harness
        .create_test_node(&make_note_node("legit user-authored note"))
        .unwrap();
    harness
        .grant_admin_ui_read_scope("store:Note:read")
        .expect("admin-UI plugin granted Note read scope");

    // ------------------------------------------------------------------
    // (3) Render via the recording-cap-recheck gate so we can assert
    //     the per-row cap-recheck fired structurally (Compromise #11
    //     closure floor — the recheck MUST be always-on, not predicate-
    //     gated). Pass declared_requires populated — envelope matches
    //     runtime composition; walk completes successfully.
    // ------------------------------------------------------------------
    let (recording_gate, counter) = AdminUiV0TestHarness::recording_cap_recheck();
    let out = harness
        .render_admin_ui_with_envelope(
            &spec,
            content_cid,
            declared_requires.clone(),
            recording_gate,
        )
        .expect(
            "T1 regression-guard: benign schema with envelope == \
             composition MUST render successfully — defense must NOT \
             over-fire and block legitimate admin UI activity",
        );

    // ------------------------------------------------------------------
    // (4) Observable consequence — rendered HTML reflects the Node's
    //     bytes; no denial frame; cap-recheck counter > 0.
    // ------------------------------------------------------------------
    let html = std::str::from_utf8(out.html_bytes()).expect("html bytes are utf-8");
    assert!(
        !html.is_empty(),
        "T1 regression-guard: rendered output MUST be non-empty"
    );
    assert!(
        html.contains("legit user-authored note"),
        "T1 regression-guard: rendered HTML MUST reflect engine-sourced \
         Node bytes; got: {html}"
    );
    assert!(
        out.cap_denials().is_empty(),
        "T1 regression-guard: benign schema walk MUST NOT emit denial \
         frames — defense over-firing breaks legitimate admin-UI \
         activity; got {} denial frames",
        out.cap_denials().len(),
    );

    // Defense-in-depth: the materialization-layer per-row gate fired
    // for the content-CID decision and its bool was CONSUMED (Safe-1
    // #527 / Qual-1 #702). The previous `>= spec.nodes().len()`
    // assertion pinned the discarded-bool per-primitive fan-out that
    // provided no production enforcement and no production
    // observability — that loop is removed (per-primitive cap-scope is
    // enforced UPSTREAM by the T1 envelope check + schema-compile
    // `derive_scope`); the authoritative materialization-layer
    // cap-decision is now the single per-row gate call. WOULD-FAIL-IF-
    // NO-OP: a regression re-introducing the discarded-bool fan-out
    // would push this back to `>= primitive_count`; a predicate-gated
    // check would record zero.
    let invocations = counter.load(Ordering::SeqCst);
    assert_eq!(
        invocations,
        1,
        "T1 regression-guard: the authoritative materialization-layer \
         cap-recheck MUST fire exactly once for the content-CID \
         decision (single consumed-bool gate per Safe-1 #527 / Qual-1 \
         #702; the discarded-bool per-primitive fan-out is removed). \
         spec has {} primitives, recorded {} invocations",
        spec.as_subgraph().nodes().len(),
        invocations,
    );
}

#[test]
fn admin_ui_v0_benign_schema_with_empty_envelope_renders_correctly() {
    // Boundary: passing `declared_requires = Vec::new()` bypasses the
    // envelope-recheck entirely (the existing behaviour pre-G24-B-FP-1
    // for tests that don't exercise the T1 surface). The benign
    // schema MUST still render — the cap-recheck closure + the
    // engine-side read_node_as drive admit/deny.
    let harness = AdminUiV0TestHarness::new();
    let spec = compile_schema(canonical_note_schema_bytes()).unwrap();
    let content_cid = harness
        .create_test_node(&make_note_node("legit body"))
        .unwrap();
    harness
        .grant_admin_ui_read_scope("store:Note:read")
        .unwrap();

    let out = harness
        .render_admin_ui_with_envelope(&spec, content_cid, Vec::new(), allow_all_cap_recheck())
        .expect("empty envelope shortcircuits T1 check; benign render OK");
    let html = std::str::from_utf8(out.html_bytes()).unwrap();
    assert!(html.contains("legit body"));
}
