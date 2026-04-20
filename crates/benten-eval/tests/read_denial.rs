//! Phase 1 security tests — Option C (symmetric-None + diagnostic capability).
//!
//! Named compromise #2 shipped Option A in the R5 base scaffold
//! (`E_CAP_DENIED_READ` error — honest but existence-leaking). 5d-J
//! workstream 1 migrates to **Option C**:
//!
//! 1. `Engine::get_node` (and the other public read surfaces) return
//!    `Ok(None)` when the policy denies the read — symmetric with a
//!    genuine backend miss. An unauthorised caller CANNOT distinguish
//!    "denied" from "never existed".
//! 2. The distinction is recoverable through [`Engine::diagnose_read`],
//!    gated on a `debug:read` capability grant. Ordinary callers can't
//!    fish the existence signal; operators who hold the diagnostic
//!    grant get three-state output (`existsInBackend`,
//!    `deniedByPolicy`, `notFound`).
//!
//! See `docs/SECURITY-POSTURE.md` §Compromise #2 for the full posture.
//!
//! The earlier Option-A `read_denied_returns_cap_denied_read` test is
//! obsolete under Option C and has been removed; its contract is
//! replaced by `get_node_on_denied_cid_returns_none_symmetric_with_miss`
//! below. The `option_a_existence_leak_is_documented_compromise` test
//! (which greps the posture doc) is superseded by the Option-C posture
//! text that now lives in `docs/SECURITY-POSTURE.md`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;
use benten_errors::ErrorCode;
use std::sync::Arc;

// A deny-all-reads policy used to exercise the Option-C read-gate. The
// write path is permissive (NoAuth-equivalent) so the fixture can still
// populate the backend via `testing_insert_privileged_fixture`.
#[derive(Debug)]
struct DenyAllReadsPolicy;

impl benten_caps::CapabilityPolicy for DenyAllReadsPolicy {
    fn check_write(&self, _ctx: &benten_caps::WriteContext) -> Result<(), benten_caps::CapError> {
        Ok(())
    }

    fn check_read(&self, ctx: &benten_caps::ReadContext) -> Result<(), benten_caps::CapError> {
        // Permit the `debug:read` diagnostic probe so
        // `Engine::diagnose_read` still works under this policy — the
        // diagnostic path is the Option-C escape hatch and we test it
        // below.
        if ctx.label == "debug" {
            return Ok(());
        }
        // Empty label = introspection read. Permit so existing Phase-1
        // tests that reach `get_node` on unlabelled Nodes still pass.
        if ctx.label.is_empty() {
            return Ok(());
        }
        Err(benten_caps::CapError::DeniedRead {
            required: format!("store:{}:read", ctx.label),
            entity: ctx
                .target_cid
                .as_ref()
                .map(benten_core::Cid::to_base32)
                .unwrap_or_default(),
        })
    }
}

#[test]
fn get_node_on_denied_cid_returns_none_symmetric_with_miss() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .capability_policy(Box::new(DenyAllReadsPolicy))
        .open(dir.path().join("benten.redb"))
        .unwrap();

    // A CID that exists in the backend but is denied by policy.
    let present_cid = engine.testing_insert_privileged_fixture();

    // A CID that was never written.
    let phantom_cid = {
        let mut props = std::collections::BTreeMap::new();
        props.insert(
            "marker".into(),
            benten_core::Value::Text("never-inserted".into()),
        );
        benten_core::Node::new(vec!["PhantomLabel".into()], props)
            .cid()
            .unwrap()
    };

    // Both return Ok(None). The caller cannot tell the denied-but-
    // existing CID apart from the never-written CID from this API.
    assert!(
        engine.get_node(&present_cid).unwrap().is_none(),
        "Option C: a denied read must collapse to Ok(None), symmetric with a miss"
    );
    assert!(
        engine.get_node(&phantom_cid).unwrap().is_none(),
        "positive control: a genuinely missing CID returns Ok(None)"
    );
}

#[test]
fn diagnose_read_requires_debug_read_capability() {
    #[derive(Debug)]
    struct DenyEverything;
    impl benten_caps::CapabilityPolicy for DenyEverything {
        fn check_write(
            &self,
            _ctx: &benten_caps::WriteContext,
        ) -> Result<(), benten_caps::CapError> {
            Ok(())
        }
        fn check_read(&self, ctx: &benten_caps::ReadContext) -> Result<(), benten_caps::CapError> {
            // Deny the debug:read probe — the caller lacks the
            // diagnostic capability.
            Err(benten_caps::CapError::DeniedRead {
                required: format!("store:{}:read", ctx.label),
                entity: String::new(),
            })
        }
    }

    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .capability_policy(Box::new(DenyEverything))
        .open(dir.path().join("benten.redb"))
        .unwrap();

    let cid = engine.testing_insert_privileged_fixture();
    let err = engine
        .diagnose_read(&cid)
        .expect_err("diagnose_read must reject when debug:read is not held");
    // The Option-C contract folds DeniedRead on the debug probe into
    // CapError::Denied at the diagnostic boundary.
    assert_eq!(err.error_code(), ErrorCode::CapDenied);
}

#[test]
fn diagnose_read_with_capability_surfaces_denied_by_policy() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .capability_policy(Box::new(DenyAllReadsPolicy))
        .open(dir.path().join("benten.redb"))
        .unwrap();

    let cid = engine.testing_insert_privileged_fixture();
    // The DenyAllReadsPolicy above permits `debug:read` but denies
    // every other label — so diagnose_read succeeds and reports a
    // `denied_by_policy` signal for the underlying post:read.
    let info = engine.diagnose_read(&cid).expect("debug:read is held");
    assert!(
        info.exists_in_backend,
        "Nodes inserted via testing_insert_privileged_fixture must exist"
    );
    assert!(
        info.denied_by_policy.is_some(),
        "caller has debug:read but not post:read — deniedByPolicy MUST be populated"
    );
    assert!(
        info.denied_by_policy.as_deref().unwrap().contains("post"),
        "the denied scope should name the Node's label"
    );
    assert!(!info.not_found);
}

#[test]
fn diagnose_read_with_capability_surfaces_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .capability_policy(Box::new(DenyAllReadsPolicy))
        .open(dir.path().join("benten.redb"))
        .unwrap();

    // A CID that was never written.
    let mut props = std::collections::BTreeMap::new();
    props.insert(
        "marker".into(),
        benten_core::Value::Text("never-inserted".into()),
    );
    let phantom = benten_core::Node::new(vec!["PhantomLabel".into()], props);
    let cid = phantom.cid().unwrap();

    let info = engine.diagnose_read(&cid).expect("debug:read is held");
    assert!(!info.exists_in_backend);
    assert!(info.not_found);
    assert!(
        info.denied_by_policy.is_none(),
        "not-found paths carry no denial signal — the backend simply has no byte-payload"
    );
}

#[test]
fn diagnose_read_under_noauth_is_open() {
    // Under NoAuth the diagnose_read surface is unrestricted (no policy
    // is plumbed in, so no gate fires). Documented in the Engine doc on
    // `diagnose_read` — embedded / single-user deployments get
    // diagnostics out of the box.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let cid = engine.testing_insert_privileged_fixture();
    let info = engine.diagnose_read(&cid).expect("NoAuth permits");
    assert!(info.exists_in_backend);
    assert!(info.denied_by_policy.is_none());
}

#[test]
fn edges_from_on_denied_cid_returns_empty_symmetric_with_zero_outgoing() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .capability_policy(Box::new(DenyAllReadsPolicy))
        .open(dir.path().join("benten.redb"))
        .unwrap();

    let cid = engine.testing_insert_privileged_fixture();
    let edges = engine
        .edges_from(&cid)
        .expect("edges_from returns Ok even when denied");
    assert!(
        edges.is_empty(),
        "Option C: denied reads produce an empty edge vec, symmetric with a Node that has no outgoing edges"
    );
}

#[test]
fn compromise_2_option_c_is_documented() {
    // The posture doc must name Option C as the shipped Phase-1
    // behaviour so operators reading SECURITY-POSTURE.md understand
    // why `engine.getNode(cid)` returns null on both denial and miss.
    let posture = std::fs::read_to_string("../../docs/SECURITY-POSTURE.md")
        .or_else(|_| std::fs::read_to_string("docs/SECURITY-POSTURE.md"))
        .expect("SECURITY-POSTURE.md must be present at repo root");
    assert!(
        posture.contains("Option C"),
        "SECURITY-POSTURE.md must document the migrated posture as Option C"
    );
    assert!(
        posture.contains("diagnose_read") || posture.contains("diagnoseRead"),
        "SECURITY-POSTURE.md must reference the Engine::diagnose_read escape hatch"
    );
}

// Re-exported so the Arc-based type is consumable from this crate's
// dependency graph without pulling in benten-caps directly as a dev-dep
// keyword. (benten-caps is already a dep via `benten-engine`'s re-export
// surface.)
#[allow(dead_code, reason = "kept for symmetry with prior test scaffolding")]
fn _keep_arc(_: Arc<()>) {}
