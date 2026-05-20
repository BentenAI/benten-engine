//! TF-1 — #989 storage-partition isolation + cross-DID non-leak (CANARY).
//!
//! ADDL Phase-4-Meta-Core, Wave R3-A, agent R3-A1, family TF-1.
//! Maps to: `r2-test-landscape.md` §TF-1 + §7 seed row S1 + the C1 exit
//! obligation (C1 cross-DID non-leak — promoted from R2-seed-only to a
//! G-CORE-1 **exit obligation** per multitenant-confidentiality-r1-2 /
//! r1-triage row 34 / §6-C1-edit; §3.6b sub-rule-4).
//!
//! ============================================================================
//! RED-PHASE — un-ignore at G-CORE-1 (pim-12 / §3.6e).
//! ============================================================================
//! Every `#[test]` here is `#[ignore]`-staged with the literal marker
//! `RED-PHASE: un-ignore at G-CORE-1`. They are written against the REAL
//! production `RedbBackend` per-DID-scoped surface (`WriteContext::namespace_did`
//! + `RedbBackend::scoped(namespace_did)` per-DID view), NOT a test double.
//! At origin/main `ed03729a` `WriteContext` has NO `namespace_did` field and
//! there is no `RedbBackend::scoped(..)` view — so these tests **do not
//! compile / would-FAIL** until G-CORE-1 lands #989. The whole module is
//! `#[cfg(feature = "tf1_989_red_phase")]`-gated so the workspace keeps
//! compiling pre-implementation (the gate is removed + the `#[ignore]`s
//! lifted at G-CORE-1 — that is the un-ignore action the closing-wave sweep
//! and the mini-reviewer verify per §3.6e: landing-status, not just
//! spec-pin presence).
//!
//! ----------------------------------------------------------------------------
//! §3-directive inherited-discipline pre-flight (this file ticks every line):
//!  - §3.6b + sub-rule 4: each pin is a PRODUCTION-ARM (real `RedbBackend`
//!    scoped view + `put_node_with_context`/`put_edge_with_context`/scan/
//!    iterate/subscriber fan-out) + OBSERVABLE-CONSEQUENCE (a Y-scoped read
//!    path returns NONE / NO X key) + WOULD-FAIL-IF-NO-OP'd (a no-op #989
//!    that ignores `namespace_did` leaks X→Y and trips every assertion).
//!    Each pin targets the SPECIFIC arm (point read / range-scan / iterate /
//!    change-subscriber fan-out / system-zone collision / Inv-13 5-row),
//!    not an umbrella "partition works".
//!  - §3.6f (pim-18) SHAPE-not-SUBSTANCE: every pin enumerates a real
//!    production call site and asserts an observable consequence — NONE is
//!    "assert a ScopedBackend type is constructible". The §4-A guard for
//!    TF-1's sibling family (TF-5) is the canonical anti-pattern; this file
//!    deliberately exercises the read PATH, not type existence.
//!  - §3.13 per-test static decomposition: this file introduces NO
//!    process-scoped shared static. Every test owns a fresh `tempdir()` +
//!    fresh `RedbBackend`; the change-subscriber pin uses a per-test
//!    `Arc<Mutex<Vec<..>>>` capture (semantic local name), never a single
//!    shared static under the parallel runner.
//!  - §3.6e (pim-12): `#[ignore]` + literal `RED-PHASE: un-ignore at
//!    G-CORE-1` marker on every test; the cfg-gate is the compile shield.
//!  - §3.5g: no TS surface in this file (the namespace_did public-shape TS
//!    mirror, if any, rides the G-CORE-1 implementer brief, not the test).
//!
//! TYPE NOTE (multitenant-r1-1, plan G-CORE-1): `namespace_did` is typed
//! `Option<Cid>` (NOT `Option<Did>` — #989 body + §4.74 are authoritative;
//! `benten-graph` must NOT gain a `benten-id` dep per arch-r1-10). These
//! tests therefore key the partition by a `Cid` value, never a `Did`.
//!
//! G-CORE-1 LANDING NOTE: this file is no longer cfg-gated nor `#[ignore]`-
//! staged — the `WriteContext::namespace_did` + `RedbBackend::scoped`
//! surface lands in the same wave that un-ignores it (per §3.6e: the
//! mini-reviewer verifies landing-status, not just spec-pin presence).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    // G-CORE-1 #989: `Cid: Copy`; the explicit `.clone()` calls below were
    // written when the RED-phase test was authored against the inferred
    // `WriteContext::namespace_did: Option<Cid>` shape, which gives the
    // pin's intent (per-DID namespace identity, deliberately re-bound for
    // the OWNING vs ADVERSARY partitions) a more readable surface than the
    // implicit `Copy`. Allow to keep the test's authored shape.
    clippy::clone_on_copy
)]

extern crate alloc;
use alloc::collections::BTreeMap;

use benten_core::{Cid, Edge, Node, Value};
use benten_graph::{
    ChangeEvent, ChangeKind, ChangeSubscriber, RedbBackend, WriteAuthority, WriteContext,
};
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

// ---------------------------------------------------------------------------
// Local helpers — no shared static (§3.13). Each call yields fresh state.
// ---------------------------------------------------------------------------

/// A node distinguishable per partition so a leak is observable by content.
fn node_titled(title: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".to_string(), Value::text(title));
    Node::new(vec!["Doc".to_string()], props)
}

/// Synthesize a stable, distinct `Cid` to stand in for a per-DID namespace
/// key. Uses a content node's CID so the value is a real `Cid` (the
/// `Option<Cid>` namespace_did type), deterministic and X≠Y.
fn namespace_cid(seed: &str) -> Cid {
    let mut props = BTreeMap::new();
    props.insert("did-seed".to_string(), Value::text(seed));
    Node::new(vec!["system:Principal".to_string()], props)
        .cid()
        .expect("namespace seed node must hash")
}

fn fresh_backend() -> (tempfile::TempDir, RedbBackend) {
    let dir = tempdir().unwrap();
    let backend = RedbBackend::create(dir.path().join("tf1.redb")).expect("open redb");
    (dir, backend)
}

/// RED-PHASE production surface contract (lands at G-CORE-1):
///   - `WriteContext::with_namespace_did(Option<Cid>) -> Self` builder
///   - `WriteContext::namespace_did(&self) -> Option<&Cid>` accessor
///   - `RedbBackend::scoped(namespace_did: Cid) -> ScopedView` per-DID view
///     whose `get_node` / `get_by_label` / `scan` / iterate / registered
///     subscriber fan-out are confined to that DID's keyspace.
/// These symbols do not exist at `ed03729a` — the file is cfg-gated so the
/// workspace still compiles; the gate + `#[ignore]` lift together at
/// G-CORE-1.
fn ctx_for(namespace: &Cid) -> WriteContext {
    WriteContext::new("Doc").with_namespace_did(Some(namespace.clone()))
}

// ---------------------------------------------------------------------------
// PIN 1 — point read does not leak X → Y (the core C1 confidentiality arm).
// Production arm: `RedbBackend::put_node_with_context` under namespace_did=X,
// then `RedbBackend::scoped(Y).get_node(cid)`. Observable: Y view returns
// `Ok(None)`. Would-FAIL if scoping is a no-op (the unscoped get returns the
// X node to the Y view).
// ---------------------------------------------------------------------------
#[test]
fn point_read_does_not_leak_across_did_partitions() {
    let (_dir, backend) = fresh_backend();
    let did_x = namespace_cid("did:key:zX");
    let did_y = namespace_cid("did:key:zY");
    assert_ne!(did_x, did_y, "test setup: the two namespaces must differ");

    let secret = node_titled("X-private-secret");
    let cid = backend
        .put_node_with_context(&secret, &ctx_for(&did_x))
        .expect("write under namespace_did=X must succeed");

    // The OWNING partition can read it (no false-positive isolation).
    let from_x = backend.scoped(did_x.clone()).get_node(&cid).unwrap();
    assert_eq!(
        from_x.as_ref(),
        Some(&secret),
        "the owning DID-X view must still see its own node"
    );

    // The ADVERSARY partition must NOT — point read returns clean miss.
    let from_y = backend.scoped(did_y).get_node(&cid).unwrap();
    assert_eq!(
        from_y, None,
        "cross-DID confidentiality-by-isolation (C1 exit obligation): a \
         node written under namespace_did=X MUST be invisible to a \
         namespace_did=Y point read. A leak here means #989 prefix \
         scoping is a no-op."
    );
}

// ---------------------------------------------------------------------------
// PIN 2 — label range-scan does not leak X → Y.
// Production arm: `RedbBackend::scoped(Y).get_by_label("Doc")` (the
// label-index range scan). Observable: the X-written CID is absent from the
// Y-scoped scan result. Would-FAIL if the label index is not partitioned.
// ---------------------------------------------------------------------------
#[test]
fn label_range_scan_does_not_leak_across_did_partitions() {
    let (_dir, backend) = fresh_backend();
    let did_x = namespace_cid("did:key:zX");
    let did_y = namespace_cid("did:key:zY");

    let x_node = node_titled("X-doc");
    let y_node = node_titled("Y-doc");
    let x_cid = backend
        .put_node_with_context(&x_node, &ctx_for(&did_x))
        .unwrap();
    let y_cid = backend
        .put_node_with_context(&y_node, &ctx_for(&did_y))
        .unwrap();

    let y_hits = backend.scoped(did_y.clone()).get_by_label("Doc").unwrap();
    assert!(
        y_hits.contains(&y_cid),
        "the Y partition must see its own label-indexed node"
    );
    assert!(
        !y_hits.contains(&x_cid),
        "cross-DID range-scan leak (C1 exit obligation): the X-written \
         CID MUST NOT appear in a namespace_did=Y `get_by_label` scan. A \
         hit here means the LABEL_INDEX_TABLE is shared across partitions."
    );

    // Symmetric: the X partition must not see Y's label-indexed node.
    let x_hits = backend.scoped(did_x).get_by_label("Doc").unwrap();
    assert!(x_hits.contains(&x_cid));
    assert!(
        !x_hits.contains(&y_cid),
        "cross-DID range-scan leak (symmetric direction)"
    );
}

// ---------------------------------------------------------------------------
// PIN 3 — raw prefix iterate (`scan`) does not leak X → Y.
// Production arm: the lowest-level `RedbBackend::scoped(Y)` key iterate over
// the node keyspace (the `n:` prefix). Observable: no key carrying the
// X-written node body is yielded to the Y-scoped iterate. Would-FAIL if the
// `n:` keyspace is not DID-prefixed (iterate is the path index-scoping
// cannot mask).
// ---------------------------------------------------------------------------
#[test]
fn raw_keyspace_iterate_does_not_leak_across_did_partitions() {
    let (_dir, backend) = fresh_backend();
    let did_x = namespace_cid("did:key:zX");
    let did_y = namespace_cid("did:key:zY");

    let secret = node_titled("X-iterate-secret");
    let x_cid = backend
        .put_node_with_context(&secret, &ctx_for(&did_x))
        .unwrap();

    // Adversary iterates its OWN partition exhaustively and tries to
    // reach the X node by CID through the iterate-yielded key set.
    let y_view = backend.scoped(did_y);
    let y_keys = y_view.iter_node_cids().expect("scoped node iterate");
    assert!(
        !y_keys.contains(&x_cid),
        "cross-DID iterate leak (C1 exit obligation): exhaustively \
         iterating the namespace_did=Y node keyspace MUST NOT surface \
         the X-written CID. Iterate is the path index-scoping alone \
         cannot mask — would-FAIL if `n:` keys are not DID-prefixed."
    );

    // And a Y-scoped point read of that CID is also a clean miss
    // (iterate + read are independently confined).
    assert_eq!(y_view.get_node(&x_cid).unwrap(), None);
}

// ---------------------------------------------------------------------------
// PIN 4 — change-subscriber fan-out does not leak X → Y.
// Production arm: register a subscriber scoped to namespace_did=Y, then
// `put_node_with_context` under namespace_did=X through the real
// post-commit fan-out. Observable: the Y-scoped subscriber receives NO
// ChangeEvent for the X write. Would-FAIL if fan-out broadcasts every
// commit to every subscriber regardless of partition (the subtlest leak —
// CDC/IVM consumers would observe another tenant's writes).
//
// §3.13: the capture buffer is a per-test `Arc<Mutex<Vec<..>>>` with a
// semantic local name, NOT a process-scoped shared static.
// ---------------------------------------------------------------------------
struct PartitionScopedRecorder {
    seen: Arc<Mutex<Vec<ChangeEvent>>>,
}
impl ChangeSubscriber for PartitionScopedRecorder {
    fn on_change(&self, event: &ChangeEvent) {
        self.seen.lock().unwrap().push(event.clone());
    }
}

#[test]
fn change_subscriber_fan_out_does_not_leak_across_did_partitions() {
    let (_dir, backend) = fresh_backend();
    let did_x = namespace_cid("did:key:zX");
    let did_y = namespace_cid("did:key:zY");

    let y_events: Arc<Mutex<Vec<ChangeEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let recorder = Arc::new(PartitionScopedRecorder {
        seen: y_events.clone(),
    });

    // RED-PHASE contract: subscribers register against a DID-scoped view so
    // fan-out is partition-confined.
    backend
        .scoped(did_y.clone())
        .register_subscriber(recorder.clone())
        .expect("register Y-scoped subscriber");

    // A write under namespace_did=X goes through the real post-commit
    // fan-out path (`put_node_with_context`).
    let x_cid = backend
        .put_node_with_context(&node_titled("X-fanout-secret"), &ctx_for(&did_x))
        .unwrap();

    let observed = y_events.lock().unwrap();
    assert!(
        !observed.iter().any(|e| e.cid == x_cid),
        "cross-DID change-subscriber fan-out leak (C1 exit obligation): a \
         subscriber scoped to namespace_did=Y MUST NOT receive a \
         ChangeEvent for a namespace_did=X write. A leak here means the \
         post-commit fan-out broadcasts every commit to every subscriber \
         regardless of partition — CDC/IVM consumers would observe \
         another tenant's writes."
    );

    // Positive control: a Y-partition write DOES reach the Y subscriber
    // (proves the subscriber is wired, so the negative above is real
    // isolation, not a dead subscriber).
    drop(observed);
    let y_cid = backend
        .put_node_with_context(&node_titled("Y-fanout-own"), &ctx_for(&did_y))
        .unwrap();
    assert!(
        y_events.lock().unwrap().iter().any(|e| e.cid == y_cid),
        "the Y-scoped subscriber MUST receive its own partition's write \
         (else the negative assertion above is vacuous)"
    );
}

// ---------------------------------------------------------------------------
// PIN 5 — edge writes are partitioned too (the edge keyspace + es:/et:
// indexes). Production arm: `put_edge_with_context` under namespace_did=X,
// then a Y-scoped `edges_from`/`get_edge`. Observable: Y view sees no X
// edge. Would-FAIL if only the `n:` keyspace is partitioned but `e:`/`es:`/
// `et:` are not (a partial-isolation regression).
// ---------------------------------------------------------------------------
#[test]
fn edge_keyspace_does_not_leak_across_did_partitions() {
    let (_dir, backend) = fresh_backend();
    let did_x = namespace_cid("did:key:zX");
    let did_y = namespace_cid("did:key:zY");

    let src = node_titled("edge-src");
    let tgt = node_titled("edge-tgt");
    let src_cid = backend
        .put_node_with_context(&src, &ctx_for(&did_x))
        .unwrap();
    let tgt_cid = backend
        .put_node_with_context(&tgt, &ctx_for(&did_x))
        .unwrap();
    let edge = Edge::new(src_cid.clone(), tgt_cid.clone(), "LINKS", None);
    let edge_cid = backend
        .put_edge_with_context(&edge, &ctx_for(&did_x))
        .unwrap();

    let y_view = backend.scoped(did_y);
    assert_eq!(
        y_view.get_edge(&edge_cid).unwrap(),
        None,
        "cross-DID edge leak (C1 exit obligation): an edge written under \
         namespace_did=X MUST be invisible to a namespace_did=Y view — \
         the `e:` keyspace must be DID-partitioned, not just `n:`."
    );
    assert!(
        y_view.edges_from(&src_cid).unwrap().is_empty(),
        "cross-DID edge-source-index leak: the `es:` index must be \
         DID-partitioned (would-FAIL if only `n:` is scoped)."
    );
}

// ---------------------------------------------------------------------------
// PIN 6 — prefix-collision vs the system-zone keyspace + SC1 ban preserved.
// A namespaced write must NOT be able to forge / collide into the reserved
// system-zone label space, and the SC1 unprivileged `system:`-label ban
// still fires under a namespaced WriteContext. Production arm:
// `put_node_with_context` of a `system:`-labelled node with namespace_did
// set + is_privileged=false. Observable: `GraphError::SystemZoneWrite`
// still returned (the DID-prefix must not become a privilege side-channel).
// Would-FAIL if #989's prefix rendering swallows / bypasses the SC1 guard.
// ---------------------------------------------------------------------------
#[test]
fn namespaced_write_does_not_bypass_sc1_system_zone_ban() {
    let (_dir, backend) = fresh_backend();
    let did_x = namespace_cid("did:key:zX");

    let mut props = BTreeMap::new();
    props.insert("k".to_string(), Value::text("v"));
    let system_node = Node::new(vec!["system:Capability".to_string()], props);

    // Unprivileged + namespaced: SC1 must STILL reject the system-zone
    // label. The DID prefix must not be a privilege side-channel.
    let ctx = WriteContext::new("system:Capability").with_namespace_did(Some(did_x.clone()));
    let res = backend.put_node_with_context(&system_node, &ctx);
    assert!(
        matches!(res, Err(benten_graph::GraphError::SystemZoneWrite { .. })),
        "SC1 system-zone ban MUST still fire for an unprivileged \
         namespaced write — #989's key-prefix rendering must not collide \
         with / bypass the reserved `system:` zone (prefix-collision-vs-\
         system-zone-keyspace check; C1/SC1 coupled exit obligation)."
    );
}

// ---------------------------------------------------------------------------
// PIN 7 — Inv-13 5-row WriteAuthority firing matrix unaffected for
// NON-namespaced writes. #989 must be additive: a default (no namespace_did)
// `User`-authority re-put of an already-stored CID still returns
// `E_INV_IMMUTABILITY` (Row 1), and an `EnginePrivileged` re-put still
// dedups (Row 3). Production arm: real `put_node_with_context` with a
// default WriteContext (namespace_did = None). Observable: identical
// pre-#989 error/dedup behaviour. Would-FAIL if #989 perturbs the
// non-namespaced path's Inv-13 dispatch.
// ---------------------------------------------------------------------------
#[test]
fn inv13_matrix_unaffected_for_non_namespaced_writes() {
    let (_dir, backend) = fresh_backend();
    let node = node_titled("inv13-row1");

    // Default ctx => namespace_did == None (the pre-#989 behaviour path).
    let ctx_user = WriteContext::default();
    assert_eq!(
        ctx_user.namespace_did(),
        None,
        "a default WriteContext MUST carry namespace_did = None — #989 is \
         additive; the un-namespaced path is the legacy path unchanged"
    );

    let cid = backend.put_node_with_context(&node, &ctx_user).unwrap();

    // Row 1: User + present → E_INV_IMMUTABILITY (unchanged by #989).
    let reput = backend.put_node_with_context(&node, &ctx_user);
    assert!(
        matches!(reput, Err(benten_graph::GraphError::InvImmutability { .. })),
        "Inv-13 Row 1 (User + present → E_INV_IMMUTABILITY) MUST be \
         unaffected for a non-namespaced write — #989 must be additive."
    );

    // Row 3: EnginePrivileged + present → dedup (Ok, same CID, no error).
    let ctx_priv = WriteContext::default().with_authority(WriteAuthority::EnginePrivileged);
    let dedup = backend
        .put_node_with_context(&node, &ctx_priv)
        .expect("EnginePrivileged re-put dedups (Row 3) — must not error");
    assert_eq!(
        dedup, cid,
        "Inv-13 Row 3 (EnginePrivileged + present → dedup, same CID) MUST \
         be unaffected for a non-namespaced write."
    );
}

// ---------------------------------------------------------------------------
// PIN 8 — `namespace_did` round-trips on `WriteContext` (the §1.A.FROZEN
// item-5 locked canary shape). The field is `Option<Cid>` (NOT `Did`) and
// survives a builder round-trip. This is the public-shape canary pin: the
// C1 canary output `WriteContext` shape is the FROZEN shape. Would-FAIL if
// the field is the wrong type or the builder drops it.
// ---------------------------------------------------------------------------
#[test]
fn write_context_namespace_did_round_trips_as_option_cid() {
    let did_x = namespace_cid("did:key:zX");

    let ctx = WriteContext::new("Doc").with_namespace_did(Some(did_x.clone()));
    // Accessor returns Option<&Cid> — the authoritative #989 type
    // (multitenant-r1-1: Option<Cid>, NOT Option<Did>; benten-graph must
    // not depend on benten-id).
    let got: Option<&Cid> = ctx.namespace_did();
    assert_eq!(
        got,
        Some(&did_x),
        "WriteContext::namespace_did must round-trip the Option<Cid> set \
         via the builder (§1.A.FROZEN item 5: the C1 canary WriteContext \
         shape is the FROZEN public shape)."
    );

    // Clearing it yields None (the default / legacy path).
    let cleared = WriteContext::new("Doc").with_namespace_did(None);
    assert_eq!(cleared.namespace_did(), None);
}

// Compile-time assertion that ChangeKind is in scope (keeps the import
// honest for the fan-out pin's event-shape expectations; cheap and makes a
// later ChangeKind variant rename a visible break here).
#[allow(dead_code)]
const _: fn() = || {
    let _ = ChangeKind::Created;
};
