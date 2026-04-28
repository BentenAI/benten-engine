//! Phase 2b R3-B — SANDBOX memory-axis unit tests (G7-A).
//!
//! Pin sources: plan §3 G7-A (memory limit), ESC-2 (linmem grow attack).
//!
//! Wave-8b: wired against the live wasmtime ResourceLimiter pipeline.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{ManifestRef, ManifestRegistry, SandboxConfig, execute};

fn dummy_attribution() -> AttributionFrame {
    let zero = Cid::from_blake3_digest([0u8; 32]);
    AttributionFrame {
        actor_cid: zero,
        handler_cid: zero,
        capability_grant_cid: zero,
        sandbox_depth: 0,
    }
}

#[test]
fn sandbox_memory_limit_kills_routes_e_sandbox_memory_exhausted() {
    // ESC-2 — module declares a memory whose minimum (post-grow attempts)
    // exceeds the per-call cap. The ResourceLimiter rejects growth above
    // the configured limit; either wasmtime's `memory.grow` returns -1
    // and the module observes the failure, or instantiation/grow fails
    // outright. We construct a fixture that loops memory.grow(1) until
    // failure, which under the limiter ALWAYS terminates with grow == -1
    // when the cap is hit. We then explicitly trap to surface the cap.
    //
    // Per the brief D21 priority resolver: a memory cap rejection
    // surfaces as `E_SANDBOX_MEMORY_EXHAUSTED` when wasmtime treats it
    // as out-of-memory; the ResourceLimiter currently lets `memory.grow`
    // return -1 (no trap) and the module must trap itself. To pin the
    // cap-enforcement contract, we use a fixture that requires a
    // minimum memory size LARGER than the cap so instantiation fails
    // outright with the memory-exceeded error.
    // Module declares 1 page initial, with `memory.grow` loop that
    // attempts to grow well beyond the cap. When grow returns -1 the
    // module's `unreachable` instruction trips. The cap is enforced by
    // wasmtime ResourceLimiter — when growth is denied wasmtime returns
    // -1 from memory.grow, AND the executor's error-mapping recognizes
    // the limiter-rejection shape via the error string.
    let bytes = wat::parse_str(
        "(module
           (memory (export \"memory\") 1)
           (func (export \"run\") (result i32)
             (loop $L
               (if (i32.eq (memory.grow (i32.const 1)) (i32.const -1))
                 (then (unreachable))
               )
               br $L
             )
             i32.const 0
           )
         )",
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let cfg = SandboxConfig {
        // Cap at 1 page (64 KiB); module declares 200 pages minimum.
        memory_bytes: 64 * 1024,
        ..SandboxConfig::default()
    };
    let attribution = dummy_attribution();
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
    assert_eq!(err.code(), ErrorCode::SandboxMemoryExhausted);
}
