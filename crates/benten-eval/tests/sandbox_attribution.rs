//! Phase 2b R3-B — SANDBOX AttributionFrame threading unit test (G7-A).
//!
//! Pin source: sec-pre-r1-03 — closes audit-trail-laundering vector.
//! AttributionFrame must thread through the host-fn dispatch boundary so
//! that audit logs of host-fn invocations carry the dispatching
//! (actor, handler, capability_grant) tuple.
//!
//! Note: D20's `sandbox_depth: u8` extension to AttributionFrame is the
//! Inv-4 / R3-B-fixtures concern (see `invariant_4_runtime.rs`).
//!
//! **G20-A1 wave-8a** (Phase 3): `#[ignore]` removed. The eval-side
//! threading is structurally enforced by the
//! `execute_with_live_cap_check` signature requiring an
//! `&AttributionFrame` argument, AND by the `HostFnContext` carrying
//! `attribution: AttributionFrame` (asserted via source-grep). This test
//! pins the contract:
//!   1. The `execute` API SIGNATURE requires the caller pass an
//!      AttributionFrame (no API path bypasses attribution).
//!   2. The `HostFnContext` actually CARRIES the frame (so trampolines
//!      can read it for audit-trail attribution).
//!   3. NO host-fn return path leaks AttributionFrame fields back into
//!      the guest (companion to attribution_non_regression.rs).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{ManifestRef, ManifestRegistry, SandboxConfig, execute};

#[test]
fn sandbox_attribution_frame_threads_through_host_fn() {
    // sec-pre-r1-03 — the eval-side `execute` signature MUST consume
    // an &AttributionFrame argument (the caller cannot pass a
    // null/default; it's a required parameter). This is the
    // structural gate that closes audit-trail-laundering.
    //
    // The companion runtime drive: invoke `execute` with a distinct
    // attribution frame against a trivial module + assert the call
    // returns Ok (the frame doesn't leak; it threads cleanly).

    let actor_digest = blake3::hash(b"benten:test:actor:sec-pre-r1-03");
    let handler_digest = blake3::hash(b"benten:test:handler:sec-pre-r1-03");
    let grant_digest = blake3::hash(b"benten:test:grant:sec-pre-r1-03");

    let attribution = AttributionFrame {
        actor_cid: Cid::from_blake3_digest(*actor_digest.as_bytes()),
        handler_cid: Cid::from_blake3_digest(*handler_digest.as_bytes()),
        capability_grant_cid: Cid::from_blake3_digest(*grant_digest.as_bytes()),
        sandbox_depth: 1,
        ..Default::default()
    };

    // The frame is non-default in all four slots; `execute` accepts
    // it as a required argument.
    let bytes = wat::parse_str(
        "(module
           (import \"host\" \"time\" (func $time (result i64)))
           (func (export \"run\") (result i64)
             call $time
           )
         )",
    )
    .unwrap();
    let registry = ManifestRegistry::new();
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
    );
    // The call must SUCCEED — non-default attribution threads cleanly
    // through the host-fn dispatch boundary. (A regression that stripped
    // the attribution from HostFnContext would either fail with a
    // missing-frame panic or strip the audit trail silently — the
    // companion absence-pin in attribution_frame_extension_does_not_leak
    // catches the silent-strip variant.)
    assert!(
        res.is_ok(),
        "non-default AttributionFrame MUST thread through cleanly; \
         got {res:?}"
    );

    // STRUCTURAL pin via source-grep: HostFnContext carries the
    // attribution frame for trampoline-side audit access.
    let host_fns_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("sandbox")
            .join("host_fns.rs"),
    )
    .expect("benten-eval/src/sandbox/host_fns.rs must be readable");
    assert!(
        host_fns_src.contains("attribution: AttributionFrame")
            || host_fns_src.contains("attribution: &AttributionFrame")
            || host_fns_src.contains("pub attribution"),
        "HostFnContext MUST carry an `attribution` field for \
         sec-pre-r1-03 audit-trail attribution; absence would mean \
         host-fn invocations lose dispatcher identity"
    );

    // ANTI-LAUNDERING: scan host-functions.toml — NO host-fn entry
    // declares a behavior that READS or RETURNS an AttributionFrame
    // field (the closure of attribution_frame_extension_does_not_leak
    // covers this in detail; this is the pairing assertion).
    let toml_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("host-functions.toml"),
    )
    .expect("workspace host-functions.toml must be readable");
    for line in toml_src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("behavior_kind") {
            assert!(
                !trimmed.contains("attribution") && !trimmed.contains("AttributionFrame"),
                "host-functions.toml MUST NOT declare a host-fn \
                 behavior that exposes AttributionFrame fields to the \
                 guest; attribution-laundering vector. Offending line: {trimmed}"
            );
        }
    }
}
