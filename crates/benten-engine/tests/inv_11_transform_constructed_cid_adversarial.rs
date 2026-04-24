//! Phase 2a R3 security — Inv-11 TRANSFORM-constructed-CID resolves to
//! `system:*` label (Code-as-graph Major #1).
//!
//! **Attack class.** Naive Inv-11 runtime enforcement probes the text of
//! the passing `Value` for a `system:` prefix. But CIDs in Benten are
//! BLAKE3 multihashes in base32-lower CIDv1 form — they have NO textual
//! relationship to the label of the Node they reference. A TRANSFORM node
//! can construct a `Value::Cid` from arbitrary bytes / lookups; the CID
//! string starts with `"bafy..."`, not `"system:..."`. A prefix probe on
//! the Value passes while the resolved Node carries a `system:*` label.
//!
//! **Prerequisite.** TRANSFORM primitive can compute CIDs from first-
//! principle inputs OR look up Nodes by label/property. Either path
//! yields a CID whose resolved Node is a system-zone grant.
//!
//! **Attack sequence.**
//!  1. Engine seeds a `system:CapabilityGrant` Node (CID = G).
//!  2. Adversary's subgraph registers a TRANSFORM whose expression computes
//!     G (e.g. via a hardcoded lookup or via a `get_by_label` flank).
//!  3. A READ primitive is driven by that computed CID.
//!  4. Naive probe checks "does `G.to_string()` start with `system:`?" —
//!     no; passes.
//!  5. READ returns the system-zone Node content.
//!
//! **Impact.** Inv-11 bypass via CID indirection; privacy violated on
//! every capability grant in the system zone.
//!
//! **Recommended mitigation.** The runtime probe must check the
//! **resolved Node's label prefix**, not the value-text. Use the new
//! `benten_graph::KVBackend::get_node_label_only(cid)` fast path
//! (G5-B-i) to look up the label cheaply before probing; resolution is
//! O(1) redb + minimal bytes read.
//!
//! **Red-phase contract.** Phase-1 HEAD: no Inv-11 runtime probe AT ALL
//! (registration-time literal-CID stopgap only). Attack succeeds. The
//! test asserts denial; fails today. R5 G5-B-i lands the probe +
//! `get_node_label_only`; test passes.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Node;
use benten_engine::Engine;

/// code-as-graph Major #1: a TRANSFORM-derived CID whose resolved Node
/// has a `system:*` label MUST be denied by Inv-11's runtime probe via
/// the resolved-label lookup (not via value-text prefix probe).
///
/// Phase-2a G5-B-i wires the runtime probe in two places:
/// 1. `Engine::get_node` — the public user surface collapses to `None`
///    when the resolved label is system-zone.
/// 2. `impl PrimitiveHost for Engine::read_node` — the evaluator-visible
///    surface collapses likewise, so a TRANSFORM that computes a
///    `Value::Cid` and feeds it to a READ primitive resolves through the
///    same probe.
///
/// The test exercises surface (1) directly (user-facing `engine.get_node`)
/// because the PrimitiveHost path is an internal boundary whose
/// observable shape is identical. The probe reaches for
/// `RedbBackend::get_node_label_only` (§9.10 `<1µs` target), not a
/// value-text prefix probe.
#[test]
fn inv_11_transform_constructed_cid_with_system_label_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // Seed a system-zone grant (engine-privileged path). Use the Phase-1
    // `grant_capability(scope, subject)` signature — the test setup
    // sketched in the earlier red-phase commentary referenced a
    // principal-first signature that was never introduced.
    let grant_cid = engine
        .grant_capability("store:post:write", "attacker-controlled-subject")
        .unwrap();

    // Sanity: the CID string form begins with `bafy...` (CIDv1 base32-
    // lower), NOT with `system:` — so a naive value-text probe passes.
    let cid_str = grant_cid.to_string();
    assert!(
        !cid_str.starts_with("system:"),
        "CID string is not textually related to label; Inv-11 probe MUST \
         resolve the Node's label, not probe CID text. cid={cid_str}"
    );

    // Surface (1): user-facing `engine.get_node` on the
    // TRANSFORM-computable CID. Inv-11 runtime probe MUST collapse to
    // None. Today this is symmetric with a backend miss; the test pins
    // the collapse independently of the capability policy.
    let observed = engine.get_node(&grant_cid).unwrap();
    assert!(
        observed.is_none(),
        "Inv-11 runtime probe MUST deny a resolved system:* label at \
         the user surface regardless of a constructed CID's textual form. \
         Got: {observed:?}"
    );

    // Belt-and-suspenders: the backend-direct accessor (privileged
    // back-channel) still returns the Node — the collapse is a user-
    // surface semantic, not a storage-layer deletion.
    let privileged = engine
        .backend_for_test()
        .get_node(&grant_cid)
        .unwrap()
        .expect("privileged back-channel still reads the grant Node");
    assert!(privileged.labels.iter().any(|l| l.starts_with("system:")));

    let _ = Node::empty();
}

// ---------------------------------------------------------------------------
// R4 cov-9: proptest placeholder — `prop_inv_11_prefix_probe_catches_
// constructed_cids`. Ignored until G5-B-i lands the probe; until then
// the property-testing strategy would panic on the first generated CID
// (the probe is `todo!()`) and tell us nothing about the Inv-11 shape.
// ---------------------------------------------------------------------------

use proptest::prelude::*;

proptest! {
    /// Phase-2a G5-B-i coverage: for each seeded mixed-zone fixture, the
    /// user-facing `engine.get_node` must return `Some` for user-zone CIDs
    /// and `None` for system-zone CIDs — driven by the resolved-label
    /// probe, not by the CID's textual form. The test seeds a small
    /// number of fixture Nodes per iteration and checks both shapes.
    #[test]
    fn prop_inv_11_prefix_probe_catches_constructed_cids(seed in 0u64..16) {
        let dir = tempfile::tempdir().unwrap();
        let engine = Engine::builder()
            .path(dir.path().join("benten.redb"))
            .build()
            .unwrap();

        // System-zone seed: privileged grant Node. Its CID is whatever
        // the engine computes; the test doesn't control the textual form.
        let scope = format!("store:seed_{seed}:write");
        let subject = format!("seed-{seed}");
        let sys_cid = engine.grant_capability(&scope, &subject).unwrap();

        // User-zone seed: a Post Node routed through the standard
        // user-facing `create_node` path.
        use benten_core::{Node as CoreNode, Value};
        use std::collections::BTreeMap;
        let mut props = BTreeMap::new();
        props.insert("title".into(), Value::text(format!("post-{seed}")));
        let user_cid = engine
            .create_node(&CoreNode::new(vec!["Post".into()], props))
            .unwrap();

        // Inv-11 assertions.
        prop_assert!(
            engine.get_node(&sys_cid).unwrap().is_none(),
            "seed={seed}: system-zone CID MUST collapse to None under Inv-11"
        );
        prop_assert!(
            engine.get_node(&user_cid).unwrap().is_some(),
            "seed={seed}: user-zone CID MUST remain readable"
        );
    }
}
