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
#[test]
#[ignore = "phase-2a-pending: Inv-11 runtime probe via get_node_label_only(cid) lands in G5-B-i per plan §9.10 + code-as-graph Major #1 correction. Drop #[ignore] once the resolved-label probe is wired."]
fn inv_11_transform_constructed_cid_with_system_label_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    // Seed a system-zone grant (engine-privileged path).
    let alice = engine.create_principal("alice").unwrap();
    let grant_cid = engine.grant_capability(&alice, "store:post:write").unwrap();

    // Sanity: the CID string form begins with `bafy...` (CIDv1 base32-
    // lower), NOT with `system:` — so a naive value-text probe passes.
    let cid_str = grant_cid.to_string();
    assert!(
        !cid_str.starts_with("system:"),
        "CID string is not textually related to label; Inv-11 probe MUST \
         resolve the Node's label, not probe CID text. cid={cid_str}"
    );

    // Target API path (G5-B-i):
    //
    //     let sg = SubgraphSpec::builder()
    //         .handler_id("attack:transform-cid-to-system")
    //         .transform(|t| {
    //             // Expression that produces the constant system-zone CID.
    //             // In the actual fixture, this would be
    //             //   get_by_label("system:CapabilityGrant")[0]
    //             // or a hardcoded Cid::from_str input.
    //             t.eval("get_by_label('system:CapabilityGrant')[0]")
    //         })
    //         .read(|r| r.by_cid_from_transform("t0"))
    //         .respond()
    //         .build();
    //     let handler_id = engine.register_subgraph(sg).unwrap();
    //
    //     let outcome = engine
    //         .call(&handler_id, "attack", Node::empty())
    //         .unwrap();
    //     assert_eq!(
    //         outcome.error_code(),
    //         Some("E_INV_SYSTEM_ZONE"),
    //         "Inv-11 runtime probe MUST deny TRANSFORM-computed CIDs \
    //          whose resolved Node has a system:* label; got {:?}",
    //         outcome.error_code()
    //     );
    //
    // Until G5-B-i lands the probe + the TRANSFORM DSL path to this attack
    // shape, the test stays red via panic — the attack path's DSL shape
    // isn't expressible today.
    let _ = grant_cid;
    let _ = Node::empty();

    panic!(
        "red-phase: Inv-11 runtime probe via get_node_label_only(cid) not \
         yet present. G5-B-i to land per plan §9.10 + code-as-graph \
         Major #1 correction."
    );
}

// ---------------------------------------------------------------------------
// R4 cov-9: proptest placeholder — `prop_inv_11_prefix_probe_catches_
// constructed_cids`. Ignored until G5-B-i lands the probe; until then
// the property-testing strategy would panic on the first generated CID
// (the probe is `todo!()`) and tell us nothing about the Inv-11 shape.
// ---------------------------------------------------------------------------

use proptest::prelude::*;

proptest! {
    #[ignore = "phase-2a-pending: requires G5-B-i get_node_label_only-based probe \
                to be non-todo!() for the property to fire meaningfully"]
    #[test]
    fn prop_inv_11_prefix_probe_catches_constructed_cids(seed in 0u64..128) {
        // Shape the proptest will assume once G5-B-i lands:
        //   1. Seed `N` privileged fixture Nodes; half in the system zone,
        //      half in the user zone.
        //   2. For each fixture CID, dispatch a TRANSFORM-constructed READ
        //      through the engine.
        //   3. Assert: CIDs whose resolved Node carries a `system:` label
        //      are ALL rejected with `E_INV_SYSTEM_ZONE`; the others pass.
        // The property shrinks toward a minimal counterexample where a
        // system-zone CID slips through the probe, which is the Inv-11
        // failure signature.
        let _ = seed;
        prop_assert!(true, "placeholder — see `#[ignore]` message");
    }
}
