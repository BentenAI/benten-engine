//! Phase 2b R3-B — Inv-7 streaming-sandbox-output integration test (G7-A).
//!
//! Pin sources: D17 + wsa-18.
//!
//! **G20-A1 wave-8a** (Phase 3): body un-ignored. The full STREAM-into-
//! SANDBOX wiring is a Phase-3+ feature (the SANDBOX primitive
//! consumes a STREAM via host-fn `chunk_emit` which is not in the D1
//! initial host-fn surface — D1 ships time / log / kv:read / random).
//! In place of the full STREAM-into-SANDBOX harness, this test covers
//! the load-bearing claim end-to-end through the same trampoline that
//! a future `chunk_emit` host-fn would route through:
//!
//!   - The SANDBOX primitive emits N chunks via repeated `log` calls
//!     (each `log` is a chunk-shaped byte emission through the
//!     CountedSink primary path).
//!   - Cumulative chunk bytes exceed the configured output budget.
//!   - The CountedSink primary path traps via Inv-7 BEFORE the next
//!     chunk lands.
//!
//! When a `chunk_emit`-style host-fn lands in Phase 3+, this test's
//! shape extends naturally to consume the STREAM upstream.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{ManifestRef, ManifestRegistry, SandboxConfig, execute};

#[test]
fn invariant_7_end_to_end_with_streaming_sandbox_output() {
    // D17 + wsa-18 — a SANDBOX module emits chunks through the `log`
    // host-fn (the streaming-shaped emission path in the D1 surface).
    // CountedSink wraps every byte-emission; cumulative bytes
    // approaching the budget trigger the Inv-7 trap BEFORE the next
    // chunk would push consumed past the limit.
    //
    // Module emits 11 chunks of 100-byte payload each = 1100 bytes
    // cumulative. Budget: 1024 bytes (≤ 11 chunks). The 11th chunk
    // attempt fires E_INV_SANDBOX_OUTPUT through the primary path.
    let bytes = wat::parse_str(
        "(module
           (import \"host\" \"log\" (func $log (param i32 i32)))
           (memory (export \"memory\") 1)
           (func (export \"run\") (result i32)
             (local $i i32)
             (loop $L
               i32.const 0
               i32.const 100
               call $log
               local.get $i
               i32.const 1
               i32.add
               local.tee $i
               i32.const 11
               i32.lt_s
               br_if $L
             )
             local.get $i
           )
         )",
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let cfg = SandboxConfig {
        output_bytes: 1024,
        fuel: 100_000_000,
        ..SandboxConfig::default()
    };
    let zero = Cid::from_blake3_digest([0u8; 32]);
    let attribution = AttributionFrame {
        actor_cid: zero,
        handler_cid: zero,
        capability_grant_cid: zero,
        sandbox_depth: 0,
    };
    let err = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        cfg,
        &[
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &attribution,
    )
    .unwrap_err();
    assert_eq!(
        err.code(),
        ErrorCode::InvSandboxOutput,
        "Inv-7 streaming end-to-end: chunk-shaped emissions cumulative \
         output exceeding budget MUST trip the CountedSink primary \
         path with E_INV_SANDBOX_OUTPUT; got {:?}",
        err.code()
    );
}
