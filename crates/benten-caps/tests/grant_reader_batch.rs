//! R3 unit tests for ucca-6 (FROZEN): `GrantReader::has_unrevoked_grant_for_any(&[scope])`
//! batch method.
//!
//! Shape-pin test: the new method on the trait returns `true` iff any scope
//! in the batch has an unrevoked grant, and performs a single backing read
//! (not N).
//!
//! TDD red-phase: the method does not yet exist on the trait. Tests will fail
//! to compile until G9-A / ucca-6 lands.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.4 ucca-6).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{CapError, GrantReader};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A minimal in-process `GrantReader` used to pin the batch contract.
/// Counts backend reads so the test can assert single-call semantics.
struct RecordingReader {
    unrevoked: Vec<String>,
    calls: Arc<AtomicUsize>,
}

impl GrantReader for RecordingReader {
    fn has_unrevoked_grant_for_scope(&self, scope: &str) -> Result<bool, CapError> {
        // Used by Phase-1 single-scope tests. Batch tests go through
        // `has_unrevoked_grant_for_any`.
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.unrevoked.iter().any(|s| s == scope))
    }

    // Phase-2a additive batch method. Default impl is the single-scope
    // fallback (N reads); concrete impls override to a single backend read.
    fn has_unrevoked_grant_for_any(&self, scopes: &[&str]) -> Result<bool, CapError> {
        // Record exactly ONE call for the whole batch — shape-pin contract.
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(scopes.iter().any(|s| self.unrevoked.iter().any(|u| u == s)))
    }
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn grant_reader_has_unrevoked_grant_for_any_batch() {
    let calls = Arc::new(AtomicUsize::new(0));
    let reader = RecordingReader {
        unrevoked: vec!["store:post:write".to_string()],
        calls: calls.clone(),
    };

    // 5-scope batch, one of which has a grant.
    let scopes = [
        "store:comment:write",
        "store:user:write",
        "store:post:write",
        "store:image:write",
        "store:vote:write",
    ];
    let out = reader.has_unrevoked_grant_for_any(&scopes).expect("query");
    assert!(
        out,
        "batch must return true when any single scope is unrevoked"
    );

    // Must be a single backend read (one call to the recording reader).
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "batch query must perform a single backend read, not N"
    );
}

#[test]
fn grant_reader_has_unrevoked_grant_for_any_empty_batch_returns_false() {
    let calls = Arc::new(AtomicUsize::new(0));
    let reader = RecordingReader {
        unrevoked: vec!["store:post:write".into()],
        calls,
    };

    let out = reader
        .has_unrevoked_grant_for_any(&[])
        .expect("empty batch");
    assert!(
        !out,
        "empty batch must return false — no grants to match against"
    );
}
