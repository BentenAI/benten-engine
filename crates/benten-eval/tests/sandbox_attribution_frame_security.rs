//! Phase 2b R3-C (consolidated) — security-framed AttributionFrame
//! integrity tests under SANDBOX (G7-A). Sibling to R3-B's
//! `sandbox_attribution.rs` (R2-aligned threading test); this file holds
//! the security-framed adversarial cases.
//!
//! Cross-territory dedup: the unit-shape `sandbox_attribution_frame_threads_through_host_fn`
//! lives in R3-B's `sandbox_attribution.rs` (R2 §1.3-aligned path). This
//! file was previously named `sandbox_attribution_frame_threads_through_host_fn.rs`
//! and contained that same test as a duplicate; consolidation renamed
//! to `sandbox_attribution_frame_security.rs` and dropped the duplicate
//! per `r3-consolidation.md` §2 item 3.
//!
//! Pin sources: sec-pre-r1-03 (audit-trail laundering — sibling); D20
//! sandbox_depth INHERITED across CALL (sec-pre-r1-08 SANDBOX → CALL →
//! SANDBOX laundering attack); sec-pre-r1-13 forward-compat regression
//! (Phase-2a sec-r6r1-01 / sec-r6r2-02 / sec-r6r3-02 closures hold);
//! r1-security-auditor.json + r2-test-landscape.md §5.4.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

// R5 surfaces consumed:
//   benten_eval::sandbox::{Sandbox, SandboxConfig, ManifestRef, SandboxResult}
//   benten_eval::{AttributionFrame, TraceStep}
//   benten_engine::Engine

#[test]
#[ignore = "Phase 2b G7-A pending — D20 + sec-pre-r1-03 sandbox_depth inheritance"]
fn sandbox_attribution_frame_sandbox_depth_inherited_not_reset_across_call() {
    // Per D20-RESOLVED: AttributionFrame.sandbox_depth: u8 increments on
    // SANDBOX entry; INHERITED across CALL boundaries (NOT reset). Closes
    // the SANDBOX → CALL → SANDBOX laundering attack class
    // (sec-pre-r1-08).
    //
    // R5 wires:
    //   1. Construct handler chain: SANDBOX(handler_a) where handler_a
    //      contains CALL(handler_b) where handler_b contains
    //      SANDBOX(handler_c).
    //   2. Invoke the outer SANDBOX; capture the trace.
    //   3. ASSERT: the inner SANDBOX's AttributionFrame.sandbox_depth
    //      == 2 (NOT 1 — would indicate CALL-boundary reset would have
    //      occurred).
    //   4. Cross-check: outer SANDBOX TraceStep frames have depth == 1.
    //
    // Pin: nesting through CALL counts cumulatively; an attacker cannot
    // launder SANDBOX nesting via CALL indirection.
    todo!("R5 G7-A — assert inner SANDBOX sandbox_depth == 2 (inherited, not reset)");
}

#[test]
#[ignore = "Phase 2b G7-A pending — sec-pre-r1-13 forward-compat regression"]
fn attribution_frame_extension_does_not_leak_to_unauthorized_consumers() {
    // sec-pre-r1-13 non-regression — Phase-2a closures must hold:
    //   * sec-r6r1-01 (Inv-14 dead-coded wiring closed)
    //   * sec-r6r2-02 (test-helpers gating sweep)
    //   * sec-r6r3-02 (parse-counter cfg-gate)
    //
    // Specific concern surfaced for Phase 2b: as AttributionFrame gains
    // new fields (D20 sandbox_depth: u8 in Phase 2b), unauthorized
    // consumers (e.g. user code in a SANDBOX module) MUST NOT be able
    // to read them.
    //
    // R5 wires:
    //   1. SANDBOX module attempts to access the host-side
    //      AttributionFrame via any host-fn (none exists; the test pins
    //      the absence).
    //   2. ASSERT: NO host-fn entry in `host-functions.toml` whose
    //      `behavior.kind` reads or returns an AttributionFrame field.
    //   3. ASSERT: AttributionFrame is NOT a wasmtime extern type
    //      (cannot be passed across the trampoline).
    //
    // Defense-in-depth: closes the case where a future host-fn might
    // accidentally surface attribution data into the SANDBOX guest.
    todo!("R5 G7-A — sweep host-functions.toml + assert no entry exposes AttributionFrame");
}
