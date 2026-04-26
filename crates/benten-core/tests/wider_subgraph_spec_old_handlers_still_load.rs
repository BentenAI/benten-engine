//! G12-D red-phase: forward-compat — pre-widening handler fixtures (with no
//! per-primitive props bag, or with empty bags) load against the WIDENED
//! `SubgraphSpec` without error.
//!
//! Per plan §3.2 G12-D must-pass tests: "wider_subgraph_spec_old_handlers_still_load
//! (forward-compat)."
//!
//! Property pin: the BTreeMap<String, Value> bag is decoded as an empty map
//! when the on-wire DAG-CBOR doesn't carry one. Pins that the widening is a
//! strict superset, NOT a breaking change.
//!
//! TDD red-phase. Owner: R5 G12-D (qa-r4-03 R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

use benten_core::Subgraph;

#[test]
#[ignore = "R5 G12-D red-phase: pre-widening fixture compat not yet asserted"]
fn pre_widening_handler_fixture_loads_against_widened_subgraph_spec() {
    // Pre-widening fixture: a Phase-2a-era SubgraphSpec serialized via the
    // older codec (no per-primitive props bag in the DAG-CBOR). Decoding
    // against the widened type should succeed with empty bags filled in.
    let _phase_2a_fixture_bytes_hex: &str = "PHASE_2A_FIXTURE_BYTES_HEX_TBD";

    todo!(
        "R5 G12-D: \
         (1) regenerate a pre-widening fixture from a Phase-2a snapshot; \
         (2) decode via Subgraph::from_dagcbor; \
         (3) assert ok + each primitive's props bag is empty (default)"
    )
}

#[test]
#[ignore = "R5 G12-D red-phase: pre-widening CID stability after re-encode not yet asserted"]
fn pre_widening_handler_re_encoded_under_widened_spec_preserves_cid() {
    // Property pin: load pre-widening fixture; re-encode under widened spec;
    // re-encoded CID matches the original CID — proves the widening doesn't
    // silently inject a new field that participates in canonical bytes for
    // pre-widening handlers.
    let _phase_2a_fixture_cid_str: &str = "bafy...PHASE_2A_FIXTURE_CID_TBD";

    todo!(
        "R5 G12-D: load fixture; sg.cid() == original CID; \
         encode + decode + re-cid round-trip preserves CID"
    )
}

#[test]
#[ignore = "R5 G12-D red-phase: post-widening additive cid drift not yet asserted"]
fn adding_per_primitive_props_to_pre_widening_handler_changes_cid() {
    // Counter-pin: pre-widening handler with NO per-primitive props produces
    // CID-A; same handler with `ttl_hours=24` added to the WAIT primitive
    // produces CID-B; CID-A != CID-B. Proves the widening properly participates
    // in canonical bytes when populated (Inv-13 collision-stability).
    todo!(
        "R5 G12-D: build handler-no-ttl; build handler-ttl-24; \
         assert cid(handler-no-ttl) != cid(handler-ttl-24)"
    )
}

#[test]
#[ignore = "R5 G12-D red-phase: subgraph_cache.rs key compat not yet asserted"]
fn subgraph_cache_key_for_pre_widening_handler_matches_post_widening_load() {
    // Pin: `crates/benten-engine/src/subgraph_cache.rs` keys by Subgraph CID;
    // a pre-widening fixture loaded post-widening keys at the same cache key.
    todo!(
        "R5 G12-D: load pre-widening fixture; compute cache key; \
         compute cache key from re-encoded post-widening shape; assert equality"
    )
}

#[test]
#[ignore = "R5 G12-D red-phase: explicit Subgraph type usage parity not yet asserted"]
fn pre_widening_handler_fixture_passes_post_widening_validation() {
    let _ = std::any::type_name::<Subgraph>(); // touch the type so unused-import doesn't trip
    todo!(
        "R5 G12-D: load pre-widening fixture; build_validated() succeeds; \
         no validation rule trips on the missing-bag default"
    )
}
