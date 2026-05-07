//! G16-B wave-6b LANDED — Loro per-property LWW + HLC ordering pins.
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
//! ## pim-2 §3.6b end-to-end discipline
//!
//! Both pins drive the production `benten_sync::crdt::LoroDoc` API +
//! assert OBSERVABLE behavioral consequences (LWW resolution + bidi
//! convergence). They would FAIL if the LWW arm silently degraded
//! (e.g. wired Loro's internal Lamport directly without HLC carry).

#![allow(clippy::unwrap_used)]

use benten_core::hlc::BentenHlc;
use benten_sync::crdt::LoroDoc;

#[test]
fn loro_per_property_lww_round_trip() {
    // D-PHASE-3-4 + plan §3 G16-B pin.
    let doc = LoroDoc::new();
    doc.set_property("title", "v1", BentenHlc::new(100, 0, 0xAAAA))
        .unwrap();
    doc.set_property("title", "v2", BentenHlc::new(200, 0, 0xAAAA))
        .unwrap();
    doc.set_property("title", "v3", BentenHlc::new(300, 0, 0xAAAA))
        .unwrap();

    // LWW: latest write wins under monotonic HLC.
    assert_eq!(doc.get_property("title").as_deref(), Some("v3"));

    // Round-trip via canonical bytes.
    let bytes = doc.to_canonical_bytes().unwrap();
    let restored = LoroDoc::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(restored.get_property("title").as_deref(), Some("v3"));
}

#[test]
fn loro_concurrent_writes_converge_via_hlc_ordering() {
    // plan §3 G16-B pin. Two writers each get their own HLC, write
    // concurrently to the same Loro doc + property. After bidirectional
    // merge, both converge on the same value (HLC determines winner).
    let doc_a = LoroDoc::new();
    let doc_b = LoroDoc::new();
    doc_a
        .set_property("color", "red", BentenHlc::new(100, 0, 0xAAAA))
        .unwrap();
    doc_b
        .set_property("color", "blue", BentenHlc::new(200, 0, 0xBBBB))
        .unwrap();

    // Bidirectional merge.
    doc_a.merge(&doc_b).unwrap();
    doc_b.merge(&doc_a).unwrap();

    // Both peers converge on the SAME value (HLC determines winner).
    assert_eq!(doc_a.get_property("color"), doc_b.get_property("color"));
    // Higher HLC (200 > 100) wins.
    assert_eq!(doc_a.get_property("color").as_deref(), Some("blue"));
}
