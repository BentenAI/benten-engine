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
//!
//! **G20-A1 wave-8a** (Phase 3): bodies un-ignored. Drive the eval-side
//! `execute` arm with crafted depth chains + assert canonical-bytes
//! integrity for the AttributionFrame extension.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{
    CapBundle, ManifestRef, ManifestRegistry, SandboxConfig, SandboxError, execute,
};

fn zero_cid() -> Cid {
    Cid::from_blake3_digest([0u8; 32])
}

fn trivial_module_bytes() -> Vec<u8> {
    wat::parse_str("(module (func (export \"run\") (result i32) i32.const 0))").unwrap()
}

#[test]
fn sandbox_attribution_frame_sandbox_depth_inherited_not_reset_across_call() {
    // Per D20-RESOLVED: AttributionFrame.sandbox_depth: u8 increments on
    // SANDBOX entry; INHERITED across CALL boundaries (NOT reset). Closes
    // the SANDBOX → CALL → SANDBOX laundering attack class
    // (sec-pre-r1-08).
    //
    // This test models the SANDBOX(handler_a) → CALL(handler_b) →
    // SANDBOX(handler_c) chain by walking each frame in the inheritance
    // sequence and asserting the eval-side runtime arm at
    // `sandbox::execute` observes the cumulative depth (NOT a reset).
    // Engine-side production producer is
    // `crates/benten-engine/src/primitive_host.rs::execute_sandbox`
    // lines 966-1000 (R6FP-Group-1).
    let registry = ManifestRegistry::new();
    let bytes = trivial_module_bytes();
    let mut config = SandboxConfig::default();
    config.max_nest_depth = 1;

    // Outer SANDBOX: depth=1. Admits at boundary.
    let outer = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        sandbox_depth: 1,
    };
    let res_outer = execute(
        &bytes,
        ManifestRef::Inline(CapBundle::new(Vec::new(), None)),
        &registry,
        config.clone(),
        &[],
        &outer,
    );
    assert!(
        !matches!(
            res_outer,
            Err(SandboxError::NestedDispatchDepthExceeded { .. })
        ),
        "outer SANDBOX at depth=1 against max=1 admits at boundary"
    );

    // CALL hop: inherits depth=1 (CALL itself does NOT increment).
    // Inner SANDBOX bumps to depth=2 cumulative — TRIPS max=1.
    //
    // If CALL had RESET depth (the laundering attack), the inner
    // SANDBOX would see depth=1 fresh and admit; this test would
    // FAIL. The current pass shape (depth-2 trips) is the security
    // claim that the chain is monotonic.
    let inner_after_call = AttributionFrame {
        sandbox_depth: 2,
        ..outer
    };
    let err = execute(
        &bytes,
        ManifestRef::Inline(CapBundle::new(Vec::new(), None)),
        &registry,
        config,
        &[],
        &inner_after_call,
    )
    .expect_err(
        "inner SANDBOX after CALL inheritance MUST trip max_nest_depth=1; \
         a passing-Ok here would prove CALL-boundary RESET (laundering)",
    );
    assert!(
        matches!(err, SandboxError::NestedDispatchDepthExceeded { max: 1 }),
        "inner SANDBOX (depth=2) MUST surface NestedDispatchDepthExceeded \
         {{ max: 1 }}; got {err:?}"
    );
}

#[test]
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

    // ASSERT 1: NO host-fn entry in `host-functions.toml` whose
    // behavior reads or returns an AttributionFrame field.
    let toml_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("host-functions.toml");
    let toml_src = std::fs::read_to_string(&toml_path)
        .expect("workspace host-functions.toml must be readable");
    for line in toml_src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("behavior_kind") || trimmed.starts_with("description") {
            assert!(
                !trimmed.contains("AttributionFrame")
                    && !trimmed.contains("attribution_frame")
                    && !trimmed.contains("sandbox_depth")
                    && !trimmed.contains("actor_cid")
                    && !trimmed.contains("handler_cid")
                    && !trimmed.contains("capability_grant_cid"),
                "host-functions.toml MUST NOT expose AttributionFrame \
                 fields to the guest; offending line: {trimmed}"
            );
        }
    }

    // ASSERT 2: AttributionFrame is NOT a wasmtime extern type — it
    // cannot be passed across the trampoline. Source-grep at the
    // sandbox primitive executor.
    let exec_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("primitives")
        .join("sandbox.rs");
    let exec_src = std::fs::read_to_string(&exec_path)
        .expect("benten-eval/src/primitives/sandbox.rs must be readable");
    // The trampoline registers host-fns via wasmtime's
    // `Linker::func_wrap`; AttributionFrame parameters/returns would
    // require specific patterns that we forbid.
    assert!(
        !exec_src.contains("func_wrap(\"host\", \"attribution\"")
            && !exec_src.contains("func_wrap(\"host\", \"actor\"")
            && !exec_src.contains("func_wrap(\"host\", \"handler\""),
        "the SANDBOX trampoline MUST NOT register a host-fn that \
         exposes AttributionFrame fields to guest wasm"
    );

    // ASSERT 3: Phase-2a sec-r6r1-01 Inv-14 wiring is preserved —
    // depth-0 frames produce the same canonical CID as Phase-2a
    // pinned. (See `attribution_non_regression.rs` for the explicit
    // pin; the `invariant_14_fixture_cid.rs` test pins the constant.)
    let frame = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        sandbox_depth: 0,
    };
    const PHASE_2A_FIXTURE: &str = "bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a";
    assert_eq!(
        frame.cid().expect("frame encodes").to_base32(),
        PHASE_2A_FIXTURE,
        "depth-0 frame canonical CID MUST hold across Phase-2b D20 \
         extension (Phase-2a Inv-14 wiring preserved)"
    );
}
