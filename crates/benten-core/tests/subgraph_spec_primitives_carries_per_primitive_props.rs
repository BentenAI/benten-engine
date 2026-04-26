//! G12-D red-phase: assert the widened `SubgraphSpec.primitives` shape
//! (per-primitive properties bag) round-trips per-primitive properties.
//!
//! Per plan §3.2 G12-D + D6-RECOMMEND: `BTreeMap<String, Value>` (sorted by
//! key for canonical bytes) carried as the per-primitive properties bag.
//! Each consumer of `SubgraphSpec.primitives` reads the wider shape.
//!
//! Per `r1-architect-reviewer.json` D6 note: "BTreeMap<String, Value> is more
//! flexible at the cost of less compile-time safety. ... Mild lean toward
//! BTreeMap for forward-compat without semver thrash."
//!
//! TDD red-phase. Owner: R5 G12-D (qa-r4-03 R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

use benten_core::{Subgraph, SubgraphBuilder};

#[test]
#[ignore = "R5 G12-D red-phase: SubgraphSpec.primitives widening not yet landed"]
fn widened_subgraph_spec_carries_wait_ttl_hours_in_per_primitive_props_bag() {
    // Build a handler with a WAIT primitive that declares ttl_hours = 24
    // (the D12-RESOLVED default-via-explicit-set path).
    let mut b = SubgraphBuilder::new("wait-with-ttl");
    let r = b.read("post");
    // The exact builder API for adding per-primitive props depends on G12-D
    // shape — the recommend lean is `b.wait(...).with_prop("ttl_hours", Value::U64(24))`.
    b.respond(r);
    let _sg = b.build_validated().expect("valid");

    todo!(
        "R5 G12-D: build handler with WAIT(ttl_hours=24); inspect spec.primitives; \
         assert the WAIT entry's BTreeMap<String, Value> bag contains \
         ('ttl_hours', Value::U64(24))"
    )
}

#[test]
#[ignore = "R5 G12-D red-phase: per-primitive props round-trip via DAG-CBOR not yet wired"]
fn widened_subgraph_spec_per_primitive_props_round_trip_via_dagcbor() {
    let mut b = SubgraphBuilder::new("wait-with-ttl-rt");
    let r = b.read("post");
    b.respond(r);
    let _sg = b.build_validated().expect("valid");

    todo!(
        "R5 G12-D: encode sg via to_dagcbor; decode; assert decoded sg's \
         per-primitive props bag preserves WAIT.ttl_hours = 24"
    )
}

#[test]
#[ignore = "R5 G12-D red-phase: SANDBOX wallclock_ms in per-primitive bag not yet wired"]
fn widened_subgraph_spec_carries_sandbox_wallclock_ms_in_per_primitive_props_bag() {
    // D24-RECOMMEND: SANDBOX per-handler wallclock_ms opt-in via
    // SubgraphSpec.primitives — pins the widening covers SANDBOX.
    let mut b = SubgraphBuilder::new("sandbox-with-wallclock");
    let _r = b.read("post"); // placeholder; real fixture builds SANDBOX primitive
    todo!(
        "R5 G12-D: build handler with SANDBOX(wallclock_ms=30000); \
         assert spec.primitives[0]'s bag contains ('wallclock_ms', Value::U64(30000))"
    )
}

#[test]
#[ignore = "R5 G12-D red-phase: SUBSCRIBE pattern in per-primitive bag not yet wired"]
fn widened_subgraph_spec_carries_subscribe_pattern_in_per_primitive_props_bag() {
    let _b = SubgraphBuilder::new("subscribe-with-pattern");
    todo!(
        "R5 G12-D: build handler with SUBSCRIBE(pattern='post:*'); \
         assert spec.primitives bag contains ('pattern', Value::String('post:*'))"
    )
}

#[test]
#[ignore = "R5 G12-D red-phase: empty props bag for primitive with no config not yet wired"]
fn widened_subgraph_spec_handles_empty_props_bag_for_primitives_without_config() {
    // READ primitive (no config) should produce an empty BTreeMap, NOT a
    // None / missing entry. Pins the bag is always present for shape stability.
    let mut b = SubgraphBuilder::new("read-no-config");
    let r = b.read("post");
    b.respond(r);
    let _sg = b.build_validated().expect("valid");

    todo!(
        "R5 G12-D: assert spec.primitives[0] (READ) carries an empty BTreeMap \
         (NOT None / missing) — shape is uniform across primitives"
    )
}
