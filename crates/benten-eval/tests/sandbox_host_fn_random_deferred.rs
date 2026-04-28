//! Phase 2b R3-B — `random` host-fn DEFERRED-to-Phase-2c regression
//! guard (G7-A).
//!
//! Pin source: D1 (defer-to-2c per sec-pre-r1-06 §2.3 reasoning —
//! shipping random before workspace-wide CSPRNG framework decision is a
//! footgun).
//!
//! Wave-8b: wired against the live `execute()` surface that fires the
//! deferral guard for any manifest claiming `host:compute:random*`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{
    CapBundle, ManifestRef, ManifestRegistry, SandboxConfig, SandboxError, execute,
};

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
fn sandbox_random_host_fn_unavailable_in_phase_2b() {
    // D1 — `random` cap claim fires E_SANDBOX_HOST_FN_NOT_FOUND with a
    // "Phase 2c" hint at the message level.
    let registry = ManifestRegistry::new();
    let inline = CapBundle::new(vec!["host:compute:random".to_string()], None);
    let bytes =
        wat::parse_str("(module (func (export \"run\") (result i32) i32.const 0))").unwrap();
    let attribution = dummy_attribution();
    let err = execute(
        &bytes,
        ManifestRef::Inline(inline),
        &registry,
        SandboxConfig::default(),
        &["host:compute:random".to_string()],
        &attribution,
    )
    .unwrap_err();
    assert_eq!(err.code(), ErrorCode::SandboxHostFnNotFound);
    if let SandboxError::HostFnNotFound { name } = err {
        assert!(
            name.contains("Phase 2c") || name.contains("deferred"),
            "operator hint MUST mention Phase 2c deferral; got: {name}"
        );
    }
}
