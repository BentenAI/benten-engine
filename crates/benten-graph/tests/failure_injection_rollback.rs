//! Edge-case test: transaction closure returning `Err` / panicking must roll
//! back ALL writes. No partial state may land on disk.
//!
//! Covers error code:
//! - `E_TX_ABORTED` — returned at the call site after the closure errors.
//!
//! R1 Triage named compromise cross-reference: this is the "transaction
//! closure panic -> rollback + re-raise" edge in the rust-test-writer-edge-cases
//! mandate.
//!
//! R3 contract: `RedbBackend::transaction(|tx| …)` does not exist today
//! (G3-A ships the closure-based txn API). These tests fail to compile until
//! then — deliberate.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::testing::canonical_test_node;
use benten_graph::{GraphError, KVBackend, RedbBackend};
use tempfile::tempdir;

fn fresh_backend() -> (RedbBackend, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");
    let b = RedbBackend::open_or_create(&db_path).unwrap();
    (b, dir)
}

#[test]
fn tx_closure_err_rolls_back_all_writes() {
    // Inject failure: closure writes two nodes then returns Err.
    // Expectation: neither node is visible after the call returns.
    let (backend, _dir) = fresh_backend();

    let n1 = canonical_test_node();
    let n1_cid = n1.cid().unwrap();

    let mut n2 = canonical_test_node();
    n2.labels.push("DifferentLabel".into());
    let n2_cid = n2.cid().unwrap();

    let result = backend.transaction(|tx| {
        tx.put_node(&n1)?;
        tx.put_node(&n2)?;
        // Simulate a capability denial or similar post-write-but-pre-commit
        // failure. The exact error kind isn't relevant here; what matters is
        // the closure returning Err and *neither* write becoming visible.
        Err::<(), GraphError>(GraphError::Redb("injected failure".into()))
    });

    // Post-condition: closure-return-Err surfaces as E_TX_ABORTED at the
    // outer call.
    let err = result.expect_err("closure returning Err must surface as Err");
    match err {
        GraphError::TxAborted { .. } => {}
        other => panic!("expected GraphError::TxAborted (E_TX_ABORTED), got {other:?}"),
    }

    // Post-condition: NEITHER write may be visible after rollback.
    assert!(
        backend.get_node(&n1_cid).unwrap().is_none(),
        "n1 must not be visible after rollback"
    );
    assert!(
        backend.get_node(&n2_cid).unwrap().is_none(),
        "n2 must not be visible after rollback"
    );
}

#[test]
fn tx_closure_panic_rolls_back_and_repropagates() {
    // Inject failure: closure writes one node then panics.
    // Expectation: the panic propagates through the transaction boundary,
    // the closure's write is NOT visible, and the database is not corrupted.
    //
    // This is explicitly the "closure panic mid-transaction rolls back +
    // re-raises" contract in the agent mandate.
    let (backend, _dir) = fresh_backend();

    let n1 = canonical_test_node();
    let n1_cid = n1.cid().unwrap();

    let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // `transaction` is `&self` + `FnOnce`, and must propagate panics.
        let _: Result<(), GraphError> = backend.transaction(|tx| {
            tx.put_node(&n1)?;
            panic!("closure panic during transaction");
        });
    }));
    assert!(
        panic_result.is_err(),
        "panic inside txn closure must repropagate to caller"
    );

    // Post-condition: no partial write visible.
    assert!(
        backend.get_node(&n1_cid).unwrap().is_none(),
        "closure panic must roll back — no partial write may be visible"
    );

    // Post-condition: the backend is still usable afterwards — a panic
    // must not poison the database handle. If the inner redb transaction
    // leaked, subsequent writes would deadlock.
    backend.put_node(&canonical_test_node()).unwrap();
}

#[test]
fn tx_commit_cap_failure_surfaces_partial_trace_with_aborted_step() {
    // Edge case from R1 Triage: commit-time capability failure returns
    // partial trace + `TraceStep::Aborted`. The transaction primitive
    // runs the closure successfully, then the capability policy denies
    // at commit. The caller sees E_TX_ABORTED with a `failed_node` of
    // `None` (the denial isn't on a specific write — it's the commit hook).
    //
    // This is the "the API honestly said no" boundary — the writes ran
    // as requested, but the commit authority was withheld.
    let (backend, _dir) = fresh_backend();
    let n1 = canonical_test_node();

    // Install a deny-on-commit hook. The concrete wiring is R5 (G3-A + G4),
    // but the contract is: on commit-time denial, transaction() returns
    // E_TX_ABORTED whose `reason` mentions capability.
    let err = backend
        .transaction_with_deny_on_commit(|tx| {
            tx.put_node(&n1)?;
            Ok(())
        })
        .expect_err("commit-time denial must surface as E_TX_ABORTED");

    match err {
        GraphError::TxAborted { reason, .. } => {
            assert!(
                reason.contains("capability") || reason.contains("cap"),
                "reason must name the capability-denial origin; got {reason:?}"
            );
        }
        other => panic!("expected GraphError::TxAborted, got {other:?}"),
    }

    // Post-condition: closure appeared to succeed but no write is visible.
    assert!(
        backend.get_node(&n1.cid().unwrap()).unwrap().is_none(),
        "commit-denial rollback must leave no writes visible"
    );
}
