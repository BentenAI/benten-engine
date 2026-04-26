//! G12-B red-phase: re-validate Phase-2a cap-grant preservation property
//! against the **real Engine** (not the in-memory HandlerTable stub).
//!
//! Per plan §3.2 G12-B must-pass tests: "devserver_hot_reload_preserves_cap_grants_through_engine_path
//! (re-validated against real engine)."
//!
//! The Phase-2a `tools/benten-dev/tests/devserver_preserves_cap_grants.rs`
//! test pinned the property against the in-memory stub. Once G12-B routes
//! through Engine::register_subgraph, the property must hold against the
//! real engine + real benten-caps grant store.
//!
//! TDD red-phase. Owner: R5 G12-B (qa-r4-01 R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "R5 G12-B red-phase: cap-grant preservation through real engine not yet wired"]
fn devserver_hot_reload_preserves_cap_grants_routed_via_engine_register_subgraph() {
    // Drive:
    //   1. Spawn devserver against handler-v1 DSL file; principal P granted
    //      cap C against handler-v1.
    //   2. Modify the DSL file (handler-v2; same handler_id, different body);
    //      devserver hot-reloads via Engine::register_subgraph.
    //   3. Assert P still holds cap C against handler-v2 (cap_grants survive
    //      the engine-side re-registration).
    //
    // Counter-property: if Engine::register_subgraph were dropped + re-added
    // naively (instead of update-in-place with grant preservation), grants
    // would reset and this test would catch the regression.
    todo!(
        "R5 G12-B: build temp DSL file (v1); start devserver; grant cap to principal; \
           rewrite file (v2); wait for hot-reload; assert principal still holds cap on v2"
    )
}

#[test]
#[ignore = "R5 G12-B red-phase: cap-revoke survives reload not yet wired"]
fn devserver_hot_reload_does_not_resurrect_revoked_caps() {
    // Counter-property: caps explicitly revoked between v1 and v2 stay revoked.
    todo!("R5 G12-B: revoke cap mid-session; trigger reload; assert cap stays revoked post-reload")
}
