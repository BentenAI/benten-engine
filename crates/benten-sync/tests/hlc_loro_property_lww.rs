//! R3-C RED-PHASE pin: HLC carries into Loro per-property LWW
//! (G14-pre-D + G16-B; per r2-test-landscape §5 + §13 R3
//! ambiguous-ownership pre-emption).
//!
//! ## Pin source
//!
//! - r2-test-landscape §5 row `hlc_carries_into_loro_per_property_lww`.
//! - r2-test-landscape §13 R3 ambiguous-ownership pre-emption:
//!
//!   > **HLC pin** (`hlc_carries_into_loro_per_property_lww`) is owned
//!   > by R3-A (G14-pre-D HLC infra) AND R3-C (G16-B Loro CRDT).
//!   > Pre-emption: R3-A writes it as a `#[ignore = "RED-PHASE: G16-B
//!   > Loro consumes HLC"]` placeholder; R3-C un-ignores + extends
//!   > with Loro-side assertion at G16-B wave.
//!
//! ## Relocation history
//!
//! At R3-A landing time the placeholder lived at
//! `crates/benten-engine/tests/hlc_loro_property_lww_placeholder.rs`
//! because `benten-sync` did not yet exist as a workspace crate.
//! At R3-C landing time the placeholder is RELOCATED here (its
//! intended home — Loro lives in `benten-sync`) per R3-A mini-review
//! observation. The R3-A placeholder file is removed in the same
//! R3-C commit.
//!
//! ## RED-PHASE discipline
//!
//! Still `#[ignore]`'d at R3-C landing time. G16-B implementer
//! un-ignores when Loro per-property LWW lands and the test body
//! drives the real `benten_sync::crdt::LoroDoc` API.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-B wave-6b consumes HLC into Loro per-property LWW"]
fn hlc_carries_into_loro_per_property_lww() {
    // G16-B implementer wires this against the real surfaces:
    //
    //   use benten_core::hlc::{Hlc, BentenHlc};
    //   use benten_sync::crdt::LoroDoc;
    //
    //   // Two writers each get their own HLC, write to the same
    //   // Loro doc + property:
    //   let mut writer_a_hlc = Hlc::new(0xAAAA_AAAA);
    //   let mut writer_b_hlc = Hlc::new(0xBBBB_BBBB);
    //   let doc_a = LoroDoc::new();
    //   let doc_b = LoroDoc::new();
    //   doc_a.set_property("color", "red", writer_a_hlc.now()).unwrap();
    //   doc_b.set_property("color", "blue", writer_b_hlc.now()).unwrap();
    //   // Bidirectional merge:
    //   doc_a.merge(&doc_b).unwrap();
    //   doc_b.merge(&doc_a).unwrap();
    //   // Both peers converge on the LWW result; the winner is
    //   // determined by HLC ordering (the higher BentenHlc wins).
    //   assert_eq!(doc_a.get_property("color"), doc_b.get_property("color"));
    //   // Property: HLC ordering determines the winner deterministically.
    //
    // OBSERVABLE consequence: Loro merge across two peers converges
    // on the same value, with the HLC determining which write wins.
    // Defends against the failure shape where Loro uses its own
    // internal logical clock (which would NOT be monotonically
    // ordered against the rest of the engine's HLC-stamped writes,
    // breaking Inv-14 attribution + cross-process resume).
    unimplemented!("G16-B wires Loro LWW-via-HLC convergence assertion");
}
