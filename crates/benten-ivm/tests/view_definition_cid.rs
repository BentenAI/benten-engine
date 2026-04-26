#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! `ViewDefinition` carries the strategy field in its CID (g8-concern-1).
//!
//! Pin source: `.addl/phase-2b/00-implementation-plan.md` §3 G8-A.
//! Landscape source: `.addl/phase-2b/r2-test-landscape.md` §1.6 row 11.
//!
//! Two definitions that differ ONLY in `strategy` must produce different
//! CIDs. Otherwise, snapshots of the IVM registry can't distinguish a
//! Phase-1 hand-written `Strategy::A` view from a Phase-2b `Strategy::B`
//! Algorithm-B view of the same `view_id`, and the Phase-3 sync layer
//! would see content-address collisions.

#![allow(clippy::unwrap_used)]

use benten_ivm::{Strategy, ViewDefinition};

#[test]
#[ignore = "Phase 2b G8-A pending"]
fn view_definition_carries_strategy_field_in_cid() {
    let def_a = ViewDefinition {
        view_id: "content_listing".into(),
        input_pattern_label: Some("post".into()),
        output_label: "system:IVMView".into(),
        strategy: Strategy::A,
    };
    let def_b = ViewDefinition {
        view_id: "content_listing".into(),
        input_pattern_label: Some("post".into()),
        output_label: "system:IVMView".into(),
        strategy: Strategy::B,
    };
    let def_c = ViewDefinition {
        view_id: "content_listing".into(),
        input_pattern_label: Some("post".into()),
        output_label: "system:IVMView".into(),
        strategy: Strategy::C,
    };

    let cid_a = def_a.cid().expect("definition cid must be infallible");
    let cid_b = def_b.cid().expect("definition cid must be infallible");
    let cid_c = def_c.cid().expect("definition cid must be infallible");

    assert_ne!(
        cid_a, cid_b,
        "ViewDefinition CID must include the `strategy` field — A vs B \
         collided, which would let a Phase-2b Algorithm-B view masquerade \
         as the Phase-1 hand-written baseline in the IVM registry snapshot."
    );
    assert_ne!(cid_b, cid_c);
    assert_ne!(cid_a, cid_c);

    // Self-equality sanity — same struct, same CID.
    let def_a_again = ViewDefinition {
        view_id: "content_listing".into(),
        input_pattern_label: Some("post".into()),
        output_label: "system:IVMView".into(),
        strategy: Strategy::A,
    };
    assert_eq!(cid_a, def_a_again.cid().unwrap());
}
