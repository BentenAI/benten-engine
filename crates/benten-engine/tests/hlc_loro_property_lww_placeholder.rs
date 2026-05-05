//! R3-A RED-PHASE PLACEHOLDER pin: HLC carries into Loro per-property LWW
//! (G14-pre-D + G16-B; placeholder lives here until R3-C / G16-A creates
//! the `benten-sync` crate).
//!
//! ## Pin source
//!
//! r2-test-landscape §2.0 wave-1pre row + §13 R3 ambiguous-ownership
//! pre-emption:
//!
//! > **HLC pin** (`hlc_carries_into_loro_per_property_lww`) is owned by
//! > R3-A (G14-pre-D HLC infra) AND R3-C (G16-B Loro CRDT). Pre-emption:
//! > R3-A writes it as a `#[ignore = "RED-PHASE: G16-B Loro consumes
//! > HLC"]` placeholder; R3-C un-ignores + extends with Loro-side
//! > assertion at G16-B wave.
//!
//! ## Where this lives at R3-A landing time
//!
//! `benten-sync` is the Phase-3 10th workspace crate that lands at
//! G16-A (wave 6); it does not exist at R3-A landing time. Per the R2
//! brief ambiguous-ownership note, R3-A authors the placeholder
//! HERE in `crates/benten-engine/tests/` (engine consumes HLC at G14-D)
//! so the placeholder compiles + runs as `#[ignore]`'d at R3-A
//! landing. R3-C / G16-A:
//!
//! 1. Creates `crates/benten-sync/`.
//! 2. Moves this file to `crates/benten-sync/tests/hlc_loro_property_lww.rs`.
//! 3. Renames the test file (drop the `_placeholder` suffix).
//! 4. Replaces the placeholder body with the real Loro-side assertion
//!    (per `r2-test-landscape.md` §3.B Loro CLR-1 cluster).
//! 5. Un-ignores the test at G16-B wave landing.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-B (R3-C) consumes HLC into Loro per-property LWW; \
            placeholder lives in benten-engine/tests/ until benten-sync crate exists"]
fn hlc_carries_into_loro_per_property_lww_placeholder() {
    // R3-C / G16-B implementer:
    //
    // 1. Move this file to `crates/benten-sync/tests/hlc_loro_property_lww.rs`
    //    (rename + relocate).
    // 2. Replace the body with:
    //
    //   use benten_core::hlc::{Hlc, BentenHlc};
    //
    //   // Two writers each get their own HLC, write to the same Loro
    //   // doc + property:
    //   let writer_a_hlc = Hlc::new(0xAAAA_AAAA, system_time_ms);
    //   let writer_b_hlc = Hlc::new(0xBBBB_BBBB, system_time_ms);
    //   let doc_a = benten_sync::loro::LoroDoc::new();
    //   let doc_b = benten_sync::loro::LoroDoc::new();
    //   doc_a.set_property("color", "red", writer_a_hlc.now());
    //   doc_b.set_property("color", "blue", writer_b_hlc.now());
    //   // Merge both:
    //   doc_a.merge(&doc_b);
    //   doc_b.merge(&doc_a);
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
    unimplemented!(
        "R3-C / G16-B relocates this file to crates/benten-sync/tests/ + wires Loro LWW-via-HLC convergence"
    );
}
