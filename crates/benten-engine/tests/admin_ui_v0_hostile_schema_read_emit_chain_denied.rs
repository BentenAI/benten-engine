//! Phase-4-Foundation R4-FP-1 — T1 LOAD-BEARING pin: hostile schema
//! READ+EMIT chain denied (subgraph injection via schema-compiler-
//! driven form generation).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-5 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T1
//! ("Subgraph injection via schema-compiler-driven form generation").
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
//! declared scope.
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

#[test]
#[ignore = "DESTINATION-REMAPPED per HARD RULE 12 clause-(b) BELONGS-NAMED-NOW: T1 hostile-schema end-to-end pin requires AdminUiV0TestHarness::new() graduation (a substantive test-harness that wires schema_compile → register_subgraph → Engine.call_as → materialize through a real Engine end-to-end). G24-A landed the substrate module (admin_ui_v0/mod.rs) + engine adapter bridge but the full hostile-schema chain test requires harness graduation. Substantive arm lands at phase-4-foundation-backlog §4.14 (G24-B-FP-1 alongside workflow-editor harness graduation). Companion benign-control pin (admin_ui_v0_benign_schema_renders_correctly.rs) un-ignores together with this one per pim-2 §3.6b sub-rule 4."]
fn admin_ui_v0_hostile_schema_with_read_emit_chain_outside_envelope_denied() {
    // G24-A wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //   let admin_ui_did = harness.admin_ui_plugin_did();
    //
    //   // Hostile schema: declares narrow `requires` envelope
    //   // (store:innocent:read) but inner subgraph contains READ+EMIT
    //   // chain that reads from store:secrets:read + EMITs the
    //   // payload to an attacker-controlled sink.
    //   let hostile_schema = common::schema_fixtures::hostile_read_emit_schema(
    //       /* declared requires */ vec!["store:innocent:read"],
    //       /* hidden read scope inside subgraph */ "store:secrets:read",
    //       /* hidden emit sink */ "attacker_sink",
    //   );
    //
    //   // Admin UI v0 attempts to render the hostile schema via
    //   // materializer. LOAD-BEARING defense: materializer's per-row
    //   // cap-recheck (Compromise #11 closure floor) MUST refuse the
    //   // walk because the runtime composition exceeds declared scope.
    //   let render_attempt = harness.dispatch_admin_ui_render_schema(
    //       admin_ui_did.clone(),
    //       hostile_schema,
    //   );
    //
    //   let err = render_attempt.expect_err(
    //       "T1 LOAD-BEARING: hostile schema with READ+EMIT chain \
    //        outside declared envelope MUST be REFUSED at materializer \
    //        entry — Compromise #11 closure floor + per-row cap-recheck"
    //   );
    //   assert!(
    //       matches!(err.code(),
    //           ErrorCode::E_MATERIALIZER_CAP_DENIED
    //           | ErrorCode::E_MATERIALIZER_SCHEMA_MISMATCH
    //       ),
    //       "T1: must surface typed denial; got {:?}",
    //       err.code()
    //   );
    //
    //   // Observable consequence: NO emit to attacker_sink occurred —
    //   // defense isn't just "deny return value", it's "halt the
    //   // composition before the EMIT fires":
    //   let emit_log = harness.captured_emits_to_sink("attacker_sink");
    //   assert!(
    //       emit_log.is_empty(),
    //       "T1: defense MUST halt composition BEFORE EMIT fires; \
    //        captured {} unauthorized emits to attacker_sink",
    //       emit_log.len()
    //   );
    //
    //   // Defense-in-depth audit-log: the rejection records the
    //   // hidden scope (`store:secrets:read`) attempted, not just
    //   // the declared one, for forensic value:
    //   let audit = harness.audit_log_since_last_dispatch();
    //   assert!(
    //       audit.iter().any(|r| r.attempted_scope == "store:secrets:read"
    //           && r.outcome.is_denied()),
    //       "T1: audit log MUST record actual attempted scope for \
    //        forensic visibility"
    //   );
    //
    // OBSERVABLE consequence: hostile schema halted before sensitive
    // data emits; audit log retains forensic record.
    unimplemented!(
        "G24-A wires hostile-schema READ+EMIT defense end-to-end \
         (T1 LOAD-BEARING). Substantive: real schema + materializer \
         walk + per-row cap-recheck + emit-not-fired observable + \
         forensic audit-log assertion."
    );
}
