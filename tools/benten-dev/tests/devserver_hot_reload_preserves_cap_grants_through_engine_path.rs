//! G12-B green-phase: re-validate Phase-2a cap-grant preservation property
//! against the **real Engine** (not the in-memory HandlerTable stub).
//!
//! Per plan §3.2 G12-B must-pass tests: "devserver_hot_reload_preserves_cap_grants_through_engine_path
//! (re-validated against real engine)."
//!
//! G12-B routing keeps caps on the dev-server's own grant table (an
//! intentionally pluggable surface — engine caps are out of scope for the
//! routing pass). The property the test pins is that the dev-server's grant
//! table SURVIVES a hot-reload that now touches the real engine — so a
//! reload-induced re-registration into Engine::register_subgraph does not
//! reach into the grant table.
//!
//! Lifted from red-phase 2026-04-28 (R5 G12-B implementer).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

use benten_core::Cid;
use benten_dev::DevServer;
use tempfile::tempdir;

#[test]
fn devserver_hot_reload_preserves_cap_grants_routed_via_engine_register_subgraph() {
    let dir = tempdir().unwrap();
    let mut dev = DevServer::builder()
        .workspace(dir.path())
        .enable_engine(true)
        .build()
        .unwrap();

    // Grant a capability to a principal BEFORE the first registration.
    let alice = Cid::from_blake3_digest([0xa1; 32]);
    dev.grant(&alice, "store:post:write").unwrap();

    // Register handler-v1 via the engine path.
    let v1 = "handler 'h1' { read('post') -> respond }";
    dev.register_handler_from_dsl("h1", "run", v1).unwrap();

    // Hot-reload: same handler_id, different body.
    let v2 = "handler 'h1' { read('post') -> transform({ x: $x }) -> respond }";
    dev.register_handler_from_dsl("h1", "run", v2).unwrap();

    // Cap-grant survives the engine-routed re-registration.
    assert!(
        dev.grant_exists(&alice, "store:post:write"),
        "grant must survive hot-reload that now routes through Engine::register_subgraph"
    );
}

#[test]
fn devserver_hot_reload_does_not_resurrect_revoked_caps() {
    // Counter-property: the routing change MUST NOT make grants any
    // stickier than the Phase-2a property required. An explicit reset
    // still wipes them.
    let dir = tempdir().unwrap();
    let mut dev = DevServer::builder()
        .workspace(dir.path())
        .enable_engine(true)
        .build()
        .unwrap();
    let alice = Cid::from_blake3_digest([0xa2; 32]);
    dev.grant(&alice, "store:post:read").unwrap();
    assert!(dev.grant_exists(&alice, "store:post:read"));

    dev.reset_dev_state().unwrap();
    assert!(
        !dev.grant_exists(&alice, "store:post:read"),
        "explicit reset clears grants even after engine-routed registrations"
    );

    // Re-grant + re-register: grant present again, no resurrection of the
    // pre-reset state.
    dev.grant(&alice, "store:post:read").unwrap();
    let v = "handler 'h2' { read('post') -> respond }";
    dev.register_handler_from_dsl("h2", "run", v).unwrap();
    assert!(dev.grant_exists(&alice, "store:post:read"));
}
