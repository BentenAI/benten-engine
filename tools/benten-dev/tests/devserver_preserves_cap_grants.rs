//! Edge-case test: devserver hot-reload preserves capability grants on disk
//! (R2 landscape §2.9 "Devserver preserves cap grants across reload",
//! dx-r1 devserver fix).
//!
//! Concern: the dev server supports hot-reload of handler subgraphs. A naive
//! implementation might also wipe the `capability_grant` table (because
//! grants look like "user state"); this would break any workflow where a
//! developer granted themselves a capability once and then iterated on the
//! handler code. The contract: handler subgraphs are replaced on reload;
//! capability grants on disk survive untouched.
//!
//! Concerns pinned:
//! - Grant present before reload is still present after reload.
//! - Grant still satisfies `check_attenuation` after reload.
//! - A handler re-registration via hot-reload does NOT advance the audit
//!   sequence of the grants table (grants row is not touched).
//! - Negative: an intentional `reset_dev_state()` call DOES clear grants
//!   (so the preservation path isn't a stuck "grants are immortal" bug).
//!
//! R3 red-phase contract: R5 (G11-A) lands `benten-dev` with hot-reload.
//! Tests compile; they fail because the crate does not yet expose a
//! `DevServer` type.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
use benten_dev::DevServer;
use tempfile::tempdir;

#[test]
fn devserver_preserves_cap_grants_across_reload() {
    let dir = tempdir().unwrap();
    let mut dev = DevServer::builder()
        .workspace(dir.path())
        .build()
        .expect("devserver must start");

    // Grant a capability to some actor.
    let alice = Cid::from_blake3_digest([0xa1; 32]);
    dev.grant(&alice, "store:post:write")
        .expect("grant must succeed");

    // Register a handler (simulate developer's initial code).
    dev.register_handler_from_str(
        "h1",
        "run",
        // Placeholder subgraph-DSL source; G11-A defines the spelling.
        "read('input') >> respond",
    )
    .expect("initial register must succeed");

    // Simulate a code change + hot-reload.
    dev.register_handler_from_str(
        "h1",
        "run",
        "read('input') >> transform('identity') >> respond",
    )
    .expect("hot-reload must succeed");

    // Grants must still be present.
    assert!(
        dev.grant_exists(&alice, "store:post:write"),
        "grant must survive hot-reload"
    );
}

#[test]
fn devserver_grant_still_satisfies_check_after_reload() {
    let dir = tempdir().unwrap();
    let mut dev = DevServer::builder().workspace(dir.path()).build().unwrap();

    let alice = Cid::from_blake3_digest([0xa2; 32]);
    dev.grant(&alice, "store:post:read").unwrap();

    // Baseline: grant is enforced.
    assert!(
        dev.check_attenuation_for_test(&alice, "store:post:read")
            .is_ok()
    );

    // Hot reload.
    dev.reload_for_test().expect("hot-reload must succeed");

    // Post-reload: grant still enforced.
    assert!(
        dev.check_attenuation_for_test(&alice, "store:post:read")
            .is_ok(),
        "grant must still satisfy check_attenuation after reload"
    );
}

#[test]
fn devserver_reload_does_not_touch_grant_audit_sequence() {
    let dir = tempdir().unwrap();
    let mut dev = DevServer::builder().workspace(dir.path()).build().unwrap();
    let alice = Cid::from_blake3_digest([0xa3; 32]);
    dev.grant(&alice, "store:post:write").unwrap();

    let seq_before = dev.grant_table_audit_sequence_for_test();
    dev.reload_for_test().expect("hot-reload must succeed");
    let seq_after = dev.grant_table_audit_sequence_for_test();

    assert_eq!(
        seq_before, seq_after,
        "hot-reload must not advance the grant table's audit sequence"
    );
}

#[test]
fn devserver_explicit_reset_dev_state_does_clear_grants() {
    // Negative pin: preservation is not "grants are immortal". The explicit
    // reset path CLEARS grants.
    let dir = tempdir().unwrap();
    let mut dev = DevServer::builder().workspace(dir.path()).build().unwrap();
    let alice = Cid::from_blake3_digest([0xa4; 32]);
    dev.grant(&alice, "store:post:write").unwrap();
    assert!(dev.grant_exists(&alice, "store:post:write"));

    dev.reset_dev_state().expect("reset must succeed");
    assert!(
        !dev.grant_exists(&alice, "store:post:write"),
        "explicit reset_dev_state must clear grants"
    );
}
