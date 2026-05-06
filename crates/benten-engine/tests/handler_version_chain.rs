//! G14-C wave-4b: handler-version chain durable via Anchor Node
//! (Compromise #18 closure; plan §3 G14-C + arch-r1-4 BLOCKER + D-C).
//!
//! Pin sources (per r2-test-landscape.md §2.2 G14-C):
//!
//! - `handler_version_chain_durable_via_anchor_node` — plan §3 G14-C
//! - `handler_version_chain_advances_on_register_subgraph_replace` — plan §3 G14-C
//! - `canonical_bytes_handler_version_chain_extensible_for_future_attribution_variants`
//!   — arch-r1-4 BLOCKER + D-C
//!
//! Per §3.6b pim-2 these tests drive the production
//! `Engine::register_subgraph` / `register_subgraph_replace` paths and
//! assert observable consequences: chain persists across restart;
//! re-registration advances + preserves history; canonical-bytes
//! encoding admits additive extension without CID drift on existing
//! Nodes.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Cid, Node, Value};
use benten_engine::Engine;
use benten_engine::handler_versions::{HANDLER_VERSION_LABEL, make_version_node};
use benten_eval::SubgraphBuilder;
use benten_eval::{SubgraphBuilderExt, SubgraphExt};

fn build_handler(handler_id: &str, label: &str) -> benten_eval::Subgraph {
    let mut sb = SubgraphBuilder::new(handler_id);
    let r = sb.read(label);
    sb.respond(r);
    sb.build_validated().expect("must build")
}

#[test]
fn handler_version_chain_durable_via_anchor_node() {
    // Compromise #18 closure: registered handler subgraph persists
    // its version chain via system:HandlerVersion zone Nodes; the
    // chain is rehydrated at engine open.
    let dir = tempfile::tempdir().unwrap();
    let store_path = dir.path().join("benten.redb");

    let v1_cid = {
        let engine = Engine::open(&store_path).unwrap();
        let sg = build_handler("demo:create_post", "post");
        let expected_cid = sg.cid().unwrap();
        engine.register_subgraph(sg).unwrap();
        let chain = engine.handler_version_chain("demo:create_post");
        assert_eq!(
            chain.len(),
            1,
            "first registration seeds chain with 1 entry"
        );
        assert_eq!(chain[0], expected_cid);
        expected_cid
    };

    // Re-open at the same path.
    let engine = Engine::open(&store_path).unwrap();
    let chain = engine.handler_version_chain("demo:create_post");
    assert_eq!(
        chain.len(),
        1,
        "Compromise #18: chain MUST be rehydrated from system:HandlerVersion zone"
    );
    assert_eq!(
        chain[0], v1_cid,
        "rehydrated chain head MUST match the originally-registered version"
    );

    // Anchor accessor exposes the same chain via the consolidated
    // anchor-store handle (cov-f3 closure).
    let with_anchor = engine
        .handler_version_chain_with_anchor("demo:create_post")
        .expect("non-empty chain returns a HandlerVersionChain");
    assert!(
        with_anchor.anchor_cid().is_some(),
        "anchor MUST be present for non-empty chain"
    );
    assert_eq!(with_anchor.current_version_cid(), Some(v1_cid));
}

#[test]
fn handler_version_chain_advances_on_register_subgraph_replace() {
    // Re-registering a handler with a new subgraph appends a new
    // Version + advances CURRENT, but old Versions remain queryable.
    let dir = tempfile::tempdir().unwrap();
    let store_path = dir.path().join("benten.redb");

    let v1_cid;
    let v2_cid;
    {
        let engine = Engine::open(&store_path).unwrap();
        let sg1 = build_handler("demo:create_post", "post");
        v1_cid = sg1.cid().unwrap();
        engine.register_subgraph_replace(sg1).unwrap();

        let sg2 = build_handler("demo:create_post", "comment");
        v2_cid = sg2.cid().unwrap();
        engine.register_subgraph_replace(sg2).unwrap();
    }
    assert_ne!(v1_cid, v2_cid, "different content => different CIDs");

    // Re-open: the chain MUST contain BOTH versions in newest-first
    // order [v2, v1].
    let engine = Engine::open(&store_path).unwrap();
    let chain = engine.handler_version_chain("demo:create_post");
    assert_eq!(
        chain.len(),
        2,
        "Compromise #18: full chain history MUST persist across restart"
    );
    assert_eq!(
        chain[0], v2_cid,
        "chain[0] (CURRENT) MUST be the most-recently registered version"
    );
    assert_eq!(
        chain[1], v1_cid,
        "chain[1] MUST be the prior version (newest-first invariant)"
    );

    let with_anchor = engine
        .handler_version_chain_with_anchor("demo:create_post")
        .expect("non-empty chain");
    // The anchor's head is the OLDEST version (chain.last()).
    assert_eq!(
        with_anchor.anchor_cid(),
        Some(v1_cid),
        "anchor head MUST equal the chain root (oldest registered version)"
    );
    assert_eq!(with_anchor.versions().len(), 2);
}

#[test]
fn canonical_bytes_handler_version_chain_extensible_for_future_attribution_variants() {
    // arch-r1-4 BLOCKER + D-C. The canonical-bytes encoding for
    // handler-version Nodes uses a CBOR map (DAG-CBOR's sorted-key
    // discipline + serde's `skip_serializing_if = "Option::is_none"`
    // omission). Future Phase-3 G16-B can add new property keys
    // (e.g. `loro_merge_attribution`) WITHOUT changing the canonical
    // bytes (or CID) of any chain Node that pre-dates the amendment.
    let cid = Cid::from_blake3_digest(*blake3::hash(b"some-version").as_bytes());
    let node = make_version_node("demo:create_post", &cid, None, 0);
    let bytes = node.canonical_bytes().unwrap();

    // (1) The first byte of a DAG-CBOR map is in the major-type-5
    // range (0xa0..=0xbf for definite-length maps). Per RFC-8949
    // §3.1, major type 5 is `0b101_xxxxx`.
    let first = bytes.first().copied().unwrap_or(0);
    let major_type = first >> 5;
    assert_eq!(
        major_type, 5,
        "Node canonical bytes MUST be a CBOR map (additive-extensible) per arch-r1-4 / D-C; \
         first byte was 0x{first:02x} (major type {major_type})"
    );

    // (2) A Node with an EXTRA property produces DIFFERENT bytes.
    let mut extended_props: std::collections::BTreeMap<String, Value> =
        std::collections::BTreeMap::new();
    extended_props.insert("handler_id".into(), Value::Text("demo:create_post".into()));
    extended_props.insert("version_cid".into(), Value::Text(cid.to_base32()));
    extended_props.insert("seq".into(), Value::Int(0));
    extended_props.insert(
        "loro_merge_attribution".into(),
        Value::Text("future-G16-B-variant".into()),
    );
    let extended = Node::new(vec![HANDLER_VERSION_LABEL.to_string()], extended_props);
    let extended_bytes = extended.canonical_bytes().unwrap();
    assert_ne!(
        bytes, extended_bytes,
        "additive extension MUST produce different canonical bytes (the new property is encoded)"
    );

    // (3) The pre-extension bytes are byte-stable (DAG-CBOR
    // determinism).
    let node_again = make_version_node("demo:create_post", &cid, None, 0);
    let bytes_again = node_again.canonical_bytes().unwrap();
    assert_eq!(
        bytes, bytes_again,
        "DAG-CBOR canonical bytes MUST be byte-stable for identical content"
    );
}

#[test]
fn make_version_node_pinned_cid_for_basic_shape() {
    // g14-c-mr-5: a literal-CID pin for the basic make_version_node
    // shape (predecessor=None, seq=0). This pin DOES NOT defeat the
    // additive-extensibility contract above: when a future Phase-3
    // amendment adds an optional property (e.g. `loro_merge_attribution`),
    // the make_version_node call site here continues NOT to set that
    // property, so the canonical-bytes encoding stays byte-stable and
    // the CID below stays valid. Catches encoder regressions
    // (e.g. seq encoding flipped from Int to Text) that the structural
    // pin in the prior test would silently pass.
    //
    // Inputs:
    //   handler_id = "demo:fix"
    //   version_cid = blake3("fixture-version")
    //   predecessor = None
    //   seq = 0
    let cid = Cid::from_blake3_digest(*blake3::hash(b"fixture-version").as_bytes());
    let node = make_version_node("demo:fix", &cid, None, 0);
    let observed_cid_b32 = node
        .cid()
        .expect("make_version_node output must have computable CID")
        .to_base32();
    // Pinned literal — extracted from the encoder at G14-C HEAD.
    // If a future change deliberately re-shapes the encoding (e.g.
    // promoting `seq` to a richer type), the rebake is intentional
    // and the literal below must update IN THE SAME PR — making
    // the encoder change observable in code review.
    const EXPECTED_CID_BASE32: &str = "bafyr4ibvdpffau7453rdofaqtmgvbrjd4opht5nk7swwtwh3gtedep75qy";
    assert_eq!(
        observed_cid_b32, EXPECTED_CID_BASE32,
        "g14-c-mr-5: make_version_node basic-shape CID drift; observed: {observed_cid_b32}"
    );
}
