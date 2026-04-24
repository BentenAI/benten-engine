//! Phase 2a G5-B-i mini-review C1: WRITE-path Inv-11 short-circuit.
//!
//! **Contract under test.** `impl PrimitiveHost::put_node` on [`Engine`] MUST
//! fire `EvalError::Invariant(InvariantViolation::SystemZone)` — the
//! Phase-2a user-surface code `E_INV_SYSTEM_ZONE` — when the candidate Node
//! carries a `system:*`-prefixed label. The check runs BEFORE the buffered
//! `PendingHostOp::PutNode` is pushed onto the active-call frame, so the
//! replay path never sees the violating op and the storage-layer stopgap
//! `guard_system_zone_node` (which fires the legacy Phase-1
//! `E_SYSTEM_ZONE_WRITE` code) is never reached through the evaluator path.
//!
//! **Why both codes exist.** The storage-layer check (`E_SYSTEM_ZONE_WRITE`)
//! is retained as defence-in-depth (see
//! `crates/benten-graph/src/redb_backend.rs::guard_system_zone_node` +
//! `docs/ERROR-CATALOG.md` §`E_SYSTEM_ZONE_WRITE`). Under Phase 2a the
//! *user-facing* contract is `E_INV_SYSTEM_ZONE`; a WRITE that makes it to
//! the storage-layer check indicates the runtime probe was bypassed. This
//! test pins the probe firing so that never happens through the normal
//! evaluator → PrimitiveHost path.
//!
//! **Companion to** `inv_11_transform_constructed_cid_adversarial.rs`, which
//! exercises the READ side of the same probe.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use benten_core::{Node, Value};
use benten_engine::Engine;
use benten_eval::{EvalError, InvariantViolation, PrimitiveHost};

/// Construct a Node whose primary label lies inside the Phase-2a system zone.
/// Matches the shape the Phase-1 storage-layer check (`guard_system_zone_node`)
/// rejects: `labels[0]` begins with `system:`.
fn system_labeled_node() -> Node {
    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    props.insert("scope".into(), Value::Text("post:write".into()));
    props.insert("subject".into(), Value::Text("attacker".into()));
    Node::new(vec!["system:CapabilityGrant".into()], props)
}

/// C1 core: the evaluator-visible WRITE path fires the Phase-2a
/// `E_INV_SYSTEM_ZONE` code (typed variant `EvalError::Invariant(SystemZone)`)
/// — NOT the Phase-1 storage-layer stopgap `E_SYSTEM_ZONE_WRITE`.
///
/// The assertion is on the `EvalError` variant because that's what the
/// evaluator sees when `write::execute` invokes `host.put_node`; the variant's
/// stable catalog code (via `EvalError::code()` / `InvariantViolation::code()`)
/// is `E_INV_SYSTEM_ZONE`. The test does NOT assert on `E_SYSTEM_ZONE_WRITE`;
/// that code is reserved for the unreachable-under-normal-flow storage-layer
/// defence-in-depth path.
#[test]
fn put_node_with_system_label_fires_inv_system_zone_not_stopgap() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let node = system_labeled_node();
    let err = <Engine as PrimitiveHost>::put_node(&engine, &node)
        .expect_err("PrimitiveHost::put_node must reject a system:* Node");

    match err {
        EvalError::Invariant(InvariantViolation::SystemZone) => {
            // Cross-check the stable catalog code as a belt-and-suspenders pin.
            assert_eq!(
                InvariantViolation::SystemZone.code().as_str(),
                "E_INV_SYSTEM_ZONE",
                "Phase 2a: InvariantViolation::SystemZone MUST map to \
                 E_INV_SYSTEM_ZONE (the user-facing Inv-11 code), NOT the \
                 Phase-1 storage-layer stopgap E_SYSTEM_ZONE_WRITE"
            );
        }
        other => panic!(
            "Phase 2a: WRITE-path Inv-11 short-circuit MUST fire \
             EvalError::Invariant(SystemZone) (→ E_INV_SYSTEM_ZONE) BEFORE \
             the PendingHostOp is buffered. Got: {other:?}"
        ),
    }
}

/// Negative: a non-system-zone label still flows through buffering unchanged,
/// so the short-circuit doesn't regress the happy path. Exercised outside a
/// dispatch frame so the `put_node` fallback arm (direct backend transaction)
/// runs; a successful `Cid` return proves the runtime probe didn't falsely
/// classify the benign label as system-zone.
#[test]
fn put_node_with_benign_label_still_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    props.insert("title".into(), Value::Text("hello".into()));
    let node = Node::new(vec!["post".into()], props);

    let cid = <Engine as PrimitiveHost>::put_node(&engine, &node)
        .expect("benign (non-system-zone) label MUST NOT trip Inv-11 probe");
    assert!(!cid.to_base32().is_empty());
}

/// Exhaustive sweep over the Phase-2a system-zone prefix set: every listed
/// prefix MUST be rejected at the WRITE path with the same code.
///
/// Mirrors the READ-side adversarial proptest's spirit: no matter which
/// specific `system:*` label the handler tries to write, the firing code is
/// the Phase-2a `E_INV_SYSTEM_ZONE`.
#[test]
fn put_node_sweeps_every_system_zone_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // The classification is intentionally broad (`starts_with("system:")`);
    // we exercise the concrete prefixes the engine itself mints plus a
    // synthetic unknown-but-still-system label to pin the broad contract.
    let labels = [
        "system:CapabilityGrant",
        "system:IVMView",
        "system:CapabilityRevocation",
        "system:Principal",
        "system:Grant",
        "system:WaitPending",
        "system:WaitResume",
        "system:ModuleManifest",
        "system:ivm:content_listing",
        "system:internal:forbidden", // unknown prefix, still system-zone
    ];
    for label in labels {
        let node = Node::new(vec![label.to_string()], BTreeMap::new());
        let err = <Engine as PrimitiveHost>::put_node(&engine, &node)
            .expect_err("every system:* label MUST be rejected at WRITE");
        assert!(
            matches!(err, EvalError::Invariant(InvariantViolation::SystemZone)),
            "label {label:?} must route through E_INV_SYSTEM_ZONE, got: {err:?}"
        );
    }
}
