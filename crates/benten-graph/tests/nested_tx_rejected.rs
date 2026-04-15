//! Edge-case test: nested transactions must be refused with a typed error.
//!
//! Covers error code:
//! - `E_NESTED_TRANSACTION_NOT_SUPPORTED` — a Phase 1 named compromise.
//!
//! Regression: Phase 1 limits transaction scopes to non-nested calls. Phase 2
//! may lift this restriction. Users calling `backend.transaction(|_| backend.transaction(…))`
//! must see the named compromise, NOT a deadlock or silent-merge.
//!
//! R3 contract: `RedbBackend::transaction` does not exist today. R5 (G3-A)
//! ships it, and the nested-rejection check is part of the initial landing.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::testing::canonical_test_node;
use benten_graph::{GraphError, RedbBackend};
use tempfile::tempdir;

fn fresh_backend() -> (RedbBackend, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");
    let b = RedbBackend::open_or_create(&db_path).unwrap();
    (b, dir)
}

#[test]
fn nested_tx_returns_error() {
    // Regression: E_NESTED_TRANSACTION_NOT_SUPPORTED is a Phase 1 named
    // compromise. Phase 2 may permit nested transaction scopes. Until then,
    // the engine must fail loudly rather than silently-merge or deadlock.
    let (backend, _dir) = fresh_backend();

    let outer_result: Result<(), GraphError> = backend.transaction(|_outer_tx| {
        // Inner `transaction` call on the same backend must be refused.
        let inner_err = backend
            .transaction(|inner_tx| {
                inner_tx.put_node(&canonical_test_node())?;
                Ok::<(), GraphError>(())
            })
            .expect_err("nested transaction must not be permitted");

        match inner_err {
            GraphError::NestedTransactionNotSupported { .. } => {}
            other => panic!(
                "expected GraphError::NestedTransactionNotSupported (E_NESTED_TRANSACTION_NOT_SUPPORTED), got {other:?}"
            ),
        }
        Ok(())
    });

    // Outer transaction itself still completes successfully (the inner
    // rejection did not poison it).
    outer_result.expect("outer transaction must still commit after rejecting the inner one");
}

#[test]
fn nested_tx_does_not_deadlock() {
    // Sharpest-edge variant: if the implementation naively re-enters
    // redb's write lock, the test will hang. Guard against that by
    // asserting the inner call returns within a single test tick.
    //
    // We can't set wall-clock timeouts in `#[test]` without `nextest`
    // configuration, so we rely on cargo-nextest's per-test timeout
    // (workspace-wide, already configured in .config/nextest.toml).
    // The assertion here is purely semantic — the test returning at all
    // is the anti-deadlock evidence.
    let (backend, _dir) = fresh_backend();

    let _: Result<(), GraphError> = backend.transaction(|_outer| {
        let result = backend.transaction(|_inner| Ok::<(), GraphError>(()));
        assert!(result.is_err(), "nested transaction must return, not block");
        Ok(())
    });
}
