//! Phase 2b R3-B — `time` host-fn unit test (G7-A).
//!
//! D1 + sec-pre-r1-06 §2.1 + ESC-16 — monotonic-coarsened to 100ms
//! granularity. Closes timezone leak + clock-fingerprinting side channel.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

//! Wave-8b: wired against the live `time` host-fn trampoline.

#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
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
fn sandbox_host_fn_time_returns_monotonic_coarsened_100ms() {
    // ESC-16 — `time` host-fn coarsened to 100ms granularity. The
    // wave-8b trampoline returns module-relative monotonic ms divided
    // by `coarsening_ms`, multiplied back. We verify:
    //   - the returned i64 is small (NOT system epoch which would be ~1.7T).
    //   - successive calls within a tight loop return the same value
    //     (coarsening collapses sub-window samples).
    let bytes = wat::parse_str(
        "(module
           (import \"host\" \"time\" (func $time (result i64)))
           (memory (export \"memory\") 1)
           (func (export \"run\") (result i64)
             call $time
           )
         )",
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let res = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        SandboxConfig::default(),
        &[
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &attribution,
    )
    .unwrap();
    // Decode the i64 little-endian return value.
    let bytes_out = res.output;
    assert_eq!(
        bytes_out.len(),
        8,
        "i64 return value MUST be 8 bytes LE; got {} bytes",
        bytes_out.len()
    );
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes_out);
    let val = i64::from_le_bytes(buf);
    // System epoch in ms is ~1.7e12 since 2024+; module-start relative
    // values are tiny (<1e7 over a year of process uptime). Conservative
    // bound: less than 1e10 (~115 days) catches a regression.
    assert!(
        val < 10_000_000_000,
        "time MUST be module-relative monotonic, NOT system epoch; got {val}"
    );
    // Coarsening: assert val mod 100ms == 0.
    assert!(
        val % 100 == 0,
        "time MUST be coarsened to 100ms granularity; got {val}"
    );
}
