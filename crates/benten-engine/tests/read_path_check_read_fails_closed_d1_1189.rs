//! Refinement-audit-2026-05 D1 #1189 closure-pin (Pattern-F Bundle 1 —
//! Qual-1 #695 + Safe-1 #534 / META #593 class-B-β auth-bypass).
//!
//! ## The bug this pins closed
//!
//! Three production read pathways
//! (`Engine::get_node` / `Engine::read_node_as` / `Engine::read_view*`)
//! gated a Node read on `policy.check_read(&ctx)` but pattern-matched
//! ONLY `Err(CapError::DeniedRead { .. })`:
//!
//! ```ignore
//! if let Err(CapError::DeniedRead { .. }) = policy.check_read(&ctx) {
//!     return Ok(None); // or empty Outcome
//! }
//! Ok(Some(node)) // <-- reached for EVERY non-DeniedRead Err
//! ```
//!
//! `CapError` is `#[non_exhaustive]` with multiple denial variants
//! (`Denied`, `Revoked`, `RevokedMidEval`, `NotImplemented`, the Phase-3
//! UCAN variants, plus any future addition). When `check_read` denied
//! via ANY non-`DeniedRead` variant, the `if let` fell through and the
//! read SUCCEEDED — a silent capability bypass for any policy that
//! revokes via `Revoked` / `RevokedMidEval`. The default `NoAuthBackend`
//! masks the bug (it returns `Ok(())` unconditionally); the latent risk
//! lives at the `CapabilityPolicy` trait surface that Phase-4-Meta /
//! external plugins exploit (CLAUDE.md baked-in #18 — `read_node_as` is
//! the Class-B-β attributed-read path, the most load-bearing site).
//!
//! ## What the fix does
//!
//! D1 #1189 collapsed the three near-duplicate bodies into the canonical
//! `Engine::read_node_inner` seam (Qual-1 #695) whose single
//! `Engine::check_read_gate` arm fails CLOSED (Safe-1 #534):
//! `Ok(()) => permit`, `Err(DeniedRead) => Option-C collapse`,
//! `Err(other) => Err(EngineError::Cap(other))`. The IVM `read_view`
//! site shares `check_read_gate` (it cannot share `read_node_inner` —
//! no resolved Node — but MUST share the fail-CLOSED decision).
//!
//! ## Why this test FAILS if the fix is reverted
//!
//! A `DenyViaRevoked` policy returns `Err(CapError::Revoked)` for the
//! target label. Pre-fix, all three sites returned `Ok(Some(node))` /
//! a populated Outcome (the `Revoked` Err fell through). This test
//! asserts each surfaces `Err(EngineError::Cap(CapError::Revoked))`
//! instead — restoring the old `if let Err(DeniedRead)` shape makes
//! every assertion below fail with the leaked Node / populated list.
//!
//! It also keeps a `DeniedRead` arm asserting the Option-C collapse is
//! preserved (named compromise #2 — a denied read is indistinguishable
//! from a clean miss), so the fix cannot over-correct `DeniedRead` into
//! a hard error.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{CapError, CapWriteContext, CapabilityPolicy, ReadContext};
use benten_core::{Node, Value};
use benten_engine::{Engine, EngineError};
use std::collections::BTreeMap;

/// Policy that DENIES every read via `CapError::Revoked` (a non-
/// `DeniedRead` denial variant — the exact class the pre-fix `if let
/// Err(DeniedRead)` shape silently permitted) and permits every write.
#[derive(Debug)]
struct DenyViaRevoked;

impl CapabilityPolicy for DenyViaRevoked {
    fn check_write(&self, _ctx: &CapWriteContext) -> Result<(), CapError> {
        Ok(())
    }
    fn check_read(&self, _ctx: &ReadContext) -> Result<(), CapError> {
        Err(CapError::Revoked)
    }
}

/// Policy that denies every read via `CapError::DeniedRead` — used to
/// pin that the Option-C collapse (named compromise #2) is preserved by
/// the fix (the fix must not turn `DeniedRead` into a hard error).
#[derive(Debug)]
struct DenyViaDeniedRead;

impl CapabilityPolicy for DenyViaDeniedRead {
    fn check_write(&self, _ctx: &CapWriteContext) -> Result<(), CapError> {
        Ok(())
    }
    fn check_read(&self, _ctx: &ReadContext) -> Result<(), CapError> {
        Err(CapError::DeniedRead {
            required: "test:read".to_string(),
            entity: "test-entity".to_string(),
        })
    }
}

fn post() -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text("hello".into()));
    Node::new(vec!["post".into()], props)
}

fn engine_with(policy: Box<dyn CapabilityPolicy>) -> (Engine, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy(policy)
        .build()
        .unwrap();
    (engine, dir)
}

/// Site 1 — `Engine::get_node` (engine_crud.rs). A `Revoked` denial
/// MUST surface `Err(EngineError::Cap(CapError::Revoked))`, NOT
/// `Ok(Some(node))`.
#[test]
fn get_node_fails_closed_on_non_denied_read_revoked() {
    let (engine, _dir) = engine_with(Box::new(DenyViaRevoked));
    let cid = engine.create_node(&post()).expect("privileged write");

    match engine.get_node(&cid) {
        Err(EngineError::Cap(CapError::Revoked)) => {}
        other => panic!(
            "D1 #1189 / Safe-1 #534 REVERTED at Engine::get_node: a \
             `CapError::Revoked` read denial MUST fail CLOSED as \
             `Err(EngineError::Cap(CapError::Revoked))`. The pre-fix \
             `if let Err(CapError::DeniedRead {{ .. }})` shape let the \
             non-DeniedRead denial fall through and silently returned \
             the Node. Got: {other:?}"
        ),
    }
}

/// Site 2 — `Engine::read_node_as` (engine_wait.rs; Class-B-β attributed
/// read, CLAUDE.md baked-in #18 — the MOST load-bearing site for this
/// bug). A `Revoked` denial MUST surface
/// `Err(EngineError::Cap(CapError::Revoked))`, NOT `Ok(Some(node))`.
#[test]
fn read_node_as_fails_closed_on_non_denied_read_revoked() {
    let (engine, _dir) = engine_with(Box::new(DenyViaRevoked));
    let cid = engine.create_node(&post()).expect("privileged write");
    let principal = benten_core::Cid::from_blake3_digest([7u8; 32]);

    match engine.read_node_as(&principal, &cid) {
        Err(EngineError::Cap(CapError::Revoked)) => {}
        other => panic!(
            "D1 #1189 / Safe-1 #534 REVERTED at Engine::read_node_as \
             (Class-B-β attributed read, CLAUDE.md #18): a \
             `CapError::Revoked` read denial MUST fail CLOSED as \
             `Err(EngineError::Cap(CapError::Revoked))`. Pre-fix the \
             non-DeniedRead denial fell through and the attributed read \
             silently succeeded — a credentialed-read bypass for any \
             revoked principal. Got: {other:?}"
        ),
    }
}

/// Site 3 — `Engine::read_view` IVM view-read (engine_views.rs). A
/// `Revoked` denial MUST surface `Err(EngineError::Cap(..))`, NOT a
/// populated / empty-but-Ok Outcome (returning rows leaks existence;
/// returning Ok-empty silently honors a bypassed read as "permitted").
///
/// `content_listing_post` resolves a non-empty label hint (`"post"`)
/// via the `content_listing_<label>` prefix fallback, so the read-gate
/// actually fires (an empty hint short-circuits the gate by design).
#[test]
fn read_view_fails_closed_on_non_denied_read_revoked() {
    let (engine, _dir) = engine_with(Box::new(DenyViaRevoked));
    let handler_id = engine.register_crud("post").unwrap();
    engine.call(&handler_id, "post:create", post()).unwrap();

    match engine.read_view("content_listing_post") {
        Err(EngineError::Cap(CapError::Revoked)) => {}
        other => panic!(
            "D1 #1189 / Safe-1 #534 REVERTED at Engine::read_view: a \
             `CapError::Revoked` read denial MUST fail CLOSED as \
             `Err(EngineError::Cap(CapError::Revoked))`. Pre-fix the \
             `if let Err(CapError::DeniedRead {{ .. }})` shape let the \
             non-DeniedRead denial fall through and the view's full row \
             set was returned. Got: {other:?}"
        ),
    }
}

/// Regression guard — the fix must NOT over-correct: a genuine
/// `DeniedRead` denial MUST still collapse to the Option-C shape
/// (`Ok(None)` for node reads; empty Outcome for view reads) per
/// Phase-1 named compromise #2 (a denied read is indistinguishable
/// from a clean miss — no CID-existence leak).
#[test]
fn denied_read_still_collapses_option_c_not_hard_error() {
    let (engine, _dir) = engine_with(Box::new(DenyViaDeniedRead));
    let cid = engine.create_node(&post()).expect("privileged write");
    let principal = benten_core::Cid::from_blake3_digest([9u8; 32]);

    assert!(
        matches!(engine.get_node(&cid), Ok(None)),
        "DeniedRead must still collapse to Ok(None) at get_node \
         (Option-C / named compromise #2) — the fail-CLOSED fix must \
         not turn DeniedRead into a hard error"
    );
    assert!(
        matches!(engine.read_node_as(&principal, &cid), Ok(None)),
        "DeniedRead must still collapse to Ok(None) at read_node_as \
         (Option-C / named compromise #2)"
    );
    let outcome = engine
        .read_view("content_listing_post")
        .expect("DeniedRead at read_view must collapse to an empty Outcome, not Err");
    assert!(
        outcome.as_list().is_none_or(|v| v.is_empty()),
        "DeniedRead at read_view must collapse to an empty Outcome list \
         (Option-C / named compromise #2), got: {outcome:?}"
    );
}
