//! R3-C RED-PHASE pins for Loro per-property LWW + HLC ordering
//! (G16-B wave-6b; per r2-test-landscape §2.4 G16-B + plan §3 G16-B row).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-B rows
//!   `loro_per_property_lww_round_trip` +
//!   `loro_concurrent_writes_converge_via_hlc_ordering`.
//! - plan §3 G16-B row.
//! - `D-PHASE-3-4` RESOLVED-at-R1 (Loro at Node-property granularity;
//!   per-property LWW + HLC).
//! - HLC carries from `benten-core::hlc` per G14-pre-D landed.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-B wave-6b lands Loro CRDT integration"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — D-PHASE-3-4 — per-property LWW round-trip"]
fn loro_per_property_lww_round_trip() {
    // D-PHASE-3-4 + plan §3 G16-B pin. G16-B implementer wires this:
    //
    //   use benten_sync::crdt::LoroDoc;
    //   use benten_core::hlc::Hlc;
    //
    //   let mut hlc = Hlc::new(0xAAAA);
    //   let doc = LoroDoc::new();
    //   doc.set_property("title", "v1", hlc.now()).unwrap();
    //   doc.set_property("title", "v2", hlc.now()).unwrap();
    //   doc.set_property("title", "v3", hlc.now()).unwrap();
    //
    //   // LWW: latest write wins under monotonic HLC.
    //   assert_eq!(doc.get_property("title").unwrap(), "v3");
    //
    //   // Round-trip via canonical bytes:
    //   let bytes = doc.to_canonical_bytes();
    //   let restored = LoroDoc::from_canonical_bytes(&bytes).unwrap();
    //   assert_eq!(restored.get_property("title").unwrap(), "v3");
    //
    // OBSERVABLE consequence: per-property LWW is observable through
    // the get_property accessor + survives canonical-bytes round-trip.
    unimplemented!("G16-B wires Loro per-property LWW + canonical-bytes round-trip");
}

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b — plan §3 G16-B — concurrent writes converge via HLC"]
fn loro_concurrent_writes_converge_via_hlc_ordering() {
    // plan §3 G16-B pin. Two writers each get their own HLC, write
    // concurrently to the same Loro doc + property. After bidirectional
    // merge, both converge on the same value (HLC determines winner).
    //
    //   use benten_sync::crdt::LoroDoc;
    //   use benten_core::hlc::Hlc;
    //
    //   let mut hlc_a = Hlc::new(0xAAAA);
    //   let mut hlc_b = Hlc::new(0xBBBB);
    //   let doc_a = LoroDoc::new();
    //   let doc_b = LoroDoc::new();
    //   doc_a.set_property("color", "red", hlc_a.now()).unwrap();
    //   doc_b.set_property("color", "blue", hlc_b.now()).unwrap();
    //   // Bidirectional merge:
    //   doc_a.merge(&doc_b).unwrap();
    //   doc_b.merge(&doc_a).unwrap();
    //   // Both peers converge on the SAME value (HLC determines winner).
    //   assert_eq!(doc_a.get_property("color"), doc_b.get_property("color"));
    //
    // OBSERVABLE consequence: after bidirectional merge, the two
    // peers agree on the property value. The agreement is
    // determined by the higher HLC (deterministic — same inputs
    // always converge to the same winner).
    unimplemented!("G16-B wires concurrent-writes-via-HLC convergence assertion");
}
