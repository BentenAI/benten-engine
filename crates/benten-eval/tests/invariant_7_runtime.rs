//! Phase 2b R3-B — Inv-7 sandbox-output runtime unit tests (G7-B).
//!
//! D15 + D17 PRIMARY:
//!   - D17 PRIMARY: streaming `CountedSink` accumulator wraps every
//!     host-fn byte-emission; traps via Inv-7 BEFORE accepting bytes.
//!   - D15 trap-loudly default: NO silent truncation. Output overflow
//!     fires the typed error every time.
//!
//! Pin sources: D15 + D17 PRIMARY, sec-pre-r1-07.
//!
//! **G20-A1 wave-8a** (Phase 3): `#[ignore]` removed; bodies drive the
//! production `benten_eval::sandbox::execute` entry point + assert
//! observable typed errors. Per pim-2 §3.6b, each test would FAIL if
//! the CountedSink primary path were silently no-op'd.

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
        ..Default::default()
    }
}

#[test]
fn invariant_7_output_traps_loudly_via_counted_sink() {
    // D15 + D17 PRIMARY — Inv-7 fires on output overflow via the
    // CountedSink primary path.
    //
    // Strategy: drive 11 successive `log` calls @ 60 KiB each (each
    // call within the per-call 65 KiB log byte cap) under a 500 KiB
    // output budget. Cumulative bytes consumed crosses the boundary
    // and the CountedSink primary path fires E_INV_SANDBOX_OUTPUT
    // BEFORE accepting any further bytes.
    let bytes = wat::parse_str(
        "(module
           (import \"host\" \"log\" (func $log (param i32 i32)))
           (memory (export \"memory\") 16)
           (func (export \"run\") (result i32)
             (local $i i32)
             (loop $L
               i32.const 0
               i32.const 60000
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
        output_bytes: 500_000,
        fuel: 100_000_000,
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
    // The CountedSink primary path fires E_INV_SANDBOX_OUTPUT before
    // any partial bytes leak into the sink.
    assert_eq!(
        err.code(),
        ErrorCode::InvSandboxOutput,
        "Inv-7 trap-loudly via CountedSink primary path; got {:?}",
        err.code()
    );
}

#[test]
fn invariant_7_output_no_silent_truncation_default() {
    // D15 + sec-pre-r1-07 — default behavior is trap-loudly.
    //
    // STRUCTURAL pin via source-grep: the SandboxConfig surface MUST
    // NOT carry a `truncate` field. Absence is the contract.
    let cfg_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("primitives")
            .join("sandbox.rs"),
    )
    .expect("benten-eval/src/primitives/sandbox.rs must be readable");
    assert!(
        !cfg_src.contains("pub truncate:"),
        "SandboxConfig MUST NOT carry a `pub truncate:` field per D15 \
         trap-loudly default; silent truncation is a covert-channel \
         vector"
    );

    // RUNTIME pin: when sink overflows, executor returns Err (typed
    // E_SANDBOX_HOST_FN_DENIED for per-call log cap, or
    // E_INV_SANDBOX_OUTPUT for cumulative output cap). It does NOT
    // return Ok with a truncated payload — that would be the silent-
    // truncation covert-channel.
    let bytes = wat::parse_str(
        "(module
           (import \"host\" \"log\" (func $log (param i32 i32)))
           (memory (export \"memory\") 4)
           (func (export \"run\") (result i32)
             i32.const 0
             i32.const 200000
             call $log
             i32.const 0
           )
         )",
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let cfg = SandboxConfig {
        output_bytes: 1024,
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
    .expect_err(
        "trap-loudly default — overflow MUST surface Err, NOT Ok with \
         a truncated payload",
    );
    assert!(
        matches!(
            err.code(),
            ErrorCode::SandboxHostFnDenied | ErrorCode::InvSandboxOutput
        ),
        "trap-loudly default routes typed error; got {:?}",
        err.code()
    );
}
