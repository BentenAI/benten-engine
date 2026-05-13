//! Phase-4-Foundation R4-FP-1 — T1 regression-guard: benign schema
//! renders correctly (defense isn't over-strict).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-5 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T1
//! ("Subgraph injection" — regression-guard arm).
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

#[test]
#[ignore = "RED-PHASE: G23-A + G23-B + G24-A wire benign-schema render path; un-ignore at G24-A landing. Pin source: r4-triage §2 r4-tc-5 + threat-model §T1 regression-guard."]
fn admin_ui_v0_benign_schema_renders_correctly() {
    // G24-A wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //   let admin_ui_did = harness.admin_ui_plugin_did();
    //
    //   // Benign schema: declares envelope (store:notes:read) and
    //   // inner composition uses EXACTLY that scope. No hidden
    //   // READ+EMIT chains. The legitimate "Workflow Editor" schema
    //   // shape per D-4F-4.
    //   let benign_schema = common::schema_fixtures::workflow_editor_schema(
    //       vec!["store:notes:read"],
    //   );
    //   harness.user_grants_admin_ui_cap("store:notes:read");
    //
    //   // Render the schema via materializer. Cap-recheck fires on
    //   // every row; declared envelope matches runtime composition;
    //   // walk completes successfully.
    //   let render = harness.dispatch_admin_ui_render_schema(
    //       admin_ui_did.clone(),
    //       benign_schema,
    //   );
    //
    //   let rendered_html = render.expect(
    //       "T1 regression-guard: benign schema with envelope == \
    //        composition MUST render successfully — defense must NOT \
    //        over-fire and block legitimate admin UI activity"
    //   );
    //   assert!(!rendered_html.is_empty(),
    //       "Rendered output must be non-empty");
    //
    //   // Defense-in-depth: cap-recheck fired on every row in the
    //   // walk (Compromise #11 closure floor; structural-always-on):
    //   let recheck_trace = harness.captured_cap_recheck_calls();
    //   assert!(
    //       !recheck_trace.is_empty(),
    //       "T1 regression-guard: cap-recheck MUST fire structurally \
    //        even on benign schema (Compromise #11 always-on); zero \
    //        invocations means the check was predicate-gated and \
    //        defense isn't actually live"
    //   );
    //
    // OBSERVABLE consequence: benign rendering succeeds + structural
    // recheck verified live; pair with hostile-schema deny pin
    // establishes full T1 closure boundary.
    unimplemented!(
        "G24-A wires benign-schema renders-correctly regression-guard \
         (T1 OK arm). Substantive: real materializer walk + render \
         succeeds + structural cap-recheck trace + non-empty output."
    );
}
