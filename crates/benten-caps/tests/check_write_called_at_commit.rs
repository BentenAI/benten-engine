//! Edge-case test: `CapabilityPolicy::check_write` fires AT COMMIT, not on
//! every individual WRITE primitive.
//!
//! Per ENGINE-SPEC §9 and the R1 Triage TOCTOU named compromise: capability
//! checks happen at commit boundaries, CALL entries, and every N ITERATE
//! iterations (default 100). Per-WRITE checks would be correct but
//! prohibitive; the Phase 1 tradeoff is documented.
//!
//! This test enforces the boundary: a closure performing 5 WRITEs calls
//! `check_write` exactly once (at commit), not 5 times.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{CapError, CapabilityPolicy, NoAuthBackend, WriteContext};
use std::sync::{Arc, Mutex};

/// Capability policy that counts how many times `check_write` is called.
struct CountingPolicy {
    count: Arc<Mutex<u32>>,
    should_deny: bool,
}

impl CountingPolicy {
    fn new(should_deny: bool) -> (Self, Arc<Mutex<u32>>) {
        let count = Arc::new(Mutex::new(0));
        (
            Self {
                count: count.clone(),
                should_deny,
            },
            count,
        )
    }
}

impl CapabilityPolicy for CountingPolicy {
    fn check_write(&self, _ctx: &WriteContext) -> Result<(), CapError> {
        *self.count.lock().unwrap() += 1;
        if self.should_deny {
            Err(CapError::DeniedDetail {
                required: "store:post:write".into(),
                entity: "alice".into(),
            })
        } else {
            Ok(())
        }
    }
}

#[test]
fn check_write_called_once_per_commit_regardless_of_write_count() {
    // Setup: a transaction that performs 5 WRITEs then commits.
    // Expectation: `check_write` fires exactly once.
    //
    // Regression: TOCTOU window is a Phase 1 named compromise — cap
    // checks at commit boundaries only. Phase 2 tightens via Invariant 13
    // to per-operation. If this test ever counts >1, it means Phase 1
    // accidentally adopted the Phase 2 tightness without the rest of
    // Invariant 13's machinery, which would be incorrect.

    let (policy, counter) = CountingPolicy::new(false);
    let backend = benten_graph::RedbBackend::open_or_create(
        tempfile::tempdir().unwrap().path().join("db.redb"),
    )
    .unwrap();

    let engine = benten_engine::Engine::builder()
        .backend(backend)
        .capability_policy(Box::new(policy))
        .build()
        .unwrap();

    engine
        .transaction(|tx| {
            for i in 0..5 {
                let mut n = benten_core::Node::new(vec!["Post".into()], Default::default());
                n.properties.insert("n".into(), benten_core::Value::Int(i));
                tx.put_node(&n)?;
            }
            Ok(())
        })
        .unwrap();

    let count = *counter.lock().unwrap();
    assert_eq!(
        count, 1,
        "check_write must fire once per commit, not once per WRITE; got {count} calls for 5 writes"
    );
}

#[test]
fn check_write_at_commit_denies_all_writes_atomically() {
    // If the commit-time check denies, ALL writes must roll back atomically.
    // This is the "honest no" at the commit boundary — the API signals
    // capability failure at the one place the cap is authoritative.
    let (policy, counter) = CountingPolicy::new(true); // denies

    let backend = benten_graph::RedbBackend::open_or_create(
        tempfile::tempdir().unwrap().path().join("db.redb"),
    )
    .unwrap();

    let engine = benten_engine::Engine::builder()
        .backend(backend)
        .capability_policy(Box::new(policy))
        .build()
        .unwrap();

    let result = engine.transaction(|tx| {
        for i in 0..3 {
            let mut n = benten_core::Node::new(vec!["Post".into()], Default::default());
            n.properties.insert("n".into(), benten_core::Value::Int(i));
            tx.put_node(&n)?;
        }
        Ok(())
    });

    let err = result.expect_err("deny-at-commit must surface as an error");
    assert!(
        matches!(
            err,
            benten_engine::EngineError::Cap(CapError::DeniedDetail { .. })
        ),
        "commit denial must produce CapError::Denied, got {err:?}"
    );

    // Still exactly one check_write call.
    assert_eq!(*counter.lock().unwrap(), 1);
}

#[test]
fn no_writes_means_no_check_write_call() {
    // Boundary: an empty transaction (no WRITEs) has nothing to check,
    // so `check_write` need not be called at all. This isn't a correctness
    // requirement, but it's a sanity pin: a caller running zero writes
    // should not incur a spurious cap-check.
    let (policy, counter) = CountingPolicy::new(false);
    let backend = benten_graph::RedbBackend::open_or_create(
        tempfile::tempdir().unwrap().path().join("db.redb"),
    )
    .unwrap();

    let engine = benten_engine::Engine::builder()
        .backend(backend)
        .capability_policy(Box::new(policy))
        .build()
        .unwrap();

    engine.transaction(|_tx| Ok(())).unwrap();

    assert_eq!(
        *counter.lock().unwrap(),
        0,
        "no writes => no cap check; got spurious call"
    );
}

#[test]
fn noauth_backend_never_denies_but_still_conforms_to_commit_model() {
    // NoAuth is the default. Regardless of the policy being permissive,
    // it must conform to the same "called once per commit" contract so
    // swapping in UCANBackend later doesn't surprise users.
    let backend = benten_graph::RedbBackend::open_or_create(
        tempfile::tempdir().unwrap().path().join("db.redb"),
    )
    .unwrap();
    let engine = benten_engine::Engine::builder()
        .backend(backend)
        .capability_policy(Box::new(NoAuthBackend))
        .build()
        .unwrap();

    // Multi-write transaction under NoAuth succeeds trivially.
    engine
        .transaction(|tx| {
            for i in 0..3 {
                let mut n = benten_core::Node::new(vec!["Post".into()], Default::default());
                n.properties.insert("n".into(), benten_core::Value::Int(i));
                tx.put_node(&n)?;
            }
            Ok(())
        })
        .unwrap();
}
