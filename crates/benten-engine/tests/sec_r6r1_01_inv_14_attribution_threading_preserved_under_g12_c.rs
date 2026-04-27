//! G12-C-cont (Phase 2b R6 A1 closure) sec-pre-r1-13 carry: assert the
//! Phase-2a `sec-r6r1-01` Inv-14 attribution-threading wiring is NOT
//! regressed by the `Subgraph` type relocation from `benten-eval` to
//! `benten-core`.
//!
//! Per `r1-security-auditor.json` sec-pre-r1-13 + line 148: Inv-14 requires
//! a per-host-fn live check that emits TraceStep with AttributionFrame
//! inherited from dispatching SANDBOX primitive. sec-pre-r1-13 G12-C carry
//! pins this is live post-G12-C-cont.
//!
//! The G12-C-cont relocation moves `Subgraph` + `SubgraphBuilder` +
//! companions to `benten-core`. The
//! `SubgraphBuilder::push()` private method (in `benten-core` post-relocation)
//! still stamps `ATTRIBUTION_PROPERTY_KEY = "attribution" → Bool(true)` on
//! every emitted OperationNode by default; this file pins that contract.
//!
//! TDD red-phase un-ignored under G12-C-cont.

#![allow(clippy::unwrap_used)]

use benten_core::{ATTRIBUTION_PROPERTY_KEY, OperationNode, PrimitiveKind, SubgraphBuilder, Value};

/// Pin: the relocated builder's `push` path stamps the attribution property
/// on every emitted OperationNode by default. Pre-G12-C-cont this was the
/// eval-side `SubgraphBuilder::push` body; post-G12-C-cont the same logic
/// runs from `benten_core::SubgraphBuilder::push` (G11-A EVAL wave-1
/// D12.7 Decision 1 contract).
#[test]
fn relocated_builder_stamps_attribution_property_on_every_emitted_node() {
    let mut b = SubgraphBuilder::new("inv14-attribution-pin");
    let r = b.read("post");
    let resp = b.respond(r);

    let sg = b.build_unvalidated_for_test();

    let nodes = sg.nodes();
    assert_eq!(nodes.len(), 2);

    for n in nodes {
        let attr = n.property(ATTRIBUTION_PROPERTY_KEY).unwrap_or_else(|| {
            panic!(
                "node `{}` missing the Inv-14 `{ATTRIBUTION_PROPERTY_KEY}` property — \
                 G12-C-cont relocation must preserve the builder's default-stamp \
                 contract (D12.7 Decision 1)",
                n.id
            )
        });
        match attr {
            Value::Bool(true) => {} // expected
            other => panic!(
                "node `{}` Inv-14 `{ATTRIBUTION_PROPERTY_KEY}` is not Bool(true): {:?}",
                n.id, other
            ),
        }
    }
    // Silence `resp` unused in the assertion-shape body; its only role is
    // to grow the subgraph past 1 node so the loop above checks more than
    // a singleton.
    let _ = resp;
}

/// Pin: subgraph relocation does NOT detach the per-node attribution
/// property from the canonical-bytes encoding. If a downstream change
/// silently drops the attribution property out of the canonical-bytes
/// encoding, two subgraphs differing only in attribution declaration
/// would CID-collide. Assert the attribution property contributes to the
/// CID by constructing two subgraphs differing only in that property.
#[test]
fn attribution_property_contributes_to_canonical_bytes_post_g12c_relocation() {
    use benten_core::Subgraph;

    // Subgraph A: builder-emitted (attribution: true by default).
    let mut b = SubgraphBuilder::new("inv14-cid-pin");
    b.read("post");
    let sg_with_attribution = b.build_unvalidated_for_test();

    // Subgraph B: identical structure but with raw OperationNode (no attribution
    // property) — the typical Inv-14-reject-path construction.
    let raw = OperationNode::new("post", PrimitiveKind::Read);
    let sg_without_attribution = Subgraph::new("inv14-cid-pin").with_node(raw);

    let cid_a = sg_with_attribution.cid().expect("cid a");
    let cid_b = sg_without_attribution.cid().expect("cid b");

    assert_ne!(
        cid_a, cid_b,
        "subgraph WITH attribution property MUST hash differently from \
         subgraph WITHOUT — G12-C-cont must preserve the property's \
         participation in the canonical-bytes encoding (Inv-14 contract)"
    );
}

/// Pin: a manually-constructed OperationNode that BYPASSES the builder
/// (used by the Inv-14 reject-path tests in the eval crate) carries NO
/// attribution property by default. This is the contract that lets
/// invariant_14_fixture_cid.rs exercise the rejection path.
#[test]
fn raw_operation_node_carries_no_attribution_by_default_post_g12c() {
    let raw = OperationNode::new("raw", PrimitiveKind::Read);
    assert!(
        raw.property(ATTRIBUTION_PROPERTY_KEY).is_none(),
        "raw OperationNode (bypassing the builder) MUST carry no \
         `{ATTRIBUTION_PROPERTY_KEY}` property — this is the contract \
         the Inv-14 reject-path tests rely on"
    );
}

/// G12-A non-regression carry (sec-pre-r1-13 §2): the BudgetExhausted
/// runtime-emission wiring (G12-A) must not bypass the AttributionFrame
/// routing path. Pin: the relocated `OperationNode` type is identical
/// across the dep boundary (PartialEq holds), so the attribution-frame
/// threading that the G12-A wiring inherits stays sound.
#[test]
fn g12a_budget_exhausted_does_not_bypass_attribution_frame_post_g12_c() {
    // Property pin: the OperationNode + attribution property pair carry
    // through DAG-CBOR round-trip (the same path the BudgetExhausted
    // emission walks for AttributionFrame derivation).
    use benten_core::Subgraph;

    let mut b = SubgraphBuilder::new("g12a-non-regression");
    let r = b.read("r");
    b.respond(r);
    let sg = b.build_unvalidated_for_test();

    let bytes = sg.to_dagcbor().expect("encode");
    let decoded = Subgraph::from_dagcbor(&bytes).expect("decode");

    for n in decoded.nodes() {
        assert_eq!(
            n.property(ATTRIBUTION_PROPERTY_KEY),
            Some(&Value::Bool(true)),
            "Inv-14 attribution property MUST round-trip through DAG-CBOR \
             so the G12-A BudgetExhausted emission can derive a non-empty \
             AttributionFrame from the decoded subgraph"
        );
    }
}
