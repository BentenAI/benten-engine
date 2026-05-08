//! Phase 2b R3-B — `log` host-fn unit test (G7-A).
//!
//! D1 + sec-pre-r1-06 §2.2 — 64 KiB per-call byte-volume cap to prevent
//! spam-based DOS or covert-channel high-bandwidth use.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

//! Wave-8b: wired against the live `log` trampoline.

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
fn sandbox_host_fn_log_respects_byte_volume_cap_64kb() {
    // D1 + sec-pre-r1-06 §2.2: per-call log byte-volume cap = 65 536.
    // Sub-test 1: 65 536-byte log succeeds (==cap).
    let bytes = wat::parse_str(
        "(module
           (import \"host\" \"log\" (func $log (param i32 i32)))
           (memory (export \"memory\") 2)
           (func (export \"run\") (result i32)
             i32.const 0
             i32.const 65536
             call $log
             i32.const 0
           )
         )",
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let cfg = SandboxConfig {
        output_bytes: 1024 * 1024,
        ..SandboxConfig::default()
    };
    let attribution = dummy_attribution();
    let res = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        cfg,
        &[
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &attribution,
    );
    assert!(res.is_ok(), "65536-byte log MUST succeed; got {res:?}");

    // Sub-test 2: 65 537-byte log fires SandboxHostFnDenied.
    let bytes = wat::parse_str(
        "(module
           (import \"host\" \"log\" (func $log (param i32 i32)))
           (memory (export \"memory\") 2)
           (func (export \"run\") (result i32)
             i32.const 0
             i32.const 65537
             call $log
             i32.const 0
           )
         )",
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let cfg = SandboxConfig {
        output_bytes: 1024 * 1024,
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
    assert_eq!(err.code(), ErrorCode::SandboxHostFnDenied);
}
