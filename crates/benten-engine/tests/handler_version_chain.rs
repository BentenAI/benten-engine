//! R3-B RED-PHASE pins: handler-version chain durable via Anchor Node
//! (G14-C wave-4b; plan §3 G14-C + arch-r1-4 BLOCKER + D-C).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-C):
//!
//! - `tests/handler_version_chain_durable_via_anchor_node` — plan §3 G14-C (integration)
//! - `tests/handler_version_chain_advances_on_register_subgraph_replace` — plan §3 G14-C (integration)
//! - `tests/canonical_bytes_handler_version_chain_extensible_for_future_attribution_variants` — arch-r1-4 BLOCKER + D-C (architectural-pin)
//!
//! ## Architectural intent
//!
//! Compromise #18 (handler-version chain durability) closes at G14-C
//! using the Phase-1-shipped Anchor + Version + CURRENT pointer
//! pattern. Each handler subgraph registration produces a Version
//! Node anchored to a per-handler Anchor; CURRENT pointer advances on
//! re-registration; old versions remain queryable.
//!
//! Per arch-r1-4 BLOCKER + D-C, the canonical-bytes encoding for the
//! handler-version chain MUST be EXTENSIBLE — additive variant slots
//! for future attribution variants land without breaking on-disk
//! bytes. Composes with §3.B CLR-1 cluster (Loro vs canonical-bytes
//! reconciliation).
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-C
//! implementer un-ignores. Per §3.6b pim-2 the un-ignored tests must
//! drive the production `Engine::register_subgraph` path.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-C — plan §3 G14-C — handler-version chain durable via Anchor"]
fn handler_version_chain_durable_via_anchor_node() {
    // plan §3 G14-C pin. Compromise #18 closure: registered handler
    // subgraph creates a Version Node hung from a per-handler Anchor;
    // the chain persists across engine restart.
    //
    // Implementer wires:
    //
    //   let store_dir = tempfile::tempdir().unwrap();
    //   let handler_id = "demo:create_post";
    //
    //   let v1_cid = {
    //       let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //       let subgraph = ...; // a TRANSFORM subgraph
    //       let cid = engine.register_subgraph(handler_id, subgraph).unwrap();
    //       cid
    //   };
    //
    //   // Re-open; chain persists.
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //   let chain = engine.handler_version_chain(handler_id).unwrap();
    //   assert_eq!(chain.current_version_cid(), v1_cid);
    //   assert_eq!(chain.versions().len(), 1);
    //   // Anchor structure: anchor → v1 → CURRENT.
    //   assert!(chain.anchor_cid().is_some());
    //
    // OBSERVABLE consequence: Re-opening the engine resurrects the
    // handler-version chain end-to-end (anchor + version + CURRENT
    // pointer). Compromise #18 closes when this test goes green.
    unimplemented!(
        "G14-C wires handler-version chain durability via Anchor Node + Version Node + CURRENT"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-C — plan §3 G14-C — chain advances on register_subgraph replace"]
fn handler_version_chain_advances_on_register_subgraph_replace() {
    // plan §3 G14-C pin. Re-registering a handler with a new subgraph
    // appends a new Version + advances CURRENT, but old Versions
    // remain queryable per the Anchor pattern.
    //
    // Implementer wires:
    //
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //   let handler_id = "demo:create_post";
    //
    //   let v1_cid = engine.register_subgraph(handler_id, &subgraph_v1).unwrap();
    //   let v2_cid = engine.register_subgraph(handler_id, &subgraph_v2).unwrap();
    //   assert_ne!(v1_cid, v2_cid, "different subgraphs must hash to different CIDs");
    //
    //   let chain = engine.handler_version_chain(handler_id).unwrap();
    //   assert_eq!(chain.current_version_cid(), v2_cid);
    //   assert_eq!(chain.versions().len(), 2);
    //   // Old version still queryable:
    //   assert!(chain.fetch_version(&v1_cid).is_some());
    //   // History order is preserved (oldest → newest):
    //   assert_eq!(chain.versions()[0].cid(), v1_cid);
    //   assert_eq!(chain.versions()[1].cid(), v2_cid);
    //
    // OBSERVABLE consequence: a handler that has been replaced
    // multiple times exposes its full version history; CURRENT names
    // the latest; old versions are content-addressable for audit /
    // rollback.
    unimplemented!(
        "G14-C wires handler-version chain advancement + old-version preservation on re-register"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-C — arch-r1-4 BLOCKER + D-C — extensible attribution variants"]
fn canonical_bytes_handler_version_chain_extensible_for_future_attribution_variants() {
    // arch-r1-4 BLOCKER + D-C pin (CLR-1 cluster). The canonical-bytes
    // encoding for handler-version chain Nodes MUST leave additive
    // variant slots for future attribution metadata. Without this,
    // Phase-3 G16-B (Loro-merge attribution-frame at new Version)
    // would force a breaking encoding change.
    //
    // Implementer wires this as an ENCODING-SHAPE pin: the canonical
    // bytes for a handler-version Node must use a CBOR map/struct
    // that allows additive fields without changing encoding identity
    // for the existing fields.
    //
    //   let v_node = ...;
    //   let bytes = v_node.canonical_bytes();
    //   let parsed: serde_ipld_dagcbor::Value = serde_ipld_dagcbor::from_slice(&bytes).unwrap();
    //   // The top-level structure is a CBOR map (additive-friendly):
    //   match parsed {
    //       serde_ipld_dagcbor::Value::Map(_) => (),
    //       _ => panic!("handler-version Node canonical bytes must be CBOR map for additive extensibility"),
    //   }
    //
    //   // Decode-with-future-variant simulation: synthesize a CBOR
    //   // map that adds a new field "loro_merge_attribution" + verify
    //   // current decoder accepts (ignores unknown fields gracefully).
    //   let mut extended = parsed_as_map(&bytes);
    //   extended.insert("loro_merge_attribution", serde_cbor_value::null());
    //   let extended_bytes = encode(&extended);
    //   let decoded = benten_engine::HandlerVersionNode::from_canonical_bytes(&extended_bytes);
    //   assert!(decoded.is_ok(), "decoder must ignore unknown additive fields");
    //
    // OBSERVABLE consequence: future Phase-3 waves (e.g. G16-B) can
    // add Loro-merge attribution variants to the handler-version
    // chain without forcing a CID-rebake of every existing
    // pinned-CID test site. arch-r1-4 BLOCKER closes when this
    // extensibility contract is asserted.
    unimplemented!(
        "G14-C wires canonical-bytes additive-extensibility check for handler-version chain"
    );
}
