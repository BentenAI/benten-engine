//! Phase 2b R3-B — `random` host-fn deferral regression guard (G7-A).
//!
//! Pin source: D1 (defer per sec-pre-r1-06 §2.3 reasoning — shipping
//! random before the workspace-wide CSPRNG framework decision is a
//! footgun). The destination for re-enabling is
//! `docs/future/phase-3-backlog.md §6.10` (Phase 3 — workspace CSPRNG
//! framework choice).
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
    // D1 — `random` cap claim fires E_SANDBOX_HOST_FN_NOT_FOUND with an
    // operator-actionable hint pointing at phase-3-backlog §6.10
    // (workspace CSPRNG framework choice) at the message level.
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
        // Operator-facing hint MUST signal (a) the host-fn isn't
        // available yet AND (b) where to find the canonical destination
        // doc — see phase-3-backlog.md §6.10 for the workspace CSPRNG
        // framework choice that gates re-enabling. NB: the literal
        // string "§6.10" is load-bearing — drift here means an operator
        // can't grep their way back to the destination doc.
        assert!(
            name.contains("not yet implemented") && name.contains("§6.10"),
            "operator hint MUST signal random-host-fn not-yet-implemented \
             + cite phase-3-backlog §6.10; got: {name}"
        );
    }
}
