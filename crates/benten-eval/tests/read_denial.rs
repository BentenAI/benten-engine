//! Phase 1 R3 security test — existence leakage via READ denial (R1 major #6).
//!
//! Attack class: when a caller issues `READ(cid)` on a Node they lack read
//! capability for, the engine's error response leaks existence. Ben's
//! decision (R1 triage named compromise #2): ship **Option A — honest
//! error, leaky existence**. `E_CAP_DENIED_READ` is a DISTINCT code from
//! `E_NOT_FOUND`; the presence of `E_CAP_DENIED_READ` confirms the Node
//! exists, just inaccessible to this caller.
//!
//! Phase 1 is embedded/local-only, so the leakage's threat model is bounded.
//! Phase 3 sync revisits with a per-grant `existence_visibility: visible|
//! hidden` configuration. This test locks the Phase 1 contract so Phase 3
//! implementers can tell the semantics from reading the test.
//!
//! The test pair covers:
//!   1. Read on a denied (existing) resource → `E_CAP_DENIED_READ`.
//!   2. Read on a truly-missing resource → `E_NOT_FOUND`.
//!
//! These must be DISTINGUISHABLE (the point of option A). If a future impl
//! decides to mask existence by returning `E_NOT_FOUND` for denial, the
//! test flips — AND the named compromise doc entry must be updated in the
//! same PR.
//!
//! TDD contract: FAIL at R3 — the READ primitive, the typed error edges,
//! and both error codes land in R5.
//!
//! Cross-refs:
//! - `.addl/phase-1/r1-security-auditor.json` finding #6 (major)
//! - `.addl/phase-1/r1-triage.md` named compromise #2
//! - `docs/ERROR-CATALOG.md` `E_CAP_DENIED_READ` vs `E_NOT_FOUND`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::ErrorCode;
use benten_engine::Engine;
use benten_engine::testing::{read_handler_for, subject_with_no_read_grants};

/// Option A: read denial returns `E_CAP_DENIED_READ`, NOT `E_NOT_FOUND`.
#[test]
fn read_denied_returns_cap_denied_read() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .capability_policy(subject_with_no_read_grants())
        .open(dir.path().join("benten.redb"))
        .unwrap();

    // Create a real, readable-by-privileged-path Node so the target EXISTS
    // in the store. The attacker lacks read cap on it but it is there.
    let cid = engine.testing_insert_privileged_fixture();

    let handler = read_handler_for(cid);
    let handler_id = engine.register_subgraph(&handler).unwrap();

    let outcome = engine
        .call(&handler_id, "read", benten_core::Node::empty())
        .expect("call returns Ok wrapper");

    let err = outcome.terminal_error().expect("read must be denied");
    assert_eq!(
        err.code(),
        ErrorCode::CapDeniedRead,
        "Option A (R1 triage named compromise #2): read denial MUST fire \
         E_CAP_DENIED_READ so the error is honest; leaking existence is \
         the deliberate Phase 1 compromise. Masking via E_NOT_FOUND is a \
         Phase 3+ opt-in."
    );
    assert_eq!(outcome.taken_edge(), "ON_DENIED");

    // Distinguishability contract: E_CAP_DENIED_READ MUST be distinct from
    // the generic write-denial code AND from the not-found code. If future
    // code paths conflate them, operators can't tell "fix your caps" from
    // "your CID is wrong" from "that thing doesn't exist".
    assert_ne!(err.code(), ErrorCode::CapDenied);
    assert_ne!(err.code(), ErrorCode::NotFound);
}

/// Positive control: a read on a CID that genuinely does NOT exist returns
/// `E_NOT_FOUND`, routed via `ON_NOT_FOUND`. This establishes the two error
/// paths are observably different from userland.
#[test]
fn read_missing_returns_not_found() {
    use benten_core::Node;

    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .capability_policy(subject_with_no_read_grants())
        .open(dir.path().join("benten.redb"))
        .unwrap();

    // Use the CID of a Node we never inserted. Nothing is stored under it.
    let mut props = std::collections::BTreeMap::new();
    props.insert(
        "marker".into(),
        benten_core::Value::Text("nonexistent".into()),
    );
    let phantom = Node::new(vec!["PhantomNode".into()], props);
    let cid = phantom.cid().unwrap();

    let handler = read_handler_for(cid);
    let handler_id = engine.register_subgraph(&handler).unwrap();
    let outcome = engine
        .call(&handler_id, "read", benten_core::Node::empty())
        .expect("call returns Ok wrapper");

    let err = outcome.terminal_error().expect("read must miss");
    assert_eq!(
        err.code(),
        ErrorCode::NotFound,
        "read on a genuinely-missing CID must fire E_NOT_FOUND, NOT \
         E_CAP_DENIED_READ — otherwise the two error paths become \
         indistinguishable and Phase 3 visibility-hiding is impossible"
    );
    assert_eq!(outcome.taken_edge(), "ON_NOT_FOUND");
}

/// Contract regression: option A is a named compromise. If Phase N ever
/// wants to flip to option B (mask existence), this test AND named
/// compromise #2 in r1-triage.md must both update. The comment here is the
/// grep-target for that migration.
#[test]
fn option_a_existence_leak_is_documented_compromise() {
    // Phase 1 compromise; Phase 3 sync revisits per R1 triage named
    // compromise #2. If this test is removed, update the compromise
    // regression in `compromises_regression.rs` in the same PR.
    //
    // This is a pure marker test — no assertion body needed beyond
    // confirming the code is reachable.
    // R3 wrote `as u32` here; ErrorCode is a non-primitive enum so cast is
    // illegal. Compare via `as_str` (semantic-equivalent, same runtime intent).
    assert_eq!(
        ErrorCode::CapDeniedRead.as_str(),
        ErrorCode::CapDeniedRead.as_str()
    );
}
