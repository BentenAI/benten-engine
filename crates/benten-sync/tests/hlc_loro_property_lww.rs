//! G16-B wave-6b LANDED — HLC carries into Loro per-property LWW.
//!
//! ## Pin source
//!
//! - r2-test-landscape §5 row `hlc_carries_into_loro_per_property_lww`.
//! - r2-test-landscape §13 R3 ambiguous-ownership pre-emption.
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! Drives the production `LoroDoc::set_property` API consuming
//! `BentenHlc` + asserts OBSERVABLE convergence under HLC ordering.

#![allow(clippy::unwrap_used)]

use benten_core::hlc::BentenHlc;
use benten_sync::crdt::LoroDoc;

#[test]
fn hlc_carries_into_loro_per_property_lww() {
    let writer_a_hlc = BentenHlc::new(100, 0, 0xAAAA_AAAA);
    let writer_b_hlc = BentenHlc::new(200, 0, 0xBBBB_BBBB);
    let doc_a = LoroDoc::new();
    let doc_b = LoroDoc::new();
    doc_a.set_property("color", "red", writer_a_hlc).unwrap();
    doc_b.set_property("color", "blue", writer_b_hlc).unwrap();
    // Bidirectional merge.
    doc_a.merge(&doc_b).unwrap();
    doc_b.merge(&doc_a).unwrap();
    // Both peers converge on the LWW result; the winner is determined
    // by HLC ordering (the higher BentenHlc wins).
    assert_eq!(doc_a.get_property("color"), doc_b.get_property("color"));
    // Higher physical_ms (200) wins.
    assert_eq!(doc_a.get_property("color").as_deref(), Some("blue"));
}
