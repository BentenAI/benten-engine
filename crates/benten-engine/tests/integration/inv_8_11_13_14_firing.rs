//! Phase 2a R3 integration — HEADLINE exit gate 2: the four new invariants
//! fire with paired positive+negative tests.
//!
//! Traces to: `.addl/phase-2a/00-implementation-plan.md` §1 exit criterion 2
//! (four new invariants firing) + §3 G4-A (Inv-8 multiplicative) + §3 G5-A
//! (Inv-13 immutability) + §3 G5-B (Inv-11 full + Inv-14 structural causal
//! attribution) + §9.11 (Inv-13 5-row matrix) + §9.10 (Inv-11 PascalCase
//! prefix table).
//!
//! Each sub-test pairs positive (the allowed shape succeeds) with negative
//! (the forbidden shape fires the right typed error code). Owned by
//! `qa-expert` per R2 landscape §8.5 rows 288-291. TDD red-phase.

#![cfg(feature = "phase_2a_pending_apis")]
// R4 fix-pass: see wait_resume_determinism.rs for the phase_2a_pending_apis
// gate rationale. Blocked on DSL closure-style SubgraphBuilder + `trace_as`
// + `testing_reput_subgraph_*` APIs landing in R5 G5-A / G5-B.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::{Engine, SubgraphSpec};
use benten_errors::ErrorCode;
use std::collections::BTreeMap;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

// ---------------------------------------------------------------------------
// Inv-8 multiplicative — through nested CALL+ITERATE
// ---------------------------------------------------------------------------

/// Positive + negative: cumulative CALL+ITERATE budget ≤ declared grant
/// succeeds; the same shape at bound+1 rejects at registration time with
/// `E_INV_ITERATE_BUDGET`.
#[test]
fn invariant_8_multiplicative_end_to_end() {
    let (_dir, engine) = fresh_engine();

    let inner = SubgraphSpec::builder()
        .handler_id("inv8:inner")
        .iterate(|i| i.over("$input.items").max(4))
        .respond(|r| r.body("$result"))
        .build();
    engine
        .register_subgraph(inner)
        .expect("inner handler registers — 4 iterations is within any reasonable budget");

    let outer_ok = SubgraphSpec::builder()
        .handler_id("inv8:outer_ok")
        .iterate(|i| i.over("$input.items").max(3))
        .call(|c| c.handler("inv8:inner").action("inv8:run"))
        .respond(|r| r.body("$result"))
        .build();
    let ok_id = engine
        .register_subgraph(outer_ok)
        .expect("3x4=12 multiplicative budget fits within Inv-8 default");
    assert!(!ok_id.is_empty());

    let outer_bad = SubgraphSpec::builder()
        .handler_id("inv8:outer_bad")
        .iterate(|i| i.over("$input.items").max(200))
        .call(|c| c.handler("inv8:inner").action("inv8:run"))
        .respond(|r| r.body("$result"))
        .build();
    let err = engine
        .register_subgraph(outer_bad)
        .expect_err("200x4 cumulative budget must exceed default cap");
    assert_eq!(
        err.code(),
        ErrorCode::InvIterateBudget,
        "Inv-8 multiplicative rejection must fire E_INV_ITERATE_BUDGET; got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Inv-11 full — registration-time literal AND runtime TRANSFORM-computed
// ---------------------------------------------------------------------------

/// Inv-11 full: both the registration-time literal-CID reject AND the
/// runtime TRANSFORM-computed-CID reject fire in one test flow. Positive
/// case: a subgraph that only touches user-zone labels registers and runs.
#[test]
fn invariant_11_full_end_to_end() {
    let (_dir, engine) = fresh_engine();

    let benign = SubgraphSpec::builder()
        .handler_id("inv11:benign")
        .read(|r| r.label("post").by("id").value("p1"))
        .respond(|r| r.body("$result"))
        .build();
    let id_ok = engine
        .register_subgraph(benign)
        .expect("benign user-zone handler registers");
    let outcome = engine
        .call(&id_ok, "inv11:run", Node::empty())
        .expect("benign call works");
    assert!(outcome.is_ok_edge());

    // Registration-time literal-CID reject.
    let literal_bad = SubgraphSpec::builder()
        .handler_id("inv11:read_system_literal")
        .read(|r| r.label("system:CapabilityGrant").by("id").value("anything"))
        .respond(|r| r.body("$result"))
        .build();
    let err = engine
        .register_subgraph(literal_bad)
        .expect_err("literal READ from system:CapabilityGrant must reject at registration");
    assert_eq!(
        err.code(),
        ErrorCode::InvSystemZone,
        "Inv-11 registration-time literal must fire E_INV_SYSTEM_ZONE; got {err:?}"
    );

    // Runtime TRANSFORM-computed-CID reject (Code-as-graph Major #1 probe).
    let runtime_bad = SubgraphSpec::builder()
        .handler_id("inv11:transform_computed_system_cid")
        .transform(|t| t.expr("{ cid: $input.system_grant_cid }"))
        .read(|r| r.by("cid").value("$result.cid"))
        .respond(|r| r.body("$result"))
        .build();
    let runtime_id = engine
        .register_subgraph(runtime_bad)
        .expect("shape is registration-legal — only the computed CID triggers Inv-11");

    let fixture_cid = engine.testing_insert_privileged_fixture();
    let mut input = BTreeMap::new();
    input.insert(
        "system_grant_cid".into(),
        Value::Text(fixture_cid.to_base32()),
    );
    let runtime_outcome = engine
        .call(
            &runtime_id,
            "inv11:run",
            Node::new(vec!["input".into()], input),
        )
        .expect("call returns Outcome (error routes through edge, not Result)");
    assert_eq!(
        runtime_outcome.error_code(),
        Some("E_INV_SYSTEM_ZONE"),
        "runtime TRANSFORM-computed system-label access must fire E_INV_SYSTEM_ZONE"
    );
}

// ---------------------------------------------------------------------------
// Inv-13 immutability — 5-row matrix rows 1+2+3 (active in 2a)
// ---------------------------------------------------------------------------

/// Inv-13: registered-subgraph CID re-put by user fires E_INV_IMMUTABILITY.
/// Privileged re-put with same bytes returns Ok (dedup). Covers plan §9.11
/// rows 1 + 2 + 3.
#[test]
fn invariant_13_immutability_end_to_end() {
    let (_dir, engine) = fresh_engine();
    let handler_id = engine.register_crud("post").unwrap();

    let sg_bytes = benten_engine::testing::subgraph_bytes_for_handler(&engine, &handler_id)
        .expect("extract the stored subgraph bytes");

    // Row 1: User / content matches = E_INV_IMMUTABILITY.
    let err = engine
        .testing_reput_subgraph_as_user(&sg_bytes)
        .expect_err("user re-put of registered subgraph bytes must fire E_INV_IMMUTABILITY");
    assert_eq!(err.code(), ErrorCode::InvImmutability);

    // Row 3: EnginePrivileged / content matches = Ok(cid_dedup); NO
    // ChangeEvent emitted; NO audit-sequence advance.
    let change_count_before = engine.change_event_count();
    let audit_seq_before = engine.testing_audit_sequence();

    let dedup_cid = engine
        .testing_reput_subgraph_privileged(&sg_bytes)
        .expect("privileged re-put of matching bytes must dedup (Ok)");
    assert!(!dedup_cid.is_empty());

    let change_count_after = engine.change_event_count();
    let audit_seq_after = engine.testing_audit_sequence();
    assert_eq!(
        change_count_before, change_count_after,
        "dedup write MUST NOT emit a ChangeEvent (§9.11 row 3)"
    );
    assert_eq!(
        audit_seq_before, audit_seq_after,
        "dedup write MUST NOT advance the audit sequence (§9.11 row 3 companion)"
    );

    // Row 2: User / content differs → still E_INV_IMMUTABILITY.
    let mut mutated_bytes = sg_bytes.clone();
    if let Some(b) = mutated_bytes.first_mut() {
        *b ^= 0xFF;
    }
    let err2 = engine
        .testing_reput_subgraph_as_user(&mutated_bytes)
        .expect_err(
            "user write that would mutate a registered subgraph must fire E_INV_IMMUTABILITY",
        );
    assert_eq!(err2.code(), ErrorCode::InvImmutability);
}

/// Row 4 (SyncReplica) — reserved shape in Phase 2a; fires when Phase 3
/// sync ships. `#[ignore]` per r2-triage decision #2.
#[test]
#[ignore = "phase-3-sync-preview — SyncReplica write-path not shipped in 2a; \
            shape pin only; firing requires SyncReplica write path not \
            shipped in 2a"]
fn invariant_13_sync_replica_dedups_reserved() {
    let (_dir, engine) = fresh_engine();
    let handler_id = engine.register_crud("post").unwrap();
    let sg_bytes =
        benten_engine::testing::subgraph_bytes_for_handler(&engine, &handler_id).expect("extract");
    let origin = benten_core::Cid::from_blake3_digest(blake3::hash(b"phase-3-peer").into());
    let _dedup_cid = engine
        .testing_reput_subgraph_as_sync_replica(&sg_bytes, &origin)
        .expect("sync-replica dedup returns Ok in Phase 3");
}

// ---------------------------------------------------------------------------
// Inv-14 structural causal attribution
// ---------------------------------------------------------------------------

/// Every TraceStep across a 5-primitive composed subgraph carries a
/// populated AttributionFrame. Negative: a handler whose primitive-type
/// refuses to declare its attribution source fails at registration.
#[test]
fn invariant_14_structural_attribution_end_to_end() {
    let (_dir, engine) = fresh_engine();

    let sg = SubgraphSpec::builder()
        .handler_id("inv14:five_step")
        .read(|r| r.label("post").by("id").value("p1"))
        .transform(|t| t.expr("{ echoed: $result }"))
        .branch(|b| b.on("$result.kind"))
        .write(|w| w.label("post"))
        .respond(|r| r.body("$result"))
        .build();
    let handler_id = engine.register_subgraph(sg).unwrap();
    let actor = engine.create_principal("inv14_actor").unwrap();

    let trace = engine
        .trace_as(&handler_id, "inv14:run", Node::empty(), &actor)
        .expect("trace_as returns a Trace carrying attribution per step");
    assert!(!trace.steps().is_empty());
    for step in trace.steps() {
        let attr = step
            .attribution()
            .expect("every TraceStep MUST carry Some(AttributionFrame) per Inv-14");
        assert_eq!(attr.actor_cid(), &actor);
        assert!(!attr.handler_cid().to_base32().is_empty());
        assert!(!attr.capability_grant_cid().to_base32().is_empty());
    }

    // Negative: missing attribution declaration rejects at registration.
    let bad = SubgraphSpec::builder()
        .handler_id("inv14:no_attribution_declared")
        .testing_raw_primitive_without_attribution_declaration()
        .respond(|r| r.body("$result"))
        .build();
    let err = engine
        .register_subgraph(bad)
        .expect_err("missing attribution declaration must reject at registration");
    assert_eq!(err.code(), ErrorCode::InvAttribution);
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn trace_step_budget_exhausted_variant_shape_pin() {
    use benten_engine::outcome::TraceStep;
    let _pin: fn(&TraceStep) -> Option<(&'static str, u64, u64)> = |step| {
        step.as_budget_exhausted()
            .map(|b| (b.budget_type(), b.consumed(), b.limit()))
    };
}
